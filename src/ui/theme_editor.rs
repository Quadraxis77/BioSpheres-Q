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
    /// Name of currently active custom theme (if any)
    pub active_custom_theme: Option<String>,
    /// Text buffer for naming new custom themes
    pub new_theme_name: String,
}

impl Default for ThemeEditorState {
    fn default() -> Self {
        Self {
            show_editor: false,
            custom_colors: CustomThemeColors::default(),
            custom_shapes: CustomThemeShapes::default(),
            active_custom_theme: None,
            new_theme_name: String::new(),
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
    mut cpu_cell_capacity: ResMut<crate::ui::scene_manager::CpuCellCapacity>,
    simulation_state: Res<crate::simulation::SimulationState>,
    mut physics_config: ResMut<crate::simulation::PhysicsConfig>,
    spatial_grid_config: Res<crate::simulation::SpatialGridConfig>,
    mut pending_grid_density: ResMut<crate::ui::scene_manager::PendingGridDensity>,
    mut threading_config: ResMut<crate::simulation::SimulationThreadingConfig>,
    gpu_physics: Res<crate::simulation::GpuPhysicsResource>,
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
            
            // Quick theme selection - Presets
            ui.text("Preset Themes:");
            for theme in ImguiTheme::all() {
                if ui.menu_item(theme.name()) {
                    theme_state.current_theme = *theme;
                    theme_state.theme_changed = true;
                    editor_state.active_custom_theme = None;
                }
            }

            // Custom themes
            let settings = crate::ui::settings::UiSettings::load();
            if !settings.custom_themes.is_empty() {
                ui.separator();
                ui.text("Custom Themes:");

                let mut theme_to_load: Option<String> = None;
                let mut theme_to_delete: Option<String> = None;

                for custom_theme in &settings.custom_themes {
                    if ui.menu_item(&custom_theme.name) {
                        theme_to_load = Some(custom_theme.name.clone());
                    }

                    // Show delete option on right-click
                    if ui.is_item_clicked_with_button(imgui::MouseButton::Right) {
                        theme_to_delete = Some(custom_theme.name.clone());
                    }

                    if ui.is_item_hovered() {
                        ui.tooltip_text("Right-click to delete");
                    }
                }

                // Handle theme loading after iteration
                if let Some(theme_name) = theme_to_load {
                    if let Some(theme) = settings.custom_themes.iter().find(|t| t.name == theme_name) {
                        load_custom_theme(&mut editor_state, theme);
                        apply_custom_theme(&editor_state.custom_colors, &editor_state.custom_shapes, ui);
                    }
                }

                // Handle theme deletion after iteration
                if let Some(theme_name) = theme_to_delete {
                    let mut settings = crate::ui::settings::UiSettings::load();
                    settings.custom_themes.retain(|t| t.name != theme_name);
                    if let Err(e) = settings.save() {
                        error!("Failed to delete custom theme: {}", e);
                    }

                    // Clear active custom theme if it was deleted
                    if editor_state.active_custom_theme.as_ref() == Some(&theme_name) {
                        editor_state.active_custom_theme = None;
                    }
                }
            }
        }
        
        // Windows menu - for toggling window visibility
        if let Some(_menu) = ui.begin_menu("Windows") {
            ui.checkbox("Cell Inspector", &mut global_ui_state.show_cell_inspector);
            
            ui.checkbox("Genome Editor", &mut global_ui_state.show_genome_editor);
            if ui.is_item_hovered() {
                ui.tooltip_text("Only available in the Genome Editor scene");
            }
            
            ui.checkbox("Scene Manager", &mut global_ui_state.show_scene_manager);
            ui.checkbox("Performance Monitor", &mut global_ui_state.show_performance_monitor);
            ui.checkbox("Rendering Controls", &mut global_ui_state.show_rendering_controls);
            ui.checkbox("Camera Settings", &mut global_ui_state.show_camera_settings);
            ui.checkbox("Lighting Settings", &mut global_ui_state.show_lighting_settings);
            
            ui.checkbox("Time Scrubber", &mut global_ui_state.show_time_scrubber);
            if ui.is_item_hovered() {
                ui.tooltip_text("Only available in the Genome Editor scene");
            }
            
            ui.checkbox("Theme Editor", &mut global_ui_state.show_theme_editor);
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

            // UI Scale radio buttons
            ui.text("UI Scale");

            let scale_options = [
                (0.75, "75%"),
                (1.0, "100%"),
                (1.25, "125%"),
                (1.5, "150%"),
                (1.75, "175%"),
                (2.0, "200%"),
                (2.5, "250%"),
                (3.0, "300%"),
                (3.5, "350%"),
                (4.0, "400%"),
            ];

            let mut current_scale = global_ui_state.ui_scale;
            for (scale_value, label) in scale_options.iter() {
                if ui.radio_button(label, &mut current_scale, *scale_value) {
                    global_ui_state.ui_scale = *scale_value;
                }
                ui.same_line();
            }
            ui.new_line();

            ui.separator();

            // CPU Cell Capacity slider (only shown in CPU mode)
            if simulation_state.mode == crate::simulation::SimulationMode::Cpu {
                ui.text("CPU Cell Capacity (Next Reset)");
                let mut capacity = cpu_cell_capacity.capacity as i32;
                if ui.slider("##CpuCellCapacity", 100, 10000, &mut capacity) {
                    cpu_cell_capacity.capacity = capacity as usize;
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text("Set maximum cells for CPU scene.\nRequires scene reset to take effect.\nUse Scene Manager to reset.");
                }
                
                ui.separator();
                
                // Spatial Grid Density slider
                ui.text("Spatial Grid Density");
                let current_density = spatial_grid_config.grid_density;
                ui.text(format!("Active: {}³ cells", current_density));
                
                let mut density = pending_grid_density.density as i32;
                if ui.slider("##GridDensity", 
                    crate::simulation::SpatialGridConfig::MIN_DENSITY as i32, 
                    crate::simulation::SpatialGridConfig::MAX_DENSITY as i32, 
                    &mut density) 
                {
                    pending_grid_density.density = density as u32;
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text("Spatial grid resolution for collision detection.\n16³ = coarse (faster), 128³ = fine (more accurate).\nRequires scene reset to take effect.");
                }
                
                // Show pending change indicator
                if pending_grid_density.density != current_density {
                    ui.text_colored([1.0, 0.8, 0.2, 1.0], format!("Pending: {}³", pending_grid_density.density));
                }
                
                ui.text_colored([1.0, 1.0, 0.0, 1.0], "⚠ Reset scene to apply");
                
                ui.separator();
            }
            
            // Disable Collisions checkbox
            ui.checkbox("Disable Collisions", &mut physics_config.disable_collisions);
            if ui.is_item_hovered() {
                ui.tooltip_text("Disable all collision detection.\nCells will pass through each other.\nUseful for performance testing.");
            }
            
            ui.separator();
            
            // GPU Physics section
            ui.text("Physics Acceleration");
            
            // GPU Physics toggle (only if GPU is available)
            if gpu_physics.context.is_some() {
                ui.checkbox("GPU Physics", &mut threading_config.gpu_physics_enabled);
                if ui.is_item_hovered() {
                    ui.tooltip_text("Use GPU compute shaders for collision detection.\nCan significantly improve performance with many cells.");
                }
            } else {
                ui.text_disabled("GPU Physics (unavailable)");
                if ui.is_item_hovered() {
                    ui.tooltip_text("GPU physics is not available.\nNo compatible GPU adapter found.");
                }
            }
            
            // CPU Multithreading toggle
            ui.checkbox("CPU Multithreading", &mut threading_config.cpu_multithreaded);
            if ui.is_item_hovered() {
                ui.tooltip_text("Use multiple CPU threads for physics.\nMay improve performance on multi-core systems.");
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
        
        // Add version text on the right side of the menu bar
        let version_text = "Bio-Spheres (v.0.1.8)";
        let text_width = ui.calc_text_size(version_text)[0];
        let window_width = ui.window_size()[0];
        let padding = 10.0;
        
        // Position cursor to the right side
        ui.set_cursor_pos([window_width - text_width - padding, ui.cursor_pos()[1]]);
        ui.text(version_text);
    }

    // Theme editor window - use global visibility state
    let windows_locked = global_ui_state.windows_locked;
    
    if global_ui_state.show_theme_editor {
        use imgui::WindowFlags;
        
        // Build flags based on lock state
        let flags = if windows_locked {
            WindowFlags::NO_MOVE | WindowFlags::NO_RESIZE
        } else {
            WindowFlags::empty()
        };
        
        ui.window("Theme Editor")
            .size([398.0, 615.0], imgui::Condition::FirstUseEver)
            .position([610.0, 30.0], imgui::Condition::FirstUseEver)
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

                // Save custom theme section
                ui.text("Save Custom Theme:");
                ui.set_next_item_width(-1.0);

                // Create a buffer for the text input
                let mut name_buffer = editor_state.new_theme_name.clone();
                name_buffer.reserve(128);

                if ui.input_text("##theme_name", &mut name_buffer).build() {
                    editor_state.new_theme_name = name_buffer;
                }

                let can_save = !editor_state.new_theme_name.trim().is_empty();

                let mut should_save = false;
                ui.disabled(!can_save, || {
                    should_save = ui.button("Save Theme");
                });

                if ui.is_item_hovered() && !can_save {
                    ui.tooltip_text("Enter a theme name to save");
                }

                ui.separator();

                // Apply and reset buttons
                let should_apply = ui.button("Apply Custom Theme");
                ui.same_line();
                let should_reset = ui.button("Reset to Default");

                // Handle save action
                if should_save && can_save {
                    let theme_name = editor_state.new_theme_name.trim().to_string();

                    // Load current settings
                    let mut settings = crate::ui::settings::UiSettings::load();

                    // Check if theme already exists
                    if let Some(existing_idx) = settings.custom_themes.iter().position(|t| t.name == theme_name) {
                        // Update existing theme
                        settings.custom_themes[existing_idx] = crate::ui::settings::SavedCustomTheme {
                            name: theme_name.clone(),
                            colors: custom_colors_to_serde(&editor_state.custom_colors),
                            shapes: custom_shapes_to_serde(&editor_state.custom_shapes),
                        };
                    } else {
                        // Add new theme
                        settings.custom_themes.push(crate::ui::settings::SavedCustomTheme {
                            name: theme_name.clone(),
                            colors: custom_colors_to_serde(&editor_state.custom_colors),
                            shapes: custom_shapes_to_serde(&editor_state.custom_shapes),
                        });
                    }

                    // Save settings
                    if let Err(e) = settings.save() {
                        error!("Failed to save custom theme: {}", e);
                    }

                    // Set this as the active custom theme
                    editor_state.active_custom_theme = Some(theme_name);

                    // Clear the name input
                    editor_state.new_theme_name.clear();
                }

                // Handle actions after UI is built
                if should_apply {
                    let colors_clone = editor_state.custom_colors.clone();
                    let shapes_clone = editor_state.custom_shapes.clone();
                    apply_custom_theme(&colors_clone, &shapes_clone, ui);

                    // Mark as unnamed custom theme
                    editor_state.active_custom_theme = None;
                }

                if should_reset {
                    editor_state.custom_colors = CustomThemeColors::default();
                    editor_state.custom_shapes = CustomThemeShapes::default();
                    editor_state.active_custom_theme = None;
                }

                ui.separator();
                ui.text_wrapped("Tip: Adjust colors and shapes, name your theme, then click 'Save Theme' to add it to the theme list!");
            });
    }
}

