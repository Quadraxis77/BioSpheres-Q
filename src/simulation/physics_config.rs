use bevy::prelude::*;

/// Spatial grid configuration for collision detection
/// 
/// Controls the density of the spatial partitioning grid used for
/// efficient collision detection. Higher density = more grid cells = 
/// better performance with many cells but more memory usage.
#[derive(Resource, Clone, Debug)]
pub struct SpatialGridConfig {
    /// Grid dimensions (NxNxN cells). Valid range: 16-128
    pub grid_density: u32,
}

impl Default for SpatialGridConfig {
    fn default() -> Self {
        Self {
            grid_density: 64, // Default 64x64x64 grid
        }
    }
}

impl SpatialGridConfig {
    pub const MIN_DENSITY: u32 = 16;
    pub const MAX_DENSITY: u32 = 128;
    
    /// Clamp grid density to valid range
    pub fn clamped_density(&self) -> u32 {
        self.grid_density.clamp(Self::MIN_DENSITY, Self::MAX_DENSITY)
    }
}

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
            world_bounds: Vec3::splat(200.0),
            sphere_radius: 100.0,
            default_stiffness: 500.0,  // Increased from 10.0 to prevent pass-through
            damping: 0.0, // Increased from 0.0 to add velocity-based resistance
            fixed_timestep: 1.0 / 64.0, // Match Bevy's default fixed timestep (64 Hz)
            velocity_damping: 0.98,
            friction_coefficient: 0.3,
            angular_damping: 0.95,
            disable_collisions: false,
        }
    }
}
