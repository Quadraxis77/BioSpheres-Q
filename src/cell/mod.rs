use bevy::prelude::*;

pub mod adhesion;
pub mod division;
pub mod types;

pub use adhesion::AdhesionPlugin;
pub use division::{DivisionPlugin, DivisionQueue, has_pending_divisions};
pub use types::TypesPlugin;

// Re-export physics types from simulation module for backwards compatibility
pub mod physics {
    pub use crate::simulation::physics::{
        PhysicsConfig, CellForces, Cytoskeleton, 
        verlet_integrate_positions_soa, verlet_integrate_velocities_soa,
        verlet_integrate_positions_soa_st, verlet_integrate_velocities_soa_st,
        apply_boundary_forces_soa, apply_boundary_forces_soa_st,
        integrate_angular_velocities_soa, integrate_angular_velocities_soa_st,
        integrate_rotations_soa, integrate_rotations_soa_st,
        sync_transforms
    };
}

/// Main cell plugin that coordinates all cell-related functionality
pub struct CellPlugin;

impl Plugin for CellPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TypesPlugin)
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

/// Signaling substances (like Cell Lab)
#[derive(Component, Default)]
pub struct CellSignaling {
    pub s1: f32,
    pub s2: f32,
    pub s3: f32,
    pub s4: f32,
}
