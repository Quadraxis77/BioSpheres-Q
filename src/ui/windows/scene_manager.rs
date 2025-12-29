use bevy::prelude::*;
use bevy_egui::egui;
use crate::simulation::SimulationMode;

/// Resource to request scene mode changes from UI
#[derive(Resource, Default)]
pub struct SceneModeRequest {
    pub requested_mode: Option<SimulationMode>,
}

pub fn render(
    ui: &mut egui::Ui,
    current_mode: SimulationMode,
    scene_request: &mut SceneModeRequest,
) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 8.0;

        // Calculate button size for rounded buttons
        let button_height = 40.0;
        let button_width = ui.available_width();

        // Light Blue button for Genome Editor (Preview mode)
        let preview_selected = current_mode == SimulationMode::Preview;
        let preview_color = if preview_selected {
            egui::Color32::from_rgb(100, 180, 220) // Lighter blue when selected
        } else {
            egui::Color32::from_rgb(135, 206, 250) // Light blue
        };

        let preview_button = egui::Button::new(
            egui::RichText::new("Genome Editor")
                .size(20.0)
                .strong()
                .color(egui::Color32::BLACK)
        )
        .fill(preview_color)
        .corner_radius(8.0);

        if ui.add_sized(egui::vec2(button_width, button_height), preview_button).clicked() {
            if current_mode != SimulationMode::Preview {
                info!("Requesting switch to Genome Editor (Preview) mode");
                scene_request.requested_mode = Some(SimulationMode::Preview);
            }
        }

        ui.add_space(4.0);

        // Green button for CPU mode
        let cpu_selected = current_mode == SimulationMode::Cpu;
        let cpu_color = if cpu_selected {
            egui::Color32::from_rgb(80, 180, 80) // Darker green when selected
        } else {
            egui::Color32::from_rgb(144, 238, 144) // Light green
        };

        let cpu_button = egui::Button::new(
            egui::RichText::new("CPU Mode")
                .size(20.0)
                .strong()
                .color(egui::Color32::BLACK)
        )
        .fill(cpu_color)
        .corner_radius(8.0);

        if ui.add_sized(egui::vec2(button_width, button_height), cpu_button).clicked() {
            if current_mode != SimulationMode::Cpu {
                info!("Requesting switch to CPU mode");
                scene_request.requested_mode = Some(SimulationMode::Cpu);
            }
        }

        ui.add_space(4.0);

        // Red button for GPU mode (disabled)
        let gpu_button = egui::Button::new(
            egui::RichText::new("GPU Mode (Coming Soon)")
                .size(20.0)
                .strong()
                .color(egui::Color32::from_gray(100))
        )
        .fill(egui::Color32::from_rgb(180, 100, 100)) // Muted red
        .corner_radius(8.0);

        let gpu_response = ui.add_enabled_ui(false, |ui| {
            ui.add_sized(egui::vec2(button_width, button_height), gpu_button)
        }).inner;
        gpu_response.on_disabled_hover_text("GPU mode is not yet implemented");
    });
}
