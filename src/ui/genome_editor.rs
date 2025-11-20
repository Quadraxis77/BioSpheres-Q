use bevy::prelude::*;
use bevy_mod_imgui::prelude::*;
use crate::genome::*;
use crate::simulation::{SimulationState, SimulationMode};
use super::imgui_widgets;

/// Genome editor plugin - modular UI component for editing genome data
pub struct GenomeEditorPlugin;

impl Plugin for GenomeEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_genome_editor);
    }
}

/// Main genome editor rendering system
fn render_genome_editor(
    mut current_genome: ResMut<CurrentGenome>,
    mut imgui_context: NonSendMut<ImguiContext>,
    mut simulation_state: ResMut<SimulationState>,
    preview_state: Res<crate::simulation::preview_sim::PreviewSimState>,
) {
    // Only show genome editor in Preview mode
    if simulation_state.mode != SimulationMode::Preview {
        return;
    }

    // Track if genome was modified during this frame
    let genome_before_edit = current_genome.genome.clone();

    let ui = imgui_context.ui();

    ui.window("Genome Editor")
        .position([4.0, 13.0], Condition::FirstUseEver)
        .size([800.0, 600.0], Condition::FirstUseEver)
        .size_constraints([800.0, 500.0], [f32::MAX, f32::MAX])
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

            ui.separator();

            // Initial mode dropdown
            ui.text("Initial Mode:");
            ui.same_line();
            let mode_names: Vec<String> = current_genome.genome.modes.iter()
                .map(|m| m.name.clone())
                .collect();

            let mut initial_mode = current_genome.genome.initial_mode as usize;
            let current_mode_name = mode_names.get(initial_mode).map(|s| s.as_str()).unwrap_or("None");
            if let Some(_token) = ui.begin_combo("##InitialMode", current_mode_name) {
                for (i, name) in mode_names.iter().enumerate() {
                    let is_selected = i == initial_mode;
                    if ui.selectable_config(name).selected(is_selected).build() {
                        initial_mode = i;
                        current_genome.genome.initial_mode = i as i32;
                    }
                }
            }

            ui.separator();

            // Mode management
            ui.text("Modes:");
            ui.same_line();
            if ui.button("Add Mode") {
                let new_mode = ModeSettings {
                    name: format!("Mode {}", current_genome.genome.modes.len()),
                    ..Default::default()
                };
                current_genome.genome.modes.push(new_mode);
            }

            ui.same_line();
            if ui.button("Remove Mode") && current_genome.genome.modes.len() > 1 {
                let selected = current_genome.selected_mode_index as usize;
                if selected < current_genome.genome.modes.len() {
                    current_genome.genome.modes.remove(selected);
                    if current_genome.selected_mode_index >= current_genome.genome.modes.len() as i32 {
                        current_genome.selected_mode_index = (current_genome.genome.modes.len() as i32) - 1;
                    }
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

                        let button_label = format!("{}: {}", i, name);
                        if ui.button_with_size(&button_label, [-1.0, 0.0]) {
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
                            draw_mode_settings(ui, selected_mode, &modes_for_ref);
                        });
                }
            }
        });

    // Check if genome was modified and trigger instant resimulation
    if current_genome.genome != genome_before_edit {
        // Trigger resimulation to current preview time to apply genome changes
        simulation_state.target_time = Some(preview_state.current_time);
    }
}

/// Draw mode settings (tabbed interface)
fn draw_mode_settings(ui: &Ui, mode: &mut ModeSettings, all_modes: &[ModeSettings]) {
    if let Some(_tab_bar) = ui.tab_bar("ModeSettingsTabs") {
        // Parent Settings Tab
        if let Some(_tab) = ui.tab_item("Parent Settings") {
            draw_parent_settings(ui, mode);
        }

        // Child A Settings Tab
        if let Some(_tab) = ui.tab_item("Child A Settings") {
            draw_child_settings(ui, "Child A", &mut mode.child_a, all_modes);

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
            draw_child_settings(ui, "Child B", &mut mode.child_b, all_modes);

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
fn draw_parent_settings(ui: &Ui, mode: &mut ModeSettings) {
    // Mode name
    ui.text("Mode Name:");
    let mut mode_name = mode.name.clone();
    if ui.input_text("##ModeName", &mut mode_name).build() {
        mode.name = mode_name;
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
        ui.slider("##SplitMass", 0.1, 10.0, &mut mode.split_mass);
    }

    // Split interval
    ui.text("Split Interval:");
    ui.slider("##SplitInterval", 1.0, 30.0, &mut mode.split_interval);

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
    ui.slider("##MaxAdhesions", 0, 20, &mut mode.max_adhesions);

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

/// Draw child settings
fn draw_child_settings(ui: &Ui, _label: &str, child: &mut ChildSettings, all_modes: &[ModeSettings]) {
    ui.text("Mode:");
    let mode_names: Vec<String> = all_modes.iter()
        .map(|m| m.name.clone())
        .collect();

    let mode_index = child.mode_number as usize;
    let current_mode_name = mode_names.get(mode_index).map(|s| s.as_str()).unwrap_or("None");

    if let Some(_token) = ui.begin_combo("##Mode", current_mode_name) {
        for (i, name) in mode_names.iter().enumerate() {
            let is_selected = i == mode_index;
            if ui.selectable_config(name).selected(is_selected).build() {
                child.mode_number = i as i32;
                // Clamp to valid range
                if child.mode_number >= all_modes.len() as i32 {
                    child.mode_number = (all_modes.len() as i32) - 1;
                }
                if child.mode_number < 0 {
                    child.mode_number = 0;
                }
            }
        }
    }

    ui.spacing();
    ui.separator();
    ui.spacing();

    ui.checkbox("Keep Adhesion", &mut child.keep_adhesion);
}

/// Draw adhesion settings
fn draw_adhesion_settings(ui: &Ui, adhesion: &mut AdhesionSettings) {
    ui.checkbox("Adhesion Can Break", &mut adhesion.can_break);

    ui.text("Adhesion Break Force:");
    ui.slider("##AdhesionBreakForce", 0.1, 100.0, &mut adhesion.break_force);

    ui.text("Adhesion Rest Length:");
    ui.slider("##AdhesionRestLength", 0.5, 5.0, &mut adhesion.rest_length);

    ui.text("Linear Spring Stiffness:");
    ui.slider("##LinearSpringStiffness", 0.1, 500.0, &mut adhesion.linear_spring_stiffness);

    ui.text("Linear Spring Damping:");
    ui.slider("##LinearSpringDamping", 0.0, 10.0, &mut adhesion.linear_spring_damping);

    ui.text("Angular Spring Stiffness:");
    ui.slider("##AngularSpringStiffness", 0.1, 100.0, &mut adhesion.orientation_spring_stiffness);

    ui.text("Angular Spring Damping:");
    ui.slider("##AngularSpringDamping", 0.0, 10.0, &mut adhesion.orientation_spring_damping);

    ui.text("Max Angular Deviation:");
    ui.slider("##MaxAngularDeviation", 0.0, 180.0, &mut adhesion.max_angular_deviation);

    ui.spacing();
    ui.separator();
    ui.spacing();

    ui.checkbox("Enable Twist Constraint", &mut adhesion.enable_twist_constraint);

    ui.text("Twist Constraint Stiffness:");
    ui.slider("##TwistConstraintStiffness", 0.0, 2.0, &mut adhesion.twist_constraint_stiffness);

    ui.text("Twist Constraint Damping:");
    ui.slider("##TwistConstraintDamping", 0.0, 10.0, &mut adhesion.twist_constraint_damping);
}
