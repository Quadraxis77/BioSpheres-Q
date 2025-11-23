use bevy::prelude::*;
use bevy_mod_imgui::prelude::*;
use imgui::InputTextFlags;
use imnodes::{Context, EditorContext, editor, PinShape, InputPinId, OutputPinId, LinkId};
use crate::genome::*;
use crate::simulation::{SimulationState, SimulationMode};
use super::imgui_widgets;
use super::camera::ImGuiWantCapture;
use super::imnodes_extensions;
use std::cell::RefCell;
use std::collections::HashMap;

/// Resource to track genome graph window state
#[derive(Resource)]
pub struct GenomeGraphState {
    pub show_window: bool,
    pub is_panning: bool,
    pub last_mouse_pos: Option<[f32; 2]>,
    pub panning_offset: [f32; 2],
    pub dragging_from_pin: Option<i32>, // Track which output pin is being dragged from
}

impl Default for GenomeGraphState {
    fn default() -> Self {
        Self {
            show_window: false,
            is_panning: false,
            last_mouse_pos: None,
            panning_offset: [0.0, 0.0],
            dragging_from_pin: None,
        }
    }
}

/// Genome editor plugin - modular UI component for editing genome data
pub struct GenomeEditorPlugin;

impl Plugin for GenomeEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GenomeGraphState>()
            .init_resource::<PreviousGenomeState>()
            .add_systems(Update, (
                update_imgui_capture_state,
                detect_genome_changes,
                render_genome_editor,
                render_genome_graph,
            ).chain());
    }
}

/// Resource to track previous genome state for change detection
#[derive(Resource, Default)]
struct PreviousGenomeState {
    mode_count: usize,
    mode_connections: Vec<(usize, i32, i32)>, // (mode_idx, child_a, child_b)
}

/// System to update ImGui input capture state - runs before other UI systems
fn update_imgui_capture_state(
    mut imgui_context: NonSendMut<ImguiContext>,
    mut imgui_capture: ResMut<ImGuiWantCapture>,
) {
    let ui = imgui_context.ui();
    imgui_capture.want_capture_mouse = ui.io().want_capture_mouse;
    imgui_capture.want_capture_keyboard = ui.io().want_capture_keyboard;
}

/// System to detect genome changes and trigger node graph rebuild
fn detect_genome_changes(
    current_genome: Res<CurrentGenome>,
    mut previous_state: ResMut<PreviousGenomeState>,
    mut node_graph: ResMut<GenomeNodeGraph>,
) {
    let current_mode_count = current_genome.genome.modes.len();
    let current_connections: Vec<(usize, i32, i32)> = current_genome.genome.modes.iter()
        .enumerate()
        .map(|(idx, mode)| (idx, mode.child_a.mode_number, mode.child_b.mode_number))
        .collect();

    // Check if mode count changed or connections changed
    let needs_rebuild = current_mode_count != previous_state.mode_count
        || current_connections != previous_state.mode_connections;

    if needs_rebuild {
        node_graph.mark_for_rebuild();
        previous_state.mode_count = current_mode_count;
        previous_state.mode_connections = current_connections;
    }
}

