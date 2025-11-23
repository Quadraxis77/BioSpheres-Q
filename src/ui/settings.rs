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
}

/// System to save UI settings when they change
#[allow(private_interfaces)]
pub fn save_ui_settings_on_change(
    global_ui_state: Res<crate::ui::GlobalUiState>,
    theme_state: Res<crate::ui::imgui_style::ImguiThemeState>,
    theme_editor_state: Res<crate::ui::theme_editor::ThemeEditorState>,
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
        });
        return;
    }

    let last = last_saved.as_ref().unwrap();

    // Check if theme changed
    let theme_changed = last.theme_name != current_theme_name || last.theme_is_custom != is_custom;

    // Only save if values actually changed
    let changed = last.windows_locked != global_ui_state.windows_locked
        || (last.ui_scale - global_ui_state.ui_scale).abs() > 0.001
        || theme_changed;

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

        if let Err(e) = settings.save() {
            error!("Failed to save UI settings: {}", e);
        }

        // Update last saved values
        *last_saved = Some(LastSavedSettings {
            windows_locked: global_ui_state.windows_locked,
            ui_scale: global_ui_state.ui_scale,
            theme_name: current_theme_name,
            theme_is_custom: is_custom,
        });
    }
}

