// ! GPU Compute Physics for Cell Simulation
//!
//! This module implements GPU-accelerated physics using WebGPU compute shaders.
//! Matches the C++ implementation's compute pipeline for 100K+ cell simulations.

use bevy::prelude::*;
use bevy::render::{
    render_resource::*,
    renderer::RenderDevice,
};
use bytemuck::{Pod, Zeroable};

/// Maximum number of cells supported by GPU simulation
pub const MAX_GPU_CELLS: usize = 100_000;

/// Spatial grid resolution (cells are distributed across 3D grid)
pub const GRID_RESOLUTION: i32 = 32;

/// Maximum cells per grid cell
pub const MAX_CELLS_PER_GRID: i32 = 64;

/// Total grid cells
pub const TOTAL_GRID_CELLS: u32 = (GRID_RESOLUTION * GRID_RESOLUTION * GRID_RESOLUTION) as u32;

/// World size (sphere boundary radius * 2)
pub const WORLD_SIZE: f32 = 100.0;

/// Grid cell size
pub const GRID_CELL_SIZE: f32 = WORLD_SIZE / GRID_RESOLUTION as f32;

/// GPU compute cell matching C++ ComputeCell structure
/// Size: 272 bytes (aligned to 16 bytes)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ComputeCell {
    // Physics (128 bytes)
    pub position_and_mass: [f32; 4],      // xyz = position, w = mass
    pub velocity: [f32; 4],                // xyz = velocity
    pub acceleration: [f32; 4],            // xyz = acceleration
    pub prev_acceleration: [f32; 4],       // xyz = previous acceleration
    pub orientation: [f32; 4],             // quaternion (w, x, y, z)
    pub genome_orientation: [f32; 4],      // genome orientation quaternion
    pub angular_velocity: [f32; 4],        // xyz = angular velocity
    pub angular_acceleration: [f32; 4],    // xyz = angular acceleration
    pub prev_angular_acceleration: [f32; 4], // xyz = previous angular accel

    // Internal state (24 bytes)
    pub signalling_substances: [f32; 4],  // 4 substances
    pub mode_index: i32,
    pub age: f32,
    pub toxins: f32,
    pub nitrates: f32,

    // Adhesion indices (80 bytes)
    pub adhesion_indices: [i32; 20],

    // Padding to maintain 16-byte alignment (16 bytes)
    pub _padding: [u32; 4],
}

impl ComputeCell {
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Create a new ComputeCell from simplified physics data
    pub fn from_physics_data(
        position: Vec3,
        velocity: Vec3,
        mass: f32,
        orientation: Quat,
        genome_orientation: Quat,
        mode_index: usize,
        age: f32,
    ) -> Self {
        Self {
            position_and_mass: [position.x, position.y, position.z, mass],
            velocity: [velocity.x, velocity.y, velocity.z, 0.0],
            acceleration: [0.0, 0.0, 0.0, 0.0],
            prev_acceleration: [0.0, 0.0, 0.0, 0.0],
            orientation: [orientation.w, orientation.x, orientation.y, orientation.z],
            genome_orientation: [genome_orientation.w, genome_orientation.x, genome_orientation.y, genome_orientation.z],
            angular_velocity: [0.0, 0.0, 0.0, 0.0],
            angular_acceleration: [0.0, 0.0, 0.0, 0.0],
            prev_angular_acceleration: [0.0, 0.0, 0.0, 0.0],
            signalling_substances: [0.0, 0.0, 0.0, 0.0],
            mode_index: mode_index as i32,
            age,
            toxins: 0.0,
            nitrates: 0.0,
            adhesion_indices: [0; 20],
            _padding: [0; 4],
        }
    }

    /// Convert to instance data for rendering (with color from mode)
    pub fn to_instance_data(&self, color: [f32; 4]) -> super::gpu_types::CellInstanceData {
        let mass = self.position_and_mass[3];
        let radius = mass.powf(1.0 / 3.0);

        super::gpu_types::CellInstanceData {
            position_and_radius: [
                self.position_and_mass[0],
                self.position_and_mass[1],
                self.position_and_mass[2],
                radius,
            ],
            color,
            orientation: self.orientation,
        }
    }
}

