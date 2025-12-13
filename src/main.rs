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
use std::env;

#[cfg(windows)]
fn allocate_console() {
    use winapi::um::consoleapi::AllocConsole;
    use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS};
    
    unsafe {
        // Try to attach to parent console first (if launched from cmd)
        if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
            // If no parent console, allocate a new one
            AllocConsole();
        }
    }
}

/// Startup system to apply window maximized state (always maximized)
fn apply_window_state(mut windows: Query<&mut Window>) {
    for mut window in windows.iter_mut() {
        println!("Setting window to maximized");
        window.set_maximized(true);
    }
}

fn main() {
    // Allocate console window on Windows
    #[cfg(windows)]
    allocate_console();
    
    // Enable verbose wgpu logging to console only
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("wgpu=debug,wgpu_core=debug,wgpu_hal=debug,bevy_render=debug"))
        .init();
    
    unsafe {
        env::set_var("RUST_LOG", "wgpu=debug,wgpu_core=debug,wgpu_hal=debug,bevy_render=debug");
        env::set_var("WGPU_VALIDATION", "1");
    }
    
    // Set up panic hook to create crash log only when there's actually a crash
    panic::set_hook(Box::new(move |panic_info| {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let crash_log_filename = format!("crash_log_{}.txt", timestamp);
        
        let mut log_content = String::new();
        log_content.push_str("=== BIOSPHERES CRASH LOG ===\n");
        log_content.push_str(&format!("Timestamp: {}\n", timestamp));
        log_content.push_str(&format!("Panic: {}\n", panic_info));
        
        if let Some(location) = panic_info.location() {
            log_content.push_str(&format!("Location: {}:{}:{}\n", 
                location.file(), location.line(), location.column()));
        }
        
        log_content.push_str(&format!("\nBacktrace:\n{:?}\n", std::backtrace::Backtrace::force_capture()));
        
        // Write crash log file
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&crash_log_filename) {
            let _ = file.write_all(log_content.as_bytes());
            let _ = file.flush();
            eprintln!("\n!!! CRASH LOG SAVED: {} !!!\n", crash_log_filename);
        }
        
        // Print to console
        eprintln!("{}", log_content);
        eprintln!("\nPress Enter to exit...");
        
        // Keep console open so user can read the error
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
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
                        // Allow fallback to DX12/DX11 on Windows if Vulkan isn't available
                        backends: Some(Backends::all()),
                        // Explicitly request only basic features to avoid compatibility issues
                        features: wgpu::Features::empty(),
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
        // Apply saved window state after startup (PostStartup ensures window is ready)
        .add_systems(PostStartup, apply_window_state)
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
