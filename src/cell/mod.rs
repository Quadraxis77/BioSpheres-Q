use bevy::prelude::*;

pub mod adhesion;
pub mod adhesion_forces;
pub mod adhesion_manager;
pub mod adhesion_zones;
pub mod division;
pub mod types;
pub mod type_registry;

pub use adhesion::{AdhesionPlugin, AdhesionSettings, AdhesionConnections, AdhesionIndices, MAX_ADHESIONS_PER_CELL, MAX_ADHESION_CONNECTIONS};
pub use adhesion_forces::{compute_adhesion_forces, compute_adhesion_forces_parallel, compute_adhesion_forces_batched};
pub use adhesion_manager::AdhesionConnectionManager;
pub use adhesion_zones::{AdhesionZone, classify_bond_direction, get_zone_color, EQUATORIAL_THRESHOLD_DEGREES};
pub use division::{DivisionPlugin, DivisionQueue, has_pending_divisions};
pub use types::TypesPlugin;
pub use type_registry::{CellTypeRegistry, CellTypeMetadata, CellTypeRegistryPlugin};

// Re-export physics types for backwards compatibility
pub mod physics {
    pub use crate::simulation::physics_config::PhysicsConfig;
    pub use crate::simulation::cpu_physics::{
        verlet_integrate_positions_soa, verlet_integrate_velocities_soa,
        verlet_integrate_positions_soa_st, verlet_integrate_velocities_soa_st,
        apply_boundary_forces_soa, apply_boundary_forces_soa_st,
        integrate_angular_velocities_soa, integrate_angular_velocities_soa_st,
        integrate_rotations_soa, integrate_rotations_soa_st,
    };
    pub use crate::rendering::sync_transforms;
    pub use super::{CellForces, Cytoskeleton};
}

/// Main cell plugin that coordinates all cell-related functionality
pub struct CellPlugin;

impl Plugin for CellPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CellTypeRegistryPlugin)
            .add_plugins(TypesPlugin)
            .add_plugins(DivisionPlugin)
            .add_plugins(AdhesionPlugin);
    }
}

/// Core cell component - every cell entity has this
#[derive(Component, Default, Clone, Copy)]
pub struct Cell {
    pub mass: f32,
    pub radius: f32,
    pub genome_id: usize,
    pub mode_index: usize,
    pub cell_type: i32, // Cache cell type for detecting changes
}

/// Cell position in 3D space
#[derive(Component, Default, Clone, Copy)]
pub struct CellPosition {
    pub position: Vec3,
    pub velocity: Vec3,
}

/// Cell orientation
#[derive(Component, Default, Clone, Copy)]
pub struct CellOrientation {
    pub rotation: Quat,
    pub angular_velocity: Vec3,
}

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

/// Signaling substances (like Cell Lab)
#[derive(Component, Default)]
pub struct CellSignaling {
    pub s1: f32,
    pub s2: f32,
    pub s3: f32,
    pub s4: f32,
}
