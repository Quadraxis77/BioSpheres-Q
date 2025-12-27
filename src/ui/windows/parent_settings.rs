use bevy_egui::egui;
use crate::genome::CurrentGenome;

pub fn render(ui: &mut egui::Ui, current_genome: &mut CurrentGenome) {
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
