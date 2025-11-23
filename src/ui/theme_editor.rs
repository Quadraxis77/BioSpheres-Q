use bevy::prelude::*;
use bevy_mod_imgui::prelude::*;
use super::{ImguiTheme, ImguiThemeState};

/// Plugin for theme customization UI
pub struct ThemeEditorPlugin;

impl Plugin for ThemeEditorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ThemeEditorState::default())
            .add_systems(Update, theme_editor_ui);
    }
}

#[derive(Resource)]
pub struct ThemeEditorState {
    pub show_editor: bool,
    pub custom_colors: CustomThemeColors,
    pub custom_shapes: CustomThemeShapes,
}

impl Default for ThemeEditorState {
    fn default() -> Self {
        Self {
            show_editor: false,
            custom_colors: CustomThemeColors::default(),
            custom_shapes: CustomThemeShapes::default(),
        }
    }
}

#[derive(Clone)]
pub struct CustomThemeColors {
    // Primary colors
    pub window_bg: [f32; 4],
    pub text: [f32; 4],
    pub border: [f32; 4],
    
    // Interactive elements
    pub button: [f32; 4],
    pub button_hovered: [f32; 4],
    pub button_active: [f32; 4],
    
    // Frames and inputs
    pub frame_bg: [f32; 4],
    pub frame_bg_hovered: [f32; 4],
    pub frame_bg_active: [f32; 4],
    
    // Sliders
    pub slider_grab: [f32; 4],
    pub slider_grab_active: [f32; 4],
    
    // Accents
    pub header: [f32; 4],
    pub header_hovered: [f32; 4],
    pub checkmark: [f32; 4],
}

#[derive(Clone)]
pub struct CustomThemeShapes {
    // Window styling
    pub window_rounding: f32,
    pub window_border_size: f32,
    pub window_padding: [f32; 2],
    
    // Frame styling (sliders, inputs)
    pub frame_rounding: f32,
    pub frame_border_size: f32,
    pub frame_padding: [f32; 2],
    
    // Slider/grab styling
    pub grab_rounding: f32,
    pub grab_min_size: f32,
    
    // Scrollbar styling
    pub scrollbar_rounding: f32,
    pub scrollbar_size: f32,
    
    // Tab styling
    pub tab_rounding: f32,
    
    // Spacing
    pub item_spacing: [f32; 2],
    pub item_inner_spacing: [f32; 2],
}

impl Default for CustomThemeColors {
    fn default() -> Self {
        Self {
            window_bg: [0.16, 0.16, 0.16, 0.97],
            text: [0.95, 0.95, 0.95, 1.00],
            border: [0.45, 0.75, 0.15, 0.80],
            button: [0.22, 0.22, 0.22, 0.90],
            button_hovered: [0.30, 0.35, 0.25, 1.00],
            button_active: [0.35, 0.45, 0.25, 1.00],
            frame_bg: [0.35, 0.60, 0.12, 1.00],
            frame_bg_hovered: [0.45, 0.75, 0.15, 1.00],
            frame_bg_active: [0.55, 0.85, 0.20, 1.00],
            slider_grab: [1.00, 0.70, 0.20, 1.00],
            slider_grab_active: [1.00, 0.80, 0.30, 1.00],
            header: [0.20, 0.20, 0.20, 0.70],
            header_hovered: [0.35, 0.60, 0.12, 0.90],
            checkmark: [1.00, 0.75, 0.25, 1.00],
        }
    }
}

impl Default for CustomThemeShapes {
    fn default() -> Self {
        Self {
            window_rounding: 8.0,
            window_border_size: 1.0,
            window_padding: [12.0, 12.0],
            frame_rounding: 12.0,
            frame_border_size: 0.0,
            frame_padding: [8.0, 6.0],
            grab_rounding: 12.0,
            grab_min_size: 20.0,
            scrollbar_rounding: 12.0,
            scrollbar_size: 14.0,
            tab_rounding: 6.0,
            item_spacing: [10.0, 6.0],
            item_inner_spacing: [6.0, 6.0],
        }
    }
}

