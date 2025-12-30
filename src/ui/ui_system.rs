use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use egui_dock::{DockArea, Style};

use crate::ui::dock::*;
use crate::ui::GlobalUiState;
use crate::genome::CurrentGenome;

/// Resource tracking viewport rect for mouse filtering
#[derive(Resource, Default)]
pub struct ViewportRect {
    pub rect: Option<egui::Rect>,
}

/// Resource to track last applied UI scale and original style values
#[derive(Resource, Default)]
pub struct LastAppliedScale {
    scale: Option<f32>,
    original_spacing: Option<egui::style::Spacing>,
    original_text_styles: Option<std::collections::BTreeMap<egui::TextStyle, egui::FontId>>,
}

/// UI state for genome editor widgets
#[derive(Resource)]
pub struct GenomeEditorState {
    // UI state for modes panel
    pub renaming_mode: Option<usize>,
    pub rename_buffer: String,
    pub copy_into_dialog_open: bool,
    pub copy_into_source: usize,
    pub color_picker_state: Option<(usize, egui::ecolor::Hsva)>,
    // UI state for quaternion balls
    pub qball_snapping: bool,
    pub qball1_locked_axis: i32,
    pub qball1_initial_distance: f32,
    pub qball2_locked_axis: i32,
    pub qball2_initial_distance: f32,
    // UI state for circular sliders
    pub enable_snapping: bool,
    // Time slider
    pub time_value: f32,
    pub max_preview_duration: f32,
    pub time_slider_dragging: bool,
}

impl Default for GenomeEditorState {
    fn default() -> Self {
        Self {
            renaming_mode: None,
            rename_buffer: String::new(),
            copy_into_dialog_open: false,
            copy_into_source: 0,
            color_picker_state: None,
            qball_snapping: true,
            qball1_locked_axis: -1,
            qball1_initial_distance: 0.0,
            qball2_locked_axis: -1,
            qball2_initial_distance: 0.0,
            enable_snapping: true,
            time_value: 0.0,
            max_preview_duration: 60.0,
            time_slider_dragging: false,
        }
    }
}

