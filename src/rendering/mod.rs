use bevy::prelude::*;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::post_process::bloom::{Bloom, BloomCompositeMode as BevyBloomCompositeMode};

pub mod cells;
pub mod debug;
pub mod adhesion_lines;
pub mod flagellocyte_mesh;
pub mod volumetric_fog;
pub mod boundary_crossing;
pub mod skybox;

/// Marker component for the world sphere entity
#[derive(Component)]
pub struct WorldSphere;

pub use cells::CellRenderingPlugin;
pub use debug::DebugRenderingPlugin;
pub use adhesion_lines::{AdhesionLineRenderPlugin, AdhesionLineSettings, AdhesionLines};
pub use volumetric_fog::{VolumetricFogPlugin, VolumetricFogSettings, SphericalFogVolume, SphericalDensityTexture};
pub use boundary_crossing::{BoundaryCrossingPlugin, BoundaryCrossingSettings, BoundaryCrossingState};
pub use skybox::{Skybox, SkyboxConfig, SkyboxConfigured, SkyboxOriginalColor, spawn_skybox, configure_skybox_children, update_skybox_materials};

/// Main rendering plugin
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin::default())
            .add_plugins(CellRenderingPlugin)
            .add_plugins(DebugRenderingPlugin)
            .add_plugins(AdhesionLineRenderPlugin)
            .add_plugins(VolumetricFogPlugin)
            .add_plugins(BoundaryCrossingPlugin)
            .init_resource::<RenderingConfig>()
            .init_resource::<AdhesionLineSettings>()
            .init_resource::<SkyboxConfig>()
            .add_systems(Startup, crate::ui::settings::load_bloom_settings_on_startup)
            .add_systems(Update, (
                update_gizmos_for_mode,
                update_wireframe_mode,
                update_world_sphere_material,
                update_bloom_settings,
                configure_skybox_children,
                update_skybox_materials,
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
    // World sphere settings
    pub world_sphere_opacity: f32,
    pub world_sphere_color: Vec3,
    pub world_sphere_emissive: f32,
    // Bloom settings (emissive-only)
    pub bloom_enabled: bool,
    pub bloom_intensity: f32,
    pub bloom_low_frequency_boost: f32,
    pub bloom_high_pass_frequency: f32,
    pub bloom_composite_mode: BloomCompositeMode,
}

/// Bloom composite mode for UI selection
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum BloomCompositeMode {
    #[default]
    Additive,
    EnergyConserving,
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
            world_sphere_opacity: 0.35,
            world_sphere_color: Vec3::new(0.2, 0.25, 0.35),
            world_sphere_emissive: 0.08,
            // Bloom defaults
            bloom_enabled: true,
            bloom_intensity: 0.3,
            bloom_low_frequency_boost: 0.5,
            bloom_high_pass_frequency: 0.8,
            bloom_composite_mode: BloomCompositeMode::EnergyConserving,
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

/// System to update world sphere material when rendering config changes
fn update_world_sphere_material(
    rendering_config: Res<RenderingConfig>,
    world_sphere_query: Query<&MeshMaterial3d<StandardMaterial>, With<WorldSphere>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !rendering_config.is_changed() {
        return;
    }
    
    for material_handle in world_sphere_query.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color = Color::srgba(
                rendering_config.world_sphere_color.x,
                rendering_config.world_sphere_color.y,
                rendering_config.world_sphere_color.z,
                rendering_config.world_sphere_opacity,
            );
            material.emissive = LinearRgba::rgb(
                rendering_config.world_sphere_emissive,
                rendering_config.world_sphere_emissive * 1.25,
                rendering_config.world_sphere_emissive * 1.5,
            );
        }
    }
}


/// System to update bloom settings on all cameras
fn update_bloom_settings(
    mut commands: Commands,
    rendering_config: Res<RenderingConfig>,
    cameras_with_bloom: Query<(Entity, &Bloom), With<Camera3d>>,
    cameras_without_bloom: Query<Entity, (With<Camera3d>, Without<Bloom>)>,
) {
    if !rendering_config.is_changed() {
        return;
    }
    
    if rendering_config.bloom_enabled {
        // Create bloom settings
        let bloom = Bloom {
            intensity: rendering_config.bloom_intensity,
            low_frequency_boost: rendering_config.bloom_low_frequency_boost,
            high_pass_frequency: rendering_config.bloom_high_pass_frequency,
            composite_mode: match rendering_config.bloom_composite_mode {
                BloomCompositeMode::Additive => BevyBloomCompositeMode::Additive,
                BloomCompositeMode::EnergyConserving => BevyBloomCompositeMode::EnergyConserving,
            },
            ..default()
        };
        
        // Add bloom to cameras that don't have it
        for entity in cameras_without_bloom.iter() {
            commands.entity(entity).insert(bloom.clone());
        }
        
        // Update existing bloom components
        for (entity, _) in cameras_with_bloom.iter() {
            commands.entity(entity).insert(bloom.clone());
        }
    } else {
        // Remove bloom from all cameras
        for (entity, _) in cameras_with_bloom.iter() {
            commands.entity(entity).remove::<Bloom>();
        }
    }
}
