use bevy::prelude::*;
use egui_dock::DockState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

const DOCK_STATE_FILE: &str = "dock_state.ron";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Panel {
    // Placeholder panels (permanent)
    LeftPanel,
    RightPanel,
    BottomPanel,
    Viewport,

    // Dynamic windows (can be opened/closed from menu) - matching BioSpheres-Q naming
    CellInspector,
    GenomeEditor,
    SceneManager,
    PerformanceMonitor,
    RenderingControls,
    TimeScrubber,
    ThemeEditor,
    CameraSettings,
    LightingSettings,
    
    // Legacy names for compatibility
    Inspector,
    Console,
    Hierarchy,
    Assets,
    CircleSliders,
    QuaternionBall,
    Modes,
    NameTypeEditor,
    AdhesionSettings,
    ParentSettings,
    TimeSlider,
}

impl Panel {
    pub fn is_placeholder(&self) -> bool {
        matches!(self, Panel::LeftPanel | Panel::RightPanel | Panel::BottomPanel | Panel::Viewport)
    }
}

impl std::fmt::Display for Panel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Panel::LeftPanel => write!(f, "Left Panel"),
            Panel::RightPanel => write!(f, "Right Panel"),
            Panel::BottomPanel => write!(f, "Bottom Panel"),
            Panel::Viewport => write!(f, "Viewport"),
            // BioSpheres-Q standard names
            Panel::CellInspector => write!(f, "Cell Inspector"),
            Panel::GenomeEditor => write!(f, "Genome Editor"),
            Panel::SceneManager => write!(f, "Scene Manager"),
            Panel::PerformanceMonitor => write!(f, "Performance Monitor"),
            Panel::RenderingControls => write!(f, "Rendering Controls"),
            Panel::TimeScrubber => write!(f, "Time Scrubber"),
            Panel::ThemeEditor => write!(f, "Theme Editor"),
            Panel::CameraSettings => write!(f, "Camera Settings"),
            Panel::LightingSettings => write!(f, "Lighting Settings"),
            // Legacy names
            Panel::Inspector => write!(f, "Inspector"),
            Panel::Console => write!(f, "Console"),
            Panel::Hierarchy => write!(f, "Hierarchy"),
            Panel::Assets => write!(f, "Assets"),
            Panel::CircleSliders => write!(f, "Parent Split Angle"),
            Panel::QuaternionBall => write!(f, "Child Settings"),
            Panel::Modes => write!(f, "Modes"),
            Panel::NameTypeEditor => write!(f, "Name & Type"),
            Panel::AdhesionSettings => write!(f, "Adhesion Settings"),
            Panel::ParentSettings => write!(f, "Parent Settings"),
            Panel::TimeSlider => write!(f, "Time Slider"),
        }
    }
}

#[derive(Resource)]
pub struct DockResource {
    pub tree: DockState<Panel>,
    pub all_hidden: bool,
}

pub fn load_dock_state() -> Option<DockState<Panel>> {
    if Path::new(DOCK_STATE_FILE).exists() {
        let data = fs::read_to_string(DOCK_STATE_FILE).ok()?;
        ron::from_str(&data).ok()
    } else {
        None
    }
}

pub fn save_dock_state(tree: &DockState<Panel>) {
    if let Ok(serialized) = ron::ser::to_string_pretty(tree, Default::default()) {
        let _ = fs::write(DOCK_STATE_FILE, serialized);
    }
}

pub fn create_default_layout() -> DockState<Panel> {
    // Create the initial layout with Viewport in the center
    let mut tree = DockState::new(vec![Panel::Viewport]);
    let surface = tree.main_surface_mut();

    // Build structure based on current baked layout:
    // Left: LeftPanel (placeholder)
    // Center: Viewport / BottomPanel (vertical split)
    // Right: RightPanel (placeholder)

    // First: Split off left panel area (~12% width)
    let [_left_panel, rest] = surface.split_left(
        egui_dock::NodeIndex::root(),
        0.118,
        vec![Panel::LeftPanel]
    );

    // Second: Split off right panel from the rest (~10% width from total)
    let [center_area, _right_panel] = surface.split_right(
        rest,
        0.896,
        vec![Panel::RightPanel]
    );

    // Third: Split center area into viewport (top ~89%) and bottom panel (bottom ~11%)
    let [_viewport, _bottom_panel] = surface.split_below(
        center_area,
        0.893,
        vec![Panel::Viewport]
    );

    // Add BottomPanel to the bottom node
    surface.set_focused_node(_bottom_panel);
    surface.push_to_focused_leaf(Panel::BottomPanel);

    tree
}

pub fn setup_dock(mut commands: Commands) {
    let tree = load_dock_state().unwrap_or_else(|| {
        info!("Creating default dock layout");
        create_default_layout()
    });

    info!("Dock state initialized");
    commands.insert_resource(DockResource {
        tree,
        all_hidden: false,
    });
    commands.init_resource::<crate::ui::ViewportRect>();
    commands.init_resource::<crate::ui::GenomeEditorState>();
}

pub fn is_panel_open(tree: &DockState<Panel>, panel: &Panel) -> bool {
    // Use public API to check all tabs
    tree.iter_all_tabs().any(|(_, tab)| tab == panel)
}

pub fn close_panel(tree: &mut DockState<Panel>, panel: &Panel) {
    // Find the panel location
    if let Some((surface_index, node_index, tab_index)) = tree.find_tab(panel) {
        tree[surface_index].remove_tab((node_index, tab_index));
    }
}

pub fn open_panel(tree: &mut DockState<Panel>, panel: &Panel) {
    // Add the panel to the focused leaf
    tree.main_surface_mut().push_to_focused_leaf(panel.clone());
}