/// GPU Mode structure matching WGSL layout
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GPUMode {
    pub color: [f32; 4],
    pub orientation_a: [f32; 4],
    pub orientation_b: [f32; 4],
    pub split_direction: [f32; 4],
    pub child_modes: [i32; 2],
    pub split_interval: f32,
    pub genome_offset: i32,
    // Adhesion settings
    pub adhesion_can_break: i32,
    pub adhesion_break_force: f32,
    pub adhesion_rest_length: f32,
    pub adhesion_linear_stiffness: f32,
    pub adhesion_linear_damping: f32,
    pub adhesion_orientation_stiffness: f32,
    pub adhesion_orientation_damping: f32,
    pub adhesion_max_angular_deviation: f32,
    pub adhesion_twist_stiffness: f32,
    pub adhesion_twist_damping: f32,
    pub adhesion_enable_twist: i32,
    pub adhesion_padding: i32,
    // Parent/child settings
    pub parent_make_adhesion: i32,
    pub child_a_keep_adhesion: i32,
    pub child_b_keep_adhesion: i32,
    pub max_adhesions: i32,
    pub flagellocyte_thrust_force: f32,
    pub _padding1: f32,
    pub _padding2: f32,
    pub _padding3: f32,
}

impl GPUMode {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

/// Cell count buffer
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CellCountBuffer {
    pub total_cell_count: u32,
    pub live_cell_count: u32,
    pub total_adhesion_count: u32,
    pub free_adhesion_top: u32,
}

/// Physics uniforms for collision shader
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PhysicsUniforms {
    pub dragged_cell_index: i32,
    pub acceleration_damping: f32,
    pub grid_resolution: i32,
    pub grid_cell_size: f32,
    pub world_size: f32,
    pub max_cells_per_grid: i32,
    pub enable_thrust_force: i32,
    pub _padding: i32,
}

/// Velocity update uniforms
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct VelocityUniforms {
    pub delta_time: f32,
    pub damping: f32,
    pub dragged_cell_index: i32,
    pub sphere_radius: f32,
    pub sphere_center: [f32; 3],
    pub _padding1: f32,  // Align sphere_center to 16 bytes (vec3 in WGSL)
    pub enable_velocity_barrier: u32,
    pub barrier_damping: f32,
    pub barrier_push_distance: f32,
    pub _padding2: f32,  // Pad to 16-byte alignment
}

/// Position update uniforms
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PositionUniforms {
    pub delta_time: f32,
    pub dragged_cell_index: i32,
    pub _padding: [f32; 2],
}

/// Grid uniforms
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GridUniforms {
    pub grid_resolution: i32,
    pub grid_cell_size: f32,
    pub world_size: f32,
    pub max_cells_per_grid: i32,
}

/// GPU compute buffers
#[derive(Resource)]
pub struct GpuComputeBuffers {
    /// Triple-buffered cell data (read, write, render)
    pub cell_buffers: [Buffer; 3],
    pub current_buffer_index: usize,

    /// Spatial grid buffers
    pub grid_cells: Buffer,
    pub grid_counts: Buffer,
    pub grid_offsets: Buffer,

    /// Cell count buffer
    pub cell_counts: Buffer,

    /// Mode buffer
    pub modes: Buffer,

