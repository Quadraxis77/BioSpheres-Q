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