/// Main UI system - renders all UI panels using egui_dock
pub fn ui_system(
    mut contexts: Query<&mut EguiContext>,
    mut dock_resource: ResMut<DockResource>,
    mut viewport_rect: ResMut<ViewportRect>,
    mut current_genome: ResMut<CurrentGenome>,
    mut genome_editor_state: ResMut<GenomeEditorState>,
    mut global_ui_state: ResMut<GlobalUiState>,
    mut ui_capture: ResMut<crate::ui::camera::UiWantCapture>,
    mut last_scale: Local<LastAppliedScale>,
    sim_state: Res<crate::simulation::SimulationState>,
    mut scene_mode_request: ResMut<crate::ui::windows::scene_manager::SceneModeRequest>,
) {
    for mut egui_context in contexts.iter_mut() {
        let ctx = egui_context.get_mut();

        // Apply UI scale only when it changes
        let scale_changed = last_scale.scale.map_or(true, |last| (last - global_ui_state.ui_scale).abs() > 0.001);
        if scale_changed {
            // Scale the entire UI by modifying the style
            ctx.global_style_mut(|style| {
                // Store original values on first run
                if last_scale.original_spacing.is_none() {
                    last_scale.original_spacing = Some(style.spacing.clone());
                    last_scale.original_text_styles = Some(style.text_styles.clone());
                }
                
                // Apply scale from original values (not multiplicatively)
                if let Some(ref original_spacing) = last_scale.original_spacing {
                    style.spacing.item_spacing = original_spacing.item_spacing * global_ui_state.ui_scale;
                    style.spacing.button_padding = original_spacing.button_padding * global_ui_state.ui_scale;
                    style.spacing.indent = original_spacing.indent * global_ui_state.ui_scale;
                    style.spacing.interact_size = original_spacing.interact_size * global_ui_state.ui_scale;
                }
                
                // Scale text sizes from original values
                if let Some(ref original_text_styles) = last_scale.original_text_styles {
                    for (text_style, font_id) in style.text_styles.iter_mut() {
                        if let Some(original_font) = original_text_styles.get(text_style) {
                            font_id.size = original_font.size * global_ui_state.ui_scale;
                        }
                    }
                }
            });
            last_scale.scale = Some(global_ui_state.ui_scale);
        }

        // Configure scroll style to use solid scrollbars that don't overlap content
        ctx.global_style_mut(|style| {
            style.spacing.scroll = egui::style::ScrollStyle::solid();
            style.spacing.scroll.bar_outer_margin = 0.0;
            style.spacing.scroll.bar_inner_margin = 0.0;
            style.spacing.scroll.floating_allocated_width = 0.0;
        });

        // Clear viewport rect at the start of each frame
        viewport_rect.rect = None;

        // Show menu bar at the top
        #[allow(deprecated)]
        egui::Panel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // Use MenuButton with IgnoreClicks behavior for Windows menu
                // so it stays open when clicking items inside
                use bevy_egui::egui::PopupCloseBehavior;
                use bevy_egui::egui::containers::menu::{MenuButton, MenuConfig};
                
                let config = MenuConfig::new()
                    .close_behavior(PopupCloseBehavior::IgnoreClicks);
                
                MenuButton::new("Windows")
                    .config(config)
                    .ui(ui, |ui| {
                        show_windows_menu(ui, &mut dock_resource, &mut global_ui_state);
                    });
            });
        });

        // Show dock area in remaining space (only if not hidden)
        if !dock_resource.all_hidden {
            let mut style = Style::from_egui(ctx.global_style().as_ref());
            // Reduce separator minimum constraint to allow smaller panels
            style.separator.extra = 75.0;

            // Apply lock settings to hide tab bar height if locked
            if global_ui_state.lock_tab_bar {
                style.tab_bar.height = 0.0;
            }

            let mut dock_area = DockArea::new(&mut dock_resource.tree)
                .style(style)
                .show_leaf_collapse_buttons(false)
                .show_leaf_close_all_buttons(false)
                .draggable_tabs(true)  // Explicitly enable tab dragging for docking
                .window_bounds(ctx.content_rect());  // Set window bounds for floating windows

            // Apply lock settings for tabs and close buttons
            if global_ui_state.lock_tabs {
                dock_area = dock_area
                    .show_tab_name_on_hover(false)
                    .draggable_tabs(false);  // Disable dragging when tabs are locked
            }

            if global_ui_state.lock_close_buttons {
                dock_area = dock_area.show_close_buttons(false);
            }

            dock_area.show(ctx, &mut TabViewer {
                viewport_rect: &mut viewport_rect,
                current_genome: &mut current_genome,
                genome_editor_state: &mut genome_editor_state,
                sim_state: &sim_state,
                scene_mode_request: &mut scene_mode_request,
                global_ui_state: &global_ui_state,
            });
        } else {
            // When hidden, set viewport to entire available screen area
            viewport_rect.rect = Some(ctx.content_rect());
        }

        // Update mouse capture state AFTER UI is rendered
        // Egui wants mouse if pointer is over any UI or if any widget is being interacted with
        // BUT exclude the viewport area - camera should work there
        let pointer_pos = ctx.pointer_hover_pos();
        let is_over_viewport = if let (Some(pos), Some(viewport)) = (pointer_pos, viewport_rect.rect) {
            viewport.contains(pos)
        } else {
            false
        };
        
        ui_capture.want_capture_mouse = !is_over_viewport && (ctx.egui_wants_pointer_input() || ctx.is_pointer_over_egui());
        ui_capture.want_capture_keyboard = ctx.egui_wants_keyboard_input();
    }
}

