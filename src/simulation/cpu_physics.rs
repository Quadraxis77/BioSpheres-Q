use bevy::prelude::*;

/// Canonical simulation state using Structure-of-Arrays (SoA) layout
/// 
/// This module implements the core deterministic physics engine using:
/// - Structure-of-Arrays (SoA) layout for cache-friendly iteration
/// - Parallel collision detection across spatial grid cells (via Rayon)
/// - Parallel force computation with deterministic accumulation
/// - Deterministic spatial partitioning for O(n) collision detection
/// 
/// All parallel operations maintain strict deterministic ordering by:
/// - Sorting collision pairs by cell indices after parallel detection
/// - Accumulating forces sequentially to preserve floating-point addition order
/// - Processing cells in index order (0 to N-1)
/// This is the core deterministic state used by both Main and Preview simulation modes
/// 
/// Design principles:
/// - Fixed capacity (10K cells) allocated upfront to avoid runtime allocations
/// - SoA layout for cache-friendly iteration and deterministic processing
/// - Cells are always processed in order by index (0 to cell_count-1)
/// - No dependencies on Bevy ECS - pure data structure
#[derive(Clone)]
pub struct CanonicalState {
    /// Number of active cells (cells 0..cell_count are valid)
    pub cell_count: usize,
    
    /// Maximum capacity (allocated upfront)
    pub capacity: usize,
    
    /// Cell unique IDs (determines iteration order)
    /// IDs are assigned sequentially and never reused
    pub cell_ids: Vec<u32>,
    
    // === Position and Motion (SoA) ===
    pub positions: Vec<Vec3>,
    pub prev_positions: Vec<Vec3>,
    pub velocities: Vec<Vec3>,
    
    // === Cell Properties (SoA) ===
    pub masses: Vec<f32>,
    pub radii: Vec<f32>,
    pub genome_ids: Vec<usize>,
    pub mode_indices: Vec<usize>,
    
    // === Orientation (SoA) ===
    pub rotations: Vec<Quat>,
    pub angular_velocities: Vec<Vec3>,
    
    // === Genome Orientation (SoA) ===
    /// Genome-derived orientations (used for adhesion calculations)
    /// This is separate from physics rotation and represents the cell's "design" orientation
    pub genome_orientations: Vec<Quat>,
    
    // === Physics State (SoA) ===
    pub forces: Vec<Vec3>,
    pub torques: Vec<Vec3>,
    pub accelerations: Vec<Vec3>,
    pub prev_accelerations: Vec<Vec3>,
    pub stiffnesses: Vec<f32>,
    
    // === Division Timers (SoA) ===
    pub birth_times: Vec<f32>,
    pub split_intervals: Vec<f32>,
    pub split_counts: Vec<i32>, // Number of times this cell has split
    
    // === Adhesion System ===
    /// Adhesion connections between cells
    pub adhesion_connections: crate::cell::AdhesionConnections,
    /// Adhesion connection manager
    pub adhesion_manager: crate::cell::AdhesionConnectionManager,
    
    /// Spatial partitioning for collision detection
    pub spatial_grid: DeterministicSpatialGrid,
    
    /// Next cell ID to assign (monotonically increasing)
    pub next_cell_id: u32,
}

impl CanonicalState {
    /// Create a new canonical state with the specified capacity
    pub fn new(capacity: usize) -> Self {
        // Calculate adhesion connection capacity (20 connections per cell)
        let adhesion_capacity = capacity * crate::cell::MAX_ADHESIONS_PER_CELL;
        
        Self {
            cell_count: 0,
            capacity,
            cell_ids: vec![0; capacity],
            positions: vec![Vec3::ZERO; capacity],
            prev_positions: vec![Vec3::ZERO; capacity],
            velocities: vec![Vec3::ZERO; capacity],
            masses: vec![1.0; capacity],
            radii: vec![1.0; capacity],
            genome_ids: vec![0; capacity],
            mode_indices: vec![0; capacity],
            rotations: vec![Quat::IDENTITY; capacity],
            angular_velocities: vec![Vec3::ZERO; capacity],
            genome_orientations: vec![Quat::IDENTITY; capacity],
            forces: vec![Vec3::ZERO; capacity],
            torques: vec![Vec3::ZERO; capacity],
            accelerations: vec![Vec3::ZERO; capacity],
            prev_accelerations: vec![Vec3::ZERO; capacity],
            stiffnesses: vec![10.0; capacity],
            birth_times: vec![0.0; capacity],
            split_intervals: vec![10.0; capacity],
            split_counts: vec![0; capacity],
            adhesion_connections: crate::cell::AdhesionConnections::new(adhesion_capacity),
            adhesion_manager: crate::cell::AdhesionConnectionManager::new(capacity),
            spatial_grid: DeterministicSpatialGrid::new(16, 100.0, 50.0), // Reduced from 64 to 16
            next_cell_id: 0,
        }
    }
    
    /// Add a new cell to the canonical state
    /// Returns the index of the new cell, or None if at capacity
    pub fn add_cell(
        &mut self,
        position: Vec3,
        velocity: Vec3,
        rotation: Quat,
        angular_velocity: Vec3,
        mass: f32,
        radius: f32,
        genome_id: usize,
        mode_index: usize,
        birth_time: f32,
        split_interval: f32,
        stiffness: f32,
        genome_orientation: Quat,
        split_count: i32,
    ) -> Option<usize> {
        if self.cell_count >= self.capacity {
            return None;
        }
        
        let idx = self.cell_count;
        self.cell_ids[idx] = self.next_cell_id;
        self.positions[idx] = position;
        self.prev_positions[idx] = position;
        self.velocities[idx] = velocity;
        self.masses[idx] = mass;
        self.radii[idx] = radius;
        self.genome_ids[idx] = genome_id;
        self.mode_indices[idx] = mode_index;
        self.rotations[idx] = rotation;
        self.angular_velocities[idx] = angular_velocity;
        self.genome_orientations[idx] = genome_orientation;
        self.forces[idx] = Vec3::ZERO;
        self.torques[idx] = Vec3::ZERO;
        self.accelerations[idx] = Vec3::ZERO;
        self.prev_accelerations[idx] = Vec3::ZERO;
        self.stiffnesses[idx] = stiffness;
        self.birth_times[idx] = birth_time;
        self.split_intervals[idx] = split_interval;
        self.split_counts[idx] = split_count;
        
        // Initialize adhesion indices for new cell
        self.adhesion_manager.init_cell_adhesion_indices(idx);
        
        self.next_cell_id += 1;
        self.cell_count += 1;
        
        Some(idx)
    }
}

/// Deterministic spatial grid using fixed-size arrays and prefix-sum algorithm
/// This provides O(1) cell lookups with zero allocations per tick
#[derive(Clone)]
pub struct DeterministicSpatialGrid {
    pub grid_dimensions: UVec3,
    pub world_size: f32,
    pub cell_size: f32,
    pub sphere_radius: f32,
    
    /// Pre-computed active cells (within sphere)
    /// Computed once at initialization
    pub active_cells: Vec<IVec3>,
    
    /// HashMap for O(1) lookup of active cell index
    pub active_cell_map: std::collections::HashMap<IVec3, usize>,
    
    /// Cell contents (flat array with prefix sums)
    /// Stores cell indices (not IDs) for direct SoA array access
    pub cell_contents: Vec<usize>,
    
    /// Prefix sum offsets for each active grid cell
    /// cell_offsets[i] = start index in cell_contents for active_cells[i]
    pub cell_offsets: Vec<usize>,
    
    /// Counts per grid cell (used during rebuild)
    pub cell_counts: Vec<usize>,
    
    /// Track which grid cells were used in last rebuild (for efficient clearing and collision detection)
    pub used_grid_cells: Vec<usize>,
}

impl DeterministicSpatialGrid {
    /// Create a new deterministic spatial grid
    pub fn new(grid_dim: u32, world_size: f32, sphere_radius: f32) -> Self {
        let grid_dimensions = UVec3::splat(grid_dim);
        let cell_size = world_size / grid_dim as f32;
        
        // Precompute active grid cells within spherical boundary
        let active_cells = Self::precompute_active_cells(grid_dimensions, cell_size, sphere_radius);
        let active_count = active_cells.len();
        
        // Build HashMap for O(1) lookups
        let mut active_cell_map = std::collections::HashMap::new();
        for (idx, &coord) in active_cells.iter().enumerate() {
            active_cell_map.insert(coord, idx);
        }
        
        // Allocate buffers for 10K cells (worst case: all cells in one grid cell)
        let max_cells = 10_000;
        
        Self {
            grid_dimensions,
            world_size,
            cell_size,
            sphere_radius,
            active_cells,
            active_cell_map,
            cell_contents: vec![0; max_cells],
            cell_offsets: vec![0; active_count],
            cell_counts: vec![0; active_count],
            used_grid_cells: Vec::with_capacity(max_cells),
        }
    }
    