/// Main genome editor rendering system
fn render_genome_editor(
    mut current_genome: ResMut<CurrentGenome>,
    mut imgui_context: NonSendMut<ImguiContext>,
    mut simulation_state: ResMut<SimulationState>,
    preview_state: Res<crate::simulation::preview_sim::PreviewSimState>,
    mut graph_state: ResMut<GenomeGraphState>,
    mut node_graph: ResMut<GenomeNodeGraph>,
    global_ui_state: Res<super::GlobalUiState>,
) {
    // Only show genome editor in Preview mode
    if simulation_state.mode != SimulationMode::Preview {
        return;
    }

    // Track if genome was modified during this frame
    let genome_before_edit = current_genome.genome.clone();

    let ui = imgui_context.ui();

    // Build flags based on lock state
    use imgui::WindowFlags;
    let flags = if global_ui_state.windows_locked {
        WindowFlags::NO_MOVE | WindowFlags::NO_RESIZE
    } else {
        WindowFlags::empty()
    };

    ui.window("Genome Editor")
        .position([0.0, 28.0], Condition::FirstUseEver)
        .size([888.0, 683.0], Condition::FirstUseEver)
        .size_constraints([800.0, 500.0], [f32::MAX, f32::MAX])
        .flags(flags)
        .build(|| {
            // Genome name input
            ui.text("Genome Name:");
            ui.same_line();
            let mut genome_name = current_genome.genome.name.clone();
            ui.set_next_item_width(200.0);
            if ui.input_text("##GenomeName", &mut genome_name).build() {
                current_genome.genome.name = genome_name;
            }

            ui.same_line();
            if ui.button("Save Genome") {
                // TODO: Implement genome saving functionality
            }

            ui.same_line();
            if ui.button("Load Genome") {
                // TODO: Implement genome loading functionality
            }

            ui.same_line();
            if ui.button("Genome Graph") {
                graph_state.show_window = !graph_state.show_window;
            }

            ui.separator();

            // Initial mode dropdown
            ui.text("Initial Mode:");
            ui.same_line();
            // Build display strings that show both index and name for clarity
            let mode_display_names: Vec<String> = current_genome.genome.modes.iter()
                .enumerate()
                .map(|(idx, m)| format!("[{}] {}", idx, m.name))
                .collect();

            let initial_mode = current_genome.genome.initial_mode as usize;
            // Clamp to valid range
            let clamped_initial_mode = initial_mode.min(current_genome.genome.modes.len().saturating_sub(1));
            let current_mode_display = mode_display_names.get(clamped_initial_mode)
                .map(|s| s.as_str())
                .unwrap_or("None");
            if let Some(_token) = ui.begin_combo("##InitialMode", current_mode_display) {
                for (i, display_name) in mode_display_names.iter().enumerate() {
                    let is_selected = i == clamped_initial_mode;
                    if ui.selectable_config(display_name).selected(is_selected).build() {
                        current_genome.genome.initial_mode = i as i32;
                    }
                }
            }

            ui.separator();

            // Mode management
            ui.text("Modes:");
            ui.same_line();
            if ui.button("Add Mode") {
                let selected_idx = current_genome.selected_mode_index as usize;
                let insert_idx = if selected_idx < current_genome.genome.modes.len() {
                    selected_idx + 1
                } else {
                    current_genome.genome.modes.len()
                };
                
                // Generate new mode name based on selected mode's default name
                let new_name = if selected_idx < current_genome.genome.modes.len() {
                    generate_next_mode_name(&current_genome.genome.modes[selected_idx].default_name, &current_genome.genome.modes)
                } else {
                    format!("Mode {}", current_genome.genome.modes.len())
                };
                
                let new_mode = ModeSettings::new_self_splitting(
                    insert_idx as i32,
                    new_name,
                );
                
                current_genome.genome.modes.insert(insert_idx, new_mode);
                
                // Update mode numbers for all modes after insertion point
                update_mode_numbers_after_insert(&mut current_genome.genome, insert_idx);
                
                // Select the newly created mode
                current_genome.selected_mode_index = insert_idx as i32;
                
                // Calculate grid position for the new node
                const NODE_SPACING_X: f32 = 250.0;
                const NODE_SPACING_Y: f32 = 200.0;
                const START_X: f32 = 50.0;
                const START_Y: f32 = 50.0;
                const COLUMNS: usize = 4;
                
                let col = insert_idx % COLUMNS;
                let row = insert_idx / COLUMNS;
                let x = START_X + (col as f32 * NODE_SPACING_X);
                let y = START_Y + (row as f32 * NODE_SPACING_Y);
                
                // Set pending position for the new node
                node_graph.pending_position = Some((insert_idx, x, y));
                
                // Mark node graph for rebuild
                node_graph.mark_for_rebuild();
            }

            ui.same_line();
            if ui.button("Remove Mode") && current_genome.genome.modes.len() > 1 {
                let selected = current_genome.selected_mode_index as usize;
                let initial_mode = current_genome.genome.initial_mode as usize;
                
                // Don't allow removing the initial mode
                if selected < current_genome.genome.modes.len() && selected != initial_mode {
                    current_genome.genome.modes.remove(selected);
                    if current_genome.selected_mode_index >= current_genome.genome.modes.len() as i32 {
                        current_genome.selected_mode_index = (current_genome.genome.modes.len() as i32) - 1;
                    }
                    // Mark node graph for rebuild
                    node_graph.mark_for_rebuild();
                }
            }
            
            // Show tooltip if trying to remove initial mode
            if ui.is_item_hovered() {
                let selected = current_genome.selected_mode_index as usize;
                let initial_mode = current_genome.genome.initial_mode as usize;
                if selected == initial_mode {
                    ui.tooltip_text("Cannot remove the initial mode");
                }
            }

            // Mode list (left panel) - extract data first to avoid borrow issues
            let modes_data: Vec<(String, Vec3)> = current_genome.genome.modes.iter()
                .map(|m| (m.name.clone(), m.color))
                .collect();
            let mut new_selected_index = current_genome.selected_mode_index;

            ui.child_window("ModeList")
                .size([200.0, 0.0])
                .border(true)
                .build(|| {
                    for (i, (name, color)) in modes_data.iter().enumerate() {
                        let is_selected = i == new_selected_index as usize;

                        // Color the mode button with mode's color
                        let button_color = if is_selected {
                            [color.x, color.y, color.z, 1.0]
                        } else {
                            [color.x * 0.8, color.y * 0.8, color.z * 0.8, 1.0]
                        };

                        let _button_style = ui.push_style_color(StyleColor::Button, button_color);
                        let _button_hovered_style = ui.push_style_color(
                            StyleColor::ButtonHovered,
                            [color.x * 0.9, color.y * 0.9, color.z * 0.9, 1.0]
                        );
                        let _button_active_style = ui.push_style_color(
                            StyleColor::ButtonActive,
                            [color.x, color.y, color.z, 1.0]
                        );

                        // Determine text color based on brightness
                        let brightness = color.x * 0.299 + color.y * 0.587 + color.z * 0.114;
                        let text_color = if brightness > 0.5 {
                            [0.0, 0.0, 0.0, 1.0]
                        } else {
                            [1.0, 1.0, 1.0, 1.0]
                        };
                        let _text_style = ui.push_style_color(StyleColor::Text, text_color);

                        if ui.button_with_size(name, [-1.0, 0.0]) {
                            new_selected_index = i as i32;
                        }

                        // Draw dashed black and white outline for selected mode
                        if is_selected {
                            let draw_list = ui.get_window_draw_list();
                            let min = ui.item_rect_min();
                            let max = ui.item_rect_max();

                            let dash_length = 6.0;
                            let black_color = 0xFF000000u32;
                            let white_color = 0xFFFFFFFFu32;

                            // Draw top edge
                            let mut x = min[0];
                            while x < max[0] {
                                let end_x = (x + dash_length).min(max[0]);
                                draw_list
                                    .add_line([x, min[1]], [end_x, min[1]], black_color)
                                    .thickness(2.0)
                                    .build();
                                x += dash_length;
                                if x < max[0] {
                                    let end_x = (x + dash_length).min(max[0]);
                                    draw_list
                                        .add_line([x, min[1]], [end_x, min[1]], white_color)
                                        .thickness(2.0)
                                        .build();
                                    x += dash_length;
                                }
                            }

                            // Draw bottom edge
                            let mut x = min[0];
                            while x < max[0] {
                                let end_x = (x + dash_length).min(max[0]);
                                draw_list
                                    .add_line([x, max[1]], [end_x, max[1]], black_color)
                                    .thickness(2.0)
                                    .build();
                                x += dash_length;
                                if x < max[0] {
                                    let end_x = (x + dash_length).min(max[0]);
                                    draw_list
                                        .add_line([x, max[1]], [end_x, max[1]], white_color)
                                        .thickness(2.0)
                                        .build();
                                    x += dash_length;
                                }
                            }

                            // Draw left edge
                            let mut y = min[1];
                            while y < max[1] {
                                let end_y = (y + dash_length).min(max[1]);
                                draw_list
                                    .add_line([min[0], y], [min[0], end_y], black_color)
                                    .thickness(2.0)
                                    .build();
                                y += dash_length;
                                if y < max[1] {
                                    let end_y = (y + dash_length).min(max[1]);
                                    draw_list
                                        .add_line([min[0], y], [min[0], end_y], white_color)
                                        .thickness(2.0)
                                        .build();
                                    y += dash_length;
                                }
                            }

                            // Draw right edge
                            let mut y = min[1];
                            while y < max[1] {
                                let end_y = (y + dash_length).min(max[1]);
                                draw_list
                                    .add_line([max[0], y], [max[0], end_y], black_color)
                                    .thickness(2.0)
                                    .build();
                                y += dash_length;
                                if y < max[1] {
                                    let end_y = (y + dash_length).min(max[1]);
                                    draw_list
                                        .add_line([max[0], y], [max[0], end_y], white_color)
                                        .thickness(2.0)
                                        .build();
                                    y += dash_length;
                                }
                            }
                        }
                    }
                });

            // Update the selection if it changed
            current_genome.selected_mode_index = new_selected_index;

            ui.same_line();

            // Mode settings panel (right panel)
            let selected_idx = current_genome.selected_mode_index as usize;
            let all_modes_count = current_genome.genome.modes.len();

            if selected_idx < all_modes_count {
                // Clone the modes list for reference
                let modes_for_ref: Vec<ModeSettings> = current_genome.genome.modes.clone();

                if let Some(selected_mode) = current_genome.genome.modes.get_mut(selected_idx) {
                    ui.child_window("ModeSettings")
                        .size([0.0, 0.0])
                        .build(|| {
                            draw_mode_settings(ui, selected_mode, &modes_for_ref, selected_idx);
                        });
                }
            }
        });

    // Check if genome was modified and trigger instant resimulation in Preview mode
    // Note: CPU mode automatically picks up genome changes on the next physics tick
    if current_genome.genome != genome_before_edit {
        if simulation_state.mode == SimulationMode::Preview {
            // Trigger resimulation to current preview time to apply genome changes
            simulation_state.target_time = Some(preview_state.current_time);
        }
    }
}

