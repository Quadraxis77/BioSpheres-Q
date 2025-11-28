//! Shared types for WebGPU rendering pipeline.
//!
//! This module contains data structures and error types used across
//! the GPU rendering system.

use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};

/// Complete physics state for a single cell in the GPU simulation.
///
/// This structure stores all data needed for physics simulation and cell division,
/// matching the C++ CPUCellPhysics_SoA structure but in AoS (Array of Structs) format.
#[derive(Clone, Debug)]
pub struct CellPhysicsData {
    /// Cell position in world space
    pub position: Vec3,
    /// Cell velocity
    pub velocity: Vec3,
    /// Cell acceleration (reset each frame)
    pub acceleration: Vec3,
    /// Previous acceleration (for Verlet integration)
    pub previous_acceleration: Vec3,
    /// Cell orientation (physical rotation)
    pub orientation: Quat,
    /// Angular velocity
    pub angular_velocity: Vec3,
    /// Angular acceleration (reset each frame)
    pub angular_acceleration: Vec3,
    /// Genome orientation (used for division and adhesion anchors)
    pub genome_orientation: Quat,
    /// Cell mass
    pub mass: f32,
    /// Cell radius (calculated from mass: radius = mass^(1/3))
    pub radius: f32,
    /// Cell age (increments with time, used for division)
    pub age: f32,
    /// Cell energy
    pub energy: f32,
    /// Cell type identifier
    pub cell_type: u32,
    /// Genome ID
    pub genome_id: u32,
    /// Current mode index within the genome
    pub mode_index: usize,
    /// Cell flags
    pub flags: u32,
    /// Cell color (RGB)
    pub color: Vec3,
}

impl CellPhysicsData {
    /// Create a new cell with default values
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            previous_acceleration: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            angular_velocity: Vec3::ZERO,
            angular_acceleration: Vec3::ZERO,
            genome_orientation: Quat::IDENTITY,
            mass: 1.0,
            radius: 1.0,
            age: 0.0,
            energy: 1.0,
            cell_type: 0,
            genome_id: 0,
            mode_index: 0,
            flags: 0,
            color: Vec3::ONE,
        }
    }

    /// Convert this physics data to instance data for rendering
    pub fn to_instance_data(&self) -> CellInstanceData {
        CellInstanceData::from_components(
            [self.position.x, self.position.y, self.position.z],
            self.radius,
            [self.color.x, self.color.y, self.color.z, 1.0],
            [self.orientation.w, self.orientation.x, self.orientation.y, self.orientation.z],
        )
    }

    /// Convert this physics data to ComputeCell format for GPU simulation
    pub fn to_compute_cell(&self) -> crate::rendering::gpu_compute::ComputeCell {
        crate::rendering::gpu_compute::ComputeCell::from_physics_data(
            self.position,
            self.velocity,
            self.mass,
            self.orientation,
            self.genome_orientation,
            self.mode_index,
            self.age,
        )
    }
}

impl Default for CellPhysicsData {
    fn default() -> Self {
        Self::new()
    }
}

/// Instance data for a single cell, matching the Biospheres.cpp CPUInstanceData format.
///
/// Layout (48 bytes total, 16-byte aligned):
/// - Offset 0:  position_and_radius (vec4: x, y, z, radius)
/// - Offset 16: color (vec4: r, g, b, a)
/// - Offset 32: orientation (vec4: w, x, y, z quaternion)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CellInstanceData {
    /// Position (xyz) and radius (w)
    pub position_and_radius: [f32; 4],
    /// RGBA color
    pub color: [f32; 4],
    /// Orientation quaternion (w, x, y, z)
    pub orientation: [f32; 4],
}

impl CellInstanceData {
    /// Size of the instance data in bytes
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Create a new instance at the origin with default values
    pub fn new() -> Self {
        Self {
            position_and_radius: [0.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            orientation: [1.0, 0.0, 0.0, 0.0], // Identity quaternion (w, x, y, z)
        }
    }

    /// Create instance data from position, radius, color, and orientation
    pub fn from_components(
        position: [f32; 3],
        radius: f32,
        color: [f32; 4],
        orientation: [f32; 4],
    ) -> Self {
        Self {
            position_and_radius: [position[0], position[1], position[2], radius],
            color,
            orientation,
        }
    }
}

impl Default for CellInstanceData {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during WebGPU operations.
#[derive(Debug, thiserror::Error)]
pub enum WebGpuError {
    /// Failed to create WebGPU adapter
    #[error("Failed to create WebGPU adapter: {0}")]
    AdapterCreation(String),

    /// Failed to create WebGPU device
    #[error("Failed to create WebGPU device: {0}")]
    DeviceCreation(#[from] wgpu::RequestDeviceError),

    /// Shader compilation failed
    #[error("Shader compilation failed at line {line:?}: {message}")]
    ShaderCompilation {
        message: String,
        line: Option<u32>,
    },

    /// Surface error during rendering
    #[error("Surface error: {0}")]
    Surface(#[from] wgpu::SurfaceError),

    /// Buffer mapping failed
    #[error("Buffer mapping failed: {0}")]
    BufferMapping(String),

    /// Window handle error
    #[error("Failed to get window handle: {0}")]
    WindowHandle(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_instance_data_size() {
        // Verify the instance data is exactly 48 bytes as specified
        assert_eq!(CellInstanceData::SIZE, 48);
    }

    #[test]
    fn test_cell_instance_data_alignment() {
        // Verify 16-byte alignment
        assert_eq!(std::mem::align_of::<CellInstanceData>(), 4);
        // Each field is 16 bytes (4 * f32)
        assert_eq!(std::mem::size_of::<[f32; 4]>(), 16);
    }

    #[test]
    fn test_cell_instance_data_default() {
        let instance = CellInstanceData::default();
        assert_eq!(instance.position_and_radius, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(instance.color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(instance.orientation, [1.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_cell_instance_data_from_components() {
        let instance = CellInstanceData::from_components(
            [1.0, 2.0, 3.0],
            0.5,
            [0.8, 0.3, 0.5, 1.0],
            [0.707, 0.707, 0.0, 0.0],
        );
        assert_eq!(instance.position_and_radius, [1.0, 2.0, 3.0, 0.5]);
        assert_eq!(instance.color, [0.8, 0.3, 0.5, 1.0]);
        assert_eq!(instance.orientation, [0.707, 0.707, 0.0, 0.0]);
    }
}