    /// Precompute which grid cells are within the spherical boundary
    fn precompute_active_cells(
        grid_dimensions: UVec3,
        cell_size: f32,
        sphere_radius: f32,
    ) -> Vec<IVec3> {
        let mut active_cells = Vec::new();

        for x in 0..grid_dimensions.x as i32 {
            for y in 0..grid_dimensions.y as i32 {
                for z in 0..grid_dimensions.z as i32 {
                    let grid_coord = IVec3::new(x, y, z);

                    // Calculate the center of this grid cell in world space
                    let grid_pos = Vec3::new(x as f32, y as f32, z as f32);
                    let world_pos = (grid_pos * cell_size) + Vec3::splat(cell_size / 2.0)
                        - Vec3::splat(grid_dimensions.x as f32 * cell_size / 2.0);

                    // Calculate the AABB bounds for this grid cell
                    let half_cell = cell_size / 2.0;
                    let min_bound = world_pos - Vec3::splat(half_cell);
                    let max_bound = world_pos + Vec3::splat(half_cell);

                    // Find the closest point on the grid cell (AABB) to the sphere center (origin)
                    // Clamp the sphere center (0,0,0) to the AABB bounds
                    let closest_point = Vec3::new(
                        0.0_f32.clamp(min_bound.x, max_bound.x),
                        0.0_f32.clamp(min_bound.y, max_bound.y),
                        0.0_f32.clamp(min_bound.z, max_bound.z),
                    );

                    // Check if the closest point on the cell is within the sphere
                    if closest_point.length() <= sphere_radius {
                        active_cells.push(grid_coord);
                    }
                }
            }
        }

        active_cells
    }
    
    /// Convert world position to grid coordinates
    fn world_to_grid(&self, position: Vec3) -> IVec3 {
        let offset_position = position + Vec3::splat(self.world_size / 2.0);
        let grid_pos = offset_position / self.cell_size;
        
        // Clamp to grid dimensions
        let max_coord = (self.grid_dimensions.x - 1) as i32;
        IVec3::new(
            (grid_pos.x as i32).clamp(0, max_coord),
            (grid_pos.y as i32).clamp(0, max_coord),
            (grid_pos.z as i32).clamp(0, max_coord),
        )
    }
    
    /// Find the index of a grid coordinate in the active_cells array
    fn active_cell_index(&self, grid_coord: IVec3) -> Option<usize> {
        self.active_cell_map.get(&grid_coord).copied()
    }
    
    /// Rebuild the spatial grid using prefix sum algorithm (zero allocations)
    /// 
    /// Algorithm:
    /// 1. Count cells per grid cell (parallel for large cell counts)
    /// 2. Compute offsets using prefix sum
    /// 3. Insert cell indices into flat array (parallel for large cell counts)
    pub fn rebuild(&mut self, positions: &[Vec3], cell_count: usize) {
        use rayon::prelude::*;
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        // Clear only previously used counts
        for &idx in &self.used_grid_cells {
            self.cell_counts[idx] = 0;
        }
        self.used_grid_cells.clear();
        
        // Use parallel counting for large cell counts (>500 cells)
        if cell_count > 500 {
            // Parallel counting with atomic operations
            let atomic_counts: Vec<AtomicUsize> = (0..self.active_cells.len())
                .map(|_| AtomicUsize::new(0))
                .collect();
            
            (0..cell_count).into_par_iter().for_each(|i| {
                let grid_coord = self.world_to_grid(positions[i]);
                if let Some(idx) = self.active_cell_index(grid_coord) {
                    atomic_counts[idx].fetch_add(1, Ordering::Relaxed);
                }
            });
            
            // Convert atomic counts and track used cells
            for (idx, atomic_count) in atomic_counts.iter().enumerate() {
                let count = atomic_count.load(Ordering::Relaxed);
                if count > 0 {
                    self.cell_counts[idx] = count;
                    self.used_grid_cells.push(idx);
                }
            }
        } else {
            // Sequential counting for small cell counts
            for i in 0..cell_count {
                let grid_coord = self.world_to_grid(positions[i]);
                if let Some(idx) = self.active_cell_index(grid_coord) {
                    if self.cell_counts[idx] == 0 {
                        self.used_grid_cells.push(idx);
                    }
                    self.cell_counts[idx] += 1;
                }
            }
        }
        
        // Compute offsets using prefix sum (sequential - fast enough)
        let mut offset = 0;
        for &idx in &self.used_grid_cells {
            self.cell_offsets[idx] = offset;
            offset += self.cell_counts[idx];
        }
        
        // Reset counts for insertion phase
        for &idx in &self.used_grid_cells {
            self.cell_counts[idx] = 0;
        }
        
        // Parallel insertion for large cell counts
        if cell_count > 500 {
            let atomic_offsets: Vec<AtomicUsize> = self.cell_offsets
                .iter()
                .map(|&offset| AtomicUsize::new(offset))
                .collect();
            
            (0..cell_count).into_par_iter().for_each(|i| {
                let grid_coord = self.world_to_grid(positions[i]);
                if let Some(idx) = self.active_cell_index(grid_coord) {
                    let insert_pos = atomic_offsets[idx].fetch_add(1, Ordering::Relaxed);
                    unsafe {
                        // Safe because each thread writes to a unique position
                        let ptr = self.cell_contents.as_ptr() as *mut usize;
                        *ptr.add(insert_pos) = i;
                    }
                }
            });
            
            // Update counts from atomic offsets
            for (idx, atomic_offset) in atomic_offsets.iter().enumerate() {
                let final_offset = atomic_offset.load(Ordering::Relaxed);
                self.cell_counts[idx] = final_offset - self.cell_offsets[idx];
            }
        } else {
            // Sequential insertion for small cell counts
            for i in 0..cell_count {
                let grid_coord = self.world_to_grid(positions[i]);
                if let Some(idx) = self.active_cell_index(grid_coord) {
                    let insert_pos = self.cell_offsets[idx] + self.cell_counts[idx];
                    self.cell_contents[insert_pos] = i;
                    self.cell_counts[idx] += 1;
                }
            }
        }
    }
    
    /// Get a slice of cell indices in a specific grid cell
    pub fn get_cell_contents(&self, grid_idx: usize) -> &[usize] {
        let start = self.cell_offsets[grid_idx];
        let count = self.cell_counts[grid_idx];
        &self.cell_contents[start..start + count]
    }
}

/// Collision pair between two cells (using indices, not entities)
#[derive(Clone, Copy, Debug)]
pub struct CanonicalCollisionPair {
    pub index_a: usize,
    pub index_b: usize,
    pub overlap: f32,
    pub normal: Vec3, // Points from A to B
}

/// Detect collisions using the deterministic spatial grid - Single-threaded version
/// Returns collision pairs using cell indices (not IDs or entities)
pub fn detect_collisions_canonical_st(
    state: &CanonicalState,
) -> Vec<CanonicalCollisionPair> {
    let mut collision_pairs = Vec::new();
    
    // Forward neighbors for half-space optimization (13 neighbors instead of 27)
    const FORWARD_NEIGHBORS: [IVec3; 13] = [
        IVec3::new(1, 0, 0),
        IVec3::new(-1, 1, 0), IVec3::new(0, 1, 0), IVec3::new(1, 1, 0),
        IVec3::new(-1, -1, 1), IVec3::new(0, -1, 1), IVec3::new(1, -1, 1),
        IVec3::new(-1, 0, 1), IVec3::new(0, 0, 1), IVec3::new(1, 0, 1),
        IVec3::new(-1, 1, 1), IVec3::new(0, 1, 1), IVec3::new(1, 1, 1),
    ];
    
    // Iterate through only the grid cells that contain simulation cells
    for &grid_idx in &state.spatial_grid.used_grid_cells {
        let grid_coord = state.spatial_grid.active_cells[grid_idx];
        let cells_in_grid = state.spatial_grid.get_cell_contents(grid_idx);
        
        // Check collisions within the same grid cell
        for i in 0..cells_in_grid.len() {
            let idx_a = cells_in_grid[i];
            for j in (i + 1)..cells_in_grid.len() {
                let idx_b = cells_in_grid[j];
                
                // Calculate distance between cells
                let delta = state.positions[idx_b] - state.positions[idx_a];
                let distance = delta.length();
                
                // Check for overlap
                let combined_radius = state.radii[idx_a] + state.radii[idx_b];
                if distance < combined_radius {
                    let overlap = combined_radius - distance;
                    let normal = if distance > 0.0001 {
                        delta / distance
                    } else {
                        Vec3::X
                    };
                    
                    collision_pairs.push(CanonicalCollisionPair {
                        index_a: idx_a,
                        index_b: idx_b,
                        overlap,
                        normal,
                    });
                }
            }
        }
        
        // Check forward neighbors to avoid duplicate checks
        for &offset in &FORWARD_NEIGHBORS {
            let neighbor_coord = grid_coord + offset;
            
            // Find neighbor in active cells
            let Some(neighbor_idx) = state.spatial_grid.active_cell_index(neighbor_coord) else {
                continue;
            };
            
            let neighbor_cells = state.spatial_grid.get_cell_contents(neighbor_idx);
            
            // Check all pairs between current cell and neighbor cell
            for &idx_a in cells_in_grid {
                for &idx_b in neighbor_cells {
                    // Calculate distance between cells
                    let delta = state.positions[idx_b] - state.positions[idx_a];
                    let distance = delta.length();
                    
                    // Check for overlap
                    let combined_radius = state.radii[idx_a] + state.radii[idx_b];
                    if distance < combined_radius {
                        let overlap = combined_radius - distance;
                        let normal = if distance > 0.0001 {
                            delta / distance
                        } else {
                            Vec3::X
                        };
                        
                        collision_pairs.push(CanonicalCollisionPair {
                            index_a: idx_a,
                            index_b: idx_b,
                            overlap,
                            normal,
                        });
                    }
                }
            }
        }
    }
    
    collision_pairs
}

