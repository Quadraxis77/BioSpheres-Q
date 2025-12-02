use bevy::prelude::*;
use bevy_mod_imgui::ImguiContext;
use imgui;
use crate::ui::camera::{CameraConfig, MainCamera};

/// Plugin for camera settings UI
pub struct CameraSettingsPlugin;

impl Plugin for CameraSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_settings_ui);
    }
}

/// System to render the camera settings UI panel
fn camera_settings_ui(
    mut imgui_context: NonSendMut<ImguiContext>,
    mut camera_config: ResMut<CameraConfig>,
    camera_query: Query<&MainCamera>,
    global_ui_state: Res<crate::ui::GlobalUiState>,
) {
    let ui = imgui_context.ui();

    // Only show if visibility is enabled
    if !global_ui_state.show_camera_settings {
        return;
    }
    
    // Build flags based on lock state
    use imgui::WindowFlags;
    let flags = if global_ui_state.windows_locked {
        WindowFlags::NO_MOVE | WindowFlags::NO_RESIZE
    } else {
        WindowFlags::empty()
    };
    
    ui.window("Camera Settings")
        .size([320.0, 400.0], imgui::Condition::FirstUseEver)
        .position([10.0, 400.0], imgui::Condition::FirstUseEver)
        .flags(flags)
        .build(|| {
            // Display current camera mode
            if let Ok(cam) = camera_query.single() {
                ui.text(format!("Mode: {:?}", cam.mode));
                ui.text_disabled("(Press Tab to switch modes)");
                ui.separator();
            }
            
            // Movement settings
            ui.text("Movement:");
            ui.slider("Move Speed", 1.0, 100.0, &mut camera_config.move_speed);
            if ui.is_item_hovered() {
                ui.tooltip_text("Base movement speed in free-fly mode (WASD)");
            }
            
            ui.slider("Sprint Multiplier", 1.0, 10.0, &mut camera_config.sprint_multiplier);
            if ui.is_item_hovered() {
                ui.tooltip_text("Speed multiplier when holding Shift");
            }
            
            ui.separator();
            
            // Mouse settings
            ui.text("Mouse Control:");
            ui.slider("Mouse Sensitivity", 0.0001, 0.01, &mut camera_config.mouse_sensitivity);
            if ui.is_item_hovered() {
                ui.tooltip_text("Mouse look sensitivity for camera rotation");
            }
            
            ui.checkbox("Invert Look", &mut camera_config.invert_look);
            if ui.is_item_hovered() {
                ui.tooltip_text("Invert vertical mouse axis");
            }
            
            ui.separator();
            
            // Field of view
            ui.text("View:");
            ui.slider("Field of View", 30.0, 120.0, &mut camera_config.fov);
            if ui.is_item_hovered() {
                ui.tooltip_text("Camera field of view in degrees (default: 70°)");
            }
            
            ui.separator();
            
            // Orbit mode settings
            ui.text("Orbit Mode:");
            ui.slider("Zoom Speed", 0.05, 1.0, &mut camera_config.zoom_speed);
            if ui.is_item_hovered() {
                ui.tooltip_text("Mouse wheel zoom speed in orbit mode");
            }
            
            ui.checkbox("Enable Spring Smoothing", &mut camera_config.enable_spring);
            if ui.is_item_hovered() {
                ui.tooltip_text("Smooth camera movement with spring physics (disable for instant movement)");
            }
            
            // Only show spring settings if spring is enabled
            if camera_config.enable_spring {
                ui.slider("Spring Stiffness", 1.0, 50.0, &mut camera_config.spring_stiffness);
                if ui.is_item_hovered() {
                    ui.tooltip_text("How quickly the camera responds to movement (higher = faster)");
                }
                
                ui.slider("Spring Damping", 0.0, 1.0, &mut camera_config.spring_damping);
                if ui.is_item_hovered() {
                    ui.tooltip_text("How much the camera smooths movement (higher = less bouncy)");
                }
            }
            
            ui.separator();
            
            // Free-fly mode settings
            ui.text("Free-Fly Mode:");
            ui.slider("Roll Speed", 0.5, 5.0, &mut camera_config.roll_speed);
            if ui.is_item_hovered() {
                ui.tooltip_text("Roll speed when pressing Q/E (radians per second)");
            }
            
            ui.separator();
            
            // Reset button
            if ui.button("Reset to Defaults") {
                *camera_config = CameraConfig::default();
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Reset all camera settings to default values");
            }
            
            ui.separator();
            ui.text_disabled("Controls:");
            ui.text_disabled("Tab - Switch camera mode");
            ui.text_disabled("Middle Mouse - Orbit (Orbit mode)");
            ui.text_disabled("Right Mouse - Look (Free-fly mode)");
            ui.text_disabled("WASD - Move (Free-fly mode)");
            ui.text_disabled("Space/C - Up/Down (Free-fly mode)");
            ui.text_disabled("Q/E - Roll (Free-fly mode)");
            ui.text_disabled("Scroll - Zoom (Orbit mode)");
            ui.text_disabled("Double-click - Follow cell");
        });
}
