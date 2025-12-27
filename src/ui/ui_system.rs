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
    global_ui_state: Res<GlobalUiState>,
) {
    for mut egui_context in contexts.iter_mut() {
        let ctx = egui_context.get_mut();

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
                    show_windows_menu(ui, &mut dock_resource, &global_ui_state);
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
                });
        } else {
            // When hidden, set viewport to entire available screen area
            viewport_rect.rect = Some(ctx.available_rect());
        }
    }
}

/// TabViewer implementation for egui_dock
struct TabViewer<'a> {
    viewport_rect: &'a mut ViewportRect,
    current_genome: &'a mut CurrentGenome,
    genome_editor_state: &'a mut GenomeEditorState,
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
            // BioSpheres-Q windows - placeholders for now, will be implemented later
            Panel::CellInspector => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.separator();
                        ui.label("Cell Inspector");
                        ui.label("Click on a cell to inspect it");
                        ui.label("(Implementation coming soon)");
                    });
            }
            Panel::GenomeEditor => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.separator();
                        ui.label("Genome Editor");
                        ui.label("Genome editing interface");
                        ui.label("(Implementation coming soon)");
                    });
            }
            Panel::SceneManager => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.separator();
                        ui.label("Scene Manager");
                        ui.label("Scene management controls");
                        ui.label("(Implementation coming soon)");
                    });
            }
            Panel::RenderingControls => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.separator();
                        ui.label("Rendering Controls");
                        ui.label("Graphics settings");
                        ui.label("(Implementation coming soon)");
                    });
            }
            Panel::TimeScrubber => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.separator();
                        ui.label("Time Scrubber");
                        ui.label("Timeline control");
                        ui.label("(Implementation coming soon)");
                    });
            }
            Panel::CameraSettings => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.separator();
                        ui.label("Camera Settings");
                        ui.label("Camera configuration");
                        ui.label("(Implementation coming soon)");
                    });
            }
            Panel::LightingSettings => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.separator();
                        ui.label("Lighting Settings");
                        ui.label("Lighting configuration");
                        ui.label("(Implementation coming soon)");
                    });
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
                crate::ui::genome_editor::render_time_slider(ui, self.genome_editor_state);
            }
            // Legacy panel names from reference - placeholders
            Panel::Inspector | Panel::Console | Panel::Hierarchy | Panel::Assets |
            Panel::PerformanceMonitor | Panel::ThemeEditor => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.separator();
                        ui.label(format!("{}", tab));
                        ui.label("Legacy panel - not used");
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