/// Helper function to draw a slider with a text input for precise value entry
fn slider_with_input_f32(ui: &Ui, label: &str, value: &mut f32, min: f32, max: f32, width: f32) -> bool {
    let mut changed = false;

    // Draw slider
    ui.set_next_item_width(width - 80.0);
    if ui.slider(label, min, max, value) {
        changed = true;
    }

    // Draw text input on same line
    ui.same_line();
    ui.set_next_item_width(70.0);
    let input_label = format!("##input{}", label);

    let mut text_buffer = format!("{:.2}", value);
    if ui.input_text(&input_label, &mut text_buffer)
        .flags(InputTextFlags::CHARS_DECIMAL | InputTextFlags::AUTO_SELECT_ALL | InputTextFlags::ENTER_RETURNS_TRUE)
        .build()
    {
        if let Ok(new_value) = text_buffer.parse::<f32>() {
            *value = new_value.clamp(min, max);
            changed = true;
        }
    }

    changed
}

/// Helper function to draw a slider with a text input for precise value entry (i32 version)
fn slider_with_input_i32(ui: &Ui, label: &str, value: &mut i32, min: i32, max: i32, width: f32) -> bool {
    let mut changed = false;

    // Draw slider
    ui.set_next_item_width(width - 80.0);
    if ui.slider(label, min, max, value) {
        changed = true;
    }

    // Draw text input on same line
    ui.same_line();
    ui.set_next_item_width(70.0);
    let input_label = format!("##input{}", label);

    let mut text_buffer = format!("{}", value);
    if ui.input_text(&input_label, &mut text_buffer)
        .flags(InputTextFlags::CHARS_DECIMAL | InputTextFlags::AUTO_SELECT_ALL | InputTextFlags::ENTER_RETURNS_TRUE)
        .build()
    {
        if let Ok(new_value) = text_buffer.parse::<i32>() {
            *value = new_value.clamp(min, max);
            changed = true;
        }
    }

    changed
}

/// Draw mode settings (tabbed interface)
fn draw_mode_settings(ui: &Ui, mode: &mut ModeSettings, all_modes: &[ModeSettings], mode_index: usize) {
    if let Some(_tab_bar) = ui.tab_bar("ModeSettingsTabs") {
        // Parent Settings Tab
        if let Some(_tab) = ui.tab_item("Parent Settings") {
            draw_parent_settings(ui, mode, mode_index);
        }

        // Child A Settings Tab
        if let Some(_tab) = ui.tab_item("Child A Settings") {
            let _mode_changed = draw_child_settings(ui, "Child A", &mut mode.child_a, all_modes);

            ui.text("Child A Orientation:");
            ui.spacing();

            ui.checkbox("Enable Angle Snapping##ChildA", &mut mode.child_a.enable_angle_snapping);
            ui.spacing();

            if imgui_widgets::quaternion_ball(ui, "##ChildAOrientation", &mut mode.child_a.orientation, 80.0, mode.child_a.enable_angle_snapping) {
                // Orientation changed
            }

            ui.spacing();
            if ui.button("Reset Orientation (Child A)") {
                mode.child_a.orientation = Quat::IDENTITY;
            }

            ui.separator();
        }

        // Child B Settings Tab
        if let Some(_tab) = ui.tab_item("Child B Settings") {
            let _mode_changed = draw_child_settings(ui, "Child B", &mut mode.child_b, all_modes);

            ui.text("Child B Orientation:");
            ui.spacing();

            ui.checkbox("Enable Angle Snapping##ChildB", &mut mode.child_b.enable_angle_snapping);
            ui.spacing();

            if imgui_widgets::quaternion_ball(ui, "##ChildBOrientation", &mut mode.child_b.orientation, 80.0, mode.child_b.enable_angle_snapping) {
                // Orientation changed
            }

            ui.spacing();
            if ui.button("Reset Orientation (Child B)") {
                mode.child_b.orientation = Quat::IDENTITY;
            }

            ui.separator();
        }

        // Adhesion Settings Tab
        let adhesion_tab_enabled = mode.parent_make_adhesion;
        if !adhesion_tab_enabled {
            let _alpha = ui.push_style_var(StyleVar::Alpha(0.5));
        }

        if let Some(_tab) = ui.tab_item("Adhesion Settings") {
            if adhesion_tab_enabled {
                draw_adhesion_settings(ui, &mut mode.adhesion_settings);
            } else {
                ui.text_disabled("Enable 'Parent Make Adhesion' to configure adhesion settings");
            }
        }
    }
}

