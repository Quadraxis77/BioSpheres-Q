use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ImguiPanelState {
    pub show_debug_info: bool,
}

/// ImGui panel plugin for BioSpheres
pub struct ImguiPanelPlugin;

impl Plugin for ImguiPanelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ImguiPanelState {
            show_debug_info: true,
        })
        .add_plugins(bevy_mod_imgui::ImguiPlugin {
            ini_filename: Some("imgui.ini".into()),
            // Use a fixed font size instead of automatic DPI scaling
            // Automatic DPI scaling can be unreliable on Windows systems with mixed DPI
            // configurations or remote desktop connections, causing panels to render at
            // massive, distorted sizes. By disabling automatic scaling and using a fixed
            // font size, we ensure consistent, predictable UI rendering across all environments.
            //font_size: 16.0,
            apply_display_scale_to_font_size: false, // Disable automatic DPI scaling
            apply_display_scale_to_font_oversample: false, // Disable oversampling scaling
            ..Default::default()
        });
    }
}
