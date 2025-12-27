use bevy::prelude::*;
use bevy_egui::egui;
use crate::genome::CurrentGenome;

pub fn render(ui: &mut egui::Ui, current_genome: &mut CurrentGenome) {
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
