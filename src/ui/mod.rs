use bevy::prelude::*;
use bevy_egui;

// Core egui modules
pub mod dock;
pub mod widgets;
pub mod ui_system;
pub mod genome_editor;

// Feature modules (still using old implementations for now)
pub mod camera;
pub mod settings;

// Temporary stubs for resource types (until full egui implementation)
#[path = "scene_manager_stub.rs"]
pub mod scene_manager;
#[path = "lighting_settings_stub.rs"]
pub mod lighting_settings;

// TODO: These will be reimplemented as egui windows
// pub mod camera_settings;
// pub mod cell_inspector;
// pub mod genome_editor;
// pub mod rendering_controls;
// pub mod time_scrubber;

// Export core egui components
pub use dock::{DockResource, Panel, setup_dock, auto_save_dock_state, save_on_exit, show_windows_menu};
pub use ui_system::{ui_system, ViewportRect, GenomeEditorState};

// Export camera (still using old implementation)
pub use camera::{CameraPlugin, MainCamera, CameraConfig, CameraState, CameraMode, FocalPlaneSettings};

// Export settings
pub use settings::UiSettings;

// Export resource types from stubs
pub use scene_manager::CpuCellCapacity;
pub use lighting_settings::LightingConfig;

// TODO: Re-enable these exports once windows are implemented
// pub use camera_settings::CameraSettingsPlugin;
// pub use cell_inspector::{CellInspectorPlugin, CellInspectorState};
// pub use genome_editor::GenomeEditorPlugin;
// pub use rendering_controls::RenderingControlsPlugin;
// pub use time_scrubber::{TimeScrubberPlugin, TimeScrubberState};

/// Global UI state shared across all UI components
#[derive(Resource)]
pub struct GlobalUiState {
    pub windows_locked: bool,
    pub ui_scale: f32,
    // Window visibility toggles
    pub show_cell_inspector: bool,
    pub show_genome_editor: bool,
    pub show_scene_manager: bool,
    pub show_rendering_controls: bool,
    pub show_time_scrubber: bool,
    pub show_camera_settings: bool,
    pub show_lighting_settings: bool,
}

impl Default for GlobalUiState {
    fn default() -> Self {
        Self {
            windows_locked: false,
            ui_scale: 1.0,
            show_cell_inspector: true,
            show_genome_editor: true,
            show_scene_manager: true,
            show_rendering_controls: false,
            show_time_scrubber: true,
            show_camera_settings: false,
            show_lighting_settings: false,
        }
    }
}

// Theme loading removed - egui will use default themes

/// Main UI plugin - provides core UI functionality with egui
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalUiState>()
            .init_resource::<ViewportRect>()
            .init_resource::<GenomeEditorState>()
            .init_resource::<CpuCellCapacity>()
            .init_resource::<LightingConfig>()
            .add_plugins(CameraPlugin)
            .add_systems(Startup, (
                setup_dock,
                settings::load_fog_settings_on_startup,
                settings::load_lighting_settings_on_startup,
                settings::load_skybox_settings_on_startup,
                settings::load_simulation_settings_on_startup,
            ))
            // CRITICAL: ui_system must run in EguiPrimaryContextPass, not Update
            .add_systems(bevy_egui::EguiPrimaryContextPass, ui_system)
            .add_systems(Update, (
                auto_save_dock_state,
                save_on_exit,
                // TODO: Re-enable after fixing for egui
                // settings::save_ui_settings_on_change,
            ));
    }
}
