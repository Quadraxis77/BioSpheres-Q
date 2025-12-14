use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persisted UI settings that are saved to disk
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UiSettings {
    pub windows_locked: bool,
    pub ui_scale: f32,
    pub theme: ThemeSettings,
    /// Library of saved custom themes
    pub custom_themes: Vec<SavedCustomTheme>,
    /// Window visibility settings
    #[serde(default)]
    pub window_visibility: WindowVisibilitySettings,
    /// Volumetric fog settings
    #[serde(default)]
    pub fog_settings: FogSettings,
    /// Bloom settings
    #[serde(default)]
    pub bloom_settings: BloomSettings,
    /// Lighting settings
    #[serde(default)]
    pub lighting_settings: LightingSettings,
    /// Skybox settings
    #[serde(default)]
    pub skybox_settings: SkyboxSettings,
    /// Simulation settings
    #[serde(default)]
    pub simulation_settings: SimulationSettings,
}

/// Window visibility settings
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WindowVisibilitySettings {
    pub show_cell_inspector: bool,
    pub show_genome_editor: bool,
    pub show_scene_manager: bool,
    pub show_performance_monitor: bool,
    pub show_rendering_controls: bool,
    pub show_time_scrubber: bool,
    pub show_theme_editor: bool,
    #[serde(default = "default_false")]
    pub show_camera_settings: bool,
    #[serde(default = "default_false")]
    pub show_lighting_settings: bool,
}

fn default_false() -> bool {
    false
}

/// Fog settings
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FogSettings {
    pub enabled: bool,
    pub density_factor: f32,
    pub absorption: f32,
    pub scattering: f32,
    pub ambient_intensity: f32,
    pub fog_color: [f32; 3],
}

impl Default for FogSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            density_factor: 0.15,
            absorption: 0.3,
            scattering: 0.3,
            ambient_intensity: 0.02,
            fog_color: [0.3, 0.4, 0.5],
        }
    }
}

/// Bloom settings
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BloomSettings {
    pub enabled: bool,
    pub intensity: f32,
    pub low_frequency_boost: f32,
    pub high_pass_frequency: f32,
    pub composite_mode: String, // "Additive" or "EnergyConserving"
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 0.3,
            low_frequency_boost: 0.5,
            high_pass_frequency: 0.8,
            composite_mode: "EnergyConserving".to_string(),
        }
    }
}

/// Lighting settings
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LightingSettings {
    pub directional_illuminance: f32,
    pub directional_color: [f32; 3],
    pub directional_rotation: [f32; 3],
    pub ambient_brightness: f32,
}

impl Default for LightingSettings {
    fn default() -> Self {
        Self {
            directional_illuminance: 11135.0,
            directional_color: [0.934, 0.934, 0.934],
            directional_rotation: [55.1, -103.5, 0.0],
            ambient_brightness: 1327.0,
        }
    }
}

/// Skybox settings
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SkyboxSettings {
    pub gamma: f32,
    pub brightness: f32,
    pub blue_tint: f32,
}

impl Default for SkyboxSettings {
    fn default() -> Self {
        Self {
            gamma: 5.178,
            brightness: 1.327,
            blue_tint: 0.618,
        }
    }
}

/// Simulation settings
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimulationSettings {
    /// Enable GPU-accelerated collision physics
    pub gpu_physics_enabled: bool,
    /// Enable CPU multithreading for physics
    pub cpu_multithreaded: bool,
    /// Cell capacity for CPU simulation
    pub cpu_cell_capacity: usize,
    /// Spatial grid density
    pub grid_density: u32,
    /// Disable collision detection
    pub disable_collisions: bool,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            gpu_physics_enabled: false,
            cpu_multithreaded: false,
            cpu_cell_capacity: 2000,
            grid_density: 32,
            disable_collisions: false,
        }
    }
}

/// Theme settings - includes both preset theme selection and custom theme data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ThemeSettings {
    /// The currently active theme name (preset or custom)
    pub current_theme_name: String,
    /// Whether the current theme is a custom theme
    pub is_custom_theme: bool,
}

/// A saved custom theme with a user-defined name
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedCustomTheme {
    pub name: String,
    pub colors: CustomThemeColorsSerde,
    pub shapes: CustomThemeShapesSerde,
}

