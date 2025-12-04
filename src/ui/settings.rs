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
            density_factor: 0.056,
            absorption: 0.023,
            scattering: 0.172,
            ambient_intensity: 0.0,
            fog_color: [0.37210405, 0.38575435, 0.6007463],
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
            // Default to unlocked on first startup
            windows_locked: false,
            // Default UI scale of 1.0 (100%)
            ui_scale: 1.0,
            // Default to Modern Dark theme
            theme: ThemeSettings::default(),
            // No custom themes saved by default
            custom_themes: Vec::new(),
            // All windows visible by default
            window_visibility: WindowVisibilitySettings::default(),
            // Default fog settings
            fog_settings: FogSettings::default(),
        }
    }
}

impl Default for WindowVisibilitySettings {
    fn default() -> Self {
        Self {
            show_cell_inspector: true,
            show_genome_editor: true,
            show_scene_manager: true,
            show_performance_monitor: false, // Disabled by default
            show_rendering_controls: true,
            show_time_scrubber: true,
            show_theme_editor: false, // Theme editor hidden by default
            show_camera_settings: false, // Camera settings hidden by default
        }
    }
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            current_theme_name: "Modern Dark".to_string(),
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
}

/// System to save UI settings when they change
#[allow(private_interfaces)]
pub fn save_ui_settings_on_change(
    global_ui_state: Res<crate::ui::GlobalUiState>,
    theme_state: Res<crate::ui::imgui_style::ImguiThemeState>,
    theme_editor_state: Res<crate::ui::theme_editor::ThemeEditorState>,
    fog_settings: Res<crate::rendering::VolumetricFogSettings>,
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
        || last.window_visibility.show_camera_settings != global_ui_state.show_camera_settings;

    // Check if fog settings changed
    let fog_changed = last.fog_settings.enabled != fog_settings.enabled
        || (last.fog_settings.density_factor - fog_settings.density_factor).abs() > 0.001
        || (last.fog_settings.absorption - fog_settings.absorption).abs() > 0.001
        || (last.fog_settings.scattering - fog_settings.scattering).abs() > 0.001
        || (last.fog_settings.ambient_intensity - fog_settings.ambient_intensity).abs() > 0.001
        || (last.fog_settings.fog_color[0] - fog_settings.fog_color.to_srgba().red).abs() > 0.001
        || (last.fog_settings.fog_color[1] - fog_settings.fog_color.to_srgba().green).abs() > 0.001
        || (last.fog_settings.fog_color[2] - fog_settings.fog_color.to_srgba().blue).abs() > 0.001;

    // Only save if values actually changed
    let changed = last.windows_locked != global_ui_state.windows_locked
        || (last.ui_scale - global_ui_state.ui_scale).abs() > 0.001
        || theme_changed
        || visibility_changed
        || fog_changed;

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
