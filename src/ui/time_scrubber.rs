use bevy::prelude::*;
use bevy_mod_imgui::ImguiContext;
use imgui::{self, StyleColor};
use crate::simulation::preview_sim::PreviewSimState;
use crate::simulation::{SimulationState, PhysicsConfig};

/// Time scrubber plugin for preview simulation
pub struct TimeScrubberPlugin;

impl Plugin for TimeScrubberPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TimeScrubberState>()
            .add_systems(Update, render_time_scrubber);
    }
}

/// State for the time scrubber UI
#[derive(Resource)]
pub struct TimeScrubberState {
    /// Maximum time to display on the scrubber (in seconds)
    pub max_time: f32,
    /// Whether the scrubber is being actively dragged
    pub is_dragging: bool,
}

impl Default for TimeScrubberState {
    fn default() -> Self {
        Self {
            max_time: 50.0,
            is_dragging: false,
        }
    }
}

/// Render the time scrubber UI panel
fn render_time_scrubber(
    mut imgui_context: NonSendMut<ImguiContext>,
    mut scrubber_state: ResMut<TimeScrubberState>,
    preview_state: Res<PreviewSimState>,
    mut sim_state: ResMut<SimulationState>,
    config: Res<PhysicsConfig>,
    global_ui_state: Res<super::GlobalUiState>,
) {
    // Only show time scrubber in Preview mode
    if sim_state.mode != crate::simulation::SimulationMode::Preview {
        return;
    }

    let ui = imgui_context.ui();
    
    // Build flags based on lock state
    // Only show if visibility is enabled
    if !global_ui_state.show_time_scrubber {
        return;
    }

    use imgui::WindowFlags;
    let flags = if global_ui_state.windows_locked {
        WindowFlags::NO_MOVE | WindowFlags::NO_RESIZE
    } else {
        WindowFlags::empty()
    };
    
    // Create time scrubber window
    ui.window("Time Scrubber")
        .size([1005.0, 210.0], imgui::Condition::FirstUseEver)
        .position([610.0, 715.0], imgui::Condition::FirstUseEver)
        .flags(flags)
        .build(|| {
            let mut current_time = preview_state.current_time;
            
            // Time display
            ui.text(format!("Current Time: {:.2}s", current_time));
            ui.same_line();
            ui.text(format!("/ {:.0}s", scrubber_state.max_time));
            
            ui.separator();
            
            // Main time slider
            ui.text("Scrub Time:");
            ui.set_next_item_width(-1.0); // Full width
            
            let slider_changed = ui
                .slider_config("##time_slider", 0.0, scrubber_state.max_time)
                .display_format("%.2fs")
                .build(&mut current_time);
            
            // Check if slider is being actively dragged
            let is_active = ui.is_item_active();
            
            if slider_changed {
                // Update target time for resimulation
                sim_state.target_time = Some(current_time);
                scrubber_state.is_dragging = is_active;
            } else if scrubber_state.is_dragging && !is_active {
                // Just finished dragging
                scrubber_state.is_dragging = false;
            }
            
            ui.separator();
            
            // Info about simulation state
            if sim_state.is_resimulating {
                let col_yellow = ui.style_color(StyleColor::PlotHistogram);
                ui.text_colored(col_yellow, "Simulating...");
            } else {
                let col_green = ui.style_color(StyleColor::PlotLines);
                ui.text_colored(col_green, "Ready");
            }
            
            // Display simulation info
            ui.separator();
            let cell_count = preview_state.canonical_state.cell_count;
            let max_capacity = preview_state.initial_state.max_cells;
            ui.text(format!("Cells: {} / {}", cell_count, max_capacity));
            ui.same_line_with_spacing(0.0, 20.0);
            ui.text(format!("Timestep: {:.4}s", config.fixed_timestep));
        });
}