/// Load a custom theme from saved settings into the editor
fn load_custom_theme(editor_state: &mut ThemeEditorState, theme: &crate::ui::settings::SavedCustomTheme) {
    // Load colors
    editor_state.custom_colors = CustomThemeColors {
        window_bg: theme.colors.window_bg,
        text: theme.colors.text,
        border: theme.colors.border,
        button: theme.colors.button,
        button_hovered: theme.colors.button_hovered,
        button_active: theme.colors.button_active,
        frame_bg: theme.colors.frame_bg,
        frame_bg_hovered: theme.colors.frame_bg_hovered,
        frame_bg_active: theme.colors.frame_bg_active,
        slider_grab: theme.colors.slider_grab,
        slider_grab_active: theme.colors.slider_grab_active,
        header: theme.colors.header,
        header_hovered: theme.colors.header_hovered,
        checkmark: theme.colors.checkmark,
    };

    // Load shapes
    editor_state.custom_shapes = CustomThemeShapes {
        window_rounding: theme.shapes.window_rounding,
        window_border_size: theme.shapes.window_border_size,
        window_padding: theme.shapes.window_padding,
        frame_rounding: theme.shapes.frame_rounding,
        frame_border_size: theme.shapes.frame_border_size,
        frame_padding: theme.shapes.frame_padding,
        grab_rounding: theme.shapes.grab_rounding,
        grab_min_size: theme.shapes.grab_min_size,
        scrollbar_rounding: theme.shapes.scrollbar_rounding,
        scrollbar_size: theme.shapes.scrollbar_size,
        tab_rounding: theme.shapes.tab_rounding,
        item_spacing: theme.shapes.item_spacing,
        item_inner_spacing: theme.shapes.item_inner_spacing,
    };

    // Set the active custom theme name
    editor_state.active_custom_theme = Some(theme.name.clone());
}