fn theme_editor_ui(
    mut imgui_context: NonSendMut<ImguiContext>,
    mut editor_state: ResMut<ThemeEditorState>,
    mut theme_state: ResMut<ImguiThemeState>,
    mut global_ui_state: ResMut<super::GlobalUiState>,
) {
    let ui = imgui_context.ui();

    // Main menu bar with theme editor toggle and options
    if let Some(_menu_bar) = ui.begin_main_menu_bar() {
        // Theme menu
        if let Some(_menu) = ui.begin_menu("Theme") {
            if ui.menu_item("Theme Editor") {
                editor_state.show_editor = !editor_state.show_editor;
            }
            ui.separator();
            
            // Quick theme selection
            for theme in ImguiTheme::all() {
                if ui.menu_item(theme.name()) {
                    theme_state.current_theme = *theme;
                    theme_state.theme_changed = true;
                }
            }
        }
        
        // Options menu
        if let Some(_menu) = ui.begin_menu("Options") {
            // Window lock toggle
            let lock_text = if global_ui_state.windows_locked {
                "🔒 Unlock Windows"
            } else {
                "🔓 Lock Windows"
            };
            
            if ui.menu_item(lock_text) {
                global_ui_state.windows_locked = !global_ui_state.windows_locked;
                println!("Windows locked: {}", global_ui_state.windows_locked);
            }
            
            if ui.is_item_hovered() {
                ui.tooltip_text("Lock windows to prevent moving/resizing");
            }
            
            ui.separator();
            
            // Show lock status
            let status = if global_ui_state.windows_locked {
                "Windows: LOCKED"
            } else {
                "Windows: Unlocked"
            };
            ui.text(status);
        }
    }

    // Theme editor window
    let mut show_editor = editor_state.show_editor;
    let windows_locked = global_ui_state.windows_locked;
    
    if show_editor {
        use imgui::WindowFlags;
        
        // Build flags based on lock state
        let flags = if windows_locked {
            WindowFlags::NO_MOVE | WindowFlags::NO_RESIZE
        } else {
            WindowFlags::empty()
        };
        
        ui.window("Theme Editor")
            .size([398.0, 615.0], imgui::Condition::FirstUseEver)
            .position([994.0, 421.0], imgui::Condition::FirstUseEver)
            .opened(&mut show_editor)
            .flags(flags)
            .build(|| {
                ui.text("Customize Your Theme");
                ui.separator();
                
                // Theme presets
                ui.text("Presets:");
                for theme in ImguiTheme::all() {
                    if ui.button(theme.name()) {
                        theme_state.current_theme = *theme;
                        theme_state.theme_changed = true;
                        
                        // Load preset colors into editor
                        load_preset_colors(&mut editor_state.custom_colors, *theme);
                    }
                    ui.same_line();
                }
                ui.new_line();
                
                ui.separator();
                ui.text("Custom Colors:");
                
                // Color pickers organized by category
                if ui.collapsing_header("Window & Background", imgui::TreeNodeFlags::empty()) {
                    ui.color_edit4("Window Background", &mut editor_state.custom_colors.window_bg);
                    ui.color_edit4("Text", &mut editor_state.custom_colors.text);
                    ui.color_edit4("Border", &mut editor_state.custom_colors.border);
                }
                
                if ui.collapsing_header("Buttons", imgui::TreeNodeFlags::empty()) {
                    ui.color_edit4("Button", &mut editor_state.custom_colors.button);
                    ui.color_edit4("Button Hovered", &mut editor_state.custom_colors.button_hovered);
                    ui.color_edit4("Button Active", &mut editor_state.custom_colors.button_active);
                }
                
                if ui.collapsing_header("Frames & Inputs", imgui::TreeNodeFlags::empty()) {
                    ui.color_edit4("Frame Background", &mut editor_state.custom_colors.frame_bg);
                    ui.color_edit4("Frame Hovered", &mut editor_state.custom_colors.frame_bg_hovered);
                    ui.color_edit4("Frame Active", &mut editor_state.custom_colors.frame_bg_active);
                }
                
                if ui.collapsing_header("Sliders", imgui::TreeNodeFlags::empty()) {
                    ui.color_edit4("Slider Grab", &mut editor_state.custom_colors.slider_grab);
                    ui.color_edit4("Slider Grab Active", &mut editor_state.custom_colors.slider_grab_active);
                }
                
                if ui.collapsing_header("Headers & Accents", imgui::TreeNodeFlags::empty()) {
                    ui.color_edit4("Header", &mut editor_state.custom_colors.header);
                    ui.color_edit4("Header Hovered", &mut editor_state.custom_colors.header_hovered);
                    ui.color_edit4("Checkmark", &mut editor_state.custom_colors.checkmark);
                }
                
                ui.separator();
                ui.text("Shape & Style:");
                
                if ui.collapsing_header("Window Shape", imgui::TreeNodeFlags::empty()) {
                    ui.slider("Window Rounding", 0.0, 20.0, &mut editor_state.custom_shapes.window_rounding);
                    ui.slider("Window Border", 0.0, 5.0, &mut editor_state.custom_shapes.window_border_size);
                    ui.slider("Window Padding X", 0.0, 30.0, &mut editor_state.custom_shapes.window_padding[0]);
                    ui.slider("Window Padding Y", 0.0, 30.0, &mut editor_state.custom_shapes.window_padding[1]);
                }
                
                if ui.collapsing_header("Slider & Frame Shape", imgui::TreeNodeFlags::empty()) {
                    ui.slider("Frame Rounding", 0.0, 20.0, &mut editor_state.custom_shapes.frame_rounding);
                    if ui.is_item_hovered() {
                        ui.tooltip_text("Controls slider track roundness");
                    }
                    
                    ui.slider("Frame Border", 0.0, 5.0, &mut editor_state.custom_shapes.frame_border_size);
                    ui.slider("Frame Padding X", 0.0, 20.0, &mut editor_state.custom_shapes.frame_padding[0]);
                    ui.slider("Frame Padding Y", 0.0, 20.0, &mut editor_state.custom_shapes.frame_padding[1]);
                    
                    ui.separator();
                    ui.slider("Grab Rounding", 0.0, 20.0, &mut editor_state.custom_shapes.grab_rounding);
                    if ui.is_item_hovered() {
                        ui.tooltip_text("Controls slider handle roundness (12.0 = perfect circle)");
                    }
                    
                    ui.slider("Grab Size", 10.0, 30.0, &mut editor_state.custom_shapes.grab_min_size);
                    if ui.is_item_hovered() {
                        ui.tooltip_text("Minimum size of slider handles");
                    }
                }
                
                if ui.collapsing_header("Scrollbar & Tab Shape", imgui::TreeNodeFlags::empty()) {
                    ui.slider("Scrollbar Rounding", 0.0, 20.0, &mut editor_state.custom_shapes.scrollbar_rounding);
                    ui.slider("Scrollbar Size", 8.0, 24.0, &mut editor_state.custom_shapes.scrollbar_size);
                    ui.slider("Tab Rounding", 0.0, 20.0, &mut editor_state.custom_shapes.tab_rounding);
                }
                
                if ui.collapsing_header("Spacing", imgui::TreeNodeFlags::empty()) {
                    ui.slider("Item Spacing X", 0.0, 20.0, &mut editor_state.custom_shapes.item_spacing[0]);
                    ui.slider("Item Spacing Y", 0.0, 20.0, &mut editor_state.custom_shapes.item_spacing[1]);
                    ui.slider("Inner Spacing X", 0.0, 20.0, &mut editor_state.custom_shapes.item_inner_spacing[0]);
                    ui.slider("Inner Spacing Y", 0.0, 20.0, &mut editor_state.custom_shapes.item_inner_spacing[1]);
                }
                
                ui.separator();
                
                // Apply button
                let should_apply = ui.button("Apply Custom Theme");
                ui.same_line();
                let should_reset = ui.button("Reset to Default");
                
                // Handle actions after UI is built
                if should_apply {
                    let colors_clone = editor_state.custom_colors.clone();
                    let shapes_clone = editor_state.custom_shapes.clone();
                    apply_custom_theme(&colors_clone, &shapes_clone, ui);
                }
                
                if should_reset {
                    editor_state.custom_colors = CustomThemeColors::default();
                    editor_state.custom_shapes = CustomThemeShapes::default();
                }
                
                ui.separator();
                ui.text_wrapped("Tip: Adjust colors and shapes, then click 'Apply Custom Theme' to see changes live!");
            });
        
        editor_state.show_editor = show_editor;
    }
}