/// Draw parent settings
fn draw_parent_settings(ui: &Ui, mode: &mut ModeSettings, _mode_index: usize) {
    // Mode name
    ui.text("Mode Name:");
    let mut mode_name = mode.name.clone();
    if ui.input_text("##ModeName", &mut mode_name).build() {
        let trimmed = mode_name.trim();
        if !trimmed.is_empty() {
            // Update with the trimmed custom name
            mode.name = trimmed.to_string();
        } else {
            // If empty, revert to the stored default name
            mode.name = mode.default_name.clone();
        }
    }
    
    // Show hint about default name when field is empty
    if ui.is_item_active() && mode_name.trim().is_empty() {
        ui.text_colored([0.7, 0.7, 0.7, 1.0], &format!("Will revert to: {}", mode.default_name));
    }

    ui.spacing();

    // Cell type dropdown
    ui.text("Cell Type:");
    ui.same_line();
    let cell_types = vec!["Test"];
    let current_cell_type = cell_types.get(mode.cell_type as usize).unwrap_or(&"Unknown");
    if let Some(_token) = ui.begin_combo("##CellType", current_cell_type) {
        for (i, cell_type_name) in cell_types.iter().enumerate() {
            let is_selected = i == mode.cell_type as usize;
            if ui.selectable_config(cell_type_name).selected(is_selected).build() {
                mode.cell_type = i as i32;
            }
        }
    }

    ui.spacing();
    ui.separator();
    ui.spacing();

    // Parent make adhesion
    ui.checkbox("Parent Make Adhesion", &mut mode.parent_make_adhesion);

    // Split mass (only for non-Test cell types)
    if mode.cell_type != 0 {
        ui.text("Split Mass:");
        slider_with_input_f32(ui, "##SplitMass", &mut mode.split_mass, 0.1, 10.0, ui.content_region_avail()[0]);
    }

    // Split interval
    ui.text("Split Interval:");
    slider_with_input_f32(ui, "##SplitInterval", &mut mode.split_interval, 1.0, 30.0, ui.content_region_avail()[0]);

    ui.spacing();
    ui.separator();
    ui.spacing();

    // Parent split angle
    ui.text("Parent Split Angle:");
    ui.checkbox("Enable Angle Snapping##Parent", &mut mode.enable_parent_angle_snapping);
    ui.spacing();

    // Use columns for layout
    ui.columns(2, "ParentSplitAngle", true);
    ui.text("Pitch");
    imgui_widgets::circular_slider_float(
        ui,
        "##ParentPitch",
        &mut mode.parent_split_direction.x,
        -180.0,
        180.0,
        60.0,
        "%.2f°",
        0.0,
        0.0,
        mode.enable_parent_angle_snapping
    );

    ui.next_column();
    ui.text("Yaw");
    imgui_widgets::circular_slider_float(
        ui,
        "##ParentYaw",
        &mut mode.parent_split_direction.y,
        -180.0,
        180.0,
        60.0,
        "%.2f°",
        0.0,
        0.0,
        mode.enable_parent_angle_snapping
    );
    ui.columns(1, "", false);

    // Max adhesions
    ui.text("Max Adhesions:");
    slider_with_input_i32(ui, "##MaxAdhesions", &mut mode.max_adhesions, 0, 20, ui.content_region_avail()[0]);

    ui.spacing();
    ui.separator();
    ui.spacing();

    // Color picker
    ui.text("Mode Color:");
    let mut color = [mode.color.x, mode.color.y, mode.color.z];
    if ui.color_picker3("##ModeColor", &mut color) {
        mode.color = Vec3::new(color[0], color[1], color[2]);
    }
}

/// Draw child settings - returns true if mode number changed
fn draw_child_settings(ui: &Ui, _label: &str, child: &mut ChildSettings, all_modes: &[ModeSettings]) -> bool {
    let mut mode_changed = false;
    
    ui.text("Mode:");
    // Build display strings that show both index and name for clarity
    let mode_display_names: Vec<String> = all_modes.iter()
        .enumerate()
        .map(|(idx, m)| format!("[{}] {}", idx, m.name))
        .collect();

    let mode_index = child.mode_number as usize;
    // Clamp mode_index to valid range before using it
    let clamped_mode_index = mode_index.min(all_modes.len().saturating_sub(1));
    let current_mode_display = mode_display_names.get(clamped_mode_index)
        .map(|s| s.as_str())
        .unwrap_or("None");

    if let Some(_token) = ui.begin_combo("##Mode", current_mode_display) {
        for (i, display_name) in mode_display_names.iter().enumerate() {
            let is_selected = i == clamped_mode_index;
            if ui.selectable_config(display_name).selected(is_selected).build() {
                let old_mode = child.mode_number;
                child.mode_number = i as i32;
                mode_changed = old_mode != child.mode_number;
            }
        }
    }

    ui.spacing();
    ui.separator();
    ui.spacing();

    ui.checkbox("Keep Adhesion", &mut child.keep_adhesion);
    
    mode_changed
}

/// Draw adhesion settings
fn draw_adhesion_settings(ui: &Ui, adhesion: &mut AdhesionSettings) {
    ui.checkbox("Adhesion Can Break", &mut adhesion.can_break);

    ui.text("Adhesion Break Force:");
    slider_with_input_f32(ui, "##AdhesionBreakForce", &mut adhesion.break_force, 0.1, 100.0, ui.content_region_avail()[0]);

    ui.text("Adhesion Rest Length:");
    slider_with_input_f32(ui, "##AdhesionRestLength", &mut adhesion.rest_length, 0.5, 5.0, ui.content_region_avail()[0]);

    ui.text("Linear Spring Stiffness:");
    slider_with_input_f32(ui, "##LinearSpringStiffness", &mut adhesion.linear_spring_stiffness, 0.1, 500.0, ui.content_region_avail()[0]);

    ui.text("Linear Spring Damping:");
    slider_with_input_f32(ui, "##LinearSpringDamping", &mut adhesion.linear_spring_damping, 0.0, 10.0, ui.content_region_avail()[0]);

    ui.text("Angular Spring Stiffness:");
    slider_with_input_f32(ui, "##AngularSpringStiffness", &mut adhesion.orientation_spring_stiffness, 0.1, 100.0, ui.content_region_avail()[0]);

    ui.text("Angular Spring Damping:");
    slider_with_input_f32(ui, "##AngularSpringDamping", &mut adhesion.orientation_spring_damping, 0.0, 10.0, ui.content_region_avail()[0]);

    ui.text("Max Angular Deviation:");
    slider_with_input_f32(ui, "##MaxAngularDeviation", &mut adhesion.max_angular_deviation, 0.0, 180.0, ui.content_region_avail()[0]);

    ui.spacing();
    ui.separator();
    ui.spacing();

    ui.checkbox("Enable Twist Constraint", &mut adhesion.enable_twist_constraint);

    ui.text("Twist Constraint Stiffness:");
    slider_with_input_f32(ui, "##TwistConstraintStiffness", &mut adhesion.twist_constraint_stiffness, 0.0, 2.0, ui.content_region_avail()[0]);

    ui.text("Twist Constraint Damping:");
    slider_with_input_f32(ui, "##TwistConstraintDamping", &mut adhesion.twist_constraint_damping, 0.0, 10.0, ui.content_region_avail()[0]);
}

