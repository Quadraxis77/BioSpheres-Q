use bevy::prelude::*;

/// Physics simulation module with multithreaded support
/// 
/// This module provides deterministic physics simulation using Verlet integration
/// with parallel processing via Rayon for improved performance on multi-core systems.
/// 
/// Key features:
/// - Parallel Verlet integration for positions and velocities (via Rayon)
/// - Parallel boundary force application
/// - Deterministic results (bit-identical across runs)
/// - Structure-of-Arrays (SoA) layout for cache efficiency
/// 
/// The parallel operations automatically scale with available CPU cores while
/// maintaining deterministic ordering through careful synchronization.

/// Cytoskeleton properties affecting collision response
#[derive(Component, Default, Clone, Copy)]
pub struct Cytoskeleton {
    pub stiffness: f32,
}

/// Accumulated forces and acceleration for Velocity Verlet integration
#[derive(Component, Default, Clone)]
pub struct CellForces {
    pub force: Vec3,
    pub acceleration: Vec3,
    pub prev_acceleration: Vec3,
}

/// Physics configuration for deterministic simulation
/// 
/// This configuration is used by both synchronous and asynchronous physics execution.
/// All values are deterministic and produce identical results across runs.
#[derive(Resource, Clone, Debug)]
pub struct PhysicsConfig {
    /// World bounds (cubic volume)
    pub world_bounds: Vec3,
    
    /// Spherical boundary radius for active simulation
    pub sphere_radius: f32,
    
    /// Default cell stiffness (matches desktop: hardness = 10.0)
    pub default_stiffness: f32,
    
    /// Collision damping coefficient (desktop has NO damping in collision forces)
    pub damping: f32,
    
    /// Fixed timestep for physics integration (64 Hz ≈ 15.6ms)
    pub fixed_timestep: f32,
    
    /// Velocity damping coefficient (matches desktop: 0.98)
    /// Applied as pow(velocity_damping, dt * 100.0)
    pub velocity_damping: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            world_bounds: Vec3::splat(100.0),
            sphere_radius: 50.0,
            default_stiffness: 10.0,
            damping: 0.0,
            fixed_timestep: 1.0 / 64.0, // Match Bevy's default fixed timestep (64 Hz)
            velocity_damping: 0.98,
        }
    }
}

/// Run a single physics step synchronously (blocking)
/// 
/// This is the standard single-threaded path. Physics runs on the calling thread
/// and blocks until complete. Use this for:
/// - Low cell counts (< 1000 cells)
/// - Debugging (simpler to trace)
/// - When latency is critical (no frame delay)
pub fn run_physics_step_sync(
    state: &mut crate::simulation::CanonicalState,
    config: &PhysicsConfig,
    genome: &crate::genome::GenomeData,
    current_time: f32,
    max_cells: usize,
    rng_seed: u64,
) {
    // Run physics step
    crate::simulation::canonical_physics::physics_step(state, config);
    
    // Run division step
    crate::simulation::canonical_physics::division_step(
        state,
        genome,
        current_time,
        max_cells,
        rng_seed,
    );
}

/// Run multiple physics steps synchronously (blocking)
/// 
/// Used for preview resimulation in single-threaded mode.
pub fn run_multi_step_sync(
    state: &mut crate::simulation::CanonicalState,
    config: &PhysicsConfig,
    genome: &crate::genome::GenomeData,
    start_time: f32,
    step_count: u32,
    max_cells: usize,
    rng_seed: u64,
) {
    for step in 0..step_count {
        let current_time = start_time + (step as f32 * config.fixed_timestep);
        run_physics_step_sync(state, config, genome, current_time, max_cells, rng_seed);
    }
}

// ============================================================================
// Pure SoA Physics Functions (no ECS dependencies)
// These functions operate on Structure-of-Arrays data and are used by
// canonical_physics for deterministic physics
// ============================================================================

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
    config: &PhysicsConfig,
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
    config: &PhysicsConfig,
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

/// System that synchronizes Transform components with CellPosition
/// Copies CellPosition.position to Transform.translation for rendering
pub fn sync_transforms(
    mut cells_query: Query<(&crate::cell::CellPosition, &mut Transform)>,
) {
    for (cell_position, mut transform) in cells_query.iter_mut() {
        transform.translation = cell_position.position;
    }
}