fn load_preset_colors(colors: &mut CustomThemeColors, theme: ImguiTheme) {
    match theme {
        ImguiTheme::CellLab => {
            colors.window_bg = [0.16, 0.16, 0.16, 0.97];
            colors.text = [0.95, 0.95, 0.95, 1.00];
            colors.border = [0.45, 0.75, 0.15, 0.80];
            colors.frame_bg = [0.35, 0.60, 0.12, 1.00];
            colors.slider_grab = [1.00, 0.70, 0.20, 1.00];
            colors.checkmark = [1.00, 0.75, 0.25, 1.00];
        }
        ImguiTheme::Industrial => {
            colors.window_bg = [0.05, 0.05, 0.05, 0.97];
            colors.text = [0.98, 0.98, 0.98, 1.00];
            colors.border = [1.00, 0.50, 0.05, 0.75];
            colors.frame_bg = [0.12, 0.12, 0.12, 1.00];
            colors.slider_grab = [1.00, 0.60, 0.10, 1.00];
            colors.checkmark = [1.00, 0.90, 0.15, 1.00];
        }
        ImguiTheme::ModernDark => {
            colors.window_bg = [0.10, 0.10, 0.12, 0.95];
            colors.text = [0.95, 0.95, 0.96, 1.00];
            colors.border = [0.20, 0.20, 0.24, 0.50];
            colors.frame_bg = [0.16, 0.16, 0.18, 1.00];
            colors.slider_grab = [0.20, 0.45, 0.80, 1.00];
            colors.checkmark = [0.28, 0.56, 0.90, 1.00];
        }
        ImguiTheme::WarmOrange => {
            colors.window_bg = [0.08, 0.05, 0.12, 0.96];
            colors.text = [0.95, 1.00, 1.00, 1.00];
            colors.border = [0.20, 0.16, 0.14, 0.30];
            colors.frame_bg = [0.12, 0.10, 0.09, 1.00];
            colors.slider_grab = [0.60, 0.38, 0.26, 1.00];
            colors.checkmark = [0.66, 0.42, 0.28, 1.00];
        }
    }
}

