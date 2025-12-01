use bevy::prelude::*;
use super::imgui_style::{self, ImguiThemeState};

#[derive(Resource, Default)]
pub struct ImguiPanelState {
    pub show_debug_info: bool,
}

/// ImGui panel plugin for BioSpheres
pub struct ImguiPanelPlugin;

impl Plugin for ImguiPanelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ImguiPanelState {
            show_debug_info: true,
        })
        .insert_resource(ImguiThemeState::default())
        .add_plugins(bevy_mod_imgui::ImguiPlugin {
            ini_filename: Some("imgui.ini".into()),
            // Use a fixed font size instead of automatic DPI scaling
            // Automatic DPI scaling can be unreliable on Windows systems with mixed DPI
            // configurations or remote desktop connections, causing panels to render at
            // massive, distorted sizes. By disabling automatic scaling and using a fixed
            // font size, we ensure consistent, predictable UI rendering across all environments.
            //font_size: 16.0,
            apply_display_scale_to_font_size: false, // Disable automatic DPI scaling
            apply_display_scale_to_font_oversample: false, // Disable oversampling scaling
            ..Default::default()
        })
        .add_systems(Update, imgui_style::apply_imgui_style_system)
        .add_systems(Update, enable_window_clamping);
    }
}

/// System to clamp all window positions to viewport bounds
/// This ensures the ENTIRE window stays within the visible viewport area
/// Only runs when a window is being actively dragged and windows are not locked
fn enable_window_clamping(
    mut context: NonSendMut<bevy_mod_imgui::prelude::ImguiContext>,
    global_ui_state: Res<crate::ui::GlobalUiState>,
) {
    // Skip entirely if windows are locked - they can't be moved anyway
    if global_ui_state.windows_locked {
        return;
    }
    
    let ui = context.ui();
    
    // Check if any mouse button is being held (indicating potential drag)
    let io = ui.io();
    let is_mouse_dragging = io.mouse_down[0]; // Left mouse button
    
    // Early exit if no dragging is happening
    if !is_mouse_dragging {
        return;
    }
    
    // Get the viewport bounds from IO
    let viewport_size = io.display_size;
    let viewport_min = [0.0, 0.0];
    let viewport_max = viewport_size;
    
    // Access the internal window list and clamp each window's position
    // We need to use the internal imgui sys API to iterate over windows
    unsafe {
        let ctx = imgui::sys::igGetCurrentContext();
        if ctx.is_null() {
            return;
        }
        
        // Get the windows vector from the context
        let windows_vec = &(*ctx).Windows;
        
        // Iterate through all windows
        for i in 0..windows_vec.Size {
            let window = *windows_vec.Data.offset(i as isize);
            if window.is_null() {
                continue;
            }
            
            // Skip if window is not active or is a child window
            if !(*window).Active || (*window).ParentWindow != std::ptr::null_mut() {
                continue;
            }
            
            // Check if this window is being moved (has the Moving flag set)
            let is_moving = ((*window).Flags & imgui::sys::ImGuiWindowFlags_NoMove as i32) == 0
                && (*window).MoveId != 0;
            
            // Only clamp if the window is actually being moved
            if !is_moving {
                continue;
            }
            
            // Get current window position and size
            let mut pos = (*window).Pos;
            let size = (*window).Size;
            
            // Clamp the window position so the entire window stays within viewport
            // Clamp X position
            if pos.x < viewport_min[0] {
                pos.x = viewport_min[0];
            } else if pos.x + size.x > viewport_max[0] {
                pos.x = viewport_max[0] - size.x;
            }
            
            // Clamp Y position
            if pos.y < viewport_min[1] {
                pos.y = viewport_min[1];
            } else if pos.y + size.y > viewport_max[1] {
                pos.y = viewport_max[1] - size.y;
            }
            
            // Update the window position if it was clamped
            if pos.x != (*window).Pos.x || pos.y != (*window).Pos.y {
                imgui::sys::igSetWindowPos_WindowPtr(
                    window,
                    pos,
                    imgui::sys::ImGuiCond_Always as i32,
                );
            }
        }
    }
}
