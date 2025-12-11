use bevy::prelude::*;
use bevy::window::{WindowMode, WindowResolution};
use bevy::render::{RenderPlugin, settings::{Backends, WgpuSettings}};
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use biospheres_bevy::*;
use biospheres_bevy::ui::{CellInspectorPlugin, GenomeEditorPlugin, SceneManagerPlugin, TimeScrubberPlugin};
use std::fs::OpenOptions;
use std::io::Write;
use std::panic;
use std::time::SystemTime;

fn main() {
    // Set up panic hook to log crashes
    panic::set_hook(Box::new(|panic_info| {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let log_filename = format!("crash_log_{}.txt", timestamp);
        
        let mut log_content = String::new();
        log_content.push_str("=== BIOSPHERES CRASH LOG ===\n");
        log_content.push_str(&format!("Timestamp: {}\n", timestamp));
        log_content.push_str(&format!("Panic: {}\n", panic_info));
        
        if let Some(location) = panic_info.location() {
            log_content.push_str(&format!("Location: {}:{}:{}\n", 
                location.file(), location.line(), location.column()));
        }
        
        log_content.push_str(&format!("\nBacktrace:\n{:?}\n", std::backtrace::Backtrace::force_capture()));
        
        // Write to file
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_filename) {
            let _ = file.write_all(log_content.as_bytes());
            eprintln!("\n!!! CRASH DETECTED - Log written to: {} !!!\n", log_filename);
        } else {
            eprintln!("\n!!! CRASH DETECTED - Failed to write log file !!!\n");
        }
        
        // Also print to stderr
        eprintln!("{}", log_content);
    }));

    App::new()
        .add_plugins(
            DefaultPlugins
                .build()
                // Embed assets in the binary for release builds
                .add_before::<bevy::asset::AssetPlugin>(EmbeddedAssetPlugin {
                    mode: PluginMode::ReplaceDefault,
                })
                .set(RenderPlugin {
                    render_creation: WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        ..default()
                    }.into(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(1920, 1080).with_scale_factor_override(1.0),
                        mode: WindowMode::Windowed,
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
