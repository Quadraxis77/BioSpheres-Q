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

/// Resource to track last applied UI scale
#[derive(Resource, Default)]
pub struct LastAppliedScale {
    scale: Option<f32>,
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
            ctx.style_mut(|style| {
                let scale_ratio = global_ui_state.ui_scale / last_scale.scale.unwrap_or(1.0);
                
                // Scale spacing values
                style.spacing.item_spacing *= scale_ratio;
                style.spacing.button_padding *= scale_ratio;
                style.spacing.indent *= scale_ratio;
                style.spacing.interact_size *= scale_ratio;
                // Don't scale these - they should remain responsive to panel width
                // style.spacing.slider_width *= scale_ratio;
                // style.spacing.combo_width *= scale_ratio;
                // style.spacing.text_edit_width *= scale_ratio;
                
                // Scale text sizes
                for (_text_style, font_id) in style.text_styles.iter_mut() {
                    font_id.size *= scale_ratio;
                }
            });
            last_scale.scale = Some(global_ui_state.ui_scale);
        }

        // Configure scroll style to use solid scrollbars that don't overlap content
        ctx.style_mut(|style| {
            style.spacing.scroll = egui::style::ScrollStyle::solid();
            style.spacing.scroll.bar_outer_margin = 0.0;
            style.spacing.scroll.bar_inner_margin = 0.0;
            style.spacing.scroll.floating_allocated_width = 0.0;
        });

        // Clear viewport rect at the start of each frame
        viewport_rect.rect = None;

        // Show menu bar at the top
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Windows", |ui| {
                    show_windows_menu(ui, &mut dock_resource, &mut global_ui_state);
                });
            });
        });

        // Show dock area in remaining space (only if not hidden)
        if !dock_resource.all_hidden {
            let mut style = Style::from_egui(ctx.style().as_ref());
            // Reduce separator minimum constraint to allow smaller panels
            style.separator.extra = 75.0;

            DockArea::new(&mut dock_resource.tree)
                .style(style)
                .show_leaf_collapse_buttons(false)
                .show_leaf_close_all_buttons(false)
                .show(ctx, &mut TabViewer {
                    viewport_rect: &mut viewport_rect,
                    current_genome: &mut current_genome,
                    genome_editor_state: &mut genome_editor_state,
                    sim_state: &sim_state,
                    scene_mode_request: &mut scene_mode_request,
                });
        } else {
            // When hidden, set viewport to entire available screen area
            viewport_rect.rect = Some(ctx.available_rect());
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
        
        ui_capture.want_capture_mouse = !is_over_viewport && (ctx.wants_pointer_input() || ctx.is_pointer_over_area());
        ui_capture.want_capture_keyboard = ctx.wants_keyboard_input();
    }
}

/// TabViewer implementation for egui_dock
struct TabViewer<'a> {
    viewport_rect: &'a mut ViewportRect,
    current_genome: &'a mut CurrentGenome,
    genome_editor_state: &'a mut GenomeEditorState,
    sim_state: &'a crate::simulation::SimulationState,
    scene_mode_request: &'a mut crate::ui::windows::scene_manager::SceneModeRequest,
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
        // Placeholder panels and Viewport should not be closeable
        !matches!(tab, Panel::LeftPanel | Panel::RightPanel | Panel::BottomPanel | Panel::Viewport)
    }

    fn is_placeholder(&self, tab: &Self::Tab) -> bool {
        // Mark structural panels as placeholders (except Viewport which gets special treatment)
        matches!(tab, Panel::LeftPanel | Panel::RightPanel | Panel::BottomPanel)
    }

    fn is_viewport(&self, tab: &Self::Tab) -> bool {
        // Mark Viewport as viewport for special rendering
        matches!(tab, Panel::Viewport)
    }
}