/// Convert CustomThemeColors to serializable format
fn custom_colors_to_serde(colors: &CustomThemeColors) -> crate::ui::settings::CustomThemeColorsSerde {
    crate::ui::settings::CustomThemeColorsSerde {
        window_bg: colors.window_bg,
        text: colors.text,
        border: colors.border,
        button: colors.button,
        button_hovered: colors.button_hovered,
        button_active: colors.button_active,
        frame_bg: colors.frame_bg,
        frame_bg_hovered: colors.frame_bg_hovered,
        frame_bg_active: colors.frame_bg_active,
        slider_grab: colors.slider_grab,
        slider_grab_active: colors.slider_grab_active,
        header: colors.header,
        header_hovered: colors.header_hovered,
        checkmark: colors.checkmark,
    }
}

/// Convert CustomThemeShapes to serializable format
fn custom_shapes_to_serde(shapes: &CustomThemeShapes) -> crate::ui::settings::CustomThemeShapesSerde {
    crate::ui::settings::CustomThemeShapesSerde {
        window_rounding: shapes.window_rounding,
        window_border_size: shapes.window_border_size,
        window_padding: shapes.window_padding,
        frame_rounding: shapes.frame_rounding,
        frame_border_size: shapes.frame_border_size,
        frame_padding: shapes.frame_padding,
        grab_rounding: shapes.grab_rounding,
        grab_min_size: shapes.grab_min_size,
        scrollbar_rounding: shapes.scrollbar_rounding,
        scrollbar_size: shapes.scrollbar_size,
        tab_rounding: shapes.tab_rounding,
        item_spacing: shapes.item_spacing,
        item_inner_spacing: shapes.item_inner_spacing,
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