#[allow(invalid_reference_casting)]
fn apply_custom_theme(colors: &CustomThemeColors, shapes: &CustomThemeShapes, ui: &imgui::Ui) {
    use imgui::StyleColor;
    
    // Get mutable access to style
    let style = unsafe { &mut *(ui.style() as *const imgui::Style as *mut imgui::Style) };
    
    // Apply custom shapes
    style.window_rounding = shapes.window_rounding;
    style.window_border_size = shapes.window_border_size;
    style.window_padding = shapes.window_padding;
    style.frame_rounding = shapes.frame_rounding;
    style.frame_border_size = shapes.frame_border_size;
    style.frame_padding = shapes.frame_padding;
    style.grab_rounding = shapes.grab_rounding;
    style.grab_min_size = shapes.grab_min_size;
    style.scrollbar_rounding = shapes.scrollbar_rounding;
    style.scrollbar_size = shapes.scrollbar_size;
    style.tab_rounding = shapes.tab_rounding;
    style.item_spacing = shapes.item_spacing;
    style.item_inner_spacing = shapes.item_inner_spacing;
    
    // Apply custom colors
    let style_colors = &mut style.colors;
    style_colors[StyleColor::WindowBg as usize] = colors.window_bg;
    style_colors[StyleColor::Text as usize] = colors.text;
    style_colors[StyleColor::Border as usize] = colors.border;
    style_colors[StyleColor::Button as usize] = colors.button;
    style_colors[StyleColor::ButtonHovered as usize] = colors.button_hovered;
    style_colors[StyleColor::ButtonActive as usize] = colors.button_active;
    style_colors[StyleColor::FrameBg as usize] = colors.frame_bg;
    style_colors[StyleColor::FrameBgHovered as usize] = colors.frame_bg_hovered;
    style_colors[StyleColor::FrameBgActive as usize] = colors.frame_bg_active;
    style_colors[StyleColor::SliderGrab as usize] = colors.slider_grab;
    style_colors[StyleColor::SliderGrabActive as usize] = colors.slider_grab_active;
    style_colors[StyleColor::Header as usize] = colors.header;
    style_colors[StyleColor::HeaderHovered as usize] = colors.header_hovered;
    style_colors[StyleColor::CheckMark as usize] = colors.checkmark;
}


/// Helper function to apply window lock flags
/// Use this when creating windows to respect the lock setting
pub fn apply_window_lock_flags(flags: imgui::WindowFlags, locked: bool) -> imgui::WindowFlags {
    if locked {
        flags | imgui::WindowFlags::NO_MOVE | imgui::WindowFlags::NO_RESIZE
    } else {
        flags
    }
}
