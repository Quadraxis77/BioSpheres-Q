use bevy::prelude::*;
use bevy_egui::egui;
use crate::genome::{CurrentGenome, ModeSettings};
use crate::ui::GenomeEditorState;
use crate::ui::widgets;

pub fn render_modes_panel(ui: &mut egui::Ui, current_genome: &mut CurrentGenome, genome_editor_state: &mut GenomeEditorState) {
    // Handle rename dialog (outside scroll area)
    let mut rename_confirmed = false;
    let mut rename_cancelled = false;

    if let Some(_rename_idx) = genome_editor_state.renaming_mode {
        egui::Window::new("Rename Mode")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label("Mode Name:");
                let response = ui.text_edit_singleline(&mut genome_editor_state.rename_buffer);

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    rename_confirmed = true;
                }

                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        rename_confirmed = true;
                    }
                    if ui.button("Cancel").clicked() {
                        rename_cancelled = true;
                    }
                });

                // Auto-focus the text field
                if !response.has_focus() {
                    response.request_focus();
                }
            });
    }

    if rename_confirmed {
        if let Some(_rename_idx) = genome_editor_state.renaming_mode {
            let trimmed = genome_editor_state.rename_buffer.trim();
            if !trimmed.is_empty() && _rename_idx < current_genome.genome.modes.len() {
                current_genome.genome.modes[_rename_idx].name = trimmed.to_string();
                info!("Renamed mode {} to {}", _rename_idx, trimmed);
            }
        }
        genome_editor_state.renaming_mode = None;
        genome_editor_state.rename_buffer.clear();
    }

    if rename_cancelled {
        genome_editor_state.renaming_mode = None;
        genome_editor_state.rename_buffer.clear();
    }

    // Draw buttons outside scroll area
    let (copy_into_clicked, reset_clicked) = widgets::modes_buttons(
        ui,
        current_genome.genome.modes.len(),
        current_genome.selected_mode_index as usize,
        current_genome.genome.initial_mode as usize,
    );

    ui.separator();

    // Show instruction text if in copy into mode (also outside scroll area)
    if genome_editor_state.copy_into_dialog_open {
        ui.colored_label(egui::Color32::YELLOW, "Select target mode to copy into:");
        ui.add_space(5.0);
    }

    // Convert modes to display format
    let modes_display: Vec<(String, egui::Color32)> = current_genome.genome.modes.iter()
        .map(|m| {
            let color = m.color;
            let r = (color.x * 255.0) as u8;
            let g = (color.y * 255.0) as u8;
            let b = (color.z * 255.0) as u8;
            (m.name.clone(), egui::Color32::from_rgb(r, g, b))
        })
        .collect();

    // Now create scroll area for the list
    let (selection_changed, initial_changed, rename_idx, color_change) = egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
        let available_width = ui.available_width();

        let mut selected_mode = current_genome.selected_mode_index as usize;
        let mut initial_mode = current_genome.genome.initial_mode as usize;

        let result = widgets::modes_list_items(
            ui,
            &modes_display,
            &mut selected_mode,
            &mut initial_mode,
            available_width,
            genome_editor_state.copy_into_dialog_open,
            &mut genome_editor_state.color_picker_state,
        );

        current_genome.selected_mode_index = selected_mode as i32;
        current_genome.genome.initial_mode = initial_mode as i32;

        result
    }).inner;

    if selection_changed {
        // If in copy into mode, this is the target selection
        if genome_editor_state.copy_into_dialog_open {
            let source_idx = genome_editor_state.copy_into_source;
            let target_idx = current_genome.selected_mode_index as usize;

            if source_idx != target_idx && source_idx < current_genome.genome.modes.len()
                && target_idx < current_genome.genome.modes.len() {
                // Copy all settings from source to target (including color, except name)
                let source_mode = current_genome.genome.modes[source_idx].clone();
                let target_name = current_genome.genome.modes[target_idx].name.clone();
                current_genome.genome.modes[target_idx] = source_mode;
                current_genome.genome.modes[target_idx].name = target_name;
                info!("Copied mode {} into mode {}", source_idx, target_idx);
            }

            // Exit copy into mode
            genome_editor_state.copy_into_dialog_open = false;
        } else {
            info!("Selected mode changed to: {}", current_genome.selected_mode_index);
        }
    }
    if initial_changed {
        info!("Initial mode changed to: {}", current_genome.genome.initial_mode);
    }

    // Handle rename request
    if let Some(idx) = rename_idx {
        genome_editor_state.renaming_mode = Some(idx);
        genome_editor_state.rename_buffer = current_genome.genome.modes[idx].name.clone();
    }

    // Handle color change from context menu color picker
    if let Some((idx, new_color)) = color_change {
        if idx < current_genome.genome.modes.len() {
            let r = new_color.r() as f32 / 255.0;
            let g = new_color.g() as f32 / 255.0;
            let b = new_color.b() as f32 / 255.0;
            current_genome.genome.modes[idx].color = Vec3::new(r, g, b);
            info!("Changed color of mode {}", idx);
        }
    }

    // Handle copy into mode
    if copy_into_clicked {
        let selected_idx = current_genome.selected_mode_index as usize;
        if selected_idx < current_genome.genome.modes.len() {
            // Enter copy into mode - user will click on target mode directly
            genome_editor_state.copy_into_dialog_open = true;
            genome_editor_state.copy_into_source = selected_idx;
        }
    }

    // Handle reset mode
    if reset_clicked {
        let selected_idx = current_genome.selected_mode_index as usize;
        if selected_idx < current_genome.genome.modes.len() {
            // Reset to default values
            let name = current_genome.genome.modes[selected_idx].name.clone();
            let color = current_genome.genome.modes[selected_idx].color;
            current_genome.genome.modes[selected_idx] = ModeSettings::default();
            current_genome.genome.modes[selected_idx].name = name;
            current_genome.genome.modes[selected_idx].color = color;
            current_genome.genome.modes[selected_idx].child_a.mode_number = selected_idx as i32;
            current_genome.genome.modes[selected_idx].child_b.mode_number = selected_idx as i32;
            info!("Reset mode {}", selected_idx);
        }
    }
}
