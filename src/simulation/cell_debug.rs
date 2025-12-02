//! Cell Debug Tracking System
//! 
//! Outputs mass, lifetime, and nutrient count for the first 32 cells every second.
//! Tracks synchronization of cell splitting with deferral compensation.

use bevy::prelude::*;
use super::cpu_physics::CanonicalState;

/// Resource for tracking debug output timing
#[derive(Resource)]
pub struct CellDebugTracker {
    /// Time since last debug output
    pub last_output_time: f32,
    /// Output interval in seconds
    pub output_interval: f32,
    /// Whether debug output is enabled
    pub enabled: bool,
    /// Number of cells to track (max 32)
    pub cells_to_track: usize,
}

impl Default for CellDebugTracker {
    fn default() -> Self {
        Self {
            last_output_time: 0.0,
            output_interval: 1.0, // Output every 1 second
            enabled: true,
            cells_to_track: 32,
        }
    }
}

/// Plugin for cell debug tracking
pub struct CellDebugPlugin;

impl Plugin for CellDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CellDebugTracker>()
            .add_systems(Update, output_cell_debug_info);
    }
}

/// Output debug info for first 32 cells every second
fn output_cell_debug_info(
    mut tracker: ResMut<CellDebugTracker>,
    main_sim_state: Option<Res<super::cpu_sim::MainSimState>>,
    preview_sim_state: Option<Res<super::preview_sim::PreviewSimState>>,
    sim_state: Option<Res<super::SimulationState>>,
    genome: Option<Res<crate::genome::CurrentGenome>>,
) {
    if !tracker.enabled {
        return;
    }

    // Get the appropriate canonical state based on simulation mode
    let (canonical_state, sim_time) = if let Some(state) = sim_state.as_ref() {
        match state.mode {
            super::SimulationMode::Preview => {
                if let Some(preview) = preview_sim_state.as_ref() {
                    (&preview.canonical_state, preview.current_time)
                } else {
                    return;
                }
            }
            super::SimulationMode::Cpu => {
                if let Some(main) = main_sim_state.as_ref() {
                    (&main.canonical_state, main.simulation_time)
                } else {
                    return;
                }
            }
            super::SimulationMode::Gpu => {
                // GPU mode doesn't use canonical state
                return;
            }
        }
    } else if let Some(main) = main_sim_state.as_ref() {
        (&main.canonical_state, main.simulation_time)
    } else {
        return;
    };

    // Check if it's time to output
    if sim_time - tracker.last_output_time < tracker.output_interval {
        return;
    }
    tracker.last_output_time = sim_time;

    // Get genome for mode info
    let genome_data = genome.as_ref().map(|g| &g.genome);

    // Output header
    println!("\n========== CELL DEBUG @ {:.2}s ==========", sim_time);
    println!("Total cells: {}", canonical_state.cell_count);
    println!("{:<4} {:>8} {:>8} {:>10} {:>10} {:>8} {:>8}",
        "ID", "Mass", "Radius", "Lifetime", "SplitInt", "Mode", "Ready");
    println!("{}", "-".repeat(70));

    // Track cells for synchronization analysis
    let cells_to_show = tracker.cells_to_track.min(canonical_state.cell_count);
    
    let mut total_mass = 0.0;
    let mut ready_count = 0;
    let mut deferred_count = 0;

    for i in 0..cells_to_show {
        let cell_id = canonical_state.cell_ids[i];
        let mass = canonical_state.masses[i];
        let radius = canonical_state.radii[i];
        let birth_time = canonical_state.birth_times[i];
        let split_interval = canonical_state.split_intervals[i];
        let mode_index = canonical_state.mode_indices[i];
        let _split_ready_frame = canonical_state.split_ready_frame[i];
        
        // Calculate lifetime (age)
        let lifetime = sim_time - birth_time;
        
        // Check if cell is ready to split
        let time_ready = lifetime >= split_interval;
        
        // Check mass threshold
        let mass_ready = if let Some(genome) = genome_data {
            if let Some(mode) = genome.modes.get(mode_index) {
                mass >= mode.split_mass
            } else {
                true
            }
        } else {
            true
        };
        
        let ready_status = if time_ready && mass_ready {
            ready_count += 1;
            "YES"
        } else if time_ready && !mass_ready {
            deferred_count += 1;
            "DEFER" // Waiting for nutrients
        } else {
            "NO"
        };

        total_mass += mass;

        println!("{:<4} {:>8.3} {:>8.3} {:>10.3} {:>10.3} {:>8} {:>8}",
            cell_id,
            mass,
            radius,
            lifetime,
            split_interval,
            mode_index,
            ready_status
        );
    }

    // Summary statistics
    println!("{}", "-".repeat(70));
    println!("Summary: Total Mass={:.3}, Ready={}, Deferred={}, Avg Mass={:.3}",
        total_mass,
        ready_count,
        deferred_count,
        if cells_to_show > 0 { total_mass / cells_to_show as f32 } else { 0.0 }
    );

    // Check synchronization - cells should split at similar times
    if cells_to_show > 1 {
        let mut lifetimes: Vec<f32> = (0..cells_to_show)
            .map(|i| sim_time - canonical_state.birth_times[i])
            .collect();
        lifetimes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let min_lifetime = lifetimes.first().copied().unwrap_or(0.0);
        let max_lifetime = lifetimes.last().copied().unwrap_or(0.0);
        let lifetime_spread = max_lifetime - min_lifetime;
        
        println!("Sync: Lifetime spread={:.3}s (min={:.3}, max={:.3})",
            lifetime_spread, min_lifetime, max_lifetime);
        
        if lifetime_spread > 0.5 {
            println!("⚠ WARNING: Cells are desynchronized (spread > 0.5s)");
        } else {
            println!("✓ Cells are synchronized");
        }
    }

    println!("==========================================\n");
}

/// Helper function to print a single cell's state (can be called from anywhere)
pub fn debug_print_cell(state: &CanonicalState, index: usize, sim_time: f32) {
    if index >= state.cell_count {
        println!("Cell {} does not exist (count={})", index, state.cell_count);
        return;
    }
    
    let lifetime = sim_time - state.birth_times[index];
    println!("Cell[{}]: id={}, mass={:.3}, radius={:.3}, lifetime={:.3}, split_interval={:.3}, mode={}",
        index,
        state.cell_ids[index],
        state.masses[index],
        state.radii[index],
        lifetime,
        state.split_intervals[index],
        state.mode_indices[index]
    );
}
