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
    
    // === Physics State (SoA) ===
    pub forces: Vec<Vec3>,
    pub accelerations: Vec<Vec3>,
    pub prev_accelerations: Vec<Vec3>,
    pub stiffnesses: Vec<f32>,
    
    // === Division Timers (SoA) ===
    pub birth_times: Vec<f32>,
    pub split_intervals: Vec<f32>,
    
    /// Spatial partitioning for collision detection
    pub spatial_grid: DeterministicSpatialGrid,
    
    /// Next cell ID to assign (monotonically increasing)
    pub next_cell_id: u32,
}

impl CanonicalState {
    /// Create a new canonical state with the specified capacity
    pub fn new(capacity: usize) -> Self {
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
            forces: vec![Vec3::ZERO; capacity],
            accelerations: vec![Vec3::ZERO; capacity],
            prev_accelerations: vec![Vec3::ZERO; capacity],
            stiffnesses: vec![10.0; capacity],
            birth_times: vec![0.0; capacity],
            split_intervals: vec![10.0; capacity],
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
        self.forces[idx] = Vec3::ZERO;
        self.accelerations[idx] = Vec3::ZERO;
        self.prev_accelerations[idx] = Vec3::ZERO;
        self.stiffnesses[idx] = stiffness;
        self.birth_times[idx] = birth_time;
        self.split_intervals[idx] = split_interval;
        
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
    /// 1. Count cells per grid cell
    /// 2. Compute offsets using prefix sum
    /// 3. Insert cell indices into flat array
    pub fn rebuild(&mut self, positions: &[Vec3], cell_count: usize) {
        // Optimization: Only clear grid cells that were used in the last rebuild
        // instead of clearing the entire array (which can be 2000+ cells)
        
        // Step 1: Clear only previously used counts
        for &idx in &self.used_grid_cells {
            self.cell_counts[idx] = 0;
        }
        self.used_grid_cells.clear();
        
        // Step 2: Count cells per grid cell and track which cells are used
        for i in 0..cell_count {
            let grid_coord = self.world_to_grid(positions[i]);
            if let Some(idx) = self.active_cell_index(grid_coord) {
                if self.cell_counts[idx] == 0 {
                    self.used_grid_cells.push(idx);
                }
                self.cell_counts[idx] += 1;
            }
        }
        
        // Step 3: Compute offsets using prefix sum (only for used cells)
        let mut offset = 0;
        for &idx in &self.used_grid_cells {
            self.cell_offsets[idx] = offset;
            offset += self.cell_counts[idx];
        }
        
        // Step 4: Reset counts for insertion phase (only used cells)
        for &idx in &self.used_grid_cells {
            self.cell_counts[idx] = 0;
        }
        
        // Step 5: Insert cell indices into flat array
        for i in 0..cell_count {
            let grid_coord = self.world_to_grid(positions[i]);
            if let Some(idx) = self.active_cell_index(grid_coord) {
                let insert_pos = self.cell_offsets[idx] + self.cell_counts[idx];
                self.cell_contents[insert_pos] = i;
                self.cell_counts[idx] += 1;
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

/// Compute collision forces from detected collision pairs - Single-threaded version
pub fn compute_collision_forces_canonical_st(
    state: &mut CanonicalState,
    collision_pairs: &[CanonicalCollisionPair],
    config: &crate::cell::physics::PhysicsConfig,
) {
    // Clear all forces
    for i in 0..state.cell_count {
        state.forces[i] = Vec3::ZERO;
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
        
        // Clamp force magnitude
        let max_force = 10000.0;
        let clamped_force_magnitude = total_force_magnitude.clamp(-max_force, max_force);
        let force = clamped_force_magnitude * pair.normal;
        
        // Apply equal and opposite forces
        state.forces[idx_b] += force;
        state.forces[idx_a] -= force;
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
    
    // Clear all forces (parallel)
    state.forces[..state.cell_count].par_iter_mut().for_each(|f| *f = Vec3::ZERO);
    
    // Compute forces for each collision pair in parallel
    // Store as (index, force) pairs to accumulate deterministically
    let force_contributions: Vec<(usize, Vec3)> = collision_pairs
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
            
            // Clamp force magnitude
            let max_force = 10000.0;
            let clamped_force_magnitude = total_force_magnitude.clamp(-max_force, max_force);
            let force = clamped_force_magnitude * pair.normal;
            
            // Return force contributions for both cells
            vec![(idx_b, force), (idx_a, -force)]
        })
        .collect();
    
    // Accumulate forces sequentially to maintain determinism
    // (Parallel accumulation would be non-deterministic due to floating-point addition order)
    for (idx, force) in force_contributions {
        state.forces[idx] += force;
    }
}

/// Deterministic physics step function - Single-threaded version
/// Used by preview simulation for simpler, more predictable performance
pub fn physics_step_st(
    state: &mut CanonicalState,
    config: &crate::cell::physics::PhysicsConfig,
) {
    // 1. Verlet integration (position update)
    crate::cell::physics::verlet_integrate_positions_soa_st(
        &mut state.positions[..state.cell_count],
        &state.velocities[..state.cell_count],
        &state.accelerations[..state.cell_count],
        config.fixed_timestep,
    );
    
    // 2. Update spatial partitioning
    state.spatial_grid.rebuild(&state.positions, state.cell_count);
    
    // 3. Detect collisions
    let collisions = detect_collisions_canonical_st(state);
    
    // 4. Compute forces
    compute_collision_forces_canonical_st(state, &collisions, config);
    
    // 5. Apply boundary conditions
    crate::cell::physics::apply_boundary_forces_soa_st(
        &state.positions[..state.cell_count],
        &mut state.velocities[..state.cell_count],
        config,
    );
    
    // 6. Verlet integration (velocity update)
    crate::cell::physics::verlet_integrate_velocities_soa_st(
        &mut state.velocities[..state.cell_count],
        &mut state.accelerations[..state.cell_count],
        &mut state.prev_accelerations[..state.cell_count],
        &state.forces[..state.cell_count],
        &state.masses[..state.cell_count],
        config.fixed_timestep,
        config.velocity_damping,
    );
}

/// Deterministic physics step function - Multithreaded version
/// This is the core function called by Main simulation mode
pub fn physics_step(
    state: &mut CanonicalState,
    config: &crate::cell::physics::PhysicsConfig,
) {
    // 1. Verlet integration (position update)
    crate::cell::physics::verlet_integrate_positions_soa(
        &mut state.positions[..state.cell_count],
        &state.velocities[..state.cell_count],
        &state.accelerations[..state.cell_count],
        config.fixed_timestep,
    );
    
    // 2. Update spatial partitioning
    state.spatial_grid.rebuild(&state.positions, state.cell_count);
    
    // 3. Detect collisions
    let collisions = detect_collisions_canonical(state);
    
    // 4. Compute forces
    compute_collision_forces_canonical(state, &collisions, config);
    
    // 5. Apply boundary conditions
    crate::cell::physics::apply_boundary_forces_soa(
        &state.positions[..state.cell_count],
        &mut state.velocities[..state.cell_count],
        config,
    );
    
    // 6. Verlet integration (velocity update)
    crate::cell::physics::verlet_integrate_velocities_soa(
        &mut state.velocities[..state.cell_count],
        &mut state.accelerations[..state.cell_count],
        &mut state.prev_accelerations[..state.cell_count],
        &state.forces[..state.cell_count],
        &state.masses[..state.cell_count],
        config.fixed_timestep,
        config.velocity_damping,
    );
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
/// * `_rng_seed` - Random seed (currently unused, reserved for future stochastic division)
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
    use crate::simulation::cell_allocation::Simulation as AllocationSim;
    
    // Early exit if at capacity
    if state.cell_count >= max_cells {
        return Vec::new();
    }
    
    // Find cells ready to divide
    let mut divisions_to_process = Vec::new();
    for i in 0..state.cell_count {
        let cell_age = current_time - state.birth_times[i];
        if cell_age >= state.split_intervals[i] {
            divisions_to_process.push(i);
        }
    }
    
    // Early exit if no divisions - avoid expensive AllocationSim creation
    if divisions_to_process.is_empty() {
        return Vec::new();
    }
    
    println!("Division at t={}: {} cells dividing", current_time, divisions_to_process.len());
    
    // Create allocation simulation for deterministic slot assignment
    // NOTE: This allocates large vectors - only do this when actually needed
    let mut alloc_sim = AllocationSim::new(max_cells, max_cells * 40);
    
    // Mark all current cells as free initially
    for i in 0..alloc_sim.cells.len() {
        alloc_sim.cells[i].age = -1; // Free
    }
    
    // Mark dividing cells as occupied (so they generate reservations)
    for &parent_idx in &divisions_to_process {
        if parent_idx < alloc_sim.cells.len() {
            alloc_sim.cells[parent_idx].age = 0; // Occupied (will divide)
        }
    }
    
    // Run allocation pipeline to get deterministic slot assignments
    alloc_sim.identify_free_cell_slots();
    alloc_sim.generate_reservations();
    alloc_sim.compact_arrays();
    alloc_sim.assign_reservations();
    
    // Collect division data before modifying state
    struct DivisionData {
        parent_idx: usize,
        child_a_slot: usize,
        child_b_slot: usize,
        parent_velocity: bevy::prelude::Vec3,
        parent_radius: f32,
        parent_genome_id: usize,
        parent_stiffness: f32,
        child_a_pos: bevy::prelude::Vec3,
        child_b_pos: bevy::prelude::Vec3,
        child_a_orientation: bevy::prelude::Quat,
        child_b_orientation: bevy::prelude::Quat,
        child_a_mode_idx: usize,
        child_b_mode_idx: usize,
        child_a_split_mass: f32,
        child_b_split_mass: f32,
        child_a_split_interval: f32,
        child_b_split_interval: f32,
    }
    
    let mut division_data_list = Vec::new();
    let mut division_events = Vec::new();
    
    // Process each division and collect data
    for &parent_idx in &divisions_to_process {
        // Get allocated slots from the allocation system
        let child_a_reservation = 2 * parent_idx;
        let child_b_reservation = 2 * parent_idx + 1;
        
        if child_a_reservation >= alloc_sim.assignments_cells.len() ||
           child_b_reservation >= alloc_sim.assignments_cells.len() {
            continue;
        }
        
        let child_a_slot = alloc_sim.assignments_cells[child_a_reservation];
        let child_b_slot = alloc_sim.assignments_cells[child_b_reservation];
        
        // Skip if slots weren't assigned (BIG sentinel value)
        if child_a_slot == u32::MAX || child_b_slot == u32::MAX {
            continue;
        }
        
        let mode_index = state.mode_indices[parent_idx];
        let mode = genome.modes.get(mode_index);
        
        if let Some(mode) = mode {
            // Save parent properties
            let parent_position = state.positions[parent_idx];
            let parent_velocity = state.velocities[parent_idx];
            let parent_rotation = state.rotations[parent_idx];
            let parent_radius = state.radii[parent_idx];
            let parent_genome_id = state.genome_ids[parent_idx];
            let parent_stiffness = state.stiffnesses[parent_idx];
            
            // Calculate split direction
            let pitch = mode.parent_split_direction.x.to_radians();
            let yaw = mode.parent_split_direction.y.to_radians();
            let split_direction = parent_rotation
                * bevy::prelude::Quat::from_euler(bevy::prelude::EulerRot::YXZ, yaw, pitch, 0.0)
                * bevy::prelude::Vec3::Z;
            
            // 75% overlap means centers are 25% of combined diameter apart
            let offset_distance = parent_radius * 0.25;
            let child_a_pos = parent_position - split_direction * offset_distance;
            let child_b_pos = parent_position + split_direction * offset_distance;
            
            // Get child mode indices
            let child_a_mode_idx = mode.child_a.mode_number.max(0) as usize;
            let child_b_mode_idx = mode.child_b.mode_number.max(0) as usize;
            
            // Get child properties
            let child_a_mode = genome.modes.get(child_a_mode_idx);
            let child_b_mode = genome.modes.get(child_b_mode_idx);
            
            let (child_a_split_interval, child_a_split_mass) = if let Some(m) = child_a_mode {
                (m.split_interval, m.split_mass)
            } else {
                (5.0, 1.0)
            };
            
            let (child_b_split_interval, child_b_split_mass) = if let Some(m) = child_b_mode {
                (m.split_interval, m.split_mass)
            } else {
                (5.0, 1.0)
            };
            
            let child_a_orientation = parent_rotation * mode.child_a.orientation;
            let child_b_orientation = parent_rotation * mode.child_b.orientation;
            
            division_data_list.push(DivisionData {
                parent_idx,
                child_a_slot: child_a_slot as usize,
                child_b_slot: child_b_slot as usize,
                parent_velocity,
                parent_radius,
                parent_genome_id,
                parent_stiffness,
                child_a_pos,
                child_b_pos,
                child_a_orientation,
                child_b_orientation,
                child_a_mode_idx,
                child_b_mode_idx,
                child_a_split_mass,
                child_b_split_mass,
                child_a_split_interval,
                child_b_split_interval,
            });
        }
    }
    
    // Now write all the children to their allocated slots
    for data in &division_data_list {
        if data.child_a_slot < state.capacity {
            // Write child A
            state.cell_ids[data.child_a_slot] = state.next_cell_id;
            state.next_cell_id += 1;
            state.positions[data.child_a_slot] = data.child_a_pos;
            state.prev_positions[data.child_a_slot] = data.child_a_pos;
            state.velocities[data.child_a_slot] = data.parent_velocity;
            state.masses[data.child_a_slot] = data.child_a_split_mass;
            state.radii[data.child_a_slot] = data.parent_radius;
            state.genome_ids[data.child_a_slot] = data.parent_genome_id;
            state.mode_indices[data.child_a_slot] = data.child_a_mode_idx;
            state.rotations[data.child_a_slot] = data.child_a_orientation;
            state.angular_velocities[data.child_a_slot] = bevy::prelude::Vec3::ZERO;
            state.forces[data.child_a_slot] = bevy::prelude::Vec3::ZERO;
            state.accelerations[data.child_a_slot] = bevy::prelude::Vec3::ZERO;
            state.prev_accelerations[data.child_a_slot] = bevy::prelude::Vec3::ZERO;
            state.stiffnesses[data.child_a_slot] = data.parent_stiffness;
            state.birth_times[data.child_a_slot] = current_time;
            state.split_intervals[data.child_a_slot] = data.child_a_split_interval;
        }
        
        if data.child_b_slot < state.capacity {
            // Write child B
            state.cell_ids[data.child_b_slot] = state.next_cell_id;
            state.next_cell_id += 1;
            state.positions[data.child_b_slot] = data.child_b_pos;
            state.prev_positions[data.child_b_slot] = data.child_b_pos;
            state.velocities[data.child_b_slot] = data.parent_velocity;
            state.masses[data.child_b_slot] = data.child_b_split_mass;
            state.radii[data.child_b_slot] = data.parent_radius;
            state.genome_ids[data.child_b_slot] = data.parent_genome_id;
            state.mode_indices[data.child_b_slot] = data.child_b_mode_idx;
            state.rotations[data.child_b_slot] = data.child_b_orientation;
            state.angular_velocities[data.child_b_slot] = bevy::prelude::Vec3::ZERO;
            state.forces[data.child_b_slot] = bevy::prelude::Vec3::ZERO;
            state.accelerations[data.child_b_slot] = bevy::prelude::Vec3::ZERO;
            state.prev_accelerations[data.child_b_slot] = bevy::prelude::Vec3::ZERO;
            state.stiffnesses[data.child_b_slot] = data.parent_stiffness;
            state.birth_times[data.child_b_slot] = current_time;
            state.split_intervals[data.child_b_slot] = data.child_b_split_interval;
        }
        
        // Record the division event
        division_events.push(DivisionEvent {
            parent_idx: data.parent_idx,
            child_a_idx: data.child_a_slot,
            child_b_idx: data.child_b_slot,
        });
    }
    
    // Compact the canonical state to remove gaps
    // After division, we have children written to various slots, but parents are still in the array
    // We need to collect all active cells (children) and compact them to indices 0..N
    
    // Mark which slots contain active cells (children)
    let mut active_slots = vec![false; state.capacity];
    for data in &division_data_list {
        active_slots[data.child_a_slot] = true;
        active_slots[data.child_b_slot] = true;
    }
    
    // Collect indices of active cells
    let mut active_indices: Vec<usize> = active_slots.iter()
        .enumerate()
        .filter_map(|(i, &active)| if active { Some(i) } else { None })
        .collect();
    
    // Sort to maintain deterministic ordering
    active_indices.sort_unstable();
    
    // Compact: move all active cells to the front of the arrays
    for (new_idx, &old_idx) in active_indices.iter().enumerate() {
        if new_idx != old_idx {
            // Swap data from old_idx to new_idx
            state.cell_ids.swap(new_idx, old_idx);
            state.positions.swap(new_idx, old_idx);
            state.prev_positions.swap(new_idx, old_idx);
            state.velocities.swap(new_idx, old_idx);
            state.masses.swap(new_idx, old_idx);
            state.radii.swap(new_idx, old_idx);
            state.genome_ids.swap(new_idx, old_idx);
            state.mode_indices.swap(new_idx, old_idx);
            state.rotations.swap(new_idx, old_idx);
            state.angular_velocities.swap(new_idx, old_idx);
            state.forces.swap(new_idx, old_idx);
            state.accelerations.swap(new_idx, old_idx);
            state.prev_accelerations.swap(new_idx, old_idx);
            state.stiffnesses.swap(new_idx, old_idx);
            state.birth_times.swap(new_idx, old_idx);
            state.split_intervals.swap(new_idx, old_idx);
        }
    }
    
    // Update cell count to reflect only active cells
    state.cell_count = active_indices.len();
    
    // Update division events with new indices after compaction
    let mut index_mapping: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for (new_idx, &old_idx) in active_indices.iter().enumerate() {
        index_mapping.insert(old_idx, new_idx);
    }
    
    for event in &mut division_events {
        event.child_a_idx = *index_mapping.get(&event.child_a_idx).unwrap_or(&event.child_a_idx);
        event.child_b_idx = *index_mapping.get(&event.child_b_idx).unwrap_or(&event.child_b_idx);
    }
    
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