/// Detect collisions using the deterministic spatial grid - Multithreaded version
/// Returns collision pairs using cell indices (not IDs or entities)
/// 
/// Uses parallel iteration over grid cells for improved performance with large cell counts.
/// Results are collected and sorted to maintain deterministic ordering.
pub fn detect_collisions_canonical(
    state: &CanonicalState,
) -> Vec<CanonicalCollisionPair> {
    use rayon::prelude::*;
    
    // Forward neighbors for half-space optimization (13 neighbors instead of 27)
    const FORWARD_NEIGHBORS: [IVec3; 13] = [
        IVec3::new(1, 0, 0),
        IVec3::new(-1, 1, 0), IVec3::new(0, 1, 0), IVec3::new(1, 1, 0),
        IVec3::new(-1, -1, 1), IVec3::new(0, -1, 1), IVec3::new(1, -1, 1),
        IVec3::new(-1, 0, 1), IVec3::new(0, 0, 1), IVec3::new(1, 0, 1),
        IVec3::new(-1, 1, 1), IVec3::new(0, 1, 1), IVec3::new(1, 1, 1),
    ];
    
    // Process each grid cell in parallel
    let mut collision_pairs: Vec<CanonicalCollisionPair> = state.spatial_grid.used_grid_cells
        .par_iter()
        .flat_map(|&grid_idx| {
            let mut local_pairs = Vec::new();
            let grid_coord = state.spatial_grid.active_cells[grid_idx];
            let cells_in_grid = state.spatial_grid.get_cell_contents(grid_idx);
            
            // Check collisions within the same grid cell
            for i in 0..cells_in_grid.len() {
                let idx_a = cells_in_grid[i];
                for j in (i + 1)..cells_in_grid.len() {
                    let idx_b = cells_in_grid[j];
                    
                    // Calculate distance between cells
                    let delta = state.positions[idx_b] - state.positions[idx_a];
                    let distance = delta.length();
                    
                    // Check for overlap
                    let combined_radius = state.radii[idx_a] + state.radii[idx_b];
                    if distance < combined_radius {
                        let overlap = combined_radius - distance;
                        let normal = if distance > 0.0001 {
                            delta / distance
                        } else {
                            Vec3::X
                        };
                        
                        local_pairs.push(CanonicalCollisionPair {
                            index_a: idx_a,
                            index_b: idx_b,
                            overlap,
                            normal,
                        });
                    }
                }
            }
            
            // Check forward neighbors to avoid duplicate checks
            for &offset in &FORWARD_NEIGHBORS {
                let neighbor_coord = grid_coord + offset;
                
                // Find neighbor in active cells
                let Some(neighbor_idx) = state.spatial_grid.active_cell_index(neighbor_coord) else {
                    continue;
                };
                
                let neighbor_cells = state.spatial_grid.get_cell_contents(neighbor_idx);
                
                // Check all pairs between current cell and neighbor cell
                for &idx_a in cells_in_grid {
                    for &idx_b in neighbor_cells {
                        // Calculate distance between cells
                        let delta = state.positions[idx_b] - state.positions[idx_a];
                        let distance = delta.length();
                        
                        // Check for overlap
                        let combined_radius = state.radii[idx_a] + state.radii[idx_b];
                        if distance < combined_radius {
                            let overlap = combined_radius - distance;
                            let normal = if distance > 0.0001 {
                                delta / distance
                            } else {
                                Vec3::X
                            };
                            
                            local_pairs.push(CanonicalCollisionPair {
                                index_a: idx_a,
                                index_b: idx_b,
                                overlap,
                                normal,
                            });
                        }
                    }
                }
            }
            
            local_pairs
        })
        .collect();
    
    // Sort collision pairs by indices to maintain deterministic ordering
    // This ensures bit-identical results across runs
    collision_pairs.sort_unstable_by_key(|pair| (pair.index_a, pair.index_b));
    
    collision_pairs
}

/// Check if two cells are connected via an active adhesion connection
/// Optimized version using adhesion manager's cell-local lookup
#[inline]
fn are_cells_connected(state: &CanonicalState, cell_a: usize, cell_b: usize) -> bool {
    state.adhesion_manager.are_cells_connected(&state.adhesion_connections, cell_a, cell_b)
}

/// Compute collision forces from detected collision pairs - Single-threaded version
pub fn compute_collision_forces_canonical_st(
    state: &mut CanonicalState,
    collision_pairs: &[CanonicalCollisionPair],
    config: &crate::cell::physics::PhysicsConfig,
) {
    // Clear all forces and torques
    for i in 0..state.cell_count {
        state.forces[i] = Vec3::ZERO;
        state.torques[i] = Vec3::ZERO;
    }
    
    // Process each collision pair
    for pair in collision_pairs {
        let idx_a = pair.index_a;
        let idx_b = pair.index_b;
        
        let stiffness_a = state.stiffnesses[idx_a];
        let stiffness_b = state.stiffnesses[idx_b];
        
        // Compute combined stiffness using harmonic mean
        let combined_stiffness = if stiffness_a > 0.0 && stiffness_b > 0.0 {
            (stiffness_a * stiffness_b) / (stiffness_a + stiffness_b)
        } else if stiffness_a > 0.0 {
            stiffness_a
        } else if stiffness_b > 0.0 {
            stiffness_b
        } else {
            config.default_stiffness
        };
        
        // Calculate spring force
        let spring_force_magnitude = combined_stiffness * pair.overlap;
        
        // Calculate relative velocity along collision normal
        let relative_velocity = state.velocities[idx_b] - state.velocities[idx_a];
        let relative_velocity_normal = relative_velocity.dot(pair.normal);
        
        // Calculate damping force
        let damping_force_magnitude = -config.damping * relative_velocity_normal;
        
        // Total force magnitude
        let total_force_magnitude = spring_force_magnitude + damping_force_magnitude;
        
        // Skip collision forces for cells connected via adhesions (adhesion forces handle their interaction)
        let cells_are_connected = are_cells_connected(state, idx_a, idx_b);
        if cells_are_connected {
            continue;
        }
        
        // Clamp force magnitude
        let max_force = 10000.0;
        let clamped_force_magnitude = total_force_magnitude.clamp(-max_force, max_force);
        let force = clamped_force_magnitude * pair.normal;
        
        // Apply equal and opposite forces
        state.forces[idx_b] += force;
        state.forces[idx_a] -= force;
        
        // === Rolling Friction Torque ===
        // Apply torque based on tangential velocity at contact point
        // This creates rolling without violating momentum conservation
        // SKIP rolling friction for cells connected via adhesions (would interfere with orientation control)
        
        let cells_are_connected = are_cells_connected(state, idx_a, idx_b);
        
        if config.friction_coefficient > 0.0 && pair.overlap > 0.0 && !cells_are_connected {
            // Contact point offsets from cell centers
            let contact_offset_a = pair.normal * state.radii[idx_a];
            let contact_offset_b = -pair.normal * state.radii[idx_b];
            
            // Velocity at contact points including rotation
            let vel_at_contact_a = state.velocities[idx_a] + 
                state.angular_velocities[idx_a].cross(contact_offset_a);
            let vel_at_contact_b = state.velocities[idx_b] + 
                state.angular_velocities[idx_b].cross(contact_offset_b);
            
            // Relative velocity at contact point
            let relative_vel_at_contact = vel_at_contact_b - vel_at_contact_a;
            
            // Tangential component (perpendicular to normal)
            let tangential_velocity = relative_vel_at_contact - pair.normal * relative_vel_at_contact.dot(pair.normal);
            let tangential_speed = tangential_velocity.length();
            
            if tangential_speed > 0.0001 {
                let tangent_direction = tangential_velocity / tangential_speed;
                
                // Maximum friction torque (Coulomb friction limit)
                let max_friction_torque = config.friction_coefficient * clamped_force_magnitude.abs();
                
                // Torque direction: perpendicular to both normal and tangent
                // This creates rotation that opposes the tangential sliding
                let torque_axis_a = contact_offset_a.cross(tangent_direction);
                let torque_axis_b = contact_offset_b.cross(tangent_direction);
                
                // Torque magnitude scaled by tangential speed and radius
                let torque_magnitude = (tangential_speed * state.radii[idx_a]).min(max_friction_torque);
                
                let resistance_torque_a = -torque_axis_a.normalize_or_zero() * torque_magnitude;
                let resistance_torque_b = -torque_axis_b.normalize_or_zero() * torque_magnitude;
                
                state.torques[idx_a] += resistance_torque_a;
                state.torques[idx_b] += resistance_torque_b;
            }
        }
    }
}

