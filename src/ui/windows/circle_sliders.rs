use bevy_egui::egui;
use crate::genome::CurrentGenome;
use crate::ui::GenomeEditorState;
use crate::ui::widgets;

pub fn render(ui: &mut egui::Ui, current_genome: &mut CurrentGenome, genome_editor_state: &mut GenomeEditorState) {
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
