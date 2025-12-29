use bevy::prelude::*;
use bevy_egui;

// Core egui modules
pub mod dock;
pub mod widgets;
pub mod ui_system;
pub mod genome_editor;
pub mod windows;

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
    // Lock settings for UI elements
    pub lock_tab_bar: bool,
    pub lock_tabs: bool,
    pub lock_close_buttons: bool,
    // Individual window lock states
    pub locked_windows: std::collections::HashSet<String>,
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
            lock_tab_bar: false,
            lock_tabs: false,
            lock_close_buttons: false,
            locked_windows: std::collections::HashSet::new(),
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
            .init_resource::<windows::scene_manager::SceneModeRequest>()
            .add_plugins(CameraPlugin)
            .add_systems(Startup, (
                setup_dock,
                load_ui_scale_on_startup,
                settings::load_fog_settings_on_startup,
                settings::load_lighting_settings_on_startup,
                settings::load_skybox_settings_on_startup,
                settings::load_simulation_settings_on_startup,
                settings::load_lock_settings_on_startup,
            ))
            // CRITICAL: ui_system must run in EguiPrimaryContextPass, not Update
            .add_systems(bevy_egui::EguiPrimaryContextPass, ui_system)
            .add_systems(Update, (
                auto_save_dock_state,
                save_on_exit,
                save_ui_scale_on_change,
                settings::save_lock_settings_on_change,
                process_scene_mode_requests,
                // TODO: Re-enable after fixing for egui
                // settings::save_ui_settings_on_change,
            ));
    }
}

/// Load UI scale from saved settings on startup
fn load_ui_scale_on_startup(mut global_ui_state: ResMut<GlobalUiState>) {
    let saved_settings = settings::UiSettings::load();
    global_ui_state.ui_scale = saved_settings.ui_scale;
    info!("Loaded UI scale: {}", global_ui_state.ui_scale);
}

/// Save UI scale when it changes
fn save_ui_scale_on_change(
    global_ui_state: Res<GlobalUiState>,
    mut last_saved_scale: Local<Option<f32>>,
) {
    // Initialize on first run
    if last_saved_scale.is_none() {
        *last_saved_scale = Some(global_ui_state.ui_scale);
        return;
    }

    // Check if scale changed
    let last = last_saved_scale.unwrap();
    if (last - global_ui_state.ui_scale).abs() > 0.001 {
        // Load current settings, update scale, and save
        let mut settings = settings::UiSettings::load();
        settings.ui_scale = global_ui_state.ui_scale;
        
        if let Err(e) = settings.save() {
            error!("Failed to save UI scale: {}", e);
        } else {
            info!("Saved UI scale: {}", global_ui_state.ui_scale);
        }
        
        *last_saved_scale = Some(global_ui_state.ui_scale);
    }
}

/// Process scene mode change requests from the UI
fn process_scene_mode_requests(
    mut scene_request: ResMut<windows::scene_manager::SceneModeRequest>,
    mut sim_state: ResMut<crate::simulation::SimulationState>,
    mut next_preview_state: ResMut<NextState<crate::simulation::PreviewSceneState>>,
    mut next_cpu_state: ResMut<NextState<crate::simulation::CpuSceneState>>,
) {
    if let Some(requested_mode) = scene_request.requested_mode.take() {
        if sim_state.mode != requested_mode {
            match requested_mode {
                crate::simulation::SimulationMode::Preview => {
                    info!("Switching to Preview mode");
                    next_cpu_state.set(crate::simulation::CpuSceneState::Inactive);
                    next_preview_state.set(crate::simulation::PreviewSceneState::Active);
                    sim_state.mode = crate::simulation::SimulationMode::Preview;
                }
                crate::simulation::SimulationMode::Cpu => {
                    info!("Switching to CPU mode");
                    next_preview_state.set(crate::simulation::PreviewSceneState::Inactive);
                    next_cpu_state.set(crate::simulation::CpuSceneState::Active);
                    sim_state.mode = crate::simulation::SimulationMode::Cpu;
                }
                crate::simulation::SimulationMode::Gpu => {
                    warn!("GPU mode not yet implemented");
                    // Don't change mode
                }
            }
        }
    }
}