/// Serializable version of CustomThemeColors
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomThemeColorsSerde {
    pub window_bg: [f32; 4],
    pub text: [f32; 4],
    pub border: [f32; 4],
    pub button: [f32; 4],
    pub button_hovered: [f32; 4],
    pub button_active: [f32; 4],
    pub frame_bg: [f32; 4],
    pub frame_bg_hovered: [f32; 4],
    pub frame_bg_active: [f32; 4],
    pub slider_grab: [f32; 4],
    pub slider_grab_active: [f32; 4],
    pub header: [f32; 4],
    pub header_hovered: [f32; 4],
    pub checkmark: [f32; 4],
}

/// Serializable version of CustomThemeShapes
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomThemeShapesSerde {
    pub window_rounding: f32,
    pub window_border_size: f32,
    pub window_padding: [f32; 2],
    pub frame_rounding: f32,
    pub frame_border_size: f32,
    pub frame_padding: [f32; 2],
    pub grab_rounding: f32,
    pub grab_min_size: f32,
    pub scrollbar_rounding: f32,
    pub scrollbar_size: f32,
    pub tab_rounding: f32,
    pub item_spacing: [f32; 2],
    pub item_inner_spacing: [f32; 2],
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            // Default to locked for cleaner first-time experience
            windows_locked: true,
            // Default UI scale of 1.25 (125%) for better readability
            ui_scale: 1.25,
            // Default to Industrial theme
            theme: ThemeSettings::default(),
            // No custom themes saved by default
            custom_themes: Vec::new(),
            // Window visibility defaults
            window_visibility: WindowVisibilitySettings::default(),
            // Default fog settings
            fog_settings: FogSettings::default(),
            // Default bloom settings
            bloom_settings: BloomSettings::default(),
            // Default lighting settings
            lighting_settings: LightingSettings::default(),
            // Default skybox settings
            skybox_settings: SkyboxSettings::default(),
            // Default simulation settings
            simulation_settings: SimulationSettings::default(),
        }
    }
}

impl Default for WindowVisibilitySettings {
    fn default() -> Self {
        Self {
            show_cell_inspector: true,
            show_genome_editor: true,
            show_scene_manager: true,
            show_performance_monitor: true, // Show performance monitor by default
            show_rendering_controls: true,
            show_time_scrubber: true,
            show_theme_editor: false, // Theme editor hidden by default
            show_camera_settings: false, // Camera settings hidden by default
            show_lighting_settings: false, // Lighting settings hidden by default
        }
    }
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            current_theme_name: "Industrial".to_string(),
            is_custom_theme: false,
        }
    }
}

impl UiSettings {
    /// Get the path to the settings file
    fn settings_path() -> PathBuf {
        PathBuf::from("ui_settings.json")
    }

    /// Load settings from disk, or create default if file doesn't exist
    pub fn load() -> Self {
        let path = Self::settings_path();

        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<UiSettings>(&contents) {
                info!("Loaded UI settings from {:?}", path);
                return settings;
            } else {
                warn!("Failed to parse UI settings, using defaults");
            }
        }

        info!("Using default UI settings (first startup)");
        Self::default()
    }

    /// Save settings to disk
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::settings_path();
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        info!("Saved UI settings to {:?}", path);
        Ok(())
    }
}

/// Local resource to track last saved settings
#[derive(Default)]
pub(crate) struct LastSavedSettings {
    pub(crate) windows_locked: bool,
    pub(crate) ui_scale: f32,
    pub(crate) theme_name: String,
    pub(crate) theme_is_custom: bool,
    pub(crate) window_visibility: WindowVisibilitySettings,
    pub(crate) fog_settings: FogSettings,
    pub(crate) bloom_settings: BloomSettings,
    pub(crate) lighting_settings: LightingSettings,
    pub(crate) skybox_settings: SkyboxSettings,
    pub(crate) simulation_settings: SimulationSettings,
}

