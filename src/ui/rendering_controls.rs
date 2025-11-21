use bevy::prelude::*;
use bevy_mod_imgui::ImguiContext;
use imgui;
use crate::rendering::RenderingConfig;

/// Plugin for rendering controls UI
pub struct RenderingControlsPlugin;

impl Plugin for RenderingControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_controls_ui);
    }
}

/// System to render the rendering controls UI panel
fn render_controls_ui(
    mut imgui_context: NonSendMut<ImguiContext>,
    mut rendering_config: ResMut<RenderingConfig>,
) {
    let ui = imgui_context.ui();

    // Create a window for rendering controls
    ui.window("Rendering Controls")
        .size([300.0, 200.0], imgui::Condition::FirstUseEver)
        .position([10.0, 100.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.text("Visualization:");
            ui.separator();
            
            if ui.checkbox("Show Orientation Gizmos", &mut rendering_config.show_orientation_gizmos) {
                rendering_config.user_has_changed_gizmos = true;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Display forward (blue), right (green), and up (red) orientation axes for each cell");
            }
            
            if ui.checkbox("Show Split Plane Gizmos", &mut rendering_config.show_split_plane_gizmos) {
                rendering_config.user_has_changed_gizmos = true;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Display split plane rings showing the division direction for each cell");
            }
            
            if ui.checkbox("Show Adhesions", &mut rendering_config.show_adhesions) {
                rendering_config.user_has_changed_gizmos = true;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Display adhesion connections between cells");
            }
            
            ui.separator();
            ui.checkbox("Wireframe Mode", &mut rendering_config.wireframe_mode);
        });
}
