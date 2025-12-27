// Temporary stub for lighting_settings - just exports the resource types
// Full egui implementation coming soon

use bevy::prelude::*;

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
            directional_rotation: [45.0, 45.0, 0.0],
            ambient_brightness: 0.1,
        }
    }
}