/// Compute collision forces from detected collision pairs - Multithreaded version
/// 
/// Uses parallel iteration for force computation, then accumulates forces sequentially
/// to maintain determinism (parallel accumulation would be non-deterministic due to
/// floating-point addition order).
pub fn compute_collision_forces_canonical(
    state: &mut CanonicalState,
    collision_pairs: &[CanonicalCollisionPair],
    config: &crate::cell::physics::PhysicsConfig,
) {
    use rayon::prelude::*;
    
    // Clear all forces and torques (parallel)
    state.forces[..state.cell_count].par_iter_mut().for_each(|f| *f = Vec3::ZERO);
    state.torques[..state.cell_count].par_iter_mut().for_each(|t| *t = Vec3::ZERO);
    
    // Compute forces and torques for each collision pair in parallel
    // Store as (index, force, torque) tuples to accumulate deterministically
    let contributions: Vec<(usize, Vec3, Vec3)> = collision_pairs
        .par_iter()
        .flat_map(|pair| {
            let idx_a = pair.index_a;
            let idx_b = pair.index_b;
            
            let stiffness_a = state.stiffnesses[idx_a];
            let stiffness_b = state.stiffnesses[idx_b];
            
            // Compute combined stiffness using harmonic mean
            let combined_stiffness = if stiffness_a > 0.0 && stiffness_b > 0.0 {
                (stiffness_a * stiffness_b) / (stiffness_a + stiffness_b)
            } else if stiffness_a > 0.0 {
                stiffness_a
            } else if stiffness_b > 0.0 {
                stiffness_b
            } else {
                config.default_stiffness
            };
            
            // Calculate spring force
            let spring_force_magnitude = combined_stiffness * pair.overlap;
            
            // Calculate relative velocity along collision normal
            let relative_velocity = state.velocities[idx_b] - state.velocities[idx_a];
            let relative_velocity_normal = relative_velocity.dot(pair.normal);
            
            // Calculate damping force
            let damping_force_magnitude = -config.damping * relative_velocity_normal;
            
            // Total force magnitude
            let total_force_magnitude = spring_force_magnitude + damping_force_magnitude;
            
            // Skip collision forces for cells connected via adhesions (adhesion forces handle their interaction)
            let cells_are_connected = are_cells_connected(state, idx_a, idx_b);
            if cells_are_connected {
                return vec![];
            }
            
            // Clamp force magnitude
            let max_force = 10000.0;
            let clamped_force_magnitude = total_force_magnitude.clamp(-max_force, max_force);
            let force = clamped_force_magnitude * pair.normal;
            
            // === Rolling Friction Torque ===
            
            let mut torque_a = Vec3::ZERO;
            let mut torque_b = Vec3::ZERO;
            
            // Apply torque based on tangential velocity at contact point
            // SKIP rolling friction for cells connected via adhesions (would interfere with orientation control)
            
            if config.friction_coefficient > 0.0 && pair.overlap > 0.0 {
                // Contact point offsets from cell centers
                let contact_offset_a = pair.normal * state.radii[idx_a];
                let contact_offset_b = -pair.normal * state.radii[idx_b];
                
                // Velocity at contact points including rotation
                let vel_at_contact_a = state.velocities[idx_a] + 
                    state.angular_velocities[idx_a].cross(contact_offset_a);
                let vel_at_contact_b = state.velocities[idx_b] + 
                    state.angular_velocities[idx_b].cross(contact_offset_b);
                
                // Relative velocity at contact point
                let relative_vel_at_contact = vel_at_contact_b - vel_at_contact_a;
                
                // Tangential component (perpendicular to normal)
                let tangential_velocity = relative_vel_at_contact - pair.normal * relative_vel_at_contact.dot(pair.normal);
                let tangential_speed = tangential_velocity.length();
                
                if tangential_speed > 0.0001 {
                    let tangent_direction = tangential_velocity / tangential_speed;
                    
                    // Maximum friction torque (Coulomb friction limit)
                    let max_friction_torque = config.friction_coefficient * clamped_force_magnitude.abs();
                    
                    // Torque direction: perpendicular to both normal and tangent
                    let torque_axis_a = contact_offset_a.cross(tangent_direction);
                    let torque_axis_b = contact_offset_b.cross(tangent_direction);
                    
                    // Torque magnitude scaled by tangential speed and radius
                    let torque_magnitude = (tangential_speed * state.radii[idx_a]).min(max_friction_torque);
                    
                    torque_a = -torque_axis_a.normalize_or_zero() * torque_magnitude;
                    torque_b = -torque_axis_b.normalize_or_zero() * torque_magnitude;
                }
            }
            
            // Return contributions for both cells: (index, force, torque)
            vec![
                (idx_b, force, torque_b),
                (idx_a, -force, torque_a),
            ]
        })
        .collect();
    
    // Accumulate forces and torques sequentially to maintain determinism
    for (idx, force, torque) in contributions {
        state.forces[idx] += force;
        state.torques[idx] += torque;
    }
}

/// Deterministic physics step function - Single-threaded version
/// Used by preview simulation for simpler, more predictable performance
pub fn physics_step_st(
    state: &mut CanonicalState,
    config: &crate::simulation::PhysicsConfig,
) {
    // 1. Verlet integration (position update)
    verlet_integrate_positions_soa_st(
        &mut state.positions[..state.cell_count],
        &state.velocities[..state.cell_count],
        &state.accelerations[..state.cell_count],
        config.fixed_timestep,
    );
    
    // 2. Update rotations from angular velocities
    integrate_rotations_soa_st(
        &mut state.rotations[..state.cell_count],
        &state.angular_velocities[..state.cell_count],
        config.fixed_timestep,
    );
    
    // 3. Update spatial partitioning
    state.spatial_grid.rebuild(&state.positions, state.cell_count);
    
    // 4. Detect collisions
    let collisions = detect_collisions_canonical_st(state);
    
    // 5. Compute forces and torques
    compute_collision_forces_canonical_st(state, &collisions, config);
    
    // 5.5. Compute adhesion forces (if any connections exist)
    if state.adhesion_connections.active_count > 0 {
        // Create default adhesion settings for now
        // TODO: Get settings from genome modes
        let default_settings = crate::cell::AdhesionSettings::default();
        let mode_settings = vec![default_settings; 10]; // Support up to 10 modes
        
        // Use batched version for single-threaded (better cache locality)
        crate::cell::compute_adhesion_forces_batched(
            &state.adhesion_connections,
            &state.positions[..state.cell_count],
            &state.velocities[..state.cell_count],
            &state.rotations[..state.cell_count],
            &state.angular_velocities[..state.cell_count],
            &state.masses[..state.cell_count],
            &mode_settings,
            &mut state.forces[..state.cell_count],
            &mut state.torques[..state.cell_count],
        );
    }
    
    // 6. Apply boundary conditions
    apply_boundary_forces_soa_st(
        &state.positions[..state.cell_count],
        &mut state.velocities[..state.cell_count],
        config,
    );
    
    // 7. Verlet integration (velocity update)
    verlet_integrate_velocities_soa_st(
        &mut state.velocities[..state.cell_count],
        &mut state.accelerations[..state.cell_count],
        &mut state.prev_accelerations[..state.cell_count],
        &state.forces[..state.cell_count],
        &state.masses[..state.cell_count],
        config.fixed_timestep,
        config.velocity_damping,
    );
    
    // 8. Update angular velocities from torques
    integrate_angular_velocities_soa_st(
        &mut state.angular_velocities[..state.cell_count],
        &state.torques[..state.cell_count],
        &state.radii[..state.cell_count],
        &state.masses[..state.cell_count],
        config.fixed_timestep,
        config.angular_damping,
    );
}

/// Genome-aware physics step function - Single-threaded version
/// Used by preview simulation with genome-specific adhesion settings
pub fn physics_step_st_with_genome(
    state: &mut CanonicalState,
    config: &crate::simulation::PhysicsConfig,
    genome: &crate::genome::GenomeData,
) {
    // 1. Verlet integration (position update)
    verlet_integrate_positions_soa_st(
        &mut state.positions[..state.cell_count],
        &state.velocities[..state.cell_count],
        &state.accelerations[..state.cell_count],
        config.fixed_timestep,
    );
    
    // 2. Update rotations from angular velocities
    integrate_rotations_soa_st(
        &mut state.rotations[..state.cell_count],
        &state.angular_velocities[..state.cell_count],
        config.fixed_timestep,
    );
    
    // 3. Update spatial partitioning
    state.spatial_grid.rebuild(&state.positions, state.cell_count);
    
    // 4. Detect collisions
    let collisions = detect_collisions_canonical_st(state);
    
    // 5. Compute forces and torques
    compute_collision_forces_canonical_st(state, &collisions, config);
    
    // 5.5. Compute adhesion forces with genome settings
    if state.adhesion_connections.active_count > 0 {
        // Extract adhesion settings from genome modes
        let mode_settings: Vec<crate::cell::AdhesionSettings> = genome.modes.iter()
            .map(|mode| crate::cell::AdhesionSettings {
                can_break: mode.adhesion_settings.can_break,
                break_force: mode.adhesion_settings.break_force,
                rest_length: mode.adhesion_settings.rest_length,
                linear_spring_stiffness: mode.adhesion_settings.linear_spring_stiffness,
                linear_spring_damping: mode.adhesion_settings.linear_spring_damping,
                orientation_spring_stiffness: mode.adhesion_settings.orientation_spring_stiffness,
                orientation_spring_damping: mode.adhesion_settings.orientation_spring_damping,
                max_angular_deviation: mode.adhesion_settings.max_angular_deviation,
                twist_constraint_stiffness: mode.adhesion_settings.twist_constraint_stiffness,
                twist_constraint_damping: mode.adhesion_settings.twist_constraint_damping,
                enable_twist_constraint: mode.adhesion_settings.enable_twist_constraint,
            })
            .collect();
        
        // Use batched version for single-threaded (better cache locality)
        crate::cell::compute_adhesion_forces_batched(
            &state.adhesion_connections,
            &state.positions[..state.cell_count],
            &state.velocities[..state.cell_count],
            &state.rotations[..state.cell_count],
            &state.angular_velocities[..state.cell_count],
            &state.masses[..state.cell_count],
            &mode_settings,
            &mut state.forces[..state.cell_count],
            &mut state.torques[..state.cell_count],
        );
    }
    
    // 6. Apply boundary conditions
    apply_boundary_forces_soa_st(
        &state.positions[..state.cell_count],
        &mut state.velocities[..state.cell_count],
        config,
    );
    
    // 7. Verlet integration (velocity update)
    verlet_integrate_velocities_soa_st(
        &mut state.velocities[..state.cell_count],
        &mut state.accelerations[..state.cell_count],
        &mut state.prev_accelerations[..state.cell_count],
        &state.forces[..state.cell_count],
        &state.masses[..state.cell_count],
        config.fixed_timestep,
        config.velocity_damping,
    );
    
    // 8. Update angular velocities from torques
    integrate_angular_velocities_soa_st(
        &mut state.angular_velocities[..state.cell_count],
        &state.torques[..state.cell_count],
        &state.radii[..state.cell_count],
        &state.masses[..state.cell_count],
        config.fixed_timestep,
        config.angular_damping,
    );
    
    // 9. Update nutrient growth for Test cells
    crate::simulation::nutrient_system::update_nutrient_growth_st(
        &mut state.masses[..state.cell_count],
        &mut state.radii[..state.cell_count],
        &state.mode_indices[..state.cell_count],
        genome,
        config.fixed_timestep,
    );
    
    // 10. Transport nutrients between adhesion-connected cells
    crate::simulation::nutrient_system::transport_nutrients_st(state, genome, config.fixed_timestep);
}

