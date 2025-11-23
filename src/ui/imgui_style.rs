use bevy::prelude::*;
use bevy_mod_imgui::prelude::*;

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ImguiTheme {
    ModernDark,
    Industrial,
    WarmOrange,
    CellLab,
}

impl ImguiTheme {
    pub fn name(&self) -> &'static str {
        match self {
            ImguiTheme::ModernDark => "Modern Dark",
            ImguiTheme::Industrial => "Industrial",
            ImguiTheme::WarmOrange => "Warm Orange",
            ImguiTheme::CellLab => "Cell Lab",
        }
    }

    pub fn all() -> &'static [ImguiTheme] {
        &[ImguiTheme::ModernDark, ImguiTheme::Industrial, ImguiTheme::WarmOrange, ImguiTheme::CellLab]
    }
}

#[derive(Resource)]
pub struct ImguiThemeState {
    pub current_theme: ImguiTheme,
    pub theme_changed: bool,
}

impl Default for ImguiThemeState {
    fn default() -> Self {
        Self {
            current_theme: ImguiTheme::ModernDark,
            theme_changed: true, // Apply theme on first frame
        }
    }
}

/// System to apply and update ImGui styling
#[allow(invalid_reference_casting)]
pub fn apply_imgui_style_system(
    mut context: NonSendMut<ImguiContext>,
    mut theme_state: ResMut<ImguiThemeState>,
) {
    if theme_state.theme_changed {
        let ui = context.ui();

        // Get a pointer to the style and modify it
        // This is safe because we have exclusive access to the ImguiContext
        unsafe {
            let style_ptr = ui.style() as *const imgui::Style as *mut imgui::Style;
            apply_theme(&mut *style_ptr, theme_state.current_theme);
        }

        theme_state.theme_changed = false;
    }
}

/// Apply rounded styling (common to all themes)
unsafe fn apply_rounded_style(style: &mut imgui::Style) {
    style.window_rounding = 12.0;
    style.window_border_size = 1.0;
    style.window_padding = [12.0, 12.0];
    style.window_title_align = [0.5, 0.5];
    
    style.frame_rounding = 6.0;
    style.frame_border_size = 0.0;
    style.frame_padding = [8.0, 6.0];
    
    style.child_rounding = 8.0;
    style.child_border_size = 1.0;
    
    style.popup_rounding = 8.0;
    style.popup_border_size = 1.0;
    
    style.scrollbar_rounding = 9.0;
    style.scrollbar_size = 14.0;
    
    style.grab_rounding = 6.0;
    style.grab_min_size = 12.0;
    
    style.tab_rounding = 6.0;
    style.tab_border_size = 0.0;
    
    style.item_spacing = [10.0, 6.0];
    style.item_inner_spacing = [6.0, 6.0];
}

/// Apply a specific theme to the ImGui style
unsafe fn apply_theme(style: &mut imgui::Style, theme: ImguiTheme) {
    // Apply default rounded style for most themes
    if theme != ImguiTheme::CellLab {
        apply_rounded_style(style);
    }
    
    match theme {
        ImguiTheme::ModernDark => apply_modern_dark_theme(style),
        ImguiTheme::Industrial => apply_industrial_theme(style),
        ImguiTheme::WarmOrange => apply_warm_orange_theme(style),
        ImguiTheme::CellLab => apply_cell_lab_theme(style),
    }
}