    /// Uniform buffers
    pub physics_uniforms: Buffer,
    pub velocity_uniforms: Buffer,
    pub position_uniforms: Buffer,
    pub grid_uniforms: Buffer,
    pub grid_clear_uniforms: Buffer,
}

impl GpuComputeBuffers {
    pub fn new(device: &RenderDevice) -> Self {
        // Create triple-buffered cell data
        let cell_buffer_size = (MAX_GPU_CELLS * ComputeCell::SIZE) as u64;
        let cell_buffers = [
            Self::create_cell_buffer(device, "Cell Buffer 0", cell_buffer_size),
            Self::create_cell_buffer(device, "Cell Buffer 1", cell_buffer_size),
            Self::create_cell_buffer(device, "Cell Buffer 2", cell_buffer_size),
        ];

        // Create spatial grid buffers
        let grid_size = (TOTAL_GRID_CELLS * MAX_CELLS_PER_GRID as u32 * 4) as u64; // u32 per cell index
        let grid_cells = device.create_buffer(&BufferDescriptor {
            label: Some("Grid Cells Buffer"),
            size: grid_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let grid_count_size = (TOTAL_GRID_CELLS * 4) as u64; // u32 per grid cell
        let grid_counts = device.create_buffer(&BufferDescriptor {
            label: Some("Grid Counts Buffer"),
            size: grid_count_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let grid_offsets = device.create_buffer(&BufferDescriptor {
            label: Some("Grid Offsets Buffer"),
            size: grid_count_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create cell count buffer
        let cell_counts = device.create_buffer(&BufferDescriptor {
            label: Some("Cell Count Buffer"),
            size: std::mem::size_of::<CellCountBuffer>() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create mode buffer (placeholder, will be filled from genome)
        let modes = device.create_buffer(&BufferDescriptor {
            label: Some("GPU Modes Buffer"),
            size: (16 * GPUMode::SIZE) as u64, // Support up to 16 modes
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create uniform buffers
        let physics_uniforms = Self::create_uniform_buffer(device, "Physics Uniforms", std::mem::size_of::<PhysicsUniforms>() as u64);
        let velocity_uniforms = Self::create_uniform_buffer(device, "Velocity Uniforms", std::mem::size_of::<VelocityUniforms>() as u64);
        let position_uniforms = Self::create_uniform_buffer(device, "Position Uniforms", std::mem::size_of::<PositionUniforms>() as u64);
        let grid_uniforms = Self::create_uniform_buffer(device, "Grid Uniforms", std::mem::size_of::<GridUniforms>() as u64);
        let grid_clear_uniforms = Self::create_uniform_buffer(device, "Grid Clear Uniforms", 32); // total_grid_cells + padding (WGSL alignment)

        Self {
            cell_buffers,
            current_buffer_index: 0,
            grid_cells,
            grid_counts,
            grid_offsets,
            cell_counts,
            modes,
            physics_uniforms,
            velocity_uniforms,
            position_uniforms,
            grid_uniforms,
            grid_clear_uniforms,
        }
    }

    fn create_cell_buffer(device: &RenderDevice, label: &str, size: u64) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size,
            // STORAGE: For compute shaders to read/write
            // VERTEX: For rendering pipeline to use as instance buffer
            // COPY_DST: To upload initial data
            // COPY_SRC: For buffer-to-buffer copies
            usage: BufferUsages::STORAGE | BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    }

    fn create_uniform_buffer(device: &RenderDevice, label: &str, size: u64) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Get current read buffer
    pub fn read_buffer(&self) -> &Buffer {
        &self.cell_buffers[self.current_buffer_index]
    }

    /// Get current write buffer
    pub fn write_buffer(&self) -> &Buffer {
        &self.cell_buffers[(self.current_buffer_index + 1) % 3]
    }

    /// Get render buffer (for extracting to rendering)
    pub fn render_buffer(&self) -> &Buffer {
        &self.cell_buffers[(self.current_buffer_index + 2) % 3]
    }

    /// Swap read and write buffers (for ping-pong between compute passes)
    /// This allows one pass to write to a buffer, then the next pass to read from it
    pub fn swap_read_write(&mut self) {
        // Advance index by 1 to swap read/write
        // This makes the old write_buffer become the new read_buffer
        self.current_buffer_index = (self.current_buffer_index + 1) % 3;
    }

    /// Advance to next buffer frame (called once per frame after all compute passes)
    pub fn advance_frame(&mut self) {
        self.current_buffer_index = (self.current_buffer_index + 1) % 3;
    }
}