/// Deterministic physics step function - Multithreaded version
/// This is the core function called by Main simulation mode
pub fn physics_step(
    state: &mut CanonicalState,
    config: &crate::simulation::PhysicsConfig,
) {
    // 1. Verlet integration (position update)
    verlet_integrate_positions_soa(
        &mut state.positions[..state.cell_count],
        &state.velocities[..state.cell_count],
        &state.accelerations[..state.cell_count],
        config.fixed_timestep,
    );
    
    // 2. Update rotations from angular velocities
    integrate_rotations_soa(
        &mut state.rotations[..state.cell_count],
        &state.angular_velocities[..state.cell_count],
        config.fixed_timestep,
    );
    
    // 3. Update spatial partitioning
    state.spatial_grid.rebuild(&state.positions, state.cell_count);
    
    // 4. Detect collisions
    let collisions = detect_collisions_canonical(state);
    
    // 5. Compute forces and torques
    compute_collision_forces_canonical(state, &collisions, config);
    
    // 5.5. Compute adhesion forces (if any connections exist)
    if state.adhesion_connections.active_count > 0 {
        // Create default adhesion settings for now
        // TODO: Get settings from genome modes
        let default_settings = crate::cell::AdhesionSettings::default();
        let mode_settings = vec![default_settings; 10]; // Support up to 10 modes
        
        // Use parallel version for multithreaded physics
        crate::cell::compute_adhesion_forces_parallel(
            &state.adhesion_connections,
            &state.positions[..state.cell_count],
            &state.velocities[..state.cell_count],
            &state.rotations[..state.cell_count],
            &state.angular_velocities[..state.cell_count],
            &state.masses[..state.cell_count],
            &mode_settings,
            &mut state.forces[..state.cell_count],
            &mut state.torques[..state.cell_count],
        );
    }
    
    // 6. Apply boundary conditions
    apply_boundary_forces_soa(
        &state.positions[..state.cell_count],
        &mut state.velocities[..state.cell_count],
        config,
    );
    
    // 7. Verlet integration (velocity update)
    verlet_integrate_velocities_soa(
        &mut state.velocities[..state.cell_count],
        &mut state.accelerations[..state.cell_count],
        &mut state.prev_accelerations[..state.cell_count],
        &state.forces[..state.cell_count],
        &state.masses[..state.cell_count],
        config.fixed_timestep,
        config.velocity_damping,
    );
    
    // 8. Update angular velocities from torques
    integrate_angular_velocities_soa(
        &mut state.angular_velocities[..state.cell_count],
        &state.torques[..state.cell_count],
        &state.radii[..state.cell_count],
        &state.masses[..state.cell_count],
        config.fixed_timestep,
        config.angular_damping,
    );
}

/// Genome-aware physics step function - Multithreaded version
/// This version uses adhesion settings from the genome
pub fn physics_step_with_genome(
    state: &mut CanonicalState,
    config: &crate::simulation::PhysicsConfig,
    genome: &crate::genome::GenomeData,
) {
    // 1. Verlet integration (position update)
    verlet_integrate_positions_soa(
        &mut state.positions[..state.cell_count],
        &state.velocities[..state.cell_count],
        &state.accelerations[..state.cell_count],
        config.fixed_timestep,
    );
    
    // 2. Update rotations from angular velocities
    integrate_rotations_soa(
        &mut state.rotations[..state.cell_count],
        &state.angular_velocities[..state.cell_count],
        config.fixed_timestep,
    );
    
    // 3. Update spatial partitioning
    state.spatial_grid.rebuild(&state.positions, state.cell_count);
    
    // 4. Detect collisions
    let collisions = detect_collisions_canonical(state);
    
    // 5. Compute forces and torques
    compute_collision_forces_canonical(state, &collisions, config);
    
    // 5.5. Compute adhesion forces with genome settings
    if state.adhesion_connections.active_count > 0 {
        // Extract adhesion settings from genome modes
        let mode_settings: Vec<crate::cell::AdhesionSettings> = genome.modes.iter()
            .map(|mode| crate::cell::AdhesionSettings {
                can_break: mode.adhesion_settings.can_break,
                break_force: mode.adhesion_settings.break_force,
                rest_length: mode.adhesion_settings.rest_length,
                linear_spring_stiffness: mode.adhesion_settings.linear_spring_stiffness,
                linear_spring_damping: mode.adhesion_settings.linear_spring_damping,
                orientation_spring_stiffness: mode.adhesion_settings.orientation_spring_stiffness,
                orientation_spring_damping: mode.adhesion_settings.orientation_spring_damping,
                max_angular_deviation: mode.adhesion_settings.max_angular_deviation,
                twist_constraint_stiffness: mode.adhesion_settings.twist_constraint_stiffness,
                twist_constraint_damping: mode.adhesion_settings.twist_constraint_damping,
                enable_twist_constraint: mode.adhesion_settings.enable_twist_constraint,
            })
            .collect();
        
        // Use parallel version for multithreaded physics
        crate::cell::compute_adhesion_forces_parallel(
            &state.adhesion_connections,
            &state.positions[..state.cell_count],
            &state.velocities[..state.cell_count],
            &state.rotations[..state.cell_count],
            &state.angular_velocities[..state.cell_count],
            &state.masses[..state.cell_count],
            &mode_settings,
            &mut state.forces[..state.cell_count],
            &mut state.torques[..state.cell_count],
        );
    }
    
    // 6. Apply boundary conditions
    apply_boundary_forces_soa(
        &state.positions[..state.cell_count],
        &mut state.velocities[..state.cell_count],
        config,
    );
    
    // 7. Verlet integration (velocity update)
    verlet_integrate_velocities_soa(
        &mut state.velocities[..state.cell_count],
        &mut state.accelerations[..state.cell_count],
        &mut state.prev_accelerations[..state.cell_count],
        &state.forces[..state.cell_count],
        &state.masses[..state.cell_count],
        config.fixed_timestep,
        config.velocity_damping,
    );
    
    // 8. Update angular velocities from torques
    integrate_angular_velocities_soa(
        &mut state.angular_velocities[..state.cell_count],
        &state.torques[..state.cell_count],
        &state.radii[..state.cell_count],
        &state.masses[..state.cell_count],
        config.fixed_timestep,
        config.angular_damping,
    );
    
    // 9. Update nutrient growth for Test cells
    crate::simulation::nutrient_system::update_nutrient_growth(
        &mut state.masses[..state.cell_count],
        &mut state.radii[..state.cell_count],
        &state.mode_indices[..state.cell_count],
        genome,
        config.fixed_timestep,
    );
    
    // 10. Transport nutrients between adhesion-connected cells
    crate::simulation::nutrient_system::transport_nutrients(state, genome, config.fixed_timestep);
}

// ============================================================================
// Deterministic Division Functions
// ============================================================================

/// Event representing a cell division that occurred
#[derive(Clone, Debug)]
pub struct DivisionEvent {
    pub parent_idx: usize,
    pub child_a_idx: usize,
    pub child_b_idx: usize,
}

/// Generate a pseudo-random rotation quaternion with magnitude ~0.001 radians
///
/// Uses a simple LCG-style hash to generate deterministic pseudo-random values
/// based on cell_id and rng_seed. This ensures reproducibility while adding
/// small perturbations to cell rotations during division.
///
/// # Arguments
/// * `cell_id` - Unique cell identifier
/// * `rng_seed` - Global random seed
///
/// # Returns
/// A quaternion representing a small random rotation (~0.001 radians)
fn pseudo_random_rotation(cell_id: u32, rng_seed: u64) -> Quat {
    // Simple hash combining cell_id and rng_seed
    let hash = ((cell_id as u64).wrapping_mul(2654435761))
        .wrapping_add(rng_seed)
        .wrapping_mul(2654435761);

    // Extract three pseudo-random values in range [0, 1)
    let x = ((hash & 0xFFFF) as f32) / 65536.0;
    let y = (((hash >> 16) & 0xFFFF) as f32) / 65536.0;
    let z = (((hash >> 32) & 0xFFFF) as f32) / 65536.0;

    // Convert to range [-1, 1) and scale to 0.001 radians
    let angle_scale = 0.001;
    let axis_x = (x - 0.5) * 2.0 * angle_scale;
    let axis_y = (y - 0.5) * 2.0 * angle_scale;
    let axis_z = (z - 0.5) * 2.0 * angle_scale;

    // Create rotation from small angle approximation
    // For small angles, sin(θ) ≈ θ and cos(θ) ≈ 1
    Quat::from_xyzw(axis_x, axis_y, axis_z, 1.0).normalize()
}