/// System to render genome graph window
fn render_genome_graph(
    mut graph_state: ResMut<GenomeGraphState>,
    mut imgui_context: NonSendMut<ImguiContext>,
    simulation_state: Res<SimulationState>,
    mut current_genome: ResMut<CurrentGenome>,
    mut node_graph: ResMut<GenomeNodeGraph>,
    global_ui_state: Res<super::GlobalUiState>,
) {
    // Only show in Preview mode and if window is open
    if simulation_state.mode != SimulationMode::Preview || !graph_state.show_window {
        return;
    }

    let ui = imgui_context.ui();

    // Rebuild graph if needed
    if node_graph.needs_rebuild {
        rebuild_node_graph(&current_genome.genome, &mut node_graph);
        node_graph.needs_rebuild = false;
    }

    // Calculate layout if needed
    if node_graph.needs_layout {
        node_graph.calculate_grid_layout();
    }

    let mut show_window = graph_state.show_window;
    
    // Build flags based on lock state
    use imgui::WindowFlags;
    let flags = if global_ui_state.windows_locked {
        WindowFlags::NO_MOVE | WindowFlags::NO_RESIZE | WindowFlags::NO_NAV
    } else {
        WindowFlags::NO_NAV
    };
    
    ui.window("Genome Graph")
        .opened(&mut show_window)
        .position([1048.0, 486.0], Condition::FirstUseEver)
        .size([1338.0, 806.0], Condition::FirstUseEver)
        .bg_alpha(1.0) // Make window fully opaque
        .flags(flags)
        .build(|| {
            // Show help text
            ui.text_colored([0.7, 0.7, 0.7, 1.0], "Shift+Click: Add mode | Shift+Right-click node: Remove | Right-click link: Self-ref | Middle drag: Pan");
            ui.separator();
            

            // Thread-local storage for imnodes context
            thread_local! {
                static IMNODES_CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
                static EDITOR_CONTEXT: RefCell<Option<EditorContext>> = RefCell::new(None);
            }

            // Initialize contexts if needed
            IMNODES_CONTEXT.with(|ctx| {
                if ctx.borrow().is_none() {
                    *ctx.borrow_mut() = Some(Context::new());
                }
            });
            
            // Configure IO every frame to enable panning
            unsafe {
                let io = imnodes_sys::imnodes_GetIO();
                if !io.is_null() {
                    // Enable link detachment with modifier click (Ctrl+Click to detach)
                    (*io).LinkDetachWithModifierClick.Modifier = std::ptr::null_mut();
                    
                    // Set middle mouse button for panning (standard behavior)
                    // ImGui mouse buttons: 0 = left, 1 = right, 2 = middle
                    (*io).AltMouseButton = 2; // Middle mouse button
                    
                    // Increase auto-panning speed for better UX
                    (*io).AutoPanningSpeed = 1000.0;
                }
            }
            
            EDITOR_CONTEXT.with(|editor_ctx| {
                if editor_ctx.borrow().is_none() {
                    IMNODES_CONTEXT.with(|ctx| {
                        if let Some(context) = ctx.borrow().as_ref() {
                            *editor_ctx.borrow_mut() = Some(context.create_editor());
                        }
                    });
                }
                
                // Set the editor context as active for this frame
                if let Some(editor_context) = editor_ctx.borrow().as_ref() {
                    unsafe {
                        // Get the raw pointer to the editor context
                        let editor_ptr = editor_context as *const EditorContext as *mut imnodes_sys::ImNodesEditorContext;
                        imnodes_sys::imnodes_EditorContextSet(editor_ptr);
                        
                        // Make grid lines very faint
                        let style = imnodes_sys::imnodes_GetStyle();
                        if !style.is_null() {
                            // Set grid line color to very faint (low alpha)
                            (*style).Colors[imnodes_sys::ImNodesCol__ImNodesCol_GridLine as usize] = 0x10FFFFFF; // Very faint white
                        }
                    }
                }
            });

            // Manual panning implementation with right mouse button as alternative
            // This provides an additional way to pan besides middle mouse button
            let is_right_mouse_down = ui.is_mouse_down(imgui::MouseButton::Right);
            let mouse_pos = ui.io().mouse_pos;
            
            // Calculate panning delta
            let mut panning_delta: Option<(f32, f32)> = None;
            
            if is_right_mouse_down {
                if !graph_state.is_panning {
                    // Start panning - no need to check window hover, just start tracking
                    graph_state.is_panning = true;
                    graph_state.last_mouse_pos = Some(mouse_pos);
                } else if let Some(last_pos) = graph_state.last_mouse_pos {
                    // Continue panning - calculate delta
                    let delta_x = mouse_pos[0] - last_pos[0];
                    let delta_y = mouse_pos[1] - last_pos[1];
                    
                    if delta_x.abs() > 0.01 || delta_y.abs() > 0.01 {
                        panning_delta = Some((delta_x, delta_y));
                        graph_state.panning_offset[0] += delta_x;
                        graph_state.panning_offset[1] += delta_y;
                    }
                    
                    graph_state.last_mouse_pos = Some(mouse_pos);
                }
            } else {
                // Stop panning when mouse button is released
                graph_state.is_panning = false;
                graph_state.last_mouse_pos = None;
            }
            
            // Apply panning by moving all node positions
            if let Some((delta_x, delta_y)) = panning_delta {
                for mode_idx in 0..current_genome.genome.modes.len() {
                    if let Some(node_id) = node_graph.get_node_for_mode(mode_idx) {
                        if let Some((x, y)) = node_graph.get_node_position(node_id) {
                            node_graph.set_node_position(node_id, x + delta_x, y + delta_y);
                        }
                    }
                }
            }

            EDITOR_CONTEXT.with(|editor_ctx| {
                if let Some(editor_context) = editor_ctx.borrow_mut().as_mut() {
                    // Collect node IDs before entering editor scope
                    let node_ids: Vec<(usize, i32)> = current_genome
                        .genome
                        .modes
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, _)| node_graph.get_node_for_mode(idx).map(|id| (idx, id)))
                        .collect();

                    // Variables to capture link events
                    let mut created_start_pin = unsafe { std::mem::transmute(0i32) };
                    let mut created_end_pin = unsafe { std::mem::transmute(0i32) };
                    let mut dropped_link_id = unsafe { std::mem::transmute(0i32) };
                    let mut hovered_node_id: i32 = 0;

                    editor(editor_context, |mut node_editor| {
                        // Draw nodes for each mode
                        for (mode_idx, mode) in current_genome.genome.modes.iter().enumerate() {
                            if let Some(node_id) = node_graph.get_node_for_mode(mode_idx) {
                                draw_genome_node(ui, &mut node_editor, node_id, mode, mode_idx, &node_graph);
                            }
                        }

                        // Draw links between nodes
                        for (link_idx, (from_node, to_node, is_child_a)) in node_graph.links.iter().enumerate() {
                            let output_pin = if *is_child_a {
                                *from_node * 100 + 1
                            } else {
                                *from_node * 100 + 2
                            };
                            let input_pin = *to_node * 100;

                            // Use unsafe transmute to convert i32 to the required types
                            unsafe {
                                let link_id: imnodes::LinkId = std::mem::transmute(link_idx as i32);
                                let out_id: imnodes::OutputPinId = std::mem::transmute(output_pin);
                                let in_id: imnodes::InputPinId = std::mem::transmute(input_pin);
                                // Note: add_link signature is (link_id, input_pin_id, output_pin_id)
                                node_editor.add_link(link_id, in_id, out_id);
                            }
                        }
                    });

                    // Check for link events after editor scope closes but while still in window
                    let link_was_created = imnodes_extensions::get_created_link_pins(&mut created_start_pin, &mut created_end_pin);
                    let link_was_dropped = imnodes_extensions::get_dropped_link_id(&mut dropped_link_id);
                    let node_is_hovered = imnodes_extensions::is_node_hovered(&mut hovered_node_id);
                    
                    let mut link_started_pin: OutputPinId = unsafe { std::mem::transmute(0i32) };
                    let link_is_started = imnodes_extensions::is_link_started(&mut link_started_pin);
                    
                    let mut hovered_link_id: i32 = 0;
                    let link_is_hovered = imnodes_extensions::is_link_hovered(&mut hovered_link_id);
                    
                    // Track which pin is being dragged from
                    if link_is_started {
                        let pin_id: i32 = unsafe { std::mem::transmute(link_started_pin) };
                        graph_state.dragging_from_pin = Some(pin_id);
                    }

                    // Handle link creation
                    if link_was_created {
                        handle_link_created(&mut current_genome, &mut node_graph, created_start_pin, created_end_pin);
                        graph_state.dragging_from_pin = None; // Clear drag state
                    }

                    // Handle link dropped - check if it was dropped over a node to auto-connect
                    if link_was_dropped {
                        // If dropped over a node and we know which pin was dragged, connect to that node
                        if node_is_hovered && graph_state.dragging_from_pin.is_some() {
                            let output_pin: imnodes::OutputPinId = unsafe { std::mem::transmute(graph_state.dragging_from_pin.unwrap()) };
                            let parent_input_pin: imnodes::InputPinId = unsafe { std::mem::transmute(hovered_node_id * 100) };
                            
                            // Create the new connection
                            handle_link_created(&mut current_genome, &mut node_graph, output_pin, parent_input_pin);
                        } else {
                            // Not over a node, so destroy the link (make it self-referential)
                            handle_link_destroyed(&mut current_genome, &mut node_graph, dropped_link_id);
                        }
                        graph_state.dragging_from_pin = None; // Clear drag state
                    }
                    
                    // Handle right-click on link to make it self-referential
                    if link_is_hovered && ui.is_mouse_clicked(imgui::MouseButton::Right) {
                        handle_link_make_self_referential(&mut current_genome, &mut node_graph, hovered_link_id);
                    }

                    // Handle node click to select mode (without shift)
                    if node_is_hovered && ui.is_mouse_clicked(imgui::MouseButton::Left) && !ui.io().key_shift {
                        if let Some(mode_idx) = node_graph.get_mode_for_node(hovered_node_id) {
                            current_genome.selected_mode_index = mode_idx as i32;
                        }
                    }

                    // Handle Shift+Click to add new mode
                    // Check if shift was held and left mouse was clicked, and we're not hovering a node
                    if ui.io().key_shift && ui.is_mouse_clicked(imgui::MouseButton::Left) && !node_is_hovered {
                        // Get mouse position in editor space using imnodes API
                        let mouse_pos_editor = unsafe {
                            let mut pos = imnodes_sys::ImVec2 { x: 0.0, y: 0.0 };
                            imnodes_sys::imnodes_EditorContextGetPanning(&mut pos as *mut _);
                            let mouse_screen = ui.io().mouse_pos;
                            let window_pos = ui.window_pos();
                            [
                                mouse_screen[0] - window_pos[0] - pos.x,
                                mouse_screen[1] - window_pos[1] - 40.0 - pos.y, // Subtract title bar and help text height
                            ]
                        };
                        
                        // Insert after selected mode
                        let selected_idx = current_genome.selected_mode_index as usize;
                        let insert_idx = if selected_idx < current_genome.genome.modes.len() {
                            selected_idx + 1
                        } else {
                            current_genome.genome.modes.len()
                        };
                        
                        // Generate new mode name based on selected mode's default name
                        let new_name = if selected_idx < current_genome.genome.modes.len() {
                            generate_next_mode_name(&current_genome.genome.modes[selected_idx].default_name, &current_genome.genome.modes)
                        } else {
                            format!("Mode {}", current_genome.genome.modes.len())
                        };
                        
                        let new_mode = ModeSettings::new_self_splitting(
                            insert_idx as i32,
                            new_name,
                        );
                        
                        current_genome.genome.modes.insert(insert_idx, new_mode);
                        
                        // Update mode numbers for all modes after insertion point
                        update_mode_numbers_after_insert(&mut current_genome.genome, insert_idx);
                        
                        // Select the newly created mode
                        current_genome.selected_mode_index = insert_idx as i32;
                        
                        // Store the desired position for the new mode before rebuild
                        node_graph.pending_position = Some((insert_idx, mouse_pos_editor[0], mouse_pos_editor[1]));
                        
                        // Mark node graph for rebuild
                        node_graph.mark_for_rebuild();
                    }

                    // Handle Shift+Right-click to remove node
                    if node_is_hovered && ui.is_mouse_clicked(imgui::MouseButton::Right) && ui.io().key_shift {
                        // Get the mode index for the hovered node
                        if let Some(mode_idx) = node_graph.get_mode_for_node(hovered_node_id) {
                            let initial_mode = current_genome.genome.initial_mode as usize;
                            
                            // Don't allow removing the last mode or the initial mode
                            if current_genome.genome.modes.len() > 1 && mode_idx != initial_mode {
                                // Remove the mode
                                current_genome.genome.modes.remove(mode_idx);
                                
                                // Update references in other modes
                                for (idx, mode) in current_genome.genome.modes.iter_mut().enumerate() {
                                    // If child references the removed mode, make it self-splitting
                                    if mode.child_a.mode_number == mode_idx as i32 {
                                        mode.child_a.mode_number = idx as i32;
                                    } else if mode.child_a.mode_number > mode_idx as i32 {
                                        // Shift down references to modes after the removed one
                                        mode.child_a.mode_number -= 1;
                                    }
                                    
                                    if mode.child_b.mode_number == mode_idx as i32 {
                                        mode.child_b.mode_number = idx as i32;
                                    } else if mode.child_b.mode_number > mode_idx as i32 {
                                        mode.child_b.mode_number -= 1;
                                    }
                                }
                                
                                // Update initial mode if needed
                                if current_genome.genome.initial_mode == mode_idx as i32 {
                                    current_genome.genome.initial_mode = 0;
                                } else if current_genome.genome.initial_mode > mode_idx as i32 {
                                    current_genome.genome.initial_mode -= 1;
                                }
                                
                                // Update selected mode if needed
                                if current_genome.selected_mode_index >= current_genome.genome.modes.len() as i32 {
                                    current_genome.selected_mode_index = (current_genome.genome.modes.len() as i32) - 1;
                                }
                                
                                // Mark node graph for rebuild
                                node_graph.mark_for_rebuild();
                            }
                        }
                    }

                    // Update stored positions after drawing (user may have moved nodes)
                    for (_mode_idx, node_id) in node_ids {
                        unsafe {
                            let node_id_typed: imnodes::NodeId = std::mem::transmute(node_id);
                            let pos = node_id_typed.get_position(imnodes::CoordinateSystem::EditorSpace);
                            node_graph.set_node_position(node_id, pos.x, pos.y);
                        }
                    }
                }
            });
        });
    
    // Update the show_window state
    graph_state.show_window = show_window;
}

