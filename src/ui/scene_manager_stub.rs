// Temporary stub for scene_manager - just exports the resource types
// Full egui implementation coming soon

use bevy::prelude::*;

#[derive(Resource)]
pub struct CpuCellCapacity {
    pub capacity: usize,
}

impl Default for CpuCellCapacity {
    fn default() -> Self {
        Self {
            capacity: 256,  // Default from original implementation
        }
    }
}
