//! Synchronization-aware nutrient transport system
//! 
//! This module provides nutrient transport that maintains perfect synchronization
//! across cells born at the same time (cohorts).

use super::cpu_physics::CanonicalState;
use bevy::prelude::*;

/// Transport nutrients while maintaining cohort synchronization
/// 
/// Cells born at the same time form a "cohort" that shares nutrients equally.
/// This ensures they accumulate mass at the same rate and split simultaneously.
pub fn transport_nutrients_synchronized(
    state: &mut CanonicalState,
    genome: &crate::genome::GenomeData,
    dt: f32,
) {
    // Step 1: Group cells into cohorts by birth_time
    let mut cohorts: std::collections::HashMap<i32, Vec<usize>> = std::collections::HashMap::new();
    for i in 0..state.cell_count {
        // Quantize birth_time to 0.01s buckets for grouping
        let birth_time_key = (state.birth_times[i] * 100.0) as i32;
        cohorts.entry(birth_time_key).or_insert_with(Vec::new).push(i);
    }
    
    // Step 2: For each cohort, pool all nutrients and redistribute equally
    for (_birth_time_key, cell_indices) in cohorts.iter() {
        if cell_indices.is_empty() {
            continue;
        }
        
        // Calculate total mass in cohort (for reference)
        let _total_mass: f32 = cell_indices.iter().map(|&idx| state.masses[idx]).sum();
        
        // Calculate nutrient gain/loss for cohort (sum of all individual gains and losses)
        let mut cohort_nutrient_gain = 0.0f32;
        for &idx in cell_indices.iter() {
            let mode_index = state.mode_indices[idx];
            if let Some(mode) = genome.modes.get(mode_index) {
                if mode.cell_type == 0 {
                    // Test cells gain nutrients
                    let storage_cap = mode.split_mass * 2.0;
                    if state.masses[idx] < storage_cap {
                        cohort_nutrient_gain += mode.nutrient_gain_rate * dt;
                    }
                } else if mode.cell_type == 1 && mode.swim_force > 0.0 {
                    // Flagellocytes consume nutrients
                    let consumption_rate = 0.2;
                    let mass_loss = mode.swim_force * consumption_rate * dt;
                    cohort_nutrient_gain -= mass_loss;
                }
            }
        }
        
        // Distribute gain equally across cohort
        let per_cell_gain = cohort_nutrient_gain / cell_indices.len() as f32;
        
        // Apply gain to each cell in cohort
        for &idx in cell_indices.iter() {
            state.masses[idx] += per_cell_gain;
            
            // Update radius
            let mode_index = state.mode_indices[idx];
            if let Some(mode) = genome.modes.get(mode_index) {
                let target_radius = state.masses[idx].min(mode.max_cell_size);
                state.radii[idx] = target_radius.clamp(0.5, 2.0);
            }
        }
        
        // After nutrient gain/loss, equalize mass within cohort to maintain perfect sync
        let new_total_mass: f32 = cell_indices.iter().map(|&idx| state.masses[idx]).sum();
        let avg_mass = new_total_mass / cell_indices.len() as f32;
        
        // Check if cohort has died (average mass below minimum threshold)
        const MIN_CELL_MASS: f32 = 0.5;
        if avg_mass < MIN_CELL_MASS {
            // Mark all cells in cohort for removal
            let mut cells_to_remove: Vec<usize> = cell_indices.clone();
            cells_to_remove.sort_by(|a, b| b.cmp(a)); // Sort in reverse order
            for &cell_idx in cells_to_remove.iter() {
                crate::simulation::nutrient_system::remove_dead_cell(state, cell_idx);
            }
            continue;
        }
        
        for &idx in cell_indices.iter() {
            state.masses[idx] = avg_mass;
            
            // Update radius to match equalized mass
            let mode_index = state.mode_indices[idx];
            if let Some(mode) = genome.modes.get(mode_index) {
                let target_radius = avg_mass.min(mode.max_cell_size);
                state.radii[idx] = target_radius.clamp(0.5, 2.0);
            }
        }
    }
    
    // Step 3: Transport nutrients between adhesion-connected cells
    // This allows nutrients to flow from Test cells to Flagellocytes they're connected to
    // Uses priority-based equilibrium transport (higher priority cells receive more nutrients)
    crate::simulation::nutrient_system::transport_nutrients_st(state, genome, dt);
}