/// Rebuild the node graph from genome data
fn rebuild_node_graph(genome: &GenomeData, node_graph: &mut GenomeNodeGraph) {
    // Save existing positions by mode name (using stored names from node graph)
    let mut saved_positions_by_name: HashMap<String, (f32, f32)> = HashMap::new();
    
    // Save positions using the mode names stored in the node graph
    // This is stable because we stored the names when the nodes were created
    for (node_id, mode_name) in &node_graph.node_to_name {
        if let Some(pos) = node_graph.get_node_position(*node_id) {
            saved_positions_by_name.insert(mode_name.clone(), pos);
        }
    }
    
    // Track if we have saved positions to restore
    let has_saved_positions = !saved_positions_by_name.is_empty();
    
    node_graph.clear();

    // Create nodes for all modes
    let mut restored_positions = 0;
    for (mode_idx, mode) in genome.modes.iter().enumerate() {
        let node_id = node_graph.create_node(mode_idx);
        
        // Store the mode name for this node
        node_graph.node_to_name.insert(node_id, mode.name.clone());
        
        // Try to restore position by name (survives reordering/deletion)
        if let Some(&(x, y)) = saved_positions_by_name.get(&mode.name) {
            node_graph.set_node_position(node_id, x, y);
            restored_positions += 1;
        }
    }
    
    // If we restored positions for existing nodes, don't trigger automatic layout
    if has_saved_positions && restored_positions > 0 {
        node_graph.needs_layout = false;
    }
    
    // Apply pending position if set (for newly created nodes)
    if let Some((mode_idx, x, y)) = node_graph.pending_position.take() {
        if let Some(node_id) = node_graph.get_node_for_mode(mode_idx) {
            node_graph.set_node_position(node_id, x, y);
            // Don't need automatic layout since we have explicit positions
            node_graph.needs_layout = false;
        }
    }

    // Create links based on child mode references
    for (mode_idx, mode) in genome.modes.iter().enumerate() {
        if let Some(parent_node) = node_graph.get_node_for_mode(mode_idx) {
            // Link to Child A
            let child_a_idx = mode.child_a.mode_number as usize;
            if child_a_idx < genome.modes.len() {
                if let Some(child_a_node) = node_graph.get_node_for_mode(child_a_idx) {
                    node_graph.add_link(parent_node, child_a_node, true);
                }
            }

            // Link to Child B
            let child_b_idx = mode.child_b.mode_number as usize;
            if child_b_idx < genome.modes.len() {
                if let Some(child_b_node) = node_graph.get_node_for_mode(child_b_idx) {
                    node_graph.add_link(parent_node, child_b_node, false);
                }
            }
        }
    }
}

