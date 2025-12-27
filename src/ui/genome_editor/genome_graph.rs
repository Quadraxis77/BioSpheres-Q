use bevy_egui::egui;
use crate::genome::CurrentGenome;

/// Placeholder for genome graph node editor
/// TODO: Implement using egui_node_graph once dependency is resolved
pub fn render_genome_graph(ui: &mut egui::Ui, _current_genome: &mut CurrentGenome) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
        ui.separator();
        ui.heading("Genome Graph");
        ui.label("Node-based genome editor");
        ui.label("(Implementation pending - requires egui_node_graph)");
    });
}
