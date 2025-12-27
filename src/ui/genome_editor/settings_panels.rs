use bevy::prelude::*;
use bevy_egui::egui;
use crate::genome::CurrentGenome;
use crate::ui::GenomeEditorState;
use crate::ui::widgets;

pub fn render_name_type_editor(ui: &mut egui::Ui, current_genome: &mut CurrentGenome) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 2.0;

        // Three buttons at the top
        ui.horizontal(|ui| {
            if ui.button("Save Genome").clicked() {
                // Open save dialog
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_file_name(&format!("{}.json", current_genome.genome.name))
                    .save_file()
                {
                    info!("Would save genome to: {:?}", path);
                    // TODO: Implement actual save
                }
            }
            if ui.button("Load Genome").clicked() {
                // Open load dialog
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .pick_file()
                {
                    info!("Would load genome from: {:?}", path);
                    // TODO: Implement actual load
                }
            }
            if ui.button("Genome Graph").clicked() {
                // TODO: Implement genome graph
            }
        });

        ui.add_space(4.0);

        // Genome Name label and field on same line
        ui.horizontal(|ui| {
            ui.label("Genome Name:");
            ui.text_edit_singleline(&mut current_genome.genome.name);
        });

        ui.add_space(4.0);

        // Get current mode
        let selected_idx = current_genome.selected_mode_index as usize;
        if selected_idx >= current_genome.genome.modes.len() {
            ui.label("No mode selected");
            return;
        }
        let mode = &mut current_genome.genome.modes[selected_idx];

        // Type dropdown and checkbox on the same line
        ui.horizontal(|ui| {
            ui.label("Type:");
            let cell_types = ["Photocyte", "Phagocyte", "Flagellocyte", "Devorocyte", "Lipocyte"];
            egui::ComboBox::from_id_salt("cell_type")
                .selected_text(cell_types[mode.cell_type as usize])
                .show_ui(ui, |ui| {
                    for (i, type_name) in cell_types.iter().enumerate() {
                        ui.selectable_value(&mut mode.cell_type, i as i32, *type_name);
                    }
                });

            ui.checkbox(&mut mode.parent_make_adhesion, "Make Adhesion");
        });
    });
}

pub fn render_adhesion_settings(ui: &mut egui::Ui, current_genome: &mut CurrentGenome) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
        // Force content to fill available width
        ui.set_width(ui.available_width());
        ui.add_space(10.0);

        // Get current mode
        let selected_idx = current_genome.selected_mode_index as usize;
        if selected_idx >= current_genome.genome.modes.len() {
            ui.label("No mode selected");
            return;
        }
        let mode = &mut current_genome.genome.modes[selected_idx];

        // Adhesion Can Break checkbox
        ui.checkbox(&mut mode.adhesion_settings.can_break, "Adhesion Can Break");

        // Adhesion Break Force (0.1 to 100.0)
        ui.label("Adhesion Break Force:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.adhesion_settings.break_force, 0.1..=100.0).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.adhesion_settings.break_force).speed(0.1).range(0.1..=100.0));
        });

        // Adhesion Rest Length (0.5 to 5.0)
        ui.label("Adhesion Rest Length:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.adhesion_settings.rest_length, 0.5..=5.0).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.adhesion_settings.rest_length).speed(0.01).range(0.5..=5.0));
        });

        // Linear Spring Stiffness (0.1 to 500.0)
        ui.label("Linear Spring Stiffness:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.adhesion_settings.linear_spring_stiffness, 0.1..=500.0).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.adhesion_settings.linear_spring_stiffness).speed(0.1).range(0.1..=500.0));
        });

        // Linear Spring Damping (0.0 to 10.0)
        ui.label("Linear Spring Damping:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.adhesion_settings.linear_spring_damping, 0.0..=10.0).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.adhesion_settings.linear_spring_damping).speed(0.01).range(0.0..=10.0));
        });

        // Orientation Spring Stiffness (0.1 to 100.0)
        ui.label("Orientation Spring Stiffness:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.adhesion_settings.orientation_spring_stiffness, 0.1..=100.0).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.adhesion_settings.orientation_spring_stiffness).speed(0.1).range(0.1..=100.0));
        });

        // Orientation Spring Damping (0.0 to 10.0)
        ui.label("Orientation Spring Damping:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.adhesion_settings.orientation_spring_damping, 0.0..=10.0).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.adhesion_settings.orientation_spring_damping).speed(0.01).range(0.0..=10.0));
        });

        // Max Angular Deviation (0.0 to 180.0)
        ui.label("Max Angular Deviation:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.adhesion_settings.max_angular_deviation, 0.0..=180.0).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.adhesion_settings.max_angular_deviation).speed(0.1).range(0.0..=180.0));
        });

        ui.add_space(10.0);

        // Enable Twist Constraint checkbox
        ui.checkbox(&mut mode.adhesion_settings.enable_twist_constraint, "Enable Twist Constraint");

        // Twist Constraint Stiffness (0.0 to 2.0)
        ui.label("Twist Constraint Stiffness:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.adhesion_settings.twist_constraint_stiffness, 0.0..=2.0).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.adhesion_settings.twist_constraint_stiffness).speed(0.01).range(0.0..=2.0));
        });

        // Twist Constraint Damping (0.0 to 10.0)
        ui.label("Twist Constraint Damping:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.adhesion_settings.twist_constraint_damping, 0.0..=10.0).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.adhesion_settings.twist_constraint_damping).speed(0.01).range(0.0..=10.0));
        });
    });
}