/// Draw a genome node in the node editor
fn draw_genome_node(
    ui: &Ui,
    node_editor: &mut imnodes::EditorScope,
    node_id: i32,
    mode: &ModeSettings,
    _mode_idx: usize,
    node_graph: &GenomeNodeGraph,
) {
    // Use unsafe transmute to convert i32 to NodeId (both are 32-bit)
    unsafe {
        let node_id_typed: imnodes::NodeId = std::mem::transmute(node_id);

        // Set node position if we have one stored
        if let Some((x, y)) = node_graph.get_node_position(node_id) {
            let _ = node_id_typed.set_position(x, y, imnodes::CoordinateSystem::EditorSpace);
        }

        // Convert mode color to u32 format for imnodes
        let node_color = color_vec3_to_u32(mode.color);
        
        // Calculate text color based on brightness for readability
        let brightness = mode.color.x * 0.299 + mode.color.y * 0.587 + mode.color.z * 0.114;
        let text_color = if brightness > 0.5 {
            [0.0, 0.0, 0.0, 1.0] // Dark text on light background
        } else {
            [1.0, 1.0, 1.0, 1.0] // Light text on dark background
        };
        
        // Push node color styles
        imnodes_sys::imnodes_PushColorStyle(
            imnodes_sys::ImNodesCol__ImNodesCol_TitleBar as i32,
            node_color,
        );
        imnodes_sys::imnodes_PushColorStyle(
            imnodes_sys::ImNodesCol__ImNodesCol_TitleBarHovered as i32,
            node_color,
        );
        imnodes_sys::imnodes_PushColorStyle(
            imnodes_sys::ImNodesCol__ImNodesCol_TitleBarSelected as i32,
            node_color,
        );

        node_editor.add_node(node_id_typed, |mut node| {
            // Title bar with mode name
            node.add_titlebar(|| {
                let _text_color = ui.push_style_color(StyleColor::Text, text_color);
                ui.text(&mode.name);
            });

            // Input pin (parent connection)
            let input_pin_id: imnodes::InputPinId = std::mem::transmute(node_id * 100);
            node.add_input(input_pin_id, PinShape::CircleFilled, || {
                ui.text("Parent");
            });

            // Node body - show key settings
            ui.spacing();
            ui.text(&format!("Type: {}", get_cell_type_name(mode.cell_type)));
            ui.text(&format!("Split: {:.1}s", mode.split_interval));
            if mode.parent_make_adhesion {
                ui.text("Adhesion: Yes");
            }
            ui.spacing();

            // Output pins (child connections)
            let child_a_pin_id: imnodes::OutputPinId = std::mem::transmute(node_id * 100 + 1);
            node.add_output(child_a_pin_id, PinShape::TriangleFilled, || {
                ui.text("Child A");
            });

            let child_b_pin_id: imnodes::OutputPinId = std::mem::transmute(node_id * 100 + 2);
            node.add_output(child_b_pin_id, PinShape::TriangleFilled, || {
                ui.text("Child B");
            });
        });
        
        // Pop the color styles (3 styles pushed)
        imnodes_sys::imnodes_PopColorStyle();
        imnodes_sys::imnodes_PopColorStyle();
        imnodes_sys::imnodes_PopColorStyle();
    }
}

