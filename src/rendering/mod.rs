use bevy::prelude::*;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};

pub mod cells;
pub mod debug;
pub mod adhesion_lines;
pub mod flagellocyte_mesh;

// GPU rendering modules
pub mod gpu_types;
pub mod gpu_renderer;
pub mod gpu_icosphere;
pub mod gpu_camera;
pub mod gpu_triple_buffer;
pub mod gpu_shaders;
pub mod gpu_compute;
pub mod gpu_compute_pipeline;
pub mod gpu_compute_dispatcher;

pub use cells::CellRenderingPlugin;
pub use debug::DebugRenderingPlugin;
pub use adhesion_lines::{AdhesionLineRenderPlugin, AdhesionLineSettings, AdhesionLines};

// GPU rendering exports
pub use gpu_types::{CellInstanceData, WebGpuError};
pub use gpu_renderer::{WebGpuRendererPlugin, GpuSceneData, GpuSceneImguiEdgePlugin};
pub use gpu_icosphere::{IcosphereMesh, IcosphereMeshBuffers, IcosphereVertex};
pub use gpu_camera::{GpuCamera, CameraUniform};
pub use gpu_triple_buffer::{TripleBufferSystem, DEFAULT_MAX_INSTANCES};
pub use gpu_shaders::{ShaderSystem, ShaderError};

/// Main rendering plugin
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin::default())
            .add_plugins(CellRenderingPlugin)
            .add_plugins(DebugRenderingPlugin)
            .add_plugins(AdhesionLineRenderPlugin)
            // Add WebGPU renderer plugin for GPU scene
            .add_plugins(WebGpuRendererPlugin)
            .init_resource::<RenderingConfig>()
            .init_resource::<AdhesionLineSettings>()
            .add_systems(Update, (
                update_gizmos_for_mode,
                update_wireframe_mode,
            ));
    }
}

/// System to toggle wireframe mode for all cell meshes
fn update_wireframe_mode(
    mut commands: Commands,
    rendering_config: Res<RenderingConfig>,
    cells_with_wireframe: Query<Entity, (With<crate::cell::Cell>, With<Wireframe>)>,
    cells_without_wireframe: Query<Entity, (With<crate::cell::Cell>, Without<Wireframe>)>,
) {
    // Only update if the config changed
    if !rendering_config.is_changed() {
        return;
    }

    if rendering_config.wireframe_mode {
        // Add Wireframe to cells that don't have it
        for entity in cells_without_wireframe.iter() {
            commands.entity(entity).insert(Wireframe);
        }
    } else {
        // Remove Wireframe from cells that have it
        for entity in cells_with_wireframe.iter() {
            commands.entity(entity).remove::<Wireframe>();
        }
    }
}



/// System to update gizmo visibility based on simulation mode
/// Orientation and split plane gizmos are enabled by default only in Preview mode
/// Adhesion lines stay on for all modes
/// Only applies defaults if user hasn't manually changed settings
fn update_gizmos_for_mode(
    sim_state: Res<crate::simulation::SimulationState>,
    mut rendering_config: ResMut<RenderingConfig>,
) {
    // Only update if the mode changed and user hasn't manually changed settings
    if !sim_state.is_changed() || rendering_config.user_has_changed_gizmos {
        return;
    }
    
    // Enable orientation/split gizmos for Preview mode only
    let enable_gizmos = sim_state.mode == crate::simulation::SimulationMode::Preview;
    
    rendering_config.show_orientation_gizmos = enable_gizmos;
    rendering_config.show_split_plane_gizmos = enable_gizmos;
    // Keep adhesion lines on for all modes
    rendering_config.show_adhesions = true;
}

/// Rendering configuration
#[derive(Resource)]
pub struct RenderingConfig {
    pub wireframe_mode: bool,
    pub show_adhesions: bool,
    pub show_orientation_gizmos: bool,
    pub show_split_plane_gizmos: bool,
    pub target_fps: f32,
    pub user_has_changed_gizmos: bool,
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            wireframe_mode: false,
            show_adhesions: true,
            show_orientation_gizmos: false,
            show_split_plane_gizmos: false,
            target_fps: 60.0,
            user_has_changed_gizmos: false,
        }
    }
}



/// System that synchronizes Transform components with CellPosition, CellOrientation, and Cell radius
/// Copies CellPosition.position to Transform.translation, CellOrientation.rotation to Transform.rotation,
/// and Cell.radius to Transform.scale
pub fn sync_transforms(
    mut cells_query: Query<(&crate::cell::CellPosition, &crate::cell::CellOrientation, &crate::cell::Cell, &mut Transform)>,
) {
    for (cell_position, cell_orientation, cell, mut transform) in cells_query.iter_mut() {
        transform.translation = cell_position.position;
        transform.rotation = cell_orientation.rotation;
        // OPTIMIZATION: All cells share the same unit sphere mesh, scaled by radius
        transform.scale = Vec3::splat(cell.radius);
    }
}
