use bevy::prelude::*;

/// Plugin for cell rendering
pub struct CellRenderingPlugin;

impl Plugin for CellRenderingPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: Add cell rendering systems
        // - Debug: colored icospheres
        // - Fancy: smooth cell deformation (future)
    }
}

/// Marker component for cell mesh
#[derive(Component)]
pub struct CellMesh;