/// System to save UI settings when they change
#[allow(private_interfaces)]
pub fn save_ui_settings_on_change(
    global_ui_state: Res<crate::ui::GlobalUiState>,
    theme_state: Res<crate::ui::imgui_style::ImguiThemeState>,
    theme_editor_state: Res<crate::ui::theme_editor::ThemeEditorState>,
    fog_settings: Res<crate::rendering::VolumetricFogSettings>,
    rendering_config: Res<crate::rendering::RenderingConfig>,
    lighting_config: Res<crate::ui::lighting_settings::LightingConfig>,
    skybox_config: Res<crate::rendering::SkyboxConfig>,
    threading_config: Res<crate::simulation::SimulationThreadingConfig>,
    physics_config: Res<crate::simulation::PhysicsConfig>,
    cpu_cell_capacity: Res<crate::ui::scene_manager::CpuCellCapacity>,
    spatial_grid_config: Res<crate::simulation::SpatialGridConfig>,
    mut last_saved: Local<Option<LastSavedSettings>>,
) {
    // Get the current theme name from theme_editor_state
    let current_theme_name = if theme_editor_state.active_custom_theme.is_some() {
        theme_editor_state.active_custom_theme.clone().unwrap()
    } else {
        theme_state.current_theme.name().to_string()
    };
    let is_custom = theme_editor_state.active_custom_theme.is_some();

    // Initialize on first run
    if last_saved.is_none() {
        *last_saved = Some(LastSavedSettings {
            windows_locked: global_ui_state.windows_locked,
            ui_scale: global_ui_state.ui_scale,
            theme_name: current_theme_name,
            theme_is_custom: is_custom,
            window_visibility: WindowVisibilitySettings {
                show_cell_inspector: global_ui_state.show_cell_inspector,
                show_genome_editor: global_ui_state.show_genome_editor,
                show_scene_manager: global_ui_state.show_scene_manager,
                show_performance_monitor: global_ui_state.show_performance_monitor,
                show_rendering_controls: global_ui_state.show_rendering_controls,
                show_time_scrubber: global_ui_state.show_time_scrubber,
                show_theme_editor: global_ui_state.show_theme_editor,
                show_camera_settings: global_ui_state.show_camera_settings,
                show_lighting_settings: global_ui_state.show_lighting_settings,
            },
            fog_settings: FogSettings {
                enabled: fog_settings.enabled,
                density_factor: fog_settings.density_factor,
                absorption: fog_settings.absorption,
                scattering: fog_settings.scattering,
                ambient_intensity: fog_settings.ambient_intensity,
                fog_color: [
                    fog_settings.fog_color.to_srgba().red,
                    fog_settings.fog_color.to_srgba().green,
                    fog_settings.fog_color.to_srgba().blue,
                ],
            },
            bloom_settings: BloomSettings {
                enabled: rendering_config.bloom_enabled,
                intensity: rendering_config.bloom_intensity,
                low_frequency_boost: rendering_config.bloom_low_frequency_boost,
                high_pass_frequency: rendering_config.bloom_high_pass_frequency,
                composite_mode: match rendering_config.bloom_composite_mode {
                    crate::rendering::BloomCompositeMode::Additive => "Additive".to_string(),
                    crate::rendering::BloomCompositeMode::EnergyConserving => "EnergyConserving".to_string(),
                },
            },
            lighting_settings: LightingSettings {
                directional_illuminance: lighting_config.directional_illuminance,
                directional_color: lighting_config.directional_color,
                directional_rotation: lighting_config.directional_rotation,
                ambient_brightness: lighting_config.ambient_brightness,
            },
            skybox_settings: SkyboxSettings {
                gamma: skybox_config.gamma,
                brightness: skybox_config.brightness,
                blue_tint: skybox_config.blue_tint,
            },
            simulation_settings: SimulationSettings {
                gpu_physics_enabled: threading_config.gpu_physics_enabled,
                cpu_multithreaded: threading_config.cpu_multithreaded,
                cpu_cell_capacity: cpu_cell_capacity.capacity,
                grid_density: spatial_grid_config.grid_density,
                disable_collisions: physics_config.disable_collisions,
            },
        });
        return;
    }

    let last = last_saved.as_ref().unwrap();

    // Check if theme changed
    let theme_changed = last.theme_name != current_theme_name || last.theme_is_custom != is_custom;

    // Check if window visibility changed
    let visibility_changed = last.window_visibility.show_cell_inspector != global_ui_state.show_cell_inspector
        || last.window_visibility.show_genome_editor != global_ui_state.show_genome_editor
        || last.window_visibility.show_scene_manager != global_ui_state.show_scene_manager
        || last.window_visibility.show_performance_monitor != global_ui_state.show_performance_monitor
        || last.window_visibility.show_rendering_controls != global_ui_state.show_rendering_controls
        || last.window_visibility.show_time_scrubber != global_ui_state.show_time_scrubber
        || last.window_visibility.show_theme_editor != global_ui_state.show_theme_editor
        || last.window_visibility.show_camera_settings != global_ui_state.show_camera_settings
        || last.window_visibility.show_lighting_settings != global_ui_state.show_lighting_settings;

    // Check if fog settings changed
    let fog_changed = last.fog_settings.enabled != fog_settings.enabled
        || (last.fog_settings.density_factor - fog_settings.density_factor).abs() > 0.001
        || (last.fog_settings.absorption - fog_settings.absorption).abs() > 0.001
        || (last.fog_settings.scattering - fog_settings.scattering).abs() > 0.001
        || (last.fog_settings.ambient_intensity - fog_settings.ambient_intensity).abs() > 0.001
        || (last.fog_settings.fog_color[0] - fog_settings.fog_color.to_srgba().red).abs() > 0.001
        || (last.fog_settings.fog_color[1] - fog_settings.fog_color.to_srgba().green).abs() > 0.001
        || (last.fog_settings.fog_color[2] - fog_settings.fog_color.to_srgba().blue).abs() > 0.001;

    // Check if bloom settings changed
    let current_bloom_mode = match rendering_config.bloom_composite_mode {
        crate::rendering::BloomCompositeMode::Additive => "Additive",
        crate::rendering::BloomCompositeMode::EnergyConserving => "EnergyConserving",
    };
    let bloom_changed = last.bloom_settings.enabled != rendering_config.bloom_enabled
        || (last.bloom_settings.intensity - rendering_config.bloom_intensity).abs() > 0.001
        || (last.bloom_settings.low_frequency_boost - rendering_config.bloom_low_frequency_boost).abs() > 0.001
        || (last.bloom_settings.high_pass_frequency - rendering_config.bloom_high_pass_frequency).abs() > 0.001
        || last.bloom_settings.composite_mode != current_bloom_mode;

    // Check if lighting settings changed
    let lighting_changed = (last.lighting_settings.directional_illuminance - lighting_config.directional_illuminance).abs() > 0.001
        || (last.lighting_settings.directional_color[0] - lighting_config.directional_color[0]).abs() > 0.001
        || (last.lighting_settings.directional_color[1] - lighting_config.directional_color[1]).abs() > 0.001
        || (last.lighting_settings.directional_color[2] - lighting_config.directional_color[2]).abs() > 0.001
        || (last.lighting_settings.directional_rotation[0] - lighting_config.directional_rotation[0]).abs() > 0.001
        || (last.lighting_settings.directional_rotation[1] - lighting_config.directional_rotation[1]).abs() > 0.001
        || (last.lighting_settings.directional_rotation[2] - lighting_config.directional_rotation[2]).abs() > 0.001
        || (last.lighting_settings.ambient_brightness - lighting_config.ambient_brightness).abs() > 0.001;

    // Check if skybox settings changed
    let skybox_changed = (last.skybox_settings.gamma - skybox_config.gamma).abs() > 0.001
        || (last.skybox_settings.brightness - skybox_config.brightness).abs() > 0.001
        || (last.skybox_settings.blue_tint - skybox_config.blue_tint).abs() > 0.001;

    // Check if simulation settings changed
    let simulation_changed = last.simulation_settings.gpu_physics_enabled != threading_config.gpu_physics_enabled
        || last.simulation_settings.cpu_multithreaded != threading_config.cpu_multithreaded
        || last.simulation_settings.cpu_cell_capacity != cpu_cell_capacity.capacity
        || last.simulation_settings.grid_density != spatial_grid_config.grid_density
        || last.simulation_settings.disable_collisions != physics_config.disable_collisions;

    // Only save if values actually changed
    let changed = last.windows_locked != global_ui_state.windows_locked
        || (last.ui_scale - global_ui_state.ui_scale).abs() > 0.001
        || theme_changed
        || visibility_changed
        || fog_changed
        || bloom_changed
        || lighting_changed
        || skybox_changed
        || simulation_changed;

    if changed {
        // Load existing settings to preserve custom themes library
        let mut settings = UiSettings::load();

        // Update the changed values
        settings.windows_locked = global_ui_state.windows_locked;
        settings.ui_scale = global_ui_state.ui_scale;
        settings.theme = ThemeSettings {
            current_theme_name: current_theme_name.clone(),
            is_custom_theme: is_custom,
        };
        settings.window_visibility = WindowVisibilitySettings {
            show_cell_inspector: global_ui_state.show_cell_inspector,
            show_genome_editor: global_ui_state.show_genome_editor,
            show_scene_manager: global_ui_state.show_scene_manager,
            show_performance_monitor: global_ui_state.show_performance_monitor,
            show_rendering_controls: global_ui_state.show_rendering_controls,
            show_time_scrubber: global_ui_state.show_time_scrubber,
            show_theme_editor: global_ui_state.show_theme_editor,
            show_camera_settings: global_ui_state.show_camera_settings,
            show_lighting_settings: global_ui_state.show_lighting_settings,
        };
        settings.fog_settings = FogSettings {
            enabled: fog_settings.enabled,
            density_factor: fog_settings.density_factor,
            absorption: fog_settings.absorption,
            scattering: fog_settings.scattering,
            ambient_intensity: fog_settings.ambient_intensity,
            fog_color: [
                fog_settings.fog_color.to_srgba().red,
                fog_settings.fog_color.to_srgba().green,
                fog_settings.fog_color.to_srgba().blue,
            ],
        };
        settings.bloom_settings = BloomSettings {
            enabled: rendering_config.bloom_enabled,
            intensity: rendering_config.bloom_intensity,
            low_frequency_boost: rendering_config.bloom_low_frequency_boost,
            high_pass_frequency: rendering_config.bloom_high_pass_frequency,
            composite_mode: match rendering_config.bloom_composite_mode {
                crate::rendering::BloomCompositeMode::Additive => "Additive".to_string(),
                crate::rendering::BloomCompositeMode::EnergyConserving => "EnergyConserving".to_string(),
            },
        };
        settings.lighting_settings = LightingSettings {
            directional_illuminance: lighting_config.directional_illuminance,
            directional_color: lighting_config.directional_color,
            directional_rotation: lighting_config.directional_rotation,
            ambient_brightness: lighting_config.ambient_brightness,
        };
        settings.skybox_settings = SkyboxSettings {
            gamma: skybox_config.gamma,
            brightness: skybox_config.brightness,
            blue_tint: skybox_config.blue_tint,
        };
        settings.simulation_settings = SimulationSettings {
            gpu_physics_enabled: threading_config.gpu_physics_enabled,
            cpu_multithreaded: threading_config.cpu_multithreaded,
            cpu_cell_capacity: cpu_cell_capacity.capacity,
            grid_density: spatial_grid_config.grid_density,
            disable_collisions: physics_config.disable_collisions,
        };

        if let Err(e) = settings.save() {
            error!("Failed to save UI settings: {}", e);
        }

        // Update last saved values
        *last_saved = Some(LastSavedSettings {
            windows_locked: global_ui_state.windows_locked,
            ui_scale: global_ui_state.ui_scale,
            theme_name: current_theme_name,
            theme_is_custom: is_custom,
            window_visibility: WindowVisibilitySettings {
                show_cell_inspector: global_ui_state.show_cell_inspector,
                show_genome_editor: global_ui_state.show_genome_editor,
                show_scene_manager: global_ui_state.show_scene_manager,
                show_performance_monitor: global_ui_state.show_performance_monitor,
                show_rendering_controls: global_ui_state.show_rendering_controls,
                show_time_scrubber: global_ui_state.show_time_scrubber,
                show_theme_editor: global_ui_state.show_theme_editor,
                show_camera_settings: global_ui_state.show_camera_settings,
                show_lighting_settings: global_ui_state.show_lighting_settings,
            },
            fog_settings: FogSettings {
                enabled: fog_settings.enabled,
                density_factor: fog_settings.density_factor,
                absorption: fog_settings.absorption,
                scattering: fog_settings.scattering,
                ambient_intensity: fog_settings.ambient_intensity,
                fog_color: [
                    fog_settings.fog_color.to_srgba().red,
                    fog_settings.fog_color.to_srgba().green,
                    fog_settings.fog_color.to_srgba().blue,
                ],
            },
            bloom_settings: BloomSettings {
                enabled: rendering_config.bloom_enabled,
                intensity: rendering_config.bloom_intensity,
                low_frequency_boost: rendering_config.bloom_low_frequency_boost,
                high_pass_frequency: rendering_config.bloom_high_pass_frequency,
                composite_mode: match rendering_config.bloom_composite_mode {
                    crate::rendering::BloomCompositeMode::Additive => "Additive".to_string(),
                    crate::rendering::BloomCompositeMode::EnergyConserving => "EnergyConserving".to_string(),
                },
            },
            lighting_settings: LightingSettings {
                directional_illuminance: lighting_config.directional_illuminance,
                directional_color: lighting_config.directional_color,
                directional_rotation: lighting_config.directional_rotation,
                ambient_brightness: lighting_config.ambient_brightness,
            },
            skybox_settings: SkyboxSettings {
                gamma: skybox_config.gamma,
                brightness: skybox_config.brightness,
                blue_tint: skybox_config.blue_tint,
            },
            simulation_settings: SimulationSettings {
                gpu_physics_enabled: threading_config.gpu_physics_enabled,
                cpu_multithreaded: threading_config.cpu_multithreaded,
                cpu_cell_capacity: cpu_cell_capacity.capacity,
                grid_density: spatial_grid_config.grid_density,
                disable_collisions: physics_config.disable_collisions,
            },
        });
    }
}


