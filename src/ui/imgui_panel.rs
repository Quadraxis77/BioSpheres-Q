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
            ..Default::default()
        });
    }
}
