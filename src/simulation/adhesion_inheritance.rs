use bevy::prelude::*;
use crate::cell::{AdhesionZone, classify_bond_direction};
use crate::simulation::cpu_physics::CanonicalState;
use crate::genome::GenomeData;

/// Handle adhesion inheritance during cell division
/// 
/// This function processes all adhesions from the parent cell and determines
/// which child(ren) should inherit each connection based on zone classification.
/// 
/// Zone inheritance rules:
/// - Zone A: Inherit to child B (adhesions pointing opposite to split direction)
/// - Zone B: Inherit to child A (adhesions pointing same as split direction)
/// - Zone C: Inherit to both children (adhesions in equatorial band)
/// 
/// CRITICAL: parent_genome_orientation must be the parent's orientation BEFORE division,
/// since child A overwrites the parent's slot and changes the genome orientation.
pub fn inherit_adhesions_on_division(
    state: &mut CanonicalState,
    genome: &GenomeData,
    parent_idx: usize,
    child_a_idx: usize,
    child_b_idx: usize,
    parent_genome_orientation: Quat,
) {
    // Debug: Log inheritance attempt
    let parent_connections_debug: Vec<_> = (0..crate::cell::MAX_ADHESIONS_PER_CELL)
        .filter_map(|slot_idx| {
            let conn_idx = state.adhesion_manager.cell_adhesion_indices[parent_idx][slot_idx];
            if conn_idx >= 0 { Some(conn_idx) } else { None }
        })
        .collect();
    
    // Get parent mode settings
    let parent_mode_idx = state.mode_indices[parent_idx];
    let parent_mode = match genome.modes.get(parent_mode_idx) {
        Some(mode) => mode,
        None => return, // Invalid mode
    };
    
    // Check if children keep adhesions
    let child_a_keep = parent_mode.child_a.keep_adhesion;
    let child_b_keep = parent_mode.child_b.keep_adhesion;
    
    if !child_a_keep && !child_b_keep {
        return; // No inheritance needed
    }
    
    // Get parent properties
    let parent_radius = state.radii[parent_idx];
    // CRITICAL: parent_genome_orientation is passed as parameter, not read from state
    // because child A has already overwritten the parent's slot with its own orientation
    
    // Calculate split direction from parent mode (in local space)
    let pitch = parent_mode.parent_split_direction.x.to_radians();
    let yaw = parent_mode.parent_split_direction.y.to_radians();
    let split_direction_local = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0) * Vec3::Z;
    
    // Extract split direction and offset for geometric calculations (matching C++)
    let split_magnitude = split_direction_local.length();
    let split_dir_parent = if split_magnitude < 0.0001 {
        Vec3::Z
    } else {
        split_direction_local / split_magnitude
    };
    let split_offset_magnitude = if split_magnitude < 0.0001 {
        0.0
    } else {
        split_magnitude * 0.5
    };
    
    // Note: Child genome orientations are already set in the state by division_step
    // We pass the orientation DELTA from genome mode to create_inherited_adhesion
    
    // CRITICAL: Collect parent's adhesion connections BEFORE initializing child indices
    // (since child A reuses parent index, initializing would clear the connections)
    let mut parent_connections = Vec::new();
    for slot_idx in 0..crate::cell::MAX_ADHESIONS_PER_CELL {
        let connection_idx = state.adhesion_manager.cell_adhesion_indices[parent_idx][slot_idx];
        if connection_idx >= 0 {
            parent_connections.push(connection_idx as usize);
        }
    }
    
    // Initialize adhesion indices for child cells (matches C++ Requirement 10.4)
    // This clears the parent's old adhesion indices
    // MUST happen AFTER collecting parent connections
    state.adhesion_manager.init_cell_adhesion_indices(child_a_idx);
    state.adhesion_manager.init_cell_adhesion_indices(child_b_idx);
    
    // Process each parent connection
    for &connection_idx in &parent_connections {
        if connection_idx >= state.adhesion_connections.active_count {
            continue;
        }
        
        if state.adhesion_connections.is_active[connection_idx] == 0 {
            continue;
        }
        
        // Determine which cell is the neighbor (not the parent)
        let cell_a_idx = state.adhesion_connections.cell_a_index[connection_idx];
        let cell_b_idx = state.adhesion_connections.cell_b_index[connection_idx];
        
        let (neighbor_idx, parent_is_a) = if cell_a_idx == parent_idx {
            (cell_b_idx, true)
        } else if cell_b_idx == parent_idx {
            (cell_a_idx, false)
        } else {
            continue; // Connection doesn't involve parent
        };
        
        // Get neighbor properties
        // In C++, child A reuses parent index, so neighborIndex automatically points to correct cell
        // We now match this behavior in Rust
        let neighbor_radius = state.radii[neighbor_idx];
        
        // CRITICAL: Use stored anchor directions in LOCAL SPACE (matches C++ implementation)
        // C++ uses the anchor direction stored in the adhesion connection, NOT world-space positions
        let (parent_anchor_direction, neighbor_anchor_direction) = if parent_is_a {
            (
                state.adhesion_connections.anchor_direction_a[connection_idx],
                state.adhesion_connections.anchor_direction_b[connection_idx],
            )
        } else {
            (
                state.adhesion_connections.anchor_direction_b[connection_idx],
                state.adhesion_connections.anchor_direction_a[connection_idx],
            )
        };
        
        // CRITICAL: Classify using LOCAL anchor direction and splitDirection from genome
        // This matches C++ implementation exactly - zones are classified in parent's local frame
        let zone = classify_bond_direction(parent_anchor_direction, split_direction_local);
        
        // Get connection properties
        // IMPORTANT: Use the CHILD's mode index, not the old connection's mode index
        // This ensures adhesion settings match the child's current mode after division
        let child_a_mode_idx = state.mode_indices[child_a_idx];
        let child_b_mode_idx = state.mode_indices[child_b_idx];
        
        // Inherit based on zone classification (matches C++ lines 170-340)
        // CRITICAL: Child A is at +offset, Child B is at -offset
        // Zone A (opposite to split) → Child B (at -offset)
        // Zone B (same as split) → Child A (at +offset)
        match zone {
            AdhesionZone::ZoneA if child_b_keep => {
                // Zone A → Child B (adhesions pointing opposite to split stay with child at opposite side)
                create_inherited_adhesion(
                    state,
                    genome,
                    child_b_idx,
                    neighbor_idx,
                    child_b_mode_idx,  // Use child B's current mode index
                    parent_is_a,
                    parent_idx,
                    parent_mode,
                    parent_genome_orientation,
                    parent_anchor_direction,
                    neighbor_anchor_direction,
                    parent_radius,
                    neighbor_radius,
                    parent_mode.child_b.orientation,  // Use orientation DELTA from genome
                    split_offset_magnitude,
                    split_dir_parent,
                    false,  // is_child_a = false (this is child B)
                );
            }
            AdhesionZone::ZoneB if child_a_keep => {
                // Zone B → Child A (adhesions pointing same as split stay with child at same side)
                create_inherited_adhesion(
                    state,
                    genome,
                    child_a_idx,
                    neighbor_idx,
                    child_a_mode_idx,  // Use child A's current mode index
                    parent_is_a,
                    parent_idx,
                    parent_mode,
                    parent_genome_orientation,
                    parent_anchor_direction,
                    neighbor_anchor_direction,
                    parent_radius,
                    neighbor_radius,
                    parent_mode.child_a.orientation,  // Use orientation DELTA from genome
                    split_offset_magnitude,
                    split_dir_parent,
                    true,  // is_child_a = true (this is child A)
                );
            }
            AdhesionZone::ZoneC => {
                // Zone C → Both children (equatorial adhesions)
                if child_b_keep {
                    create_inherited_adhesion(
                        state,
                        genome,
                        child_b_idx,
                        neighbor_idx,
                        child_b_mode_idx,  // Use child B's current mode index
                        parent_is_a,
                        parent_idx,
                        parent_mode,
                        parent_genome_orientation,
                        parent_anchor_direction,
                        neighbor_anchor_direction,
                        parent_radius,
                        neighbor_radius,
                        parent_mode.child_b.orientation,  // Use orientation DELTA from genome
                        split_offset_magnitude,
                        split_dir_parent,
                        false,  // is_child_a = false (this is child B)
                    );
                }
                if child_a_keep {
                    create_inherited_adhesion(
                        state,
                        genome,
                        child_a_idx,
                        neighbor_idx,
                        child_a_mode_idx,  // Use child A's current mode index
                        parent_is_a,
                        parent_idx,
                        parent_mode,
                        parent_genome_orientation,
                        parent_anchor_direction,
                        neighbor_anchor_direction,
                        parent_radius,
                        neighbor_radius,
                        parent_mode.child_a.orientation,  // Use orientation DELTA from genome
                        split_offset_magnitude,
                        split_dir_parent,
                        true,  // is_child_a = true (this is child A)
                    );
                }
            }
            _ => {} // No inheritance for this combination
        }
        
        // Mark original connection as inactive
        state.adhesion_connections.is_active[connection_idx] = 0;
    }
}