/// Deterministic division step for canonical state
///
/// This function handles cell division in a deterministic manner:
/// - Checks which cells are ready to divide based on age
/// - Uses the cell_allocation system for deterministic slot assignment
/// - Creates child cells with properties derived from parent and genome
/// - Respects capacity limits
/// - Assigns unique IDs maintaining deterministic ordering
///
/// # Arguments
/// * `state` - Mutable reference to the canonical state
/// * `genome` - Reference to the genome data
/// * `current_time` - Current simulation time
/// * `max_cells` - Maximum cell capacity
/// * `_rng_seed` - Random seed (used for pseudo-random rotation perturbations)
///
/// # Returns
/// Vector of DivisionEvent describing which divisions occurred
pub fn division_step(
    state: &mut CanonicalState,
    genome: &crate::genome::GenomeData,
    current_time: f32,
    max_cells: usize,
    _rng_seed: u64,
) -> Vec<DivisionEvent> {
    
    // Early exit if at capacity
    if state.cell_count >= max_cells {
        return Vec::new();
    }
    
    // Find cells ready to divide
    let mut divisions_to_process = Vec::new();
    for i in 0..state.cell_count {
        let cell_age = current_time - state.birth_times[i];
        let mode_index = state.mode_indices[i];
        let mode = genome.modes.get(mode_index);
        
        // Check max_splits limit (-1 means infinite)
        let can_split_by_count = if let Some(m) = mode {
            m.max_splits < 0 || state.split_counts[i] < m.max_splits
        } else {
            true
        };
        
        // Check adhesion limits - prevent splitting if at/above max or below min connections
        let adhesion_count = state.adhesion_manager.count_active_adhesions(i);
        let can_split_by_adhesions = if let Some(m) = mode {
            adhesion_count >= m.min_adhesions as usize && adhesion_count < m.max_adhesions as usize
        } else {
            true
        };
        
        // Skip division if split_interval > 25 (never-split condition), max_splits reached, or max_adhesions reached
        if can_split_by_count && can_split_by_adhesions && state.split_intervals[i] <= 25.0 && cell_age >= state.split_intervals[i] {
            divisions_to_process.push(i);
        }
    }
    
    // CRITICAL: Prevent simultaneous splits of adhered cells (matches C++ GPU implementation)
    // Use priority-based system where lower cell index has higher priority
    let mut filtered_divisions = Vec::new();
    for &cell_idx in &divisions_to_process {
        let mut should_defer = false;
        
        for adhesion_idx in &state.adhesion_manager.cell_adhesion_indices[cell_idx] {
            if *adhesion_idx < 0 {
                continue;
            }
            let adhesion_idx = *adhesion_idx as usize;
            
            if state.adhesion_connections.is_active[adhesion_idx] == 0 {
                continue;
            }
            
            let other_idx = if state.adhesion_connections.cell_a_index[adhesion_idx] == cell_idx {
                state.adhesion_connections.cell_b_index[adhesion_idx]
            } else {
                state.adhesion_connections.cell_a_index[adhesion_idx]
            };
            
            let other_age = current_time - state.birth_times[other_idx];
            // Check if other cell is ready to divide (respecting never-split condition)
            if state.split_intervals[other_idx] <= 25.0 && other_age >= state.split_intervals[other_idx] {
                if other_idx < cell_idx {
                    should_defer = true;
                    break;
                }
            }
        }
        
        if !should_defer {
            filtered_divisions.push(cell_idx);
        }
    }
    
    let divisions_to_process = filtered_divisions;
    
    // Early exit if no divisions - avoid expensive AllocationSim creation
    if divisions_to_process.is_empty() {
        return Vec::new();
    }

    // For staggered divisions, we use a simpler allocation strategy:
    // Write children to slots at the end of the array (starting from cell_count)
    // Then compact to remove parents and consolidate everything
    //
    // This avoids the complex reservation system which was designed for
    // simultaneous divisions where all parents are freed at once.
    
    // Collect division data before modifying state
    struct DivisionData {
        parent_idx: usize,
        parent_mode_idx: usize,
        child_a_slot: usize,
        child_b_slot: usize,
        parent_velocity: bevy::prelude::Vec3,
        parent_radius: f32,
        parent_genome_id: usize,
        parent_stiffness: f32,
        parent_split_count: i32,
        parent_genome_orientation: bevy::prelude::Quat,  // CRITICAL: Save parent's genome orientation before overwriting
        child_a_pos: bevy::prelude::Vec3,
        child_b_pos: bevy::prelude::Vec3,
        child_a_orientation: bevy::prelude::Quat,
        child_b_orientation: bevy::prelude::Quat,
        child_a_genome_orientation: bevy::prelude::Quat,
        child_b_genome_orientation: bevy::prelude::Quat,
        child_a_mode_idx: usize,
        child_b_mode_idx: usize,
        child_a_split_mass: f32,
        child_b_split_mass: f32,
        child_a_radius: f32,
        child_b_radius: f32,
        child_a_split_interval: f32,
        child_b_split_interval: f32,
    }
    
    let mut division_data_list = Vec::new();
    let mut division_events = Vec::new();

    // Calculate available slots for children
    // Child A reuses parent index (matches C++), Child B gets new slot
    let mut next_available_slot = state.cell_count;

    // Process each division and collect data
    for &parent_idx in &divisions_to_process {
        // Check if we have space for 1 more cell (child B)
        if next_available_slot >= state.capacity {
            break;
        }

        // Child A reuses parent index, Child B gets new slot (matches C++ behavior)
        let child_a_slot = parent_idx;
        let child_b_slot = next_available_slot;
        next_available_slot += 1;
        
        let mode_index = state.mode_indices[parent_idx];
        let mode = genome.modes.get(mode_index);
        
        if let Some(mode) = mode {
            // Save parent properties
            let parent_position = state.positions[parent_idx];
            let parent_velocity = state.velocities[parent_idx];
            let parent_rotation = state.rotations[parent_idx];
            let parent_genome_orientation = state.genome_orientations[parent_idx];
            let parent_radius = state.radii[parent_idx];
            let parent_mass = state.masses[parent_idx];
            let parent_genome_id = state.genome_ids[parent_idx];
            let parent_stiffness = state.stiffnesses[parent_idx];
            let parent_split_count = state.split_counts[parent_idx];
            
            // Calculate split direction using physics rotation (for positioning)
            let pitch = mode.parent_split_direction.x.to_radians();
            let yaw = mode.parent_split_direction.y.to_radians();
            let split_direction = parent_rotation
                * bevy::prelude::Quat::from_euler(bevy::prelude::EulerRot::YXZ, yaw, pitch, 0.0)
                * bevy::prelude::Vec3::Z;
            
            // 75% overlap means centers are 25% of combined diameter apart
            // Match C++ convention: Child A at +offset, Child B at -offset
            let offset_distance = parent_radius * 0.25;
            let child_a_pos = parent_position + split_direction * offset_distance;
            let child_b_pos = parent_position - split_direction * offset_distance;
            
            // Get child mode indices
            // Check if Child A will reach max_splits after this division
            let will_reach_max_splits = mode.max_splits >= 0 && (parent_split_count + 1) >= mode.max_splits;
            
            // If max_splits is reached and mode_after_splits is set, use that mode for Child A
            let child_a_mode_idx = if will_reach_max_splits && mode.mode_after_splits >= 0 {
                mode.mode_after_splits.max(0) as usize
            } else {
                mode.child_a.mode_number.max(0) as usize
            };
            let child_b_mode_idx = mode.child_b.mode_number.max(0) as usize;
            
            // Get child properties
            let child_a_mode = genome.modes.get(child_a_mode_idx);
            let child_b_mode = genome.modes.get(child_b_mode_idx);
            
            // Split parent's mass according to split_ratio
            // split_ratio determines what fraction goes to Child A (0.0 to 1.0)
            let split_ratio = mode.split_ratio.clamp(0.0, 1.0);
            let child_a_split_mass = parent_mass * split_ratio;
            let child_b_split_mass = parent_mass * (1.0 - split_ratio);
            
            // Calculate child radii based on their masses
            let child_a_radius = if let Some(m) = child_a_mode {
                child_a_split_mass.min(m.max_cell_size).clamp(1.0, 2.0)
            } else {
                child_a_split_mass.clamp(1.0, 2.0)
            };
            
            let child_b_radius = if let Some(m) = child_b_mode {
                child_b_split_mass.min(m.max_cell_size).clamp(1.0, 2.0)
            } else {
                child_b_split_mass.clamp(1.0, 2.0)
            };
            
            let child_a_split_interval = if let Some(m) = child_a_mode {
                m.split_interval
            } else {
                5.0
            };
            
            let child_b_split_interval = if let Some(m) = child_b_mode {
                m.split_interval
            } else {
                5.0
            };
            
            // CRITICAL: Use parent's GENOME orientation for child genome orientations
            // This ensures genome orientations stay fixed and don't inherit physics rotation
            let child_a_genome_orientation = parent_genome_orientation * mode.child_a.orientation;
            let child_b_genome_orientation = parent_genome_orientation * mode.child_b.orientation;
            
            // Physics rotations inherit from parent's physics rotation + child orientation delta
            // This preserves the parent's spin while applying the genome-specified orientation change
            let child_a_orientation = parent_rotation * mode.child_a.orientation;
            let child_b_orientation = parent_rotation * mode.child_b.orientation;
            
            division_data_list.push(DivisionData {
                parent_idx,
                parent_mode_idx: mode_index,
                child_a_slot: child_a_slot as usize,
                child_b_slot: child_b_slot as usize,
                parent_velocity,
                parent_radius,
                parent_genome_id,
                parent_stiffness,
                parent_split_count,
                parent_genome_orientation,  // CRITICAL: Save parent's genome orientation before overwriting
                child_a_pos,
                child_b_pos,
                child_a_orientation,
                child_b_orientation,
                child_a_genome_orientation,
                child_b_genome_orientation,
                child_a_mode_idx,
                child_b_mode_idx,
                child_a_split_mass,
                child_b_split_mass,
                child_a_radius,
                child_b_radius,
                child_a_split_interval,
                child_b_split_interval,
            });
        }
    }
    
    // Now write all the children to their allocated slots
    for data in &division_data_list {
        if data.child_a_slot < state.capacity {
            // Write child A
            let child_a_id = state.next_cell_id;
            state.cell_ids[data.child_a_slot] = child_a_id;
            state.next_cell_id += 1;
            state.positions[data.child_a_slot] = data.child_a_pos;
            state.prev_positions[data.child_a_slot] = data.child_a_pos;
            state.velocities[data.child_a_slot] = data.parent_velocity;
            state.masses[data.child_a_slot] = data.child_a_split_mass;
            state.radii[data.child_a_slot] = data.child_a_radius;
            state.genome_ids[data.child_a_slot] = data.parent_genome_id;
            state.mode_indices[data.child_a_slot] = data.child_a_mode_idx;

            // Apply pseudo-random rotation perturbation (0.001 radians)
            let random_rotation_a = pseudo_random_rotation(child_a_id, _rng_seed);
            state.rotations[data.child_a_slot] = data.child_a_orientation * random_rotation_a;

            state.genome_orientations[data.child_a_slot] = data.child_a_genome_orientation;
            state.angular_velocities[data.child_a_slot] = bevy::prelude::Vec3::ZERO;
            state.forces[data.child_a_slot] = bevy::prelude::Vec3::ZERO;
            state.torques[data.child_a_slot] = bevy::prelude::Vec3::ZERO;
            state.accelerations[data.child_a_slot] = bevy::prelude::Vec3::ZERO;
            state.prev_accelerations[data.child_a_slot] = bevy::prelude::Vec3::ZERO;
            state.stiffnesses[data.child_a_slot] = data.parent_stiffness;
            state.birth_times[data.child_a_slot] = current_time;
            state.split_intervals[data.child_a_slot] = data.child_a_split_interval;
            // Child A inherits parent's split count + 1
            state.split_counts[data.child_a_slot] = data.parent_split_count + 1;

            // Adhesion indices will be initialized in inheritance function (matches C++)
        }

        if data.child_b_slot < state.capacity {
            // Write child B
            let child_b_id = state.next_cell_id;
            state.cell_ids[data.child_b_slot] = child_b_id;
            state.next_cell_id += 1;
            state.positions[data.child_b_slot] = data.child_b_pos;
            state.prev_positions[data.child_b_slot] = data.child_b_pos;
            state.velocities[data.child_b_slot] = data.parent_velocity;
            state.masses[data.child_b_slot] = data.child_b_split_mass;
            state.radii[data.child_b_slot] = data.child_b_radius;
            state.genome_ids[data.child_b_slot] = data.parent_genome_id;
            state.mode_indices[data.child_b_slot] = data.child_b_mode_idx;

            // Apply pseudo-random rotation perturbation (0.001 radians)
            let random_rotation_b = pseudo_random_rotation(child_b_id, _rng_seed);
            state.rotations[data.child_b_slot] = data.child_b_orientation * random_rotation_b;

            state.genome_orientations[data.child_b_slot] = data.child_b_genome_orientation;
            state.angular_velocities[data.child_b_slot] = bevy::prelude::Vec3::ZERO;
            state.forces[data.child_b_slot] = bevy::prelude::Vec3::ZERO;
            state.torques[data.child_b_slot] = bevy::prelude::Vec3::ZERO;
            state.accelerations[data.child_b_slot] = bevy::prelude::Vec3::ZERO;
            state.prev_accelerations[data.child_b_slot] = bevy::prelude::Vec3::ZERO;
            state.stiffnesses[data.child_b_slot] = data.parent_stiffness;
            state.birth_times[data.child_b_slot] = current_time;
            state.split_intervals[data.child_b_slot] = data.child_b_split_interval;
            // Child B starts with fresh split count of 0
            state.split_counts[data.child_b_slot] = 0;

            // Initialize adhesion indices for child B
            state.adhesion_manager.init_cell_adhesion_indices(data.child_b_slot);
        }
        
        // Record the division event
        division_events.push(DivisionEvent {
            parent_idx: data.parent_idx,
            child_a_idx: data.child_a_slot,
            child_b_idx: data.child_b_slot,
        });
    }
    
    // Now handle adhesion inheritance and creation AFTER all children are written
    // Child A reuses parent index (matches C++), so neighborIndex automatically points to correct cell
    for data in &division_data_list {
        // Create child-to-child adhesion if parent mode allows it
        let mode_index = data.parent_mode_idx;
        let mode = genome.modes.get(mode_index);

        // Inherit adhesions from parent to children based on zone classification
        // CRITICAL: Pass parent's saved genome orientation, not from state (child A has overwritten it)
        crate::simulation::inherit_adhesions_on_division(
            state,
            genome,
            data.parent_mode_idx,
            data.child_a_slot,
            data.child_b_slot,
            data.parent_genome_orientation,
        );


        if let Some(mode) = mode {
            if mode.parent_make_adhesion && mode.child_a.keep_adhesion && mode.child_b.keep_adhesion {
                // CRITICAL: Use split direction from parent's GENOME orientation (not world positions!)
                // This ensures anchors stay aligned with the genome's intended split direction
                // even if physics has moved the cells slightly
                let pitch = mode.parent_split_direction.x.to_radians();
                let yaw = mode.parent_split_direction.y.to_radians();
                let split_dir_local = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0) * Vec3::Z;
                
                // CRITICAL: Match C++ implementation exactly
                // Direction vectors in parent's local frame:
                // Child A is at +offset, child B is at -offset
                // Child A points toward B (at -offset): -splitDirLocal
                // Child B points toward A (at +offset): +splitDirLocal
                // Transform to each child's local space using genome-derived orientation deltas
                let direction_a_to_b_parent_local = -split_dir_local;
                let direction_b_to_a_parent_local = split_dir_local;
                
                let anchor_direction_a = (mode.child_a.orientation.inverse() * direction_a_to_b_parent_local).normalize();
                let anchor_direction_b = (mode.child_b.orientation.inverse() * direction_b_to_a_parent_local).normalize();
                
                // Get genome orientations for twist references
                let child_a_genome_orientation = state.genome_orientations[data.child_a_slot];
                let child_b_genome_orientation = state.genome_orientations[data.child_b_slot];
                
                // Get child mode split directions for zone classification
                let child_a_mode = genome.modes.get(data.child_a_mode_idx);
                let child_b_mode = genome.modes.get(data.child_b_mode_idx);
                
                let child_a_split_dir = if let Some(m) = child_a_mode {
                    let pitch = m.parent_split_direction.x.to_radians();
                    let yaw = m.parent_split_direction.y.to_radians();
                    Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0) * Vec3::Z
                } else {
                    Vec3::Z
                };
                
                let child_b_split_dir = if let Some(m) = child_b_mode {
                    let pitch = m.parent_split_direction.x.to_radians();
                    let yaw = m.parent_split_direction.y.to_radians();
                    Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0) * Vec3::Z
                } else {
                    Vec3::Z
                };
                
                // Create child-to-child connection with parent's mode index
                let result = state.adhesion_manager.add_adhesion_with_directions(
                    &mut state.adhesion_connections,
                    data.child_a_slot,
                    data.child_b_slot,
                    mode_index,  // Use parent's mode index
                    anchor_direction_a,
                    anchor_direction_b,
                    child_a_split_dir,
                    child_b_split_dir,
                    child_a_genome_orientation,
                    child_b_genome_orientation,
                );
                
                let _ = result; // Suppress unused warning
            }
        }
    }
    
    // Optimized compaction: Since child A reuses parent index, we only need to move child B cells
    // from temporary slots to fill gaps left by non-dividing parents
    
    // Simply update cell count (child B cells are already written)
    let new_cell_count = state.cell_count + division_data_list.len();
    state.cell_count = new_cell_count;
    
    // No index remapping needed for adhesion connections since:
    // - Child A reuses parent index (adhesions already point to correct cell)
    // - Child B is at a new index (adhesions were created with correct index)
    // - Non-dividing cells keep their indices
    
    division_events
}