/// System to load fog settings from saved UI settings on startup
pub fn load_fog_settings_on_startup(
    mut fog_settings: ResMut<crate::rendering::VolumetricFogSettings>,
) {
    let saved_settings = UiSettings::load();
    fog_settings.enabled = saved_settings.fog_settings.enabled;
    fog_settings.density_factor = saved_settings.fog_settings.density_factor;
    fog_settings.absorption = saved_settings.fog_settings.absorption;
    fog_settings.scattering = saved_settings.fog_settings.scattering;
    fog_settings.ambient_intensity = saved_settings.fog_settings.ambient_intensity;
    fog_settings.fog_color = Color::srgb(
        saved_settings.fog_settings.fog_color[0],
        saved_settings.fog_settings.fog_color[1],
        saved_settings.fog_settings.fog_color[2],
    );
}

/// System to load bloom settings from saved UI settings on startup
pub fn load_bloom_settings_on_startup(
    mut rendering_config: ResMut<crate::rendering::RenderingConfig>,
) {
    let saved_settings = UiSettings::load();
    rendering_config.bloom_enabled = saved_settings.bloom_settings.enabled;
    rendering_config.bloom_intensity = saved_settings.bloom_settings.intensity;
    rendering_config.bloom_low_frequency_boost = saved_settings.bloom_settings.low_frequency_boost;
    rendering_config.bloom_high_pass_frequency = saved_settings.bloom_settings.high_pass_frequency;
    rendering_config.bloom_composite_mode = match saved_settings.bloom_settings.composite_mode.as_str() {
        "Additive" => crate::rendering::BloomCompositeMode::Additive,
        _ => crate::rendering::BloomCompositeMode::EnergyConserving,
    };
}


