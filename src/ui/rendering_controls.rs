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
    mut fog_settings: ResMut<crate::rendering::VolumetricFogSettings>,
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
        .size([212.0, 550.0], imgui::Condition::FirstUseEver)
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
            
            // Volumetric Fog Settings
            ui.separator();
            ui.text("Volumetric Fog:");
            
            let mut enabled = fog_settings.enabled;
            if ui.checkbox("Enable Fog", &mut enabled) {
                fog_settings.enabled = enabled;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Toggle volumetric fog rendering");
            }
            
            ui.text("Density:");
            ui.slider("##fog_density", 0.0, 1.0, &mut fog_settings.density_factor);
            if ui.is_item_hovered() {
                ui.tooltip_text("Overall fog density");
            }
            
            ui.text("Absorption:");
            ui.slider("##fog_absorption", 0.0, 1.0, &mut fog_settings.absorption);
            if ui.is_item_hovered() {
                ui.tooltip_text("How much light is absorbed by the fog");
            }
            
            ui.text("Scattering:");
            ui.slider("##fog_scattering", 0.0, 1.0, &mut fog_settings.scattering);
            if ui.is_item_hovered() {
                ui.tooltip_text("How much light is scattered by the fog");
            }
            
            ui.text("Ambient:");
            ui.slider("##fog_ambient", 0.0, 0.1, &mut fog_settings.ambient_intensity);
            if ui.is_item_hovered() {
                ui.tooltip_text("Ambient light intensity in fog");
            }
            
            ui.text("Fog Color:");
            let mut fog_color = [
                fog_settings.fog_color.to_srgba().red,
                fog_settings.fog_color.to_srgba().green,
                fog_settings.fog_color.to_srgba().blue,
            ];
            if ui.color_edit3("##fog_color", &mut fog_color) {
                fog_settings.fog_color = Color::srgb(fog_color[0], fog_color[1], fog_color[2]);
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Color of the volumetric fog");
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
