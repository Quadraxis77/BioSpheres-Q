//! Nutrient transport system
//! 
//! This module provides nutrient growth and transport between cells.
//! Each cell grows independently, then nutrients flow between connected cells
//! based on priority ratios to establish equilibrium.

use super::cpu_physics::CanonicalState;

/// Transport nutrients with individual cell growth (no cohort synchronization)
/// 
/// Each cell gains/loses nutrients independently based on its mode settings.
/// Nutrients then flow between adhesion-connected cells based on priority ratios.
pub fn transport_nutrients_synchronized(
    state: &mut CanonicalState,
    genome: &crate::genome::GenomeData,
    dt: f32,
) {
    // Step 1: Individual nutrient gain for Test cells
    crate::simulation::nutrient_system::update_nutrient_growth_st(
        &mut state.masses[..state.cell_count],
        &mut state.radii[..state.cell_count],
        &state.mode_indices[..state.cell_count],
        genome,
        dt,
    );
    
    // Step 2: Consume nutrients for Flagellocytes with swim force
    let dead_cells = crate::simulation::nutrient_system::consume_swim_nutrients_st(
        &mut state.masses[..state.cell_count],
        &mut state.radii[..state.cell_count],
        &state.mode_indices[..state.cell_count],
        genome,
        dt,
    );
    
    // Remove dead cells (in reverse order to maintain indices)
    for &cell_idx in dead_cells.iter().rev() {
        crate::simulation::nutrient_system::remove_dead_cell(state, cell_idx);
    }
    
    // Step 3: Transport nutrients between adhesion-connected cells
    // Nutrients flow to establish equilibrium based on priority ratios
    crate::simulation::nutrient_system::transport_nutrients_st(state, genome, dt);
}