// ============================================================================
// Deterministic RNG Functions
// ============================================================================

/// Deterministic random value for a cell at a specific tick
/// 
/// This function generates pseudo-random values that are:
/// - Deterministic: same inputs always produce same output
/// - Independent: different cells/ticks produce different values
/// - Well-distributed: values are uniformly distributed in [0, 1)
/// 
/// # Arguments
/// * `cell_id` - Unique cell identifier
/// * `tick` - Current simulation tick
/// * `seed` - Global random seed
/// * `index` - Additional index for multiple random values per cell/tick
/// 
/// # Returns
/// A pseudo-random f32 value in the range [0, 1)
pub fn deterministic_random(cell_id: u32, tick: u64, seed: u64, index: u32) -> f32 {
    let hash = hash_u64(seed ^ (cell_id as u64) ^ (tick << 32) ^ (index as u64));
    (hash as f32) / (u64::MAX as f32)
}

/// FNV-1a hash for deterministic randomness
/// 
/// This is a simple, fast hash function that provides good distribution
/// for our deterministic RNG needs.
fn hash_u64(mut x: u64) -> u64 {
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;
    
    let mut hash = FNV_OFFSET;
    for _ in 0..8 {
        hash ^= x & 0xFF;
        hash = hash.wrapping_mul(FNV_PRIME);
        x >>= 8;
    }
    hash
}

