use bevy::prelude::*;

pub mod camera;
pub mod debug_info;
pub mod imgui_panel;
pub mod imgui_style;
pub mod imgui_widgets;
pub mod genome_editor;
pub mod imnodes_extensions;
pub mod performance_monitor;
pub mod scene_manager;
pub mod theme_editor;
pub mod time_scrubber;
pub mod rendering_controls;

pub use camera::{CameraPlugin, MainCamera, CameraConfig, CameraState};
pub use debug_info::DebugInfoPlugin;
pub use imgui_panel::{ImguiPanelPlugin, ImguiPanelState};
pub use imgui_style::{ImguiTheme, ImguiThemeState};
pub use genome_editor::GenomeEditorPlugin;
pub use performance_monitor::PerformanceMonitorPlugin;
pub use scene_manager::{SceneManagerPlugin, SceneManagerState};
pub use theme_editor::{ThemeEditorPlugin, ThemeEditorState};
pub use time_scrubber::{TimeScrubberPlugin, TimeScrubberState};
pub use rendering_controls::RenderingControlsPlugin;

/// Global UI state shared across all UI components
#[derive(Resource)]
pub struct GlobalUiState {
    pub windows_locked: bool,
}

impl Default for GlobalUiState {
    fn default() -> Self {
        Self {
            windows_locked: true,
        }
    }
}

/// Main UI plugin - provides core UI functionality
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalUiState>()
            .add_plugins(CameraPlugin)
            .add_plugins(DebugInfoPlugin)
            .add_plugins(ImguiPanelPlugin)
            .add_plugins(PerformanceMonitorPlugin)
            .add_plugins(RenderingControlsPlugin)
            .add_plugins(ThemeEditorPlugin);
    }
}
