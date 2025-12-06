use bevy::prelude::*;

pub mod camera;
pub mod camera_settings;
pub mod cell_inspector;
pub mod debug_info;
pub mod imgui_panel;
pub mod imgui_style;
pub mod imgui_widgets;
pub mod genome_editor;
pub mod imnodes_extensions;
pub mod lighting_settings;
pub mod performance_monitor;
pub mod scene_manager;
pub mod theme_editor;
pub mod time_scrubber;
pub mod rendering_controls;
pub mod settings;

pub use camera::{CameraPlugin, MainCamera, CameraConfig, CameraState, CameraMode, FocalPlaneSettings};
pub use camera_settings::CameraSettingsPlugin;
pub use cell_inspector::{CellInspectorPlugin, CellInspectorState};
pub use debug_info::DebugInfoPlugin;
pub use imgui_panel::{ImguiPanelPlugin, ImguiPanelState};
pub use imgui_style::{ImguiTheme, ImguiThemeState};
pub use imgui_widgets::range_slider;
pub use genome_editor::GenomeEditorPlugin;
pub use lighting_settings::{LightingSettingsPlugin, LightingConfig};
pub use performance_monitor::PerformanceMonitorPlugin;
pub use scene_manager::{SceneManagerPlugin, SceneManagerState};
pub use theme_editor::{ThemeEditorPlugin, ThemeEditorState};
pub use time_scrubber::{TimeScrubberPlugin, TimeScrubberState};
pub use rendering_controls::RenderingControlsPlugin;

/// Global UI state shared across all UI components
#[derive(Resource)]
pub struct GlobalUiState {
    pub windows_locked: bool,
    pub ui_scale: f32,
    // Window visibility toggles
    pub show_cell_inspector: bool,
    pub show_genome_editor: bool,
    pub show_scene_manager: bool,
    pub show_performance_monitor: bool,
    pub show_rendering_controls: bool,
    pub show_time_scrubber: bool,
    pub show_theme_editor: bool,
    pub show_camera_settings: bool,
    pub show_lighting_settings: bool,
}

impl Default for GlobalUiState {
    fn default() -> Self {
        // Load settings from disk, defaults to unlocked on first startup
        let saved_settings = settings::UiSettings::load();
        Self {
            windows_locked: saved_settings.windows_locked,
            ui_scale: saved_settings.ui_scale,
            show_cell_inspector: saved_settings.window_visibility.show_cell_inspector,
            show_genome_editor: saved_settings.window_visibility.show_genome_editor,
            show_scene_manager: saved_settings.window_visibility.show_scene_manager,
            show_performance_monitor: saved_settings.window_visibility.show_performance_monitor,
            show_rendering_controls: saved_settings.window_visibility.show_rendering_controls,
            show_time_scrubber: saved_settings.window_visibility.show_time_scrubber,
            show_theme_editor: saved_settings.window_visibility.show_theme_editor,
            show_camera_settings: saved_settings.window_visibility.show_camera_settings,
            show_lighting_settings: saved_settings.window_visibility.show_lighting_settings,
        }
    }
}

/// System to load theme from saved settings on startup
/// This runs once on startup and applies custom theme on first Update frame
pub(crate) fn load_theme_from_settings(
    mut theme_state: ResMut<imgui_style::ImguiThemeState>,
    mut theme_editor_state: ResMut<theme_editor::ThemeEditorState>,
    mut commands: Commands,
) {
    let saved_settings = settings::UiSettings::load();

    // Check if the saved theme is a custom theme
    if saved_settings.theme.is_custom_theme {
        // Find the custom theme by name
        if let Some(custom_theme) = saved_settings.custom_themes.iter()
            .find(|t| t.name == saved_settings.theme.current_theme_name) {
            // Load custom theme data
            theme_editor_state.custom_colors = theme_editor::CustomThemeColors {
                window_bg: custom_theme.colors.window_bg,
                text: custom_theme.colors.text,
                border: custom_theme.colors.border,
                button: custom_theme.colors.button,
                button_hovered: custom_theme.colors.button_hovered,
                button_active: custom_theme.colors.button_active,
                frame_bg: custom_theme.colors.frame_bg,
                frame_bg_hovered: custom_theme.colors.frame_bg_hovered,
                frame_bg_active: custom_theme.colors.frame_bg_active,
                slider_grab: custom_theme.colors.slider_grab,
                slider_grab_active: custom_theme.colors.slider_grab_active,
                header: custom_theme.colors.header,
                header_hovered: custom_theme.colors.header_hovered,
                checkmark: custom_theme.colors.checkmark,
            };

            theme_editor_state.custom_shapes = theme_editor::CustomThemeShapes {
                window_rounding: custom_theme.shapes.window_rounding,
                window_border_size: custom_theme.shapes.window_border_size,
                window_padding: custom_theme.shapes.window_padding,
                frame_rounding: custom_theme.shapes.frame_rounding,
                frame_border_size: custom_theme.shapes.frame_border_size,
                frame_padding: custom_theme.shapes.frame_padding,
                grab_rounding: custom_theme.shapes.grab_rounding,
                grab_min_size: custom_theme.shapes.grab_min_size,
                scrollbar_rounding: custom_theme.shapes.scrollbar_rounding,
                scrollbar_size: custom_theme.shapes.scrollbar_size,
                tab_rounding: custom_theme.shapes.tab_rounding,
                item_spacing: custom_theme.shapes.item_spacing,
                item_inner_spacing: custom_theme.shapes.item_inner_spacing,
            };

            theme_editor_state.active_custom_theme = Some(custom_theme.name.clone());

            // Schedule custom theme application on first Update frame
            commands.insert_resource(ApplyCustomThemeOnStartup);
        }
    } else {
        // Load preset theme by matching name
        for theme in imgui_style::ImguiTheme::all() {
            if theme.name() == saved_settings.theme.current_theme_name {
                theme_state.current_theme = *theme;
                theme_state.theme_changed = true;
                break;
            }
        }
    }
}