// ============================================================================
// Structure-of-Arrays (SoA) Integration Functions
// ============================================================================
// These pure functions operate on slices and are used by the physics_step functions above.
// They provide both single-threaded and multi-threaded versions for flexibility.

/// Verlet integration position update (SoA version) - Single-threaded
/// Position update: x(t+dt) = x(t) + v(t)*dt + 0.5*a(t)*dt²
pub fn verlet_integrate_positions_soa_st(
    positions: &mut [Vec3],
    velocities: &[Vec3],
    accelerations: &[Vec3],
    dt: f32,
) {
    let dt_sq = dt * dt;
    for i in 0..positions.len() {
        if velocities[i].is_finite() && accelerations[i].is_finite() {
            positions[i] += velocities[i] * dt + 0.5 * accelerations[i] * dt_sq;
        }
    }
}

/// Verlet integration position update (SoA version) - Multithreaded
/// Position update: x(t+dt) = x(t) + v(t)*dt + 0.5*a(t)*dt²
/// 
/// Uses parallel iteration for improved performance with large cell counts.
pub fn verlet_integrate_positions_soa(
    positions: &mut [Vec3],
    velocities: &[Vec3],
    accelerations: &[Vec3],
    dt: f32,
) {
    use rayon::prelude::*;
    
    let dt_sq = dt * dt;
    
    // Use parallel iteration for better performance with many cells
    positions.par_iter_mut()
        .zip(velocities.par_iter())
        .zip(accelerations.par_iter())
        .for_each(|((pos, vel), acc)| {
            if vel.is_finite() && acc.is_finite() {
                *pos += *vel * dt + 0.5 * *acc * dt_sq;
            }
        });
}

/// Verlet integration velocity update (SoA version) - Single-threaded
/// Velocity update: v(t+dt) = v(t) + 0.5*(a(t) + a(t+dt))*dt
pub fn verlet_integrate_velocities_soa_st(
    velocities: &mut [Vec3],
    accelerations: &mut [Vec3],
    prev_accelerations: &mut [Vec3],
    forces: &[Vec3],
    masses: &[f32],
    dt: f32,
    velocity_damping: f32,
) {
    let velocity_damping_factor = velocity_damping.powf(dt * 100.0);
    
    for i in 0..velocities.len() {
        // Skip invalid masses
        if masses[i] <= 0.0 || !masses[i].is_finite() {
            continue;
        }
        
        let old_acceleration = accelerations[i];
        let new_acceleration = forces[i] / masses[i];
        
        if new_acceleration.is_finite() {
            let velocity_change = 0.5 * (old_acceleration + new_acceleration) * dt;
            velocities[i] = (velocities[i] + velocity_change) * velocity_damping_factor;
            accelerations[i] = new_acceleration;
            prev_accelerations[i] = old_acceleration;
        }
    }
}

/// Verlet integration velocity update (SoA version) - Multithreaded
/// Velocity update: v(t+dt) = v(t) + 0.5*(a(t) + a(t+dt))*dt
/// 
/// Uses parallel iteration for improved performance with large cell counts.
pub fn verlet_integrate_velocities_soa(
    velocities: &mut [Vec3],
    accelerations: &mut [Vec3],
    prev_accelerations: &mut [Vec3],
    forces: &[Vec3],
    masses: &[f32],
    dt: f32,
    velocity_damping: f32,
) {
    use rayon::prelude::*;
    
    let velocity_damping_factor = velocity_damping.powf(dt * 100.0);
    
    // Use parallel iteration for better performance with many cells
    velocities.par_iter_mut()
        .zip(accelerations.par_iter_mut())
        .zip(prev_accelerations.par_iter_mut())
        .zip(forces.par_iter())
        .zip(masses.par_iter())
        .for_each(|((((vel, acc), prev_acc), force), mass)| {
            // Skip invalid masses
            if *mass <= 0.0 || !mass.is_finite() {
                return;
            }
            
            let old_acceleration = *acc;
            let new_acceleration = *force / *mass;
            
            if new_acceleration.is_finite() {
                let velocity_change = 0.5 * (old_acceleration + new_acceleration) * dt;
                *vel = (*vel + velocity_change) * velocity_damping_factor;
                *acc = new_acceleration;
                *prev_acc = old_acceleration;
            }
        });
}

/// Apply boundary velocity reversal for cells crossing the spherical boundary (SoA version) - Single-threaded
pub fn apply_boundary_forces_soa_st(
    positions: &[Vec3],
    velocities: &mut [Vec3],
    config: &crate::simulation::PhysicsConfig,
) {
    for i in 0..positions.len() {
        let distance_from_origin = positions[i].length();
        
        if distance_from_origin > config.sphere_radius {
            let r_hat = if distance_from_origin > 0.0001 {
                positions[i] / distance_from_origin
            } else {
                continue;
            };
            
            // Decompose velocity into radial and tangential components
            let radial_component_magnitude = velocities[i].dot(r_hat);
            let v_radial = radial_component_magnitude * r_hat;
            let v_tangential = velocities[i] - v_radial;
            
            // Reverse radial component
            velocities[i] = v_tangential - v_radial;
        }
    }
}

/// Apply boundary velocity reversal for cells crossing the spherical boundary (SoA version) - Multithreaded
/// 
/// Uses parallel iteration for improved performance with large cell counts.
pub fn apply_boundary_forces_soa(
    positions: &[Vec3],
    velocities: &mut [Vec3],
    config: &crate::simulation::PhysicsConfig,
) {
    use rayon::prelude::*;
    
    let sphere_radius = config.sphere_radius;
    
    // Use parallel iteration for better performance with many cells
    positions.par_iter()
        .zip(velocities.par_iter_mut())
        .for_each(|(pos, vel)| {
            let distance_from_origin = pos.length();
            
            if distance_from_origin > sphere_radius {
                let r_hat = if distance_from_origin > 0.0001 {
                    *pos / distance_from_origin
                } else {
                    return;
                };
                
                // Decompose velocity into radial and tangential components
                let radial_component_magnitude = vel.dot(r_hat);
                let v_radial = radial_component_magnitude * r_hat;
                let v_tangential = *vel - v_radial;
                
                // Reverse radial component
                *vel = v_tangential - v_radial;
            }
        });
}

/// Update angular velocities from torques (SoA version) - Single-threaded
pub fn integrate_angular_velocities_soa_st(
    angular_velocities: &mut [Vec3],
    torques: &[Vec3],
    radii: &[f32],
    masses: &[f32],
    dt: f32,
    angular_damping: f32,
) {
    let angular_damping_factor = angular_damping.powf(dt * 100.0);
    
    for i in 0..angular_velocities.len() {
        if masses[i] <= 0.0 || !masses[i].is_finite() || radii[i] <= 0.0 {
            continue;
        }
        
        // Moment of inertia for a sphere: I = (2/5) * m * r²
        let moment_of_inertia = 0.4 * masses[i] * radii[i] * radii[i];
        
        if moment_of_inertia > 0.0 {
            let angular_acceleration = torques[i] / moment_of_inertia;
            angular_velocities[i] = (angular_velocities[i] + angular_acceleration * dt) * angular_damping_factor;
        }
    }
}

/// Update angular velocities from torques (SoA version) - Multithreaded
pub fn integrate_angular_velocities_soa(
    angular_velocities: &mut [Vec3],
    torques: &[Vec3],
    radii: &[f32],
    masses: &[f32],
    dt: f32,
    angular_damping: f32,
) {
    use rayon::prelude::*;
    
    let angular_damping_factor = angular_damping.powf(dt * 100.0);
    
    angular_velocities.par_iter_mut()
        .zip(torques.par_iter())
        .zip(radii.par_iter())
        .zip(masses.par_iter())
        .for_each(|(((ang_vel, torque), radius), mass)| {
            if *mass <= 0.0 || !mass.is_finite() || *radius <= 0.0 {
                return;
            }
            
            // Moment of inertia for a sphere: I = (2/5) * m * r²
            let moment_of_inertia = 0.4 * *mass * *radius * *radius;
            
            if moment_of_inertia > 0.0 {
                let angular_acceleration = *torque / moment_of_inertia;
                *ang_vel = (*ang_vel + angular_acceleration * dt) * angular_damping_factor;
            }
        });
}

/// Update rotations from angular velocities (SoA version) - Single-threaded
pub fn integrate_rotations_soa_st(
    rotations: &mut [Quat],
    angular_velocities: &[Vec3],
    dt: f32,
) {
    for i in 0..rotations.len() {
        let ang_vel = angular_velocities[i];
        if ang_vel.length_squared() > 0.0001 {
            let angle = ang_vel.length() * dt;
            let axis = ang_vel.normalize();
            let delta_rotation = Quat::from_axis_angle(axis, angle);
            rotations[i] = (delta_rotation * rotations[i]).normalize();
        }
    }
}

/// Update rotations from angular velocities (SoA version) - Multithreaded
pub fn integrate_rotations_soa(
    rotations: &mut [Quat],
    angular_velocities: &[Vec3],
    dt: f32,
) {
    use rayon::prelude::*;
    
    rotations.par_iter_mut()
        .zip(angular_velocities.par_iter())
        .for_each(|(rotation, ang_vel)| {
            if ang_vel.length_squared() > 0.0001 {
                let angle = ang_vel.length() * dt;
                let axis = ang_vel.normalize();
                let delta_rotation = Quat::from_axis_angle(axis, angle);
                *rotation = (delta_rotation * *rotation).normalize();
            }
        });
}

