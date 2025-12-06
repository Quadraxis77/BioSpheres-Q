use bevy::prelude::*;
use bevy_mod_imgui::ImguiContext;
use imgui;
use crate::rendering::SkyboxConfig;

/// Plugin for lighting and skybox settings UI
pub struct LightingSettingsPlugin;

impl Plugin for LightingSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightingConfig>()
            .add_systems(Update, (lighting_settings_ui, update_directional_light));
    }
}

/// Lighting configuration resource
#[derive(Resource)]
pub struct LightingConfig {
    /// Directional light illuminance
    pub directional_illuminance: f32,
    /// Directional light color
    pub directional_color: [f32; 3],
    /// Directional light rotation (euler angles in degrees)
    pub directional_rotation: [f32; 3],
    /// Ambient light brightness
    pub ambient_brightness: f32,
}

impl Default for LightingConfig {
    fn default() -> Self {
        Self {
            directional_illuminance: 10000.0,
            directional_color: [1.0, 1.0, 1.0],
            directional_rotation: [-28.6, 28.6, 0.0], // Roughly -0.5, 0.5, 0.0 radians
            ambient_brightness: 500.0,
        }
    }
}

/// System to render the lighting settings UI panel
fn lighting_settings_ui(
    mut imgui_context: NonSendMut<ImguiContext>,
    mut lighting_config: ResMut<LightingConfig>,
    mut skybox_config: ResMut<SkyboxConfig>,
    global_ui_state: Res<crate::ui::GlobalUiState>,
) {
    let ui = imgui_context.ui();

    if !global_ui_state.show_lighting_settings {
        return;
    }
    
    use imgui::WindowFlags;
    let flags = if global_ui_state.windows_locked {
        WindowFlags::NO_MOVE | WindowFlags::NO_RESIZE
    } else {
        WindowFlags::empty()
    };
    
    ui.window("Lighting Settings")
        .size([320.0, 450.0], imgui::Condition::FirstUseEver)
        .position([10.0, 500.0], imgui::Condition::FirstUseEver)
        .flags(flags)
        .build(|| {
            // Directional Light section
            if ui.collapsing_header("Directional Light", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                ui.slider("Illuminance", 0.0, 50000.0, &mut lighting_config.directional_illuminance);
                if ui.is_item_hovered() {
                    ui.tooltip_text("Brightness of the directional light (lux)");
                }
                
                ui.color_edit3("Color", &mut lighting_config.directional_color);
                if ui.is_item_hovered() {
                    ui.tooltip_text("Color of the directional light");
                }
                
                ui.slider("Rotation X", -90.0, 90.0, &mut lighting_config.directional_rotation[0]);
                ui.slider("Rotation Y", -180.0, 180.0, &mut lighting_config.directional_rotation[1]);
                ui.slider("Rotation Z", -180.0, 180.0, &mut lighting_config.directional_rotation[2]);
                if ui.is_item_hovered() {
                    ui.tooltip_text("Direction of the light (euler angles in degrees)");
                }
            }
            
            ui.separator();
            
            // Ambient Light section
            if ui.collapsing_header("Ambient Light", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                ui.slider("Brightness", 0.0, 2000.0, &mut lighting_config.ambient_brightness);
                if ui.is_item_hovered() {
                    ui.tooltip_text("Ambient light brightness");
                }
            }
            
            ui.separator();
            
            // Skybox section
            if ui.collapsing_header("Skybox", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                let mut gamma = skybox_config.gamma;
                let mut brightness = skybox_config.brightness;
                let mut blue_tint = skybox_config.blue_tint;
                
                let gamma_changed = ui.slider("Gamma", 0.5, 10.0, &mut gamma);
                if ui.is_item_hovered() {
                    ui.tooltip_text("Higher values darken midtones while preserving bright areas");
                }
                
                let brightness_changed = ui.slider("Brightness", 0.0, 2.0, &mut brightness);
                if ui.is_item_hovered() {
                    ui.tooltip_text("Overall skybox brightness multiplier");
                }
                
                let tint_changed = ui.slider("Blue Tint", 0.0, 1.0, &mut blue_tint);
                if ui.is_item_hovered() {
                    ui.tooltip_text("Shift colors towards blue (0 = none, 1 = full)");
                }
                
                // Apply changes to trigger Bevy's change detection
                if gamma_changed || brightness_changed || tint_changed {
                    skybox_config.gamma = gamma;
                    skybox_config.brightness = brightness;
                    skybox_config.blue_tint = blue_tint;
                }
            }
            
            ui.separator();
            
            // Reset button
            if ui.button("Reset to Defaults") {
                *lighting_config = LightingConfig::default();
                *skybox_config = SkyboxConfig::default();
            }
        });
}

/// System to update directional and ambient lights based on config
fn update_directional_light(
    lighting_config: Res<LightingConfig>,
    mut directional_query: Query<(&mut DirectionalLight, &mut Transform), Without<AmbientLight>>,
    mut ambient_query: Query<&mut AmbientLight>,
) {
    if !lighting_config.is_changed() {
        return;
    }
    
    // Update directional lights
    for (mut light, mut transform) in directional_query.iter_mut() {
        light.illuminance = lighting_config.directional_illuminance;
        light.color = Color::srgb(
            lighting_config.directional_color[0],
            lighting_config.directional_color[1],
            lighting_config.directional_color[2],
        );
        
        // Convert degrees to radians and apply rotation
        let rotation = Quat::from_euler(
            EulerRot::XYZ,
            lighting_config.directional_rotation[0].to_radians(),
            lighting_config.directional_rotation[1].to_radians(),
            lighting_config.directional_rotation[2].to_radians(),
        );
        transform.rotation = rotation;
    }
    
    // Update ambient lights
    for mut ambient in ambient_query.iter_mut() {
        ambient.brightness = lighting_config.ambient_brightness;
    }
}