/// Resource marker to trigger custom theme application on first Update frame
#[derive(Resource)]
pub(crate) struct ApplyCustomThemeOnStartup;

/// System to apply custom theme on first Update frame after startup
/// This runs once on the first Update frame to apply custom themes loaded from disk
#[allow(invalid_reference_casting)]
pub(crate) fn apply_custom_theme_on_startup(
    mut commands: Commands,
    marker: Option<Res<ApplyCustomThemeOnStartup>>,
    mut context: Option<NonSendMut<bevy_mod_imgui::ImguiContext>>,
    theme_editor_state: Res<theme_editor::ThemeEditorState>,
) {
    if marker.is_none() {
        return;
    }

    if let Some(context) = context.as_mut() {
        let ui = context.ui();

        // Apply custom theme
        let colors = &theme_editor_state.custom_colors;
        let shapes = &theme_editor_state.custom_shapes;

        use imgui::StyleColor;

        // Apply the custom theme directly
        unsafe {
            let style_ptr = ui.style() as *const imgui::Style as *mut imgui::Style;
            let style = &mut *style_ptr;

            // Apply custom shapes
            style.window_rounding = shapes.window_rounding;
            style.window_border_size = shapes.window_border_size;
            style.window_padding = shapes.window_padding;
            style.frame_rounding = shapes.frame_rounding;
            style.frame_border_size = shapes.frame_border_size;
            style.frame_padding = shapes.frame_padding;
            style.grab_rounding = shapes.grab_rounding;
            style.grab_min_size = shapes.grab_min_size;
            style.scrollbar_rounding = shapes.scrollbar_rounding;
            style.scrollbar_size = shapes.scrollbar_size;
            style.tab_rounding = shapes.tab_rounding;
            style.item_spacing = shapes.item_spacing;
            style.item_inner_spacing = shapes.item_inner_spacing;

            // Apply custom colors
            let style_colors = &mut style.colors;
            style_colors[StyleColor::WindowBg as usize] = colors.window_bg;
            style_colors[StyleColor::Text as usize] = colors.text;
            style_colors[StyleColor::Border as usize] = colors.border;
            style_colors[StyleColor::Button as usize] = colors.button;
            style_colors[StyleColor::ButtonHovered as usize] = colors.button_hovered;
            style_colors[StyleColor::ButtonActive as usize] = colors.button_active;
            style_colors[StyleColor::FrameBg as usize] = colors.frame_bg;
            style_colors[StyleColor::FrameBgHovered as usize] = colors.frame_bg_hovered;
            style_colors[StyleColor::FrameBgActive as usize] = colors.frame_bg_active;
            style_colors[StyleColor::SliderGrab as usize] = colors.slider_grab;
            style_colors[StyleColor::SliderGrabActive as usize] = colors.slider_grab_active;
            style_colors[StyleColor::Header as usize] = colors.header;
            style_colors[StyleColor::HeaderHovered as usize] = colors.header_hovered;
            style_colors[StyleColor::CheckMark as usize] = colors.checkmark;
        }

        // Remove the marker so this doesn't run again
        commands.remove_resource::<ApplyCustomThemeOnStartup>();
    }
}

/// Main UI plugin - provides core UI functionality
pub struct UiPlugin;

impl Plugin for UiPlugin {
    #[allow(private_interfaces)]
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalUiState>()
            .add_plugins(CameraPlugin)
            .add_plugins(CameraSettingsPlugin)
            .add_plugins(DebugInfoPlugin)
            .add_plugins(ImguiPanelPlugin)
            .add_plugins(LightingSettingsPlugin)
            .add_plugins(PerformanceMonitorPlugin)
            .add_plugins(RenderingControlsPlugin)
            .add_plugins(ThemeEditorPlugin)
            .add_systems(Startup, (
                load_theme_from_settings, 
                settings::load_fog_settings_on_startup,
                settings::load_lighting_settings_on_startup,
                settings::load_skybox_settings_on_startup,
                settings::load_simulation_settings_on_startup,
            ))
            .add_systems(Update, (
                apply_custom_theme_on_startup,
                settings::save_ui_settings_on_change,
            ));
    }
}