/// Modern Dark theme - Clean, professional, with blue accents
unsafe fn apply_modern_dark_theme(style: &mut imgui::Style) {
    use imgui::StyleColor;
    let colors = &mut style.colors;

    // Backgrounds
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

/// Industrial theme - Orange, yellow, and black construction/caution aesthetic
unsafe fn apply_industrial_theme(style: &mut imgui::Style) {
    use imgui::StyleColor;
    let colors = &mut style.colors;

    // Backgrounds - deep black
    colors[StyleColor::WindowBg as usize] = [0.05, 0.05, 0.05, 0.97];
    colors[StyleColor::ChildBg as usize] = [0.07, 0.07, 0.07, 0.93];
    colors[StyleColor::PopupBg as usize] = [0.05, 0.05, 0.05, 0.98];
    colors[StyleColor::TitleBg as usize] = [0.20, 0.15, 0.00, 1.00];
    colors[StyleColor::TitleBgActive as usize] = [0.95, 0.75, 0.05, 1.00];
    colors[StyleColor::TitleBgCollapsed as usize] = [0.20, 0.15, 0.00, 0.75];
    colors[StyleColor::MenuBarBg as usize] = [0.08, 0.08, 0.08, 1.00];
    colors[StyleColor::Border as usize] = [1.00, 0.50, 0.05, 0.75];
    colors[StyleColor::BorderShadow as usize] = [0.00, 0.00, 0.00, 0.50];
    colors[StyleColor::FrameBg as usize] = [0.12, 0.12, 0.12, 1.00];
    colors[StyleColor::FrameBgHovered as usize] = [0.20, 0.18, 0.10, 1.00];
    colors[StyleColor::FrameBgActive as usize] = [0.28, 0.24, 0.12, 1.00];
    colors[StyleColor::Tab as usize] = [0.15, 0.13, 0.05, 1.00];
    colors[StyleColor::TabHovered as usize] = [0.90, 0.70, 0.10, 0.90];
    colors[StyleColor::TabActive as usize] = [0.85, 0.65, 0.08, 1.00];
    colors[StyleColor::TabUnfocused as usize] = [0.10, 0.10, 0.10, 1.00];
    colors[StyleColor::TabUnfocusedActive as usize] = [0.35, 0.30, 0.10, 1.00];
    colors[StyleColor::Button as usize] = [0.95, 0.45, 0.05, 0.90];
    colors[StyleColor::ButtonHovered as usize] = [1.00, 0.55, 0.15, 1.00];
    colors[StyleColor::ButtonActive as usize] = [0.85, 0.35, 0.00, 1.00];
    colors[StyleColor::Header as usize] = [0.80, 0.55, 0.05, 0.70];
    colors[StyleColor::HeaderHovered as usize] = [0.95, 0.65, 0.10, 0.90];
    colors[StyleColor::HeaderActive as usize] = [0.90, 0.60, 0.08, 1.00];
    colors[StyleColor::Separator as usize] = [0.90, 0.75, 0.10, 0.85];
    colors[StyleColor::SeparatorHovered as usize] = [1.00, 0.85, 0.20, 0.95];
    colors[StyleColor::SeparatorActive as usize] = [1.00, 0.90, 0.30, 1.00];
    colors[StyleColor::ResizeGrip as usize] = [1.00, 0.40, 0.00, 0.65];
    colors[StyleColor::ResizeGripHovered as usize] = [1.00, 0.50, 0.10, 0.85];
    colors[StyleColor::ResizeGripActive as usize] = [1.00, 0.60, 0.20, 1.00];
    colors[StyleColor::ScrollbarBg as usize] = [0.06, 0.06, 0.06, 0.75];
    colors[StyleColor::ScrollbarGrab as usize] = [0.40, 0.35, 0.20, 1.00];
    colors[StyleColor::ScrollbarGrabHovered as usize] = [0.60, 0.45, 0.15, 1.00];
    colors[StyleColor::ScrollbarGrabActive as usize] = [0.80, 0.55, 0.10, 1.00];
    colors[StyleColor::SliderGrab as usize] = [1.00, 0.60, 0.10, 1.00];
    colors[StyleColor::SliderGrabActive as usize] = [1.00, 0.75, 0.25, 1.00];
    colors[StyleColor::CheckMark as usize] = [1.00, 0.90, 0.15, 1.00];
    colors[StyleColor::Text as usize] = [0.98, 0.98, 0.98, 1.00];
    colors[StyleColor::TextDisabled as usize] = [0.45, 0.45, 0.45, 1.00];
    colors[StyleColor::TextSelectedBg as usize] = [0.80, 0.50, 0.10, 0.50];
    colors[StyleColor::PlotLines as usize] = [0.85, 0.65, 0.20, 1.00];
    colors[StyleColor::PlotLinesHovered as usize] = [1.00, 0.80, 0.30, 1.00];
    colors[StyleColor::PlotHistogram as usize] = [1.00, 0.55, 0.05, 1.00];
    colors[StyleColor::PlotHistogramHovered as usize] = [1.00, 0.70, 0.20, 1.00];
    colors[StyleColor::DragDropTarget as usize] = [1.00, 0.15, 0.05, 0.95];
    colors[StyleColor::NavHighlight as usize] = [1.00, 0.85, 0.15, 1.00];
    colors[StyleColor::NavWindowingHighlight as usize] = [1.00, 0.90, 0.50, 0.85];
    colors[StyleColor::NavWindowingDimBg as usize] = [0.10, 0.10, 0.10, 0.50];
    colors[StyleColor::ModalWindowDimBg as usize] = [0.00, 0.00, 0.00, 0.80];
}

/// Warm Orange theme - Cozy, warm browns and oranges inspired by Discord themes
unsafe fn apply_warm_orange_theme(style: &mut imgui::Style) {
    use imgui::StyleColor;
    let colors = &mut style.colors;

    // Text colors - warm tones
    colors[StyleColor::Text as usize] = [0.85, 0.75, 0.68, 1.00]; // text-1: warm light text
    colors[StyleColor::TextDisabled as usize] = [0.40, 0.35, 0.30, 1.00]; // text-5: muted
    colors[StyleColor::TextSelectedBg as usize] = [0.54, 0.35, 0.24, 0.45]; // orange selection

    // Backgrounds - dark warm browns
    colors[StyleColor::WindowBg as usize] = [0.05, 0.04, 0.04, 0.97]; // bg-4: main background
    colors[StyleColor::ChildBg as usize] = [0.08, 0.07, 0.06, 0.93]; // bg-3: secondary
    colors[StyleColor::PopupBg as usize] = [0.05, 0.04, 0.04, 0.98]; // bg-4
    colors[StyleColor::MenuBarBg as usize] = [0.12, 0.10, 0.09, 1.00]; // bg-2
    
    // Title bars - subtle warm gradient
    colors[StyleColor::TitleBg as usize] = [0.12, 0.10, 0.09, 1.00]; // bg-2
    colors[StyleColor::TitleBgActive as usize] = [0.20, 0.16, 0.14, 1.00]; // bg-1
    colors[StyleColor::TitleBgCollapsed as usize] = [0.12, 0.10, 0.09, 0.75];

    // Borders - transparent/subtle
    colors[StyleColor::Border as usize] = [0.20, 0.16, 0.14, 0.30];
    colors[StyleColor::BorderShadow as usize] = [0.00, 0.00, 0.00, 0.00];

    // Frames - dark with warm tint
    colors[StyleColor::FrameBg as usize] = [0.12, 0.10, 0.09, 1.00]; // bg-2
    colors[StyleColor::FrameBgHovered as usize] = [0.20, 0.16, 0.14, 1.00]; // bg-1 hover
    colors[StyleColor::FrameBgActive as usize] = [0.25, 0.20, 0.17, 1.00]; // active state

    // Tabs - warm orange accents
    colors[StyleColor::Tab as usize] = [0.12, 0.10, 0.09, 1.00];
    colors[StyleColor::TabHovered as usize] = [0.66, 0.42, 0.28, 0.85]; // orange-1 hover
    colors[StyleColor::TabActive as usize] = [0.54, 0.35, 0.24, 1.00]; // orange-3 active
    colors[StyleColor::TabUnfocused as usize] = [0.10, 0.08, 0.07, 1.00];
    colors[StyleColor::TabUnfocusedActive as usize] = [0.20, 0.16, 0.14, 1.00];

    // Buttons - orange accent (accent-3, accent-4, accent-5)
    colors[StyleColor::Button as usize] = [0.54, 0.35, 0.24, 0.90]; // orange-3
    colors[StyleColor::ButtonHovered as usize] = [0.48, 0.32, 0.22, 1.00]; // orange-4
    colors[StyleColor::ButtonActive as usize] = [0.42, 0.28, 0.20, 1.00]; // orange-5

    // Headers - orange accent
    colors[StyleColor::Header as usize] = [0.54, 0.35, 0.24, 0.70]; // orange-3
    colors[StyleColor::HeaderHovered as usize] = [0.60, 0.38, 0.26, 0.85]; // orange-2
    colors[StyleColor::HeaderActive as usize] = [0.54, 0.35, 0.24, 1.00]; // orange-3

    // Separator - subtle warm
    colors[StyleColor::Separator as usize] = [0.40, 0.32, 0.27, 0.80]; // text-4
    colors[StyleColor::SeparatorHovered as usize] = [0.60, 0.38, 0.26, 0.85]; // orange-2
    colors[StyleColor::SeparatorActive as usize] = [0.66, 0.42, 0.28, 1.00]; // orange-1

    // Resize grip - orange accent
    colors[StyleColor::ResizeGrip as usize] = [0.54, 0.35, 0.24, 0.60];
    colors[StyleColor::ResizeGripHovered as usize] = [0.60, 0.38, 0.26, 0.80];
    colors[StyleColor::ResizeGripActive as usize] = [0.66, 0.42, 0.28, 1.00];

    // Scrollbar - dark with warm accents
    colors[StyleColor::ScrollbarBg as usize] = [0.08, 0.07, 0.06, 0.70]; // bg-3
    colors[StyleColor::ScrollbarGrab as usize] = [0.25, 0.20, 0.17, 1.00];
    colors[StyleColor::ScrollbarGrabHovered as usize] = [0.35, 0.28, 0.24, 1.00];
    colors[StyleColor::ScrollbarGrabActive as usize] = [0.45, 0.36, 0.30, 1.00];

    // Sliders - orange accent
    colors[StyleColor::SliderGrab as usize] = [0.60, 0.38, 0.26, 1.00]; // orange-2
    colors[StyleColor::SliderGrabActive as usize] = [0.66, 0.42, 0.28, 1.00]; // orange-1

    // Checkmark - bright orange
    colors[StyleColor::CheckMark as usize] = [0.66, 0.42, 0.28, 1.00]; // orange-1

    // Plot colors - warm palette
    colors[StyleColor::PlotLines as usize] = [0.60, 0.50, 0.45, 1.00]; // text-2
    colors[StyleColor::PlotLinesHovered as usize] = [0.66, 0.42, 0.28, 1.00]; // orange-1
    colors[StyleColor::PlotHistogram as usize] = [0.54, 0.35, 0.24, 1.00]; // orange-3
    colors[StyleColor::PlotHistogramHovered as usize] = [0.60, 0.38, 0.26, 1.00]; // orange-2

    // Drag and drop - bright orange for visibility
    colors[StyleColor::DragDropTarget as usize] = [0.66, 0.42, 0.28, 0.95]; // orange-1

    // Navigation - orange accent
    colors[StyleColor::NavHighlight as usize] = [0.60, 0.38, 0.26, 1.00]; // orange-2
    colors[StyleColor::NavWindowingHighlight as usize] = [0.85, 0.75, 0.68, 0.80]; // text-1
    colors[StyleColor::NavWindowingDimBg as usize] = [0.20, 0.16, 0.14, 0.40];

    // Modal dimming - deep warm black
    colors[StyleColor::ModalWindowDimBg as usize] = [0.03, 0.02, 0.02, 0.75];
}

/// Cell Lab theme - Matching the actual Cell Lab game interface
unsafe fn apply_cell_lab_theme(style: &mut imgui::Style) {
    use imgui::StyleColor;
    
    // Cell Lab specific rounding - very round sliders and handles!
    style.window_rounding = 8.0;
    style.window_border_size = 1.0;
    style.window_padding = [12.0, 12.0];
    style.window_title_align = [0.5, 0.5];
    
    style.frame_rounding = 12.0; // Very round for slider tracks!
    style.frame_border_size = 0.0;
    style.frame_padding = [8.0, 6.0];
    
    style.child_rounding = 6.0;
    style.child_border_size = 1.0;
    
    style.popup_rounding = 6.0;
    style.popup_border_size = 1.0;
    
    style.scrollbar_rounding = 12.0; // Round scrollbars
    style.scrollbar_size = 14.0;
    
    style.grab_rounding = 12.0; // VERY ROUND handles like Cell Lab!
    style.grab_min_size = 20.0; // Bigger handles
    
    style.tab_rounding = 6.0;
    style.tab_border_size = 0.0;
    
    style.item_spacing = [10.0, 6.0];
    style.item_inner_spacing = [6.0, 6.0];
    
    let colors = &mut style.colors;

    // Backgrounds - dark grey like the game interface
    colors[StyleColor::WindowBg as usize] = [0.16, 0.16, 0.16, 0.97]; // medium dark grey
    colors[StyleColor::ChildBg as usize] = [0.18, 0.18, 0.18, 0.93]; // slightly lighter
    colors[StyleColor::PopupBg as usize] = [0.16, 0.16, 0.16, 0.98];
    colors[StyleColor::MenuBarBg as usize] = [0.14, 0.14, 0.14, 1.00];
    
    // Title bars - dark with lime green active
    colors[StyleColor::TitleBg as usize] = [0.12, 0.12, 0.12, 1.00];
    colors[StyleColor::TitleBgActive as usize] = [0.20, 0.25, 0.15, 1.00]; // subtle green tint
    colors[StyleColor::TitleBgCollapsed as usize] = [0.12, 0.12, 0.12, 0.75];

    // Borders - bright lime green like the game
    colors[StyleColor::Border as usize] = [0.45, 0.75, 0.15, 0.80]; // bright lime border
    colors[StyleColor::BorderShadow as usize] = [0.00, 0.00, 0.00, 0.00];

    // Frames - LIME GREEN for slider tracks (like the game!)
    colors[StyleColor::FrameBg as usize] = [0.35, 0.60, 0.12, 1.00]; // bright lime green!
    colors[StyleColor::FrameBgHovered as usize] = [0.45, 0.75, 0.15, 1.00]; // brighter lime
    colors[StyleColor::FrameBgActive as usize] = [0.55, 0.85, 0.20, 1.00]; // very bright lime

    // Tabs - matching the game's tab style
    colors[StyleColor::Tab as usize] = [0.14, 0.14, 0.14, 1.00];
    colors[StyleColor::TabHovered as usize] = [0.45, 0.75, 0.15, 0.90]; // lime green hover
    colors[StyleColor::TabActive as usize] = [0.35, 0.60, 0.12, 1.00]; // active lime
    colors[StyleColor::TabUnfocused as usize] = [0.12, 0.12, 0.12, 1.00];
    colors[StyleColor::TabUnfocusedActive as usize] = [0.18, 0.18, 0.18, 1.00];

    // Buttons - subtle like the game's Save/Load buttons
    colors[StyleColor::Button as usize] = [0.22, 0.22, 0.22, 0.90]; // dark grey
    colors[StyleColor::ButtonHovered as usize] = [0.30, 0.35, 0.25, 1.00]; // green tint hover
    colors[StyleColor::ButtonActive as usize] = [0.35, 0.45, 0.25, 1.00]; // brighter green

    // Headers - subtle with lime green highlights
    colors[StyleColor::Header as usize] = [0.20, 0.20, 0.20, 0.70];
    colors[StyleColor::HeaderHovered as usize] = [0.35, 0.60, 0.12, 0.90]; // lime green hover
    colors[StyleColor::HeaderActive as usize] = [0.45, 0.75, 0.15, 1.00]; // bright lime active

    // Separator - lime green like the game borders
    colors[StyleColor::Separator as usize] = [0.45, 0.75, 0.15, 0.80];
    colors[StyleColor::SeparatorHovered as usize] = [0.55, 0.85, 0.20, 0.90];
    colors[StyleColor::SeparatorActive as usize] = [0.65, 0.95, 0.25, 1.00];

    // Resize grip - orange/yellow like slider handles
    colors[StyleColor::ResizeGrip as usize] = [0.95, 0.65, 0.15, 0.70];
    colors[StyleColor::ResizeGripHovered as usize] = [1.00, 0.75, 0.25, 0.85];
    colors[StyleColor::ResizeGripActive as usize] = [1.00, 0.80, 0.30, 1.00];

    // Scrollbar - dark with lime green
    colors[StyleColor::ScrollbarBg as usize] = [0.10, 0.10, 0.10, 0.75];
    colors[StyleColor::ScrollbarGrab as usize] = [0.35, 0.60, 0.12, 1.00];
    colors[StyleColor::ScrollbarGrabHovered as usize] = [0.45, 0.75, 0.15, 1.00];
    colors[StyleColor::ScrollbarGrabActive as usize] = [0.55, 0.85, 0.20, 1.00];

    // Sliders - BRIGHT LIME GREEN bars with ORANGE/YELLOW handles (like the game!)
    colors[StyleColor::SliderGrab as usize] = [1.00, 0.70, 0.20, 1.00]; // orange/yellow handle!
    colors[StyleColor::SliderGrabActive as usize] = [1.00, 0.80, 0.30, 1.00]; // brighter orange!

    // Checkmark - orange/yellow like the game's checkboxes
    colors[StyleColor::CheckMark as usize] = [1.00, 0.75, 0.25, 1.00];

    // Text - WHITE for visibility on dark backgrounds (text on sliders will still be readable)
    colors[StyleColor::Text as usize] = [0.95, 0.95, 0.95, 1.00]; // bright white text
    colors[StyleColor::TextDisabled as usize] = [0.50, 0.50, 0.50, 1.00]; // medium grey
    colors[StyleColor::TextSelectedBg as usize] = [0.20, 0.35, 0.10, 0.60]; // dark green selection for contrast

    // Plot colors - lime green bars with orange handles
    colors[StyleColor::PlotLines as usize] = [0.45, 0.75, 0.15, 1.00]; // lime green line
    colors[StyleColor::PlotLinesHovered as usize] = [0.55, 0.85, 0.20, 1.00]; // bright lime
    colors[StyleColor::PlotHistogram as usize] = [0.45, 0.75, 0.15, 1.00]; // lime green bars
    colors[StyleColor::PlotHistogramHovered as usize] = [0.55, 0.85, 0.20, 1.00]; // bright lime

    // Drag and drop - orange highlight
    colors[StyleColor::DragDropTarget as usize] = [1.00, 0.70, 0.20, 0.95];

    // Navigation - lime green
    colors[StyleColor::NavHighlight as usize] = [0.45, 0.75, 0.15, 1.00];
    colors[StyleColor::NavWindowingHighlight as usize] = [0.85, 0.85, 0.85, 0.80];
    colors[StyleColor::NavWindowingDimBg as usize] = [0.10, 0.10, 0.10, 0.50];

    // Modal dimming - dark like the game
    colors[StyleColor::ModalWindowDimBg as usize] = [0.00, 0.00, 0.00, 0.75];
}
