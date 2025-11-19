use bevy::prelude::*;

/// Plugin for debug information display
pub struct DebugInfoPlugin;

impl Plugin for DebugInfoPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: Add debug info UI systems
        // - FPS counter
        // - TPS counter
        // - Current simulation mode
        // - Execution time of simulation steps
        // - Cell count
        // - 3D compass for orientation
    }
}

/// Debug info visibility settings
#[derive(Resource, Default)]
pub struct DebugInfoSettings {
    pub show_fps: bool,
    pub show_tps: bool,
    pub show_simulation_mode: bool,
    pub show_profiling: bool,
    pub show_cell_count: bool,
    pub show_compass: bool,
}
