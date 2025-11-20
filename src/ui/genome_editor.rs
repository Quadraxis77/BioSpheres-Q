use bevy::prelude::*;
use bevy_mod_imgui::prelude::*;
use imgui::InputTextFlags;
use imnodes::{Context, EditorContext, editor};
use crate::genome::*;
use crate::simulation::{SimulationState, SimulationMode};
use super::imgui_widgets;
use super::camera::ImGuiWantCapture;

/// Resource to track genome graph window state
#[derive(Resource, Default)]
pub struct GenomeGraphState {
    pub show_window: bool,
}

/// Genome editor plugin - modular UI component for editing genome data
pub struct GenomeEditorPlugin;

impl Plugin for GenomeEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GenomeGraphState>()
            .add_systems(Update, (
                update_imgui_capture_state,
                render_genome_editor,
                render_genome_graph,
            ).chain());
    }
}

/// System to update ImGui mouse capture state - runs before other UI systems
fn update_imgui_capture_state(
    mut imgui_context: NonSendMut<ImguiContext>,
    mut imgui_capture: ResMut<ImGuiWantCapture>,
) {
    let ui = imgui_context.ui();
    imgui_capture.want_capture_mouse = ui.io().want_capture_mouse;
}

/// Main genome editor rendering system
fn render_genome_editor(
    mut current_genome: ResMut<CurrentGenome>,
    mut imgui_context: NonSendMut<ImguiContext>,
    mut simulation_state: ResMut<SimulationState>,
    preview_state: Res<crate::simulation::preview_sim::PreviewSimState>,
    mut graph_state: ResMut<GenomeGraphState>,
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

            ui.same_line();
            if ui.button("Genome Graph") {
                graph_state.show_window = !graph_state.show_window;
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
) {
    // Only show in Preview mode and if window is open
    if simulation_state.mode != SimulationMode::Preview || !graph_state.show_window {
        return;
    }

    let ui = imgui_context.ui();

    // Create static context and editor for persistent state
    // We can't store these in a resource because they're not thread-safe (not Send/Sync)
    // This is safe because Bevy systems with NonSendMut run on the main thread only
    static mut IMNODES_CONTEXT: Option<Context> = None;
    static mut EDITOR_CONTEXT: Option<EditorContext> = None;

    // Safety: This system uses NonSendMut<ImguiContext>, which means Bevy guarantees
    // it only runs on the main thread, so there's no risk of data races
    unsafe {
        if IMNODES_CONTEXT.is_none() {
            let context = Context::new();
            let editor = context.create_editor();
            IMNODES_CONTEXT = Some(context);
            EDITOR_CONTEXT = Some(editor);
        }
    }

    ui.window("Genome Graph")
        .opened(&mut graph_state.show_window)
        .position([820.0, 13.0], Condition::FirstUseEver)
        .size([600.0, 600.0], Condition::FirstUseEver)
        .build(|| {
            // Create the node editor
            // Safety: Same as above - main thread only access
            unsafe {
                if let Some(editor_context) = &mut EDITOR_CONTEXT {
                    editor(editor_context, |_editor| {
                        // Empty node graph for now
                    });
                }
            }
        });
}