/// Create an inherited adhesion connection from parent to child
/// 
/// This matches the C++ implementation exactly:
/// - Calculates child anchor from child position to neighbor in parent frame
/// - Calculates neighbor anchor from neighbor position to child in parent frame
/// - Transforms both to their respective local frames
/// - Uses genome orientations for proper transformations
/// - Preserves original side assignment
fn create_inherited_adhesion(
    state: &mut CanonicalState,
    genome: &GenomeData,
    child_idx: usize,
    neighbor_idx: usize,
    mode_index: usize,
    parent_was_a: bool,
    _parent_idx: usize,
    parent_mode: &crate::genome::ModeSettings,
    parent_genome_orientation: Quat,
    parent_anchor_direction: Vec3,
    _neighbor_anchor_direction: Vec3,
    parent_radius: f32,
    neighbor_radius: f32,
    child_orientation_delta: Quat,
    split_offset_magnitude: f32,
    split_dir_parent: Vec3,
    is_child_a: bool,
) {
    // CRITICAL: Match C++ implementation for Zone C cases
    // In Zone C, the neighbor needs TWO separate anchors (one to each child)
    // We must calculate geometric positions in parent frame and derive anchors
    
    // Get rest length from parent mode
    let rest_length = parent_mode.adhesion_settings.rest_length;
    
    // Calculate center-to-center distance using parent's adhesion rest length
    let center_to_center_dist = rest_length + parent_radius + neighbor_radius;
    
    // Calculate positions in parent frame for geometric anchor placement (MATCHES C++)
    let child_pos_parent_frame = if is_child_a {
        split_dir_parent * split_offset_magnitude  // Child A at +offset
    } else {
        -split_dir_parent * split_offset_magnitude  // Child B at -offset
    };
    let neighbor_pos_parent_frame = parent_anchor_direction * center_to_center_dist;
    
    // Child anchor: direction from child to neighbor, transformed by genome orientation
    let direction_to_neighbor_parent_frame = (neighbor_pos_parent_frame - child_pos_parent_frame).normalize();
    let child_anchor_direction = (child_orientation_delta.inverse() * direction_to_neighbor_parent_frame).normalize();
    
    // Neighbor anchor: direction from neighbor to child, transformed to neighbor's frame
    let direction_to_child_parent_frame = (child_pos_parent_frame - neighbor_pos_parent_frame).normalize();
    let neighbor_genome_orientation = state.genome_orientations[neighbor_idx];
    let relative_rotation = neighbor_genome_orientation.inverse() * parent_genome_orientation;
    let neighbor_anchor_direction = (relative_rotation * direction_to_child_parent_frame).normalize();
    
    // Get child and neighbor mode indices for zone classification
    let child_mode_idx = state.mode_indices[child_idx];
    let neighbor_mode_idx = state.mode_indices[neighbor_idx];
    
    // Get split directions from each cell's mode
    let child_mode = genome.modes.get(child_mode_idx);
    let neighbor_mode = genome.modes.get(neighbor_mode_idx);
    
    let child_split_dir = if let Some(mode) = child_mode {
        let pitch = mode.parent_split_direction.x.to_radians();
        let yaw = mode.parent_split_direction.y.to_radians();
        Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0) * Vec3::Z
    } else {
        Vec3::Z
    };
    
    let neighbor_split_dir = if let Some(mode) = neighbor_mode {
        let pitch = mode.parent_split_direction.x.to_radians();
        let yaw = mode.parent_split_direction.y.to_radians();
        Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0) * Vec3::Z
    } else {
        Vec3::Z
    };
    
    // Get genome orientations for twist references
    let child_genome_orientation = state.genome_orientations[child_idx];
    let neighbor_genome_orientation_for_twist = state.genome_orientations[neighbor_idx];
    
    // Preserve original side assignment: if neighbor was originally cellA, keep them as cellA
    let result = if parent_was_a {
        // Parent was cellA, neighbor was cellB, so neighbor becomes cellB
        state.adhesion_manager.add_adhesion_with_directions(
            &mut state.adhesion_connections,
            child_idx,
            neighbor_idx,
            mode_index,
            child_anchor_direction,
            neighbor_anchor_direction,
            child_split_dir,
            neighbor_split_dir,
            child_genome_orientation,
            neighbor_genome_orientation_for_twist,
        )
    } else {
        // Parent was cellB, neighbor was cellA, so neighbor becomes cellA
        state.adhesion_manager.add_adhesion_with_directions(
            &mut state.adhesion_connections,
            neighbor_idx,
            child_idx,
            mode_index,
            neighbor_anchor_direction,
            child_anchor_direction,
            neighbor_split_dir,
            child_split_dir,
            neighbor_genome_orientation_for_twist,
            child_genome_orientation,
        )
    };
    
    if let Some(conn_idx) = result {
        // Debug: Record anchor positions for inherited adhesion
        let anchor_a_local = state.adhesion_connections.anchor_direction_a[conn_idx];
        let anchor_b_local = state.adhesion_connections.anchor_direction_b[conn_idx];
        let cell_a_idx = state.adhesion_connections.cell_a_index[conn_idx];
        let cell_b_idx = state.adhesion_connections.cell_b_index[conn_idx];
        let genome_rot_a = state.genome_orientations[cell_a_idx];
        let genome_rot_b = state.genome_orientations[cell_b_idx];
        let anchor_a_world = genome_rot_a * anchor_a_local;
        let anchor_b_world = genome_rot_b * anchor_b_local;
        
        println!("[ANCHOR DEBUG] Inherited adhesion created:");
        println!("  Connection: {} <-> {}", cell_a_idx, cell_b_idx);
        println!("  Parent anchor (local): {:?}", parent_anchor_direction);
        println!("  Child anchor (local): {:?}", anchor_a_local);
        println!("  Neighbor anchor (local): {:?}", anchor_b_local);
        println!("  Child anchor (world): {:?}", anchor_a_world);
        println!("  Neighbor anchor (world): {:?}", anchor_b_world);
        println!("  Expected: Should preserve parent's anchor direction");
    }
}