pub fn render_parent_settings(ui: &mut egui::Ui, current_genome: &mut CurrentGenome) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
        // Force content to fill available width
        ui.set_width(ui.available_width());
        ui.add_space(10.0);

        // Get current mode
        let selected_idx = current_genome.selected_mode_index as usize;
        if selected_idx >= current_genome.genome.modes.len() {
            ui.label("No mode selected");
            return;
        }
        let mode = &mut current_genome.genome.modes[selected_idx];

        // Split Mass (1.0 to 3.0)
        ui.label("Split Mass:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.split_mass, 1.0..=3.0).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.split_mass).speed(0.01).range(1.0..=3.0));
        });

        // Split Interval (1.0 to 60.0 seconds)
        ui.label("Split Interval:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.split_interval, 1.0..=60.0).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.split_interval).speed(0.1).range(1.0..=60.0).suffix("s"));
        });

        // Nutrient Priority (0.1 to 10.0)
        ui.label("Nutrient Priority:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.nutrient_priority, 0.1..=10.0).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.nutrient_priority).speed(0.01).range(0.1..=10.0));
        });

        // Prioritize When Low checkbox
        ui.checkbox(&mut mode.prioritize_when_low, "Prioritize When Low");

        ui.add_space(10.0);

        // Max Connections (0 to 20)
        ui.label("Max Connections:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.max_adhesions, 0..=20).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.max_adhesions).speed(1).range(0..=20));
        });

        // Min Connections (0 to 20)
        ui.label("Min Connections:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.min_adhesions, 0..=20).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.min_adhesions).speed(1).range(0..=20));
        });

        // Max Splits (-1 to 20, where -1 = infinite)
        ui.label("Max Splits:");
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut mode.max_splits, -1..=20).show_value(false));
            ui.add(egui::DragValue::new(&mut mode.max_splits).speed(0.1).range(-1.0..=20.0));
        });
    });
}

pub fn render_circle_sliders(ui: &mut egui::Ui, current_genome: &mut CurrentGenome, genome_editor_state: &mut GenomeEditorState) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
        ui.checkbox(&mut genome_editor_state.enable_snapping, "Enable Snapping (11.25°)");
        ui.add_space(10.0);

        // Get current mode
        let selected_idx = current_genome.selected_mode_index as usize;
        if selected_idx < current_genome.genome.modes.len() {
            let mode = &mut current_genome.genome.modes[selected_idx];

            // Calculate responsive radius based on available width
            let available_width = ui.available_width();
            // Reserve space for padding and two sliders side by side
            let max_radius = ((available_width - 40.0) / 2.0 - 20.0) / 2.0;
            let radius = max_radius.clamp(20.0, 60.0);

            // Always side by side
            ui.horizontal(|ui| {
                ui.add_space(10.0);

                ui.vertical(|ui| {
                    ui.label("Pitch:");
                    let mut pitch = mode.parent_split_direction.x;
                    widgets::circular_slider_float(
                        ui,
                        &mut pitch,
                        -180.0,
                        180.0,
                        radius,
                        genome_editor_state.enable_snapping,
                    );
                    mode.parent_split_direction.x = pitch;
                });

                ui.vertical(|ui| {
                    ui.label("Yaw:");
                    let mut yaw = mode.parent_split_direction.y;
                    widgets::circular_slider_float(
                        ui,
                        &mut yaw,
                        -180.0,
                        180.0,
                        radius,
                        genome_editor_state.enable_snapping,
                    );
                    mode.parent_split_direction.y = yaw;
                });
            });
        }
    });
}

