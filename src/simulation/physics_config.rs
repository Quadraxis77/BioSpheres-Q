use bevy::prelude::*;

/// Physics configuration for deterministic simulation
/// 
/// This configuration is shared by both CPU and GPU physics implementations.
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
    
    /// Tangential friction coefficient for rolling contact
    pub friction_coefficient: f32,
    
    /// Angular velocity damping coefficient
    pub angular_damping: f32,
    
    /// Disable collision detection (for performance testing or specific scenarios)
    pub disable_collisions: bool,
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
            friction_coefficient: 0.3,
            angular_damping: 0.95,
            disable_collisions: false,
        }
    }
}
