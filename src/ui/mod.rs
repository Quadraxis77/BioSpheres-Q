use bevy::prelude::*;

pub mod camera;
pub mod debug_info;
pub mod imgui_panel;
pub mod imgui_widgets;
pub mod genome_editor;
pub mod imnodes_extensions;
pub mod performance_monitor;
pub mod scene_manager;
pub mod time_scrubber;

pub use camera::{CameraPlugin, MainCamera, CameraConfig, CameraState};
pub use debug_info::DebugInfoPlugin;
pub use imgui_panel::{ImguiPanelPlugin, ImguiPanelState};
pub use genome_editor::GenomeEditorPlugin;
pub use performance_monitor::PerformanceMonitorPlugin;
pub use scene_manager::{SceneManagerPlugin, SceneManagerState};
pub use time_scrubber::{TimeScrubberPlugin, TimeScrubberState};

/// Main UI plugin - provides core UI functionality
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraPlugin)
            .add_plugins(DebugInfoPlugin)
            .add_plugins(ImguiPanelPlugin)
            .add_plugins(PerformanceMonitorPlugin);
    }
}