pub fn render_quaternion_ball(ui: &mut egui::Ui, current_genome: &mut CurrentGenome, genome_editor_state: &mut GenomeEditorState) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
        ui.checkbox(&mut genome_editor_state.qball_snapping, "Enable Snapping (11.25°)");
        ui.add_space(10.0);

        // Calculate responsive ball size - match circle slider calculation
        let available_width = ui.available_width();
        // Reserve space for padding and two balls side by side
        let max_radius = ((available_width - 40.0) / 2.0 - 20.0) / 2.0;
        let ball_radius = max_radius.clamp(20.0, 60.0);

        let ball_container_width = ball_radius * 2.0 + 20.0;

        // Get current mode index
        let selected_mode_idx = current_genome.selected_mode_index as usize;

        // Collect mode display data before mutable borrows
        let mode_display_data: Vec<(String, egui::Color32)> = current_genome.genome.modes.iter()
            .map(|m| {
                let color = m.color;
                let r = (color.x * 255.0) as u8;
                let g = (color.y * 255.0) as u8;
                let b = (color.z * 255.0) as u8;
                (m.name.clone(), egui::Color32::from_rgb(r, g, b))
            })
            .collect();

        // Display balls horizontally with coordinates directly below each ball
        ui.horizontal_top(|ui| {
            ui.add_space(10.0);

            // Ball 1 (Child A) with mode dropdown below
            ui.allocate_ui_with_layout(
                egui::vec2(ball_container_width, 0.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.label("Child A");

                    if selected_mode_idx >= current_genome.genome.modes.len() {
                        return;
                    }
                    let mode = &mut current_genome.genome.modes[selected_mode_idx];

                    widgets::quaternion_ball(
                        ui,
                        &mut mode.child_a.orientation,
                        &mut mode.child_a.x_axis_lat,
                        &mut mode.child_a.x_axis_lon,
                        &mut mode.child_a.y_axis_lat,
                        &mut mode.child_a.y_axis_lon,
                        &mut mode.child_a.z_axis_lat,
                        &mut mode.child_a.z_axis_lon,
                        ball_radius,
                        genome_editor_state.qball_snapping,
                        &mut genome_editor_state.qball1_locked_axis,
                        &mut genome_editor_state.qball1_initial_distance,
                    );

                    ui.add_space(5.0);

                    // Keep Adhesion checkbox for ball 1
                    ui.checkbox(&mut mode.child_a.keep_adhesion, "Keep Adhesion");

                    // Mode label and dropdown for ball 1
                    ui.label("Mode:");
                    let child_a_mode_idx = mode.child_a.mode_number as usize;
                    let mode_color = mode_display_data[child_a_mode_idx].1;
                    let brightness = mode_color.r() as f32 * 0.299 + mode_color.g() as f32 * 0.587 + mode_color.b() as f32 * 0.114;
                    let text_color = if brightness > 127.5 {
                        egui::Color32::BLACK
                    } else {
                        egui::Color32::WHITE
                    };
                    egui::ComboBox::from_id_salt("qball1_mode")
                        .selected_text(
                            egui::RichText::new(&mode_display_data[child_a_mode_idx].0)
                                .color(text_color)
                                .background_color(mode_color)
                        )
                        .width(ball_container_width - 20.0)
                        .show_ui(ui, |ui| {
                            for (i, (mode_name, mode_color)) in mode_display_data.iter().enumerate() {
                                // Calculate brightness to determine text color
                                let brightness = mode_color.r() as f32 * 0.299 + mode_color.g() as f32 * 0.587 + mode_color.b() as f32 * 0.114;
                                let text_color = if brightness > 127.5 {
                                    egui::Color32::BLACK
                                } else {
                                    egui::Color32::WHITE
                                };

                                let mut current_mode = mode.child_a.mode_number as usize;
                                let _response = ui.selectable_value(
                                    &mut current_mode,
                                    i,
                                    egui::RichText::new(mode_name).color(text_color).background_color(*mode_color)
                                );
                                if current_mode != mode.child_a.mode_number as usize {
                                    mode.child_a.mode_number = current_mode as i32;
                                }
                            }
                        });
                }
            );

            // Ball 2 (Child B) with mode dropdown below
            ui.allocate_ui_with_layout(
                egui::vec2(ball_container_width, 0.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.label("Child B");

                    if selected_mode_idx >= current_genome.genome.modes.len() {
                        return;
                    }
                    let mode = &mut current_genome.genome.modes[selected_mode_idx];

                    widgets::quaternion_ball(
                        ui,
                        &mut mode.child_b.orientation,
                        &mut mode.child_b.x_axis_lat,
                        &mut mode.child_b.x_axis_lon,
                        &mut mode.child_b.y_axis_lat,
                        &mut mode.child_b.y_axis_lon,
                        &mut mode.child_b.z_axis_lat,
                        &mut mode.child_b.z_axis_lon,
                        ball_radius,
                        genome_editor_state.qball_snapping,
                        &mut genome_editor_state.qball2_locked_axis,
                        &mut genome_editor_state.qball2_initial_distance,
                    );

                    ui.add_space(5.0);

                    // Keep Adhesion checkbox for ball 2
                    ui.checkbox(&mut mode.child_b.keep_adhesion, "Keep Adhesion");

                    // Mode label and dropdown for ball 2
                    ui.label("Mode:");
                    let child_b_mode_idx = mode.child_b.mode_number as usize;
                    let mode_color = mode_display_data[child_b_mode_idx].1;
                    let brightness = mode_color.r() as f32 * 0.299 + mode_color.g() as f32 * 0.587 + mode_color.b() as f32 * 0.114;
                    let text_color = if brightness > 127.5 {
                        egui::Color32::BLACK
                    } else {
                        egui::Color32::WHITE
                    };
                    egui::ComboBox::from_id_salt("qball2_mode")
                        .selected_text(
                            egui::RichText::new(&mode_display_data[child_b_mode_idx].0)
                                .color(text_color)
                                .background_color(mode_color)
                        )
                        .width(ball_container_width - 20.0)
                        .show_ui(ui, |ui| {
                            for (i, (mode_name, mode_color)) in mode_display_data.iter().enumerate() {
                                // Calculate brightness to determine text color
                                let brightness = mode_color.r() as f32 * 0.299 + mode_color.g() as f32 * 0.587 + mode_color.b() as f32 * 0.114;
                                let text_color = if brightness > 127.5 {
                                    egui::Color32::BLACK
                                } else {
                                    egui::Color32::WHITE
                                };

                                let mut current_mode = mode.child_b.mode_number as usize;
                                let _response = ui.selectable_value(
                                    &mut current_mode,
                                    i,
                                    egui::RichText::new(mode_name).color(text_color).background_color(*mode_color)
                                );
                                if current_mode != mode.child_b.mode_number as usize {
                                    mode.child_b.mode_number = current_mode as i32;
                                }
                            }
                        });
                }
            );
        });
    });
}

pub fn render_time_slider(ui: &mut egui::Ui, genome_editor_state: &mut GenomeEditorState) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Time:");

            let available = ui.available_width();
            let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
            ui.style_mut().spacing.slider_width = slider_width;
            ui.add(egui::Slider::new(&mut genome_editor_state.time_value, 0.0..=100.0).show_value(false));
            ui.add(egui::DragValue::new(&mut genome_editor_state.time_value).speed(0.1).range(0.0..=100.0));
        });
    });
}
