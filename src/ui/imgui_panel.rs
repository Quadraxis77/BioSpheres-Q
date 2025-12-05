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
        // Create default imgui.ini for first-time users with docked layout
        Self::ensure_default_imgui_ini();
        
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

impl ImguiPanelPlugin {
    /// Create default imgui.ini for first-time users with proper docked layout
    fn ensure_default_imgui_ini() {
        use std::path::Path;
        
        let imgui_ini = Path::new("imgui.ini");
        
        // Only create if it doesn't exist (first-time user)
        if !imgui_ini.exists() {
            let default_layout = r#"[Window][Debug##Default]
Pos=60,60
Size=400,400
Collapsed=0

[Window][Genome Editor]
Pos=0,27
Size=700,1053
Collapsed=0

[Window][Scene Manager]
Pos=1704,26
Size=212,321
Collapsed=0
DockId=0x00000002,0

[Window][Time Scrubber]
Pos=702,910
Size=1000,163
Collapsed=0

[Window][Rendering Controls]
Pos=1704,349
Size=212,390
Collapsed=0
DockId=0x00000005,0

[Window][Cell Inspector]
Pos=1704,741
Size=212,336
Collapsed=0
DockId=0x00000008,0

[Window][Advanced Performance Monitor]
Pos=1704,349
Size=212,390
Collapsed=0
DockId=0x00000003,0

[Window][Theme Editor]
Pos=994,421
Size=398,615
Collapsed=0

[Window][Camera Settings]
Pos=2223,215
Size=815,613
Collapsed=0

[Docking][Data]
DockNode        ID=0x00000001 Pos=1704,26 Size=212,1051 Split=Y
  DockNode      ID=0x00000007 Parent=0x00000001 SizeRef=388,954 Split=Y
    DockNode    ID=0x00000004 Parent=0x00000007 SizeRef=403,430 Split=Y
      DockNode  ID=0x00000002 Parent=0x00000004 SizeRef=401,418 Selected=0x6B58BA6D
      DockNode  ID=0x00000003 Parent=0x00000004 SizeRef=401,273 Selected=0x9B936203
    DockNode    ID=0x00000005 Parent=0x00000007 SizeRef=403,522 Selected=0x018F13E1
  DockNode      ID=0x00000008 Parent=0x00000001 SizeRef=388,450 Selected=0x0CE0C78D
"#;
            
            if let Err(e) = std::fs::write(imgui_ini, default_layout) {
                eprintln!("Failed to create default imgui.ini: {}", e);
            }
        }
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
    let io = ui.io();
    
    // Check if left mouse button is being held AND the mouse is actually dragging
    // This prevents the system from running during scrollbar interactions
    let is_mouse_dragging = io.mouse_down[0] && 
        (io.mouse_delta[0].abs() > 0.1 || io.mouse_delta[1].abs() > 0.1);
    
    // Early exit if no actual dragging is happening
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