/// Convert Vec3 color to u32 for imnodes
#[allow(dead_code)]
fn color_vec3_to_u32(color: Vec3) -> u32 {
    let r = (color.x * 255.0) as u32;
    let g = (color.y * 255.0) as u32;
    let b = (color.z * 255.0) as u32;
    0xFF000000 | (b << 16) | (g << 8) | r
}

/// Get cell type name from index
fn get_cell_type_name(cell_type: i32) -> &'static str {
    match cell_type {
        0 => "Test",
        _ => "Unknown",
    }
}

/// Handle link creation in the node graph
fn handle_link_created(
    current_genome: &mut ResMut<CurrentGenome>,
    node_graph: &mut ResMut<GenomeNodeGraph>,
    output_pin: OutputPinId,
    input_pin: InputPinId,
) {
    unsafe {
        // Convert pin IDs back to i32
        let output_pin_id: i32 = std::mem::transmute(output_pin);
        let input_pin_id: i32 = std::mem::transmute(input_pin);

        // Decode pin IDs: node_id * 100 for input, node_id * 100 + 1/2 for outputs
        let parent_node_id = output_pin_id / 100;
        let child_node_id = input_pin_id / 100;
        let is_child_a = (output_pin_id % 100) == 1;

        // Get mode indices from node IDs
        if let (Some(parent_mode_idx), Some(child_mode_idx)) = (
            node_graph.get_mode_for_node(parent_node_id),
            node_graph.get_mode_for_node(child_node_id),
        ) {
            // Update the genome data
            if parent_mode_idx < current_genome.genome.modes.len() {
                let mode = &mut current_genome.genome.modes[parent_mode_idx];
                if is_child_a {
                    mode.child_a.mode_number = child_mode_idx as i32;
                } else {
                    mode.child_b.mode_number = child_mode_idx as i32;
                }

                // Update the node graph
                node_graph.add_link(parent_node_id, child_node_id, is_child_a);
            }
        }
    }
}

/// Handle link destruction in the node graph
fn handle_link_destroyed(
    current_genome: &mut ResMut<CurrentGenome>,
    node_graph: &mut ResMut<GenomeNodeGraph>,
    link_id: LinkId,
) {
    unsafe {
        let link_idx: i32 = std::mem::transmute(link_id);

        // Find and remove the link
        if let Some((from_node, _to_node, is_child_a)) =
            node_graph.links.get(link_idx as usize).copied()
        {
            if let Some(parent_mode_idx) = node_graph.get_mode_for_node(from_node) {
                // Set the child back to self-splitting (point to same mode)
                if parent_mode_idx < current_genome.genome.modes.len() {
                    let mode = &mut current_genome.genome.modes[parent_mode_idx];
                    if is_child_a {
                        mode.child_a.mode_number = parent_mode_idx as i32;
                    } else {
                        mode.child_b.mode_number = parent_mode_idx as i32;
                    }
                }
            }

            // Rebuild the graph to reflect changes
            node_graph.mark_for_rebuild();
        }
    }
}

/// Handle right-click on link to make it self-referential
fn handle_link_make_self_referential(
    current_genome: &mut ResMut<CurrentGenome>,
    node_graph: &mut ResMut<GenomeNodeGraph>,
    link_id: i32,
) {
    // Find the link
    if let Some((from_node, _to_node, is_child_a)) =
        node_graph.links.get(link_id as usize).copied()
    {
        if let Some(parent_mode_idx) = node_graph.get_mode_for_node(from_node) {
            // Set the child to point to the same mode (self-referential)
            if parent_mode_idx < current_genome.genome.modes.len() {
                let mode = &mut current_genome.genome.modes[parent_mode_idx];
                if is_child_a {
                    mode.child_a.mode_number = parent_mode_idx as i32;
                } else {
                    mode.child_b.mode_number = parent_mode_idx as i32;
                }
            }
        }

        // Rebuild the graph to reflect changes
        node_graph.mark_for_rebuild();
    }
}

/// Generate the next available mode name based on a base name
/// If base name is "Mode 5", tries "Mode 6", then "Mode 5.1", "Mode 5.2", etc.
fn generate_next_mode_name(base_name: &str, existing_modes: &[ModeSettings]) -> String {
    // Extract the base number from the name (e.g., "Mode 5" -> 5)
    let base_number = if let Some(num_str) = base_name.split_whitespace().last() {
        // Try to parse as integer first
        if let Ok(num) = num_str.parse::<i32>() {
            num
        } else {
            // Try to parse as float (e.g., "5.1")
            if let Ok(num) = num_str.parse::<f32>() {
                num.floor() as i32
            } else {
                0
            }
        }
    } else {
        0
    };
    
    // Helper to check if a name is already used (checks both name and default_name)
    let is_name_taken = |candidate: &str| {
        existing_modes.iter().any(|m| m.name == candidate || m.default_name == candidate)
    };
    
    // Try the next integer first (e.g., "Mode 5" -> "Mode 6")
    let next_int_name = format!("Mode {}", base_number + 1);
    if !is_name_taken(&next_int_name) {
        return next_int_name;
    }
    
    // If that's taken, try decimal suffixes (e.g., "Mode 5.1", "Mode 5.2", etc.)
    for i in 1..100 {
        let candidate_name = format!("Mode {}.{}", base_number, i);
        if !is_name_taken(&candidate_name) {
            return candidate_name;
        }
    }
    
    // Fallback: use total mode count
    format!("Mode {}", existing_modes.len())
}

/// Update mode numbers after inserting a new mode
/// All modes at or after the insertion point need their references updated
fn update_mode_numbers_after_insert(genome: &mut GenomeData, insert_idx: usize) {
    // Update all child references that point to modes at or after the insertion point
    for (idx, mode) in genome.modes.iter_mut().enumerate() {
        // Skip the newly inserted mode - it should remain self-referential
        if idx == insert_idx {
            // Ensure the newly inserted mode is self-referential
            mode.child_a.mode_number = insert_idx as i32;
            mode.child_b.mode_number = insert_idx as i32;
            continue;
        }
        
        // For all other modes, update references that point to modes at or after insertion
        if mode.child_a.mode_number >= insert_idx as i32 {
            mode.child_a.mode_number += 1;
        }
        if mode.child_b.mode_number >= insert_idx as i32 {
            mode.child_b.mode_number += 1;
        }
    }
    
    // Update initial mode if needed
    if genome.initial_mode >= insert_idx as i32 {
        genome.initial_mode += 1;
    }
}
