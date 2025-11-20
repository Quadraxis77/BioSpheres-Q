use bevy::prelude::*;

pub mod cells;
pub mod debug;

pub use cells::CellRenderingPlugin;
pub use debug::DebugRenderingPlugin;

/// Main rendering plugin
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CellRenderingPlugin)
            .add_plugins(DebugRenderingPlugin)
            .init_resource::<RenderingConfig>();
    }
}

/// Rendering configuration
#[derive(Resource)]
pub struct RenderingConfig {
    pub wireframe_mode: bool,
    pub show_adhesions: bool,
    pub show_orientation_gizmos: bool,
    pub show_split_plane_gizmos: bool,
    pub target_fps: f32,
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            wireframe_mode: false,
            show_adhesions: true,
            show_orientation_gizmos: false,
            show_split_plane_gizmos: false,
            target_fps: 60.0,
        }
    }
}

/// System that synchronizes Transform components with CellPosition
/// Copies CellPosition.position to Transform.translation for rendering
pub fn sync_transforms(
    mut cells_query: Query<(&crate::cell::CellPosition, &mut Transform)>,
) {
    for (cell_position, mut transform) in cells_query.iter_mut() {
        transform.translation = cell_position.position;
    }
}
