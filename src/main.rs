use bevy::prelude::*;
use bevy::window::{WindowMode, WindowResolution, MonitorSelection};
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use biospheres_bevy::*;
use biospheres_bevy::ui::{CellInspectorPlugin, GenomeEditorPlugin, SceneManagerPlugin, TimeScrubberPlugin};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .build()
                // Embed assets in the binary for release builds
                .add_before::<bevy::asset::AssetPlugin>(EmbeddedAssetPlugin {
                    mode: PluginMode::ReplaceDefault,
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(1920, 1080).with_scale_factor_override(1.0),
                        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                        ..default()
                    }),
                    ..default()
                })
        )
        // Core simulation plugins
        .add_plugins(SimulationPlugin)
        .add_plugins(CellPlugin)
        .add_plugins(GenomePlugin)
         // Rendering and UI plugins
        .add_plugins(RenderingPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(InputPlugin)
        // Optional UI plugins
        .add_plugins(CellInspectorPlugin)
        .add_plugins(GenomeEditorPlugin)
        .add_plugins(SceneManagerPlugin)
        .add_plugins(TimeScrubberPlugin)
        .run();
}
