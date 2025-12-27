use bevy::prelude::*;
use crate::simulation::{SimulationState, SimulationMode};
use crate::simulation::preview_sim::PreviewSimState;
use crate::ui::ui_system::GenomeEditorState;

/// System that syncs time slider UI to simulation
/// Runs when user drags the time slider, triggers resimulation to that point
pub fn sync_time_slider_to_simulation(
    genome_editor_state: Res<GenomeEditorState>,
    mut sim_state: ResMut<SimulationState>,
    mut last_time_value: Local<f32>,
) {
    // Only run in Preview mode
    if sim_state.mode != SimulationMode::Preview {
        return;
    }

    // Check if time_value actually changed (not just any field in GenomeEditorState)
    let current_time_value = genome_editor_state.time_value;
    if (current_time_value - *last_time_value).abs() > 0.01 {
        // Convert slider value (0-100) to simulation time (seconds)
        let target_sim_time = (current_time_value / 100.0)
            * genome_editor_state.max_preview_duration;

        // Only update if different from current target (avoid redundant resimulations)
        let needs_update = match sim_state.target_time {
            Some(current_target) => (current_target - target_sim_time).abs() > 0.01,
            None => true,
        };

        if needs_update {
            sim_state.target_time = Some(target_sim_time);
        }

        *last_time_value = current_time_value;
    }
}

/// System that syncs simulation time back to slider
/// Updates slider when simulation advances (e.g., during playback)
pub fn sync_simulation_to_time_slider(
    mut genome_editor_state: ResMut<GenomeEditorState>,
    sim_state: Res<SimulationState>,
    preview_state: Res<PreviewSimState>,
    mut last_sim_time: Local<f32>,
) {
    // Only run in Preview mode
    if sim_state.mode != SimulationMode::Preview {
        return;
    }

    // Skip if currently resimulating (let user control finish)
    if sim_state.is_resimulating {
        return;
    }

    // Skip if slider is being actively dragged (prevent fighting user input)
    if genome_editor_state.time_slider_dragging {
        return;
    }

    // Only update if simulation time actually changed
    let current_sim_time = preview_state.current_time;
    if (current_sim_time - *last_sim_time).abs() > 0.01 {
        let slider_value = if genome_editor_state.max_preview_duration > 0.0 {
            (current_sim_time / genome_editor_state.max_preview_duration) * 100.0
        } else {
            0.0
        };
        let slider_value = slider_value.clamp(0.0, 100.0);

        // Only update if significantly different (avoid jitter)
        if (genome_editor_state.time_value - slider_value).abs() > 0.1 {
            genome_editor_state.time_value = slider_value;
        }

        *last_sim_time = current_sim_time;
    }
}