/// TabViewer implementation for egui_dock
struct TabViewer<'a> {
    viewport_rect: &'a mut ViewportRect,
    current_genome: &'a mut CurrentGenome,
    genome_editor_state: &'a mut GenomeEditorState,
    sim_state: &'a crate::simulation::SimulationState,
    scene_mode_request: &'a mut crate::ui::windows::scene_manager::SceneModeRequest,
    global_ui_state: &'a GlobalUiState,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = Panel;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.to_string().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Panel::Viewport => {
                // Capture the viewport rect for mouse interaction
                let rect = ui.available_rect_before_wrap();
                self.viewport_rect.rect = Some(rect);

                // Don't draw anything else - let the 3D scene show through
            }
            // Placeholder panels are empty - they just hold space for other tabs
            Panel::LeftPanel | Panel::RightPanel | Panel::BottomPanel => {
                // No content - empty placeholder
            }
            // Genome editor panels - using actual implementations
            Panel::Modes => {
                crate::ui::genome_editor::render_modes_panel(ui, self.current_genome, self.genome_editor_state);
            }
            Panel::NameTypeEditor => {
                crate::ui::genome_editor::render_name_type_editor(ui, self.current_genome);
            }
            Panel::AdhesionSettings => {
                crate::ui::genome_editor::render_adhesion_settings(ui, self.current_genome);
            }
            Panel::ParentSettings => {
                crate::ui::genome_editor::render_parent_settings(ui, self.current_genome);
            }
            Panel::CircleSliders => {
                crate::ui::genome_editor::render_circle_sliders(ui, self.current_genome, self.genome_editor_state);
            }
            Panel::QuaternionBall => {
                crate::ui::genome_editor::render_quaternion_ball(ui, self.current_genome, self.genome_editor_state);
            }
            Panel::TimeSlider => {
                crate::ui::genome_editor::render_time_slider(ui, self.genome_editor_state, self.sim_state);
            }
            Panel::SceneManager => {
                crate::ui::windows::render_scene_manager(ui, self.sim_state.mode, self.scene_mode_request);
            }
            // Unused stub panels - show placeholder message
            _ => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.separator();
                        ui.label(format!("{}", tab));
                        ui.label("This panel is not yet implemented.");
                    });
            }
        }
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        // Only the Viewport panel should be transparent
        !matches!(tab, Panel::Viewport)
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        // Check if this specific window is locked
        let panel_name = tab.to_string();
        
        // Get the appropriate locked windows set based on current scene
        let locked_windows = match self.sim_state.mode {
            crate::simulation::SimulationMode::Preview => &self.global_ui_state.locked_windows_preview,
            crate::simulation::SimulationMode::Cpu => &self.global_ui_state.locked_windows_cpu,
            _ => &self.global_ui_state.locked_windows_preview, // fallback
        };
        
        let is_locked = locked_windows.contains(&panel_name);
        
        // If window is locked, it's not closeable
        // If global lock_close_buttons is enabled, nothing is closeable
        !is_locked && !self.global_ui_state.lock_close_buttons
    }

    fn is_placeholder(&self, _tab: &Self::Tab) -> bool {
        // No panels are placeholders anymore - all can be toggled
        false
    }

    fn is_viewport(&self, tab: &Self::Tab) -> bool {
        // Mark Viewport as viewport for special rendering
        matches!(tab, Panel::Viewport)
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        // Allow all tabs to be moved to floating windows
        true
    }

    fn hide_tab_button(&self, tab: &Self::Tab) -> bool {
        // Check if this specific window is locked
        let panel_name = tab.to_string();
        
        // Get the appropriate locked windows set based on current scene
        let locked_windows = match self.sim_state.mode {
            crate::simulation::SimulationMode::Preview => &self.global_ui_state.locked_windows_preview,
            crate::simulation::SimulationMode::Cpu => &self.global_ui_state.locked_windows_cpu,
            _ => &self.global_ui_state.locked_windows_preview, // fallback
        };
        
        let is_locked = locked_windows.contains(&panel_name);
        
        // Hide tab button if window is locked OR if global lock_tabs is enabled
        is_locked || self.global_ui_state.lock_tabs
    }

    fn is_draggable(&self, tab: &Self::Tab) -> bool {
        // Allow all tabs to be draggable, including Viewport
        // Check if this specific window is locked
        let panel_name = tab.to_string();
        
        // Get the appropriate locked windows set based on current scene
        let locked_windows = match self.sim_state.mode {
            crate::simulation::SimulationMode::Preview => &self.global_ui_state.locked_windows_preview,
            crate::simulation::SimulationMode::Cpu => &self.global_ui_state.locked_windows_cpu,
            _ => &self.global_ui_state.locked_windows_preview, // fallback
        };
        
        let is_locked = locked_windows.contains(&panel_name);
        
        // Not draggable if locked or if global lock_tabs is enabled
        !is_locked && !self.global_ui_state.lock_tabs
    }
}
