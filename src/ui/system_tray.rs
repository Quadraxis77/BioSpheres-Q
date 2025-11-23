use bevy::prelude::*;
use tray_icon::{TrayIcon, TrayIconBuilder, menu::{Menu, MenuItem}};

/// Plugin for system tray icon functionality
pub struct SystemTrayPlugin;

impl Plugin for SystemTrayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_system_tray)
            .add_systems(Update, handle_tray_events);
    }
}

// We'll keep the TrayIcon in a static variable since it needs to persist
// and can't be stored in a Bevy Resource (not Send)
static mut TRAY_ICON: Option<TrayIcon> = None;

fn setup_system_tray() {
    // Create tray menu
    let tray_menu = Menu::new();

    let show_item = MenuItem::new("Show Window", true, None);
    let quit_item = MenuItem::new("Quit", true, None);

    if let Err(e) = tray_menu.append(&show_item) {
        error!("Failed to add show item to tray menu: {}", e);
        return;
    }

    if let Err(e) = tray_menu.append(&quit_item) {
        error!("Failed to add quit item to tray menu: {}", e);
        return;
    }

    // Create tray icon (using a simple default icon for now)
    // Note: You'll want to replace this with an actual icon file
    let icon = match load_icon() {
        Ok(icon) => icon,
        Err(e) => {
            error!("Failed to load tray icon: {}", e);
            return;
        }
    };

    match TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("BioSpheres")
        .with_icon(icon)
        .build()
    {
        Ok(tray_icon) => {
            info!("System tray icon created successfully");
            unsafe {
                TRAY_ICON = Some(tray_icon);
            }
        }
        Err(e) => {
            error!("Failed to create system tray icon: {}", e);
        }
    }
}

fn handle_tray_events(
    mut windows: Query<&mut Window>,
    mut exit: bevy::prelude::MessageWriter<bevy::app::AppExit>,
) {
    use tray_icon::menu::MenuEvent;

    while let Ok(event) = MenuEvent::receiver().try_recv() {
        let event_id = &event.id().0;

        // Check the event ID string
        if event_id == "Show Window" {
            info!("Restore window requested from tray");
            if let Ok(mut window) = windows.single_mut() {
                window.visible = true;
                window.focused = true;
            }
        } else if event_id == "Quit" {
            info!("Quit requested from tray");
            exit.write(bevy::app::AppExit::Success);
        } else {
            warn!("Unknown tray menu event: {:?}", event_id);
        }
    }
}

fn load_icon() -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
    // Create a simple 32x32 RGBA icon with a green circle
    // This is a placeholder - you should replace with an actual icon file
    let width = 32;
    let height = 32;
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    let center_x = width / 2;
    let center_y = height / 2;
    let radius = 12.0;

    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 - center_x as f32).abs();
            let dy = (y as f32 - center_y as f32).abs();
            let distance = (dx * dx + dy * dy).sqrt();

            let idx = ((y * width + x) * 4) as usize;

            if distance <= radius {
                // Green circle
                rgba[idx] = 50;      // R
                rgba[idx + 1] = 200; // G
                rgba[idx + 2] = 50;  // B
                rgba[idx + 3] = 255; // A
            } else {
                // Transparent background
                rgba[idx + 3] = 0;
            }
        }
    }

    tray_icon::Icon::from_rgba(rgba, width, height)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
