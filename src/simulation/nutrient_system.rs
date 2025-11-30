use bevy::prelude::*;
use super::cpu_physics::CanonicalState;

/// Update cell mass and radius based on nutrient gain (for Test cells) - Single-threaded
/// Test cells (cell_type == 0) automatically gain mass over time and grow in size
pub fn update_nutrient_growth_st(
    masses: &mut [f32],
    radii: &mut [f32],
    mode_indices: &[usize],
    genome: &crate::genome::GenomeData,
    dt: f32,
) {
    for i in 0..masses.len() {
        let mode_index = mode_indices[i];
        if let Some(mode) = genome.modes.get(mode_index) {
            // Only apply nutrient growth to Test cells (cell_type == 0)
            if mode.cell_type == 0 {
                // Gain mass based on nutrient gain rate
                let mass_gain = mode.nutrient_gain_rate * dt;
                masses[i] += mass_gain;
                
                // Calculate target radius based on mass (linear relationship)
                // Clamp to max_cell_size
                let target_radius = masses[i].min(mode.max_cell_size);
                radii[i] = target_radius.clamp(1.0, 2.0);
            }
        }
    }
}

/// Update cell mass and radius based on nutrient gain (for Test cells) - Multithreaded
/// Test cells (cell_type == 0) automatically gain mass over time and grow in size
pub fn update_nutrient_growth(
    masses: &mut [f32],
    radii: &mut [f32],
    mode_indices: &[usize],
    genome: &crate::genome::GenomeData,
    dt: f32,
) {
    use rayon::prelude::*;
    
    masses.par_iter_mut()
        .zip(radii.par_iter_mut())
        .zip(mode_indices.par_iter())
        .for_each(|((mass, radius), mode_index)| {
            if let Some(mode) = genome.modes.get(*mode_index) {
                // Only apply nutrient growth to Test cells (cell_type == 0)
                if mode.cell_type == 0 {
                    // Gain mass based on nutrient gain rate
                    let mass_gain = mode.nutrient_gain_rate * dt;
                    *mass += mass_gain;
                    
                    // Calculate target radius based on mass (linear relationship)
                    // Clamp to max_cell_size
                    let target_radius = (*mass).min(mode.max_cell_size);
                    *radius = target_radius.clamp(1.0, 2.0);
                }
            }
        });
}

/// Consume nutrients for Flagellocyte cells based on swim force - Single-threaded
/// Flagellocytes (cell_type == 1) consume mass proportional to their swim force
/// Returns a list of cell indices that died (mass <= 0)
pub fn consume_swim_nutrients_st(
    masses: &mut [f32],
    radii: &mut [f32],
    mode_indices: &[usize],
    genome: &crate::genome::GenomeData,
    dt: f32,
) -> Vec<usize> {
    let mut cells_to_remove = Vec::new();
    
    for i in 0..masses.len() {
        let mode_index = mode_indices[i];
        if let Some(mode) = genome.modes.get(mode_index) {
            // Only apply nutrient consumption to Flagellocyte cells (cell_type == 1)
            if mode.cell_type == 1 && mode.swim_force > 0.0 {
                // Consume mass proportional to swim force
                // Consumption rate: 0.2 mass per second at full swim force (1.0)
                let consumption_rate = 0.2;
                let mass_loss = mode.swim_force * consumption_rate * dt;
                masses[i] -= mass_loss;
                
                // Check if cell has died
                if masses[i] <= 0.0 {
                    cells_to_remove.push(i);
                    continue;
                }
                
                // Update radius based on new mass
                // Flagellocytes have a minimum visual size of 0.5 regardless of mass
                let target_radius = masses[i].min(mode.max_cell_size);
                radii[i] = target_radius.clamp(0.5, 2.0);
            }
        }
    }
    
    cells_to_remove
}

/// Consume nutrients for Flagellocyte cells based on swim force - Multithreaded
/// Flagellocytes (cell_type == 1) consume mass proportional to their swim force
/// Returns a list of cell indices that died (mass <= 0)
pub fn consume_swim_nutrients(
    masses: &mut [f32],
    radii: &mut [f32],
    mode_indices: &[usize],
    genome: &crate::genome::GenomeData,
    dt: f32,
) -> Vec<usize> {
    use rayon::prelude::*;
    use std::sync::Mutex;
    
    let cells_to_remove = Mutex::new(Vec::new());
    
    masses.par_iter_mut()
        .zip(radii.par_iter_mut())
        .zip(mode_indices.par_iter())
        .enumerate()
        .for_each(|(i, ((mass, radius), mode_index))| {
            if let Some(mode) = genome.modes.get(*mode_index) {
                // Only apply nutrient consumption to Flagellocyte cells (cell_type == 1)
                if mode.cell_type == 1 && mode.swim_force > 0.0 {
                    // Consume mass proportional to swim force
                    // Consumption rate: 0.2 mass per second at full swim force (1.0)
                    let consumption_rate = 0.2;
                    let mass_loss = mode.swim_force * consumption_rate * dt;
                    *mass -= mass_loss;
                    
                    // Check if cell has died
                    if *mass <= 0.0 {
                        cells_to_remove.lock().unwrap().push(i);
                        return;
                    }
                    
                    // Update radius based on new mass
                    // Flagellocytes have a minimum visual size of 0.5 regardless of mass
                    let target_radius = (*mass).min(mode.max_cell_size);
                    *radius = target_radius.clamp(0.5, 2.0);
                }
            }
        });
    
    cells_to_remove.into_inner().unwrap()
}

