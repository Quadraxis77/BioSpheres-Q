use bevy::prelude::*;
use bevy_mod_imgui::prelude::*;
use std::sync::Once;

static STYLE_INIT: Once = Once::new();

/// System to apply custom modern styling to ImGui
/// This runs every frame but only applies the style once
#[allow(invalid_reference_casting)]
pub fn apply_imgui_style_system(mut context: NonSendMut<ImguiContext>) {
    STYLE_INIT.call_once(|| {
        // This is a workaround to modify the style permanently
        // We access the UI which gives us a way to modify the underlying context
        let ui = context.ui();
        
        // Get a pointer to the style and modify it
        // This is safe because:
        // 1. We're only doing it once during initialization (via Once)
        // 2. We have exclusive access to the ImguiContext (NonSendMut)
        // 3. No other code is accessing the style at this point
        unsafe {
            let style_ptr = ui.style() as *const imgui::Style as *mut imgui::Style;
            apply_modern_style(&mut *style_ptr);
        }
    });
}

/// Apply modern, rounded styling to ImGui
unsafe fn apply_modern_style(style: &mut imgui::Style) {
    use imgui::StyleColor;
    
    // Window styling
    style.window_rounding = 12.0;
    style.window_border_size = 1.0;
    style.window_padding = [12.0, 12.0];
    style.window_title_align = [0.5, 0.5];
    
    // Frame styling
    style.frame_rounding = 6.0;
    style.frame_border_size = 0.0;
    style.frame_padding = [8.0, 6.0];
    
    // Child window styling
    style.child_rounding = 8.0;
    style.child_border_size = 1.0;
    
    // Popup styling
    style.popup_rounding = 8.0;
    style.popup_border_size = 1.0;
    
    // Scrollbar styling
    style.scrollbar_rounding = 9.0;
    style.scrollbar_size = 14.0;
    
    // Grab styling
    style.grab_rounding = 6.0;
    style.grab_min_size = 12.0;
    
    // Tab styling
    style.tab_rounding = 6.0;
    style.tab_border_size = 0.0;
    
    // Item spacing
    style.item_spacing = [10.0, 6.0];
    style.item_inner_spacing = [6.0, 6.0];
    
    // Modern color scheme - Dark theme with blue accents
    let colors = &mut style.colors;
    
    colors[StyleColor::WindowBg as usize] = [0.10, 0.10, 0.12, 0.95];
    colors[StyleColor::ChildBg as usize] = [0.12, 0.12, 0.14, 0.90];
    colors[StyleColor::PopupBg as usize] = [0.10, 0.10, 0.12, 0.98];
    colors[StyleColor::TitleBg as usize] = [0.08, 0.08, 0.10, 1.00];
    colors[StyleColor::TitleBgActive as usize] = [0.12, 0.12, 0.15, 1.00];
    colors[StyleColor::TitleBgCollapsed as usize] = [0.08, 0.08, 0.10, 0.75];
    colors[StyleColor::MenuBarBg as usize] = [0.12, 0.12, 0.14, 1.00];
    colors[StyleColor::Border as usize] = [0.20, 0.20, 0.24, 0.50];
    colors[StyleColor::BorderShadow as usize] = [0.00, 0.00, 0.00, 0.00];
    colors[StyleColor::FrameBg as usize] = [0.16, 0.16, 0.18, 1.00];
    colors[StyleColor::FrameBgHovered as usize] = [0.20, 0.20, 0.24, 1.00];
    colors[StyleColor::FrameBgActive as usize] = [0.24, 0.24, 0.28, 1.00];
    colors[StyleColor::Tab as usize] = [0.12, 0.12, 0.14, 1.00];
    colors[StyleColor::TabHovered as usize] = [0.28, 0.56, 0.90, 0.80];
    colors[StyleColor::TabActive as usize] = [0.20, 0.45, 0.80, 1.00];
    colors[StyleColor::TabUnfocused as usize] = [0.10, 0.10, 0.12, 1.00];
    colors[StyleColor::TabUnfocusedActive as usize] = [0.14, 0.14, 0.16, 1.00];
    colors[StyleColor::Button as usize] = [0.20, 0.45, 0.80, 0.80];
    colors[StyleColor::ButtonHovered as usize] = [0.28, 0.56, 0.90, 1.00];
    colors[StyleColor::ButtonActive as usize] = [0.16, 0.36, 0.70, 1.00];
    colors[StyleColor::Header as usize] = [0.20, 0.45, 0.80, 0.60];
    colors[StyleColor::HeaderHovered as usize] = [0.28, 0.56, 0.90, 0.80];
    colors[StyleColor::HeaderActive as usize] = [0.24, 0.50, 0.85, 1.00];
    colors[StyleColor::Separator as usize] = [0.20, 0.20, 0.24, 1.00];
    colors[StyleColor::SeparatorHovered as usize] = [0.28, 0.56, 0.90, 0.78];
    colors[StyleColor::SeparatorActive as usize] = [0.28, 0.56, 0.90, 1.00];
    colors[StyleColor::ResizeGrip as usize] = [0.20, 0.45, 0.80, 0.40];
    colors[StyleColor::ResizeGripHovered as usize] = [0.28, 0.56, 0.90, 0.67];
    colors[StyleColor::ResizeGripActive as usize] = [0.28, 0.56, 0.90, 0.95];
    colors[StyleColor::ScrollbarBg as usize] = [0.08, 0.08, 0.10, 0.60];
    colors[StyleColor::ScrollbarGrab as usize] = [0.20, 0.20, 0.24, 1.00];
    colors[StyleColor::ScrollbarGrabHovered as usize] = [0.28, 0.28, 0.32, 1.00];
    colors[StyleColor::ScrollbarGrabActive as usize] = [0.36, 0.36, 0.40, 1.00];
    colors[StyleColor::SliderGrab as usize] = [0.20, 0.45, 0.80, 1.00];
    colors[StyleColor::SliderGrabActive as usize] = [0.28, 0.56, 0.90, 1.00];
    colors[StyleColor::CheckMark as usize] = [0.28, 0.56, 0.90, 1.00];
    colors[StyleColor::Text as usize] = [0.95, 0.95, 0.96, 1.00];
    colors[StyleColor::TextDisabled as usize] = [0.50, 0.50, 0.52, 1.00];
    colors[StyleColor::TextSelectedBg as usize] = [0.20, 0.45, 0.80, 0.35];
    colors[StyleColor::PlotLines as usize] = [0.61, 0.61, 0.64, 1.00];
    colors[StyleColor::PlotLinesHovered as usize] = [0.28, 0.56, 0.90, 1.00];
    colors[StyleColor::PlotHistogram as usize] = [0.28, 0.56, 0.90, 1.00];
    colors[StyleColor::PlotHistogramHovered as usize] = [0.36, 0.64, 0.95, 1.00];
    colors[StyleColor::DragDropTarget as usize] = [0.28, 0.56, 0.90, 0.90];
    colors[StyleColor::NavHighlight as usize] = [0.28, 0.56, 0.90, 1.00];
    colors[StyleColor::NavWindowingHighlight as usize] = [1.00, 1.00, 1.00, 0.70];
    colors[StyleColor::NavWindowingDimBg as usize] = [0.80, 0.80, 0.80, 0.20];
    colors[StyleColor::ModalWindowDimBg as usize] = [0.00, 0.00, 0.00, 0.60];
}