#[derive(Resource)]
pub struct SaveTimer {
    timer: Timer,
}

impl Default for SaveTimer {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs(2), TimerMode::Repeating),
        }
    }
}

pub fn auto_save_dock_state(
    time: Res<Time>,
    mut save_timer: Local<SaveTimer>,
    dock_resource: Res<DockResource>,
) {
    save_timer.timer.tick(time.delta());

    if save_timer.timer.just_finished() {
        save_dock_state(&dock_resource.tree);
    }
}

pub fn save_on_exit(
    dock_resource: Res<DockResource>,
    mut exit_events: MessageReader<bevy::app::AppExit>,
) {
    for _ in exit_events.read() {
        save_dock_state(&dock_resource.tree);
        info!("Saved dock state on exit");
    }
}

pub fn show_windows_menu(ui: &mut bevy_egui::egui::Ui, dock_resource: &mut DockResource, global_ui_state: &mut crate::ui::GlobalUiState) {
    // UI Scale slider
    ui.label("UI Scale:");
    ui.add(bevy_egui::egui::Slider::new(&mut global_ui_state.ui_scale, 0.5..=4.0)
        .text("Scale")
        .suffix("x"));
    
    ui.separator();
    
    // List of genome editor panels that can be toggled
    let genome_editor_panels = [
        Panel::Modes,
        Panel::NameTypeEditor,
        Panel::AdhesionSettings,
        Panel::ParentSettings,
        Panel::CircleSliders,
        Panel::QuaternionBall,
        Panel::TimeSlider,
    ];

    ui.label("Genome Editor:");
    for panel in &genome_editor_panels {
        let is_open = is_panel_open(&dock_resource.tree, panel);
        let panel_name = panel.to_string();
        let is_locked = global_ui_state.locked_windows.contains(&panel_name);

        ui.horizontal(|ui| {
            // Window toggle button
            if ui.selectable_label(is_open, format!("  {}", panel)).clicked() {
                if is_open {
                    close_panel(&mut dock_resource.tree, panel);
                } else {
                    open_panel(&mut dock_resource.tree, panel);
                }
            }
            
            // Lock/Unlock button
            let lock_icon = if is_locked { "🔒" } else { "🔓" };
            if ui.small_button(lock_icon).clicked() {
                if is_locked {
                    global_ui_state.locked_windows.remove(&panel_name);
                } else {
                    global_ui_state.locked_windows.insert(panel_name);
                }
            }
        });
    }

    ui.separator();

    // Placeholder panels (structural layout panels)
    ui.label("Layout Panels:");
    
    let placeholder_panels = [
        Panel::LeftPanel,
        Panel::RightPanel,
        Panel::BottomPanel,
        Panel::Viewport,
    ];
    
    for panel in &placeholder_panels {
        let is_open = is_panel_open(&dock_resource.tree, panel);
        let panel_name = panel.to_string();
        let is_locked = global_ui_state.locked_windows.contains(&panel_name);

        ui.horizontal(|ui| {
            // Window toggle button
            if ui.selectable_label(is_open, format!("  {}", panel)).clicked() {
                if is_open {
                    close_panel(&mut dock_resource.tree, panel);
                } else {
                    open_panel(&mut dock_resource.tree, panel);
                }
            }
            
            // Lock/Unlock button
            let lock_icon = if is_locked { "🔒" } else { "🔓" };
            if ui.small_button(lock_icon).clicked() {
                if is_locked {
                    global_ui_state.locked_windows.remove(&panel_name);
                } else {
                    global_ui_state.locked_windows.insert(panel_name);
                }
            }
        });
    }

    ui.separator();

    // Other windows
    ui.label("Other Windows:");

    // Scene Manager
    let scene_manager_open = is_panel_open(&dock_resource.tree, &Panel::SceneManager);
    let scene_manager_name = Panel::SceneManager.to_string();
    let scene_manager_locked = global_ui_state.locked_windows.contains(&scene_manager_name);
    
    ui.horizontal(|ui| {
        if ui.selectable_label(scene_manager_open, "  Scene Manager").clicked() {
            if scene_manager_open {
                close_panel(&mut dock_resource.tree, &Panel::SceneManager);
            } else {
                open_panel(&mut dock_resource.tree, &Panel::SceneManager);
            }
        }
        
        let lock_icon = if scene_manager_locked { "🔒" } else { "🔓" };
        if ui.small_button(lock_icon).clicked() {
            if scene_manager_locked {
                global_ui_state.locked_windows.remove(&scene_manager_name);
            } else {
                global_ui_state.locked_windows.insert(scene_manager_name);
            }
        }
    });

    ui.separator();

    // Individual hide options
    ui.checkbox(&mut global_ui_state.lock_tab_bar, "Hide Tab Bar");
    ui.checkbox(&mut global_ui_state.lock_tabs, "Hide Tabs");
    ui.checkbox(&mut global_ui_state.lock_close_buttons, "Hide Close Buttons");

    ui.separator();

    // Lock All option - toggles all three settings at once
    let all_locked = global_ui_state.lock_tab_bar 
        && global_ui_state.lock_tabs 
        && global_ui_state.lock_close_buttons;
    
    let lock_all_label = if all_locked {
        "Unlock All"
    } else {
        "Lock All"
    };

    if ui.button(lock_all_label).clicked() {
        let new_state = !all_locked;
        global_ui_state.lock_tab_bar = new_state;
        global_ui_state.lock_tabs = new_state;
        global_ui_state.lock_close_buttons = new_state;
    }

    ui.separator();

    let hide_all_label = if dock_resource.all_hidden {
        "Show All"
    } else {
        "Hide All"
    };

    if ui.button(hide_all_label).clicked() {
        dock_resource.all_hidden = !dock_resource.all_hidden;
    }
}
