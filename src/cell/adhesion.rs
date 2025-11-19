use bevy::prelude::*;

/// Plugin for cell adhesion mechanics
pub struct AdhesionPlugin;

impl Plugin for AdhesionPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: Add adhesion systems
        // - Linear springs between adhered cells
        // - Angular constraints
        // - Break after max force threshold
    }
}

/// Adhesion connection between cells
#[derive(Component)]
pub struct Adhesion {
    pub other_cell: Entity,
    pub spring_constant: f32,
    pub rest_length: f32,
    pub max_force: f32,
}

/// Adhesion anchor point on a cell
#[derive(Component, Default)]
pub struct AdhesionAnchor {
    pub local_position: Vec3,
}
