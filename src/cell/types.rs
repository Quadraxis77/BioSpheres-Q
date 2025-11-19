use bevy::prelude::*;

/// Plugin for cell type definitions
pub struct TypesPlugin;

impl Plugin for TypesPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: Register cell type components
    }
}

/// Cell type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellType {
    Chronocyte,  // Splits after set time
    Phagocyte,   // Eats food to gain biomass
    Photocyte,   // Absorbs light to gain biomass
    Flagellocyte, // Propels itself forward
}

/// Chronocyte - splits after a set time
#[derive(Component)]
pub struct Chronocyte {
    pub split_time: f32,
    pub time_elapsed: f32,
}