/// System to load lighting settings from saved UI settings on startup
pub fn load_lighting_settings_on_startup(
    mut lighting_config: ResMut<crate::ui::lighting_settings::LightingConfig>,
) {
    let saved_settings = UiSettings::load();
    lighting_config.directional_illuminance = saved_settings.lighting_settings.directional_illuminance;
    lighting_config.directional_color = saved_settings.lighting_settings.directional_color;
    lighting_config.directional_rotation = saved_settings.lighting_settings.directional_rotation;
    lighting_config.ambient_brightness = saved_settings.lighting_settings.ambient_brightness;
}

/// System to load skybox settings from saved UI settings on startup
pub fn load_skybox_settings_on_startup(
    mut skybox_config: ResMut<crate::rendering::SkyboxConfig>,
) {
    let saved_settings = UiSettings::load();
    skybox_config.gamma = saved_settings.skybox_settings.gamma;
    skybox_config.brightness = saved_settings.skybox_settings.brightness;
    skybox_config.blue_tint = saved_settings.skybox_settings.blue_tint;
}

/// System to load simulation settings from saved UI settings on startup
pub fn load_simulation_settings_on_startup(
    mut threading_config: ResMut<crate::simulation::SimulationThreadingConfig>,
    mut physics_config: ResMut<crate::simulation::PhysicsConfig>,
    mut cpu_cell_capacity: ResMut<crate::ui::scene_manager::CpuCellCapacity>,
    mut spatial_grid_config: ResMut<crate::simulation::SpatialGridConfig>,
) {
    let saved_settings = UiSettings::load();
    threading_config.gpu_physics_enabled = saved_settings.simulation_settings.gpu_physics_enabled;
    threading_config.cpu_multithreaded = saved_settings.simulation_settings.cpu_multithreaded;
    cpu_cell_capacity.capacity = saved_settings.simulation_settings.cpu_cell_capacity;
    spatial_grid_config.grid_density = saved_settings.simulation_settings.grid_density;
    physics_config.disable_collisions = saved_settings.simulation_settings.disable_collisions;
}
