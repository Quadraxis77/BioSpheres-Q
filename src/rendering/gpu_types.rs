//! Shared types for WebGPU rendering pipeline.
//!
//! This module contains data structures and error types used across
//! the GPU rendering system.

use bytemuck::{Pod, Zeroable};

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