/// Transport nutrients between adhesion-connected cells - Single-threaded
/// Nutrients flow based on weighted mass difference: from high-mass to low-mass cells,
/// and from low-priority to high-priority cells. The flow rate is proportional to
/// the weighted difference using opposite cell's priority as the weight.
pub fn transport_nutrients_st(
    state: &mut CanonicalState,
    genome: &crate::genome::GenomeData,
    dt: f32,
) {
    // Calculate mass changes for each cell (accumulate transfers)
    let mut mass_deltas = vec![0.0f32; state.cell_count];
    
    // Process each active adhesion connection
    let adhesion_capacity = state.adhesion_connections.is_active.len();
    for adhesion_idx in 0..adhesion_capacity {
        if state.adhesion_connections.is_active[adhesion_idx] == 0 {
            continue;
        }
        
        let cell_a_idx = state.adhesion_connections.cell_a_index[adhesion_idx];
        let cell_b_idx = state.adhesion_connections.cell_b_index[adhesion_idx];
        
        // Skip if either cell is out of range
        if cell_a_idx >= state.cell_count || cell_b_idx >= state.cell_count {
            continue;
        }
        
        // Get mode settings for both cells
        let mode_a = genome.modes.get(state.mode_indices[cell_a_idx]);
        let mode_b = genome.modes.get(state.mode_indices[cell_b_idx]);
        
        // Only transport between Test cells
        let is_test_a = mode_a.map(|m| m.cell_type == 0).unwrap_or(false);
        let is_test_b = mode_b.map(|m| m.cell_type == 0).unwrap_or(false);
        
        if !is_test_a || !is_test_b {
            continue;
        }
        
        // Get base priorities (default to 1.0 if mode not found)
        let base_priority_a = mode_a.map(|m| m.nutrient_priority).unwrap_or(1.0);
        let base_priority_b = mode_b.map(|m| m.nutrient_priority).unwrap_or(1.0);
        
        // Get prioritize_when_low flags
        let prioritize_a = mode_a.map(|m| m.prioritize_when_low).unwrap_or(true);
        let prioritize_b = mode_b.map(|m| m.prioritize_when_low).unwrap_or(true);
        
        // Get masses
        let mass_a = state.masses[cell_a_idx];
        let mass_b = state.masses[cell_b_idx];
        
        // Apply dynamic priority boost when cells are dangerously low on nutrients
        // Threshold: below 0.3 mass is considered "dangerously low"
        // Boost: multiply priority by 10x when below threshold
        let danger_threshold = 0.3;
        let priority_boost = 10.0;
        
        let priority_a = if prioritize_a && mass_a < danger_threshold {
            base_priority_a * priority_boost
        } else {
            base_priority_a
        };
        
        let priority_b = if prioritize_b && mass_b < danger_threshold {
            base_priority_b * priority_boost
        } else {
            base_priority_b
        };
        
        // Calculate weighted mass difference
        // Weight A's mass by B's priority, and B's mass by A's priority
        // This creates flow from low-priority to high-priority cells
        let weighted_mass_a = mass_a * priority_b;
        let weighted_mass_b = mass_b * priority_a;
        
        // Flow is proportional to weighted difference
        // Positive flow means A -> B, negative means B -> A
        let weighted_diff = weighted_mass_a - weighted_mass_b;
        
        // Transport rate constant (tune this for desired equilibration speed)
        // Higher values = faster equilibration
        let transport_rate = 0.5;
        
        // Calculate mass transfer (positive = A loses, B gains)
        let mass_transfer = weighted_diff * transport_rate * dt;
        
        // Apply transfer with different minimum thresholds based on prioritize_when_low
        let min_mass_a = if prioritize_a { 0.1 } else { 0.0 };
        let min_mass_b = if prioritize_b { 0.1 } else { 0.0 };
        
        let actual_transfer = if mass_transfer > 0.0 {
            // A -> B: limit by A's mass (respect minimum threshold)
            mass_transfer.min(mass_a - min_mass_a)
        } else {
            // B -> A: limit by B's mass (respect minimum threshold)
            mass_transfer.max(-(mass_b - min_mass_b))
        };
        
        // Accumulate deltas
        mass_deltas[cell_a_idx] -= actual_transfer;
        mass_deltas[cell_b_idx] += actual_transfer;
    }
    
    // Apply mass changes and update radii
    // Track cells that die (mass <= 0)
    let mut cells_to_remove = Vec::new();
    
    for i in 0..state.cell_count {
        if mass_deltas[i].abs() > 0.0001 {
            state.masses[i] += mass_deltas[i];
            
            // Check if cell has died (mass <= 0)
            if state.masses[i] <= 0.0 {
                cells_to_remove.push(i);
                continue;
            }
            
            // Update radius based on new mass
            if let Some(mode) = genome.modes.get(state.mode_indices[i]) {
                if mode.cell_type == 0 {
                    let target_radius = state.masses[i].min(mode.max_cell_size);
                    state.radii[i] = target_radius.clamp(1.0, 2.0);
                }
            }
        }
    }
    
    // Remove dead cells (in reverse order to maintain indices)
    for &cell_idx in cells_to_remove.iter().rev() {
        remove_dead_cell(state, cell_idx);
    }
}

