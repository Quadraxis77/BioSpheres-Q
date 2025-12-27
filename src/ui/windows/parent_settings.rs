use bevy_egui::egui;
use crate::genome::CurrentGenome;

/// Helper function to create a color-coded group container
fn group_container(ui: &mut egui::Ui, title: &str, color: egui::Color32, content: impl FnOnce(&mut egui::Ui)) {
    let frame = egui::Frame::default()
        .fill(egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 30u8))
        .stroke(egui::Stroke::new(1.5, color))
        .corner_radius(egui::CornerRadius::same(4u8))
        .inner_margin(egui::Margin::same(8i8));

    frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(title).strong().color(color));
        ui.add_space(4.0);
        content(ui);
    });
    ui.add_space(6.0);
}

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

        // Division Settings Group (Yellow)
        group_container(ui, "Division Settings", egui::Color32::from_rgb(200, 180, 80), |ui| {
            ui.label("Split Mass:");
            ui.horizontal(|ui| {
                let available = ui.available_width();
                let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
                ui.style_mut().spacing.slider_width = slider_width;
                ui.add(egui::Slider::new(&mut mode.split_mass, 1.0..=3.0).show_value(false));
                ui.add(egui::DragValue::new(&mut mode.split_mass).speed(0.01).range(1.0..=3.0));
            });

            ui.label("Split Interval:");
            ui.horizontal(|ui| {
                let available = ui.available_width();
                let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
                ui.style_mut().spacing.slider_width = slider_width;
                ui.add(egui::Slider::new(&mut mode.split_interval, 1.0..=60.0).show_value(false));
                ui.add(egui::DragValue::new(&mut mode.split_interval).speed(0.1).range(1.0..=60.0).suffix("s"));
            });
        });

        // Nutrient Settings Group (Green)
        group_container(ui, "Nutrient Settings", egui::Color32::from_rgb(100, 180, 120), |ui| {
            ui.label("Nutrient Priority:");
            ui.horizontal(|ui| {
                let available = ui.available_width();
                let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
                ui.style_mut().spacing.slider_width = slider_width;
                ui.add(egui::Slider::new(&mut mode.nutrient_priority, 0.1..=10.0).show_value(false));
                ui.add(egui::DragValue::new(&mut mode.nutrient_priority).speed(0.01).range(0.1..=10.0));
            });

            ui.checkbox(&mut mode.prioritize_when_low, "Prioritize When Low");
        });

        // Connection Settings Group (Cyan)
        group_container(ui, "Connection Settings", egui::Color32::from_rgb(100, 180, 200), |ui| {
            ui.label("Max Connections:");
            ui.horizontal(|ui| {
                let available = ui.available_width();
                let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
                ui.style_mut().spacing.slider_width = slider_width;
                ui.add(egui::Slider::new(&mut mode.max_adhesions, 0..=20).show_value(false));
                ui.add(egui::DragValue::new(&mut mode.max_adhesions).speed(1).range(0..=20));
            });

            ui.label("Min Connections:");
            ui.horizontal(|ui| {
                let available = ui.available_width();
                let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
                ui.style_mut().spacing.slider_width = slider_width;
                ui.add(egui::Slider::new(&mut mode.min_adhesions, 0..=20).show_value(false));
                ui.add(egui::DragValue::new(&mut mode.min_adhesions).speed(1).range(0..=20));
            });

            ui.label("Max Splits:");
            ui.horizontal(|ui| {
                let available = ui.available_width();
                let slider_width = if available > 80.0 { available - 70.0 } else { 50.0 };
                ui.style_mut().spacing.slider_width = slider_width;
                ui.add(egui::Slider::new(&mut mode.max_splits, -1..=20).show_value(false));
                ui.add(egui::DragValue::new(&mut mode.max_splits).speed(0.1).range(-1.0..=20.0));
            });
        });
    });
}
