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
    mut theme_state: ResMut<crate::ui::ImguiThemeState>,
    global_ui_state: Res<crate::ui::GlobalUiState>,
) {
    let ui = imgui_context.ui();

    // Create a window for rendering controls
    use imgui::WindowFlags;
    // Only show if visibility is enabled
    if !global_ui_state.show_rendering_controls {
        return;
    }
    
    // Build flags based on lock state
    let flags = if global_ui_state.windows_locked {
        WindowFlags::NO_MOVE | WindowFlags::NO_RESIZE
    } else {
        WindowFlags::empty()
    };
    
    ui.window("Rendering Controls")
        .size([212.0, 390.0], imgui::Condition::FirstUseEver)
        .position([1704.0, 349.0], imgui::Condition::FirstUseEver)
        .flags(flags)
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
            
            // World Sphere Settings
            ui.separator();
            ui.text("World Sphere:");
            
            ui.text("Opacity:");
            ui.slider("##world_opacity", 0.0, 1.0, &mut rendering_config.world_sphere_opacity);
            if ui.is_item_hovered() {
                ui.tooltip_text("Transparency of the world boundary sphere");
            }
            
            ui.text("Color:");
            let mut color = [
                rendering_config.world_sphere_color.x,
                rendering_config.world_sphere_color.y,
                rendering_config.world_sphere_color.z,
            ];
            if ui.color_edit3("##world_color", &mut color) {
                rendering_config.world_sphere_color = Vec3::new(color[0], color[1], color[2]);
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Base color of the world sphere");
            }
            
            ui.text("Edge Glow:");
            ui.slider("##world_emissive", 0.0, 0.5, &mut rendering_config.world_sphere_emissive);
            if ui.is_item_hovered() {
                ui.tooltip_text("Emissive lighting intensity for Fresnel edge glow");
            }
            
            // Theme selector
            ui.separator();
            ui.text("UI Theme:");
            
            for theme in crate::ui::ImguiTheme::all() {
                let is_selected = theme_state.current_theme == *theme;
                if ui.radio_button_bool(theme.name(), is_selected) && !is_selected {
                    theme_state.current_theme = *theme;
                    theme_state.theme_changed = true;
                }
            }
        });
}