/// Remove a dead cell from the canonical state
/// Uses swap-and-pop strategy: swap with last cell, then decrement count
pub fn remove_dead_cell(state: &mut CanonicalState, cell_idx: usize) {
    if cell_idx >= state.cell_count {
        return;
    }
    
    // Remove all adhesion connections for this cell
    state.adhesion_manager.remove_all_connections_for_cell(&mut state.adhesion_connections, cell_idx);
    
    let last_idx = state.cell_count - 1;
    
    if cell_idx != last_idx {
        // Swap with last cell
        state.cell_ids[cell_idx] = state.cell_ids[last_idx];
        state.positions[cell_idx] = state.positions[last_idx];
        state.prev_positions[cell_idx] = state.prev_positions[last_idx];
        state.velocities[cell_idx] = state.velocities[last_idx];
        state.masses[cell_idx] = state.masses[last_idx];
        state.radii[cell_idx] = state.radii[last_idx];
        state.genome_ids[cell_idx] = state.genome_ids[last_idx];
        state.mode_indices[cell_idx] = state.mode_indices[last_idx];
        state.rotations[cell_idx] = state.rotations[last_idx];
        state.angular_velocities[cell_idx] = state.angular_velocities[last_idx];
        state.genome_orientations[cell_idx] = state.genome_orientations[last_idx];
        state.forces[cell_idx] = state.forces[last_idx];
        state.torques[cell_idx] = state.torques[last_idx];
        state.accelerations[cell_idx] = state.accelerations[last_idx];
        state.prev_accelerations[cell_idx] = state.prev_accelerations[last_idx];
        state.stiffnesses[cell_idx] = state.stiffnesses[last_idx];
        state.birth_times[cell_idx] = state.birth_times[last_idx];
        state.split_intervals[cell_idx] = state.split_intervals[last_idx];
        state.split_counts[cell_idx] = state.split_counts[last_idx];
        
        // Update adhesion indices: all references to last_idx should now point to cell_idx
        if last_idx < state.adhesion_manager.cell_adhesion_indices.len() {
            state.adhesion_manager.cell_adhesion_indices[cell_idx] = 
                state.adhesion_manager.cell_adhesion_indices[last_idx];
            
            // Update all adhesion connections that reference last_idx to now reference cell_idx
            for adhesion_idx in 0..state.adhesion_connections.is_active.len() {
                if state.adhesion_connections.is_active[adhesion_idx] == 0 {
                    continue;
                }
                
                if state.adhesion_connections.cell_a_index[adhesion_idx] == last_idx {
                    state.adhesion_connections.cell_a_index[adhesion_idx] = cell_idx;
                }
                if state.adhesion_connections.cell_b_index[adhesion_idx] == last_idx {
                    state.adhesion_connections.cell_b_index[adhesion_idx] = cell_idx;
                }
            }
        }
    }
    
    // Clear the adhesion indices for the removed cell slot
    if cell_idx < state.adhesion_manager.cell_adhesion_indices.len() {
        state.adhesion_manager.init_cell_adhesion_indices(last_idx);
    }
    
    // Decrement cell count
    state.cell_count -= 1;
}

/// Transport nutrients between adhesion-connected cells - Multithreaded
/// This is a simplified parallel version that processes adhesions in parallel
pub fn transport_nutrients(
    state: &mut CanonicalState,
    genome: &crate::genome::GenomeData,
    dt: f32,
) {
    // For thread safety, we use the single-threaded version
    // A fully parallel version would require atomic operations or more complex synchronization
    transport_nutrients_st(state, genome, dt);
}
