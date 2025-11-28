//! GPU Compute Dispatcher
//!
//! Manages compute pass execution and buffer synchronization

use bevy::render::renderer::{RenderDevice, RenderQueue};

use super::gpu_compute::*;
use super::gpu_compute_pipeline::GpuComputePipelines;

/// Bind groups for all compute pipelines
/// Note: This is no longer a Resource - bind groups are created dynamically during dispatch
pub struct GpuComputeBindGroups {
    pub grid_clear: wgpu::BindGroup,
    pub grid_insert: wgpu::BindGroup,
    pub collision: wgpu::BindGroup,
    pub velocity: wgpu::BindGroup,
    pub position: wgpu::BindGroup,
}

impl GpuComputeBindGroups {
    pub fn new(
        device: &RenderDevice,
        pipelines: &GpuComputePipelines,
        buffers: &GpuComputeBuffers,
    ) -> Self {
        Self {
            grid_clear: Self::create_grid_clear_bind_group(device, pipelines, buffers),
            grid_insert: Self::create_grid_insert_bind_group(device, pipelines, buffers),
            collision: Self::create_collision_bind_group(device, pipelines, buffers),
            velocity: Self::create_velocity_bind_group(device, pipelines, buffers),
            position: Self::create_position_bind_group(device, pipelines, buffers),
        }
    }

    fn create_grid_clear_bind_group(
        device: &RenderDevice,
        pipelines: &GpuComputePipelines,
        buffers: &GpuComputeBuffers,
    ) -> wgpu::BindGroup {
        device.wgpu_device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Clear Bind Group"),
            layout: &pipelines.grid_clear_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.grid_counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.grid_clear_uniforms.as_entire_binding(),
                },
            ],
        })
    }

    fn create_grid_insert_bind_group(
        device: &RenderDevice,
        pipelines: &GpuComputePipelines,
        buffers: &GpuComputeBuffers,
    ) -> wgpu::BindGroup {
        device.wgpu_device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Insert Bind Group"),
            layout: &pipelines.grid_insert_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.read_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.grid_cells.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.grid_offsets.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.cell_counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: buffers.grid_uniforms.as_entire_binding(),
                },
            ],
        })
    }

    fn create_collision_bind_group(
        device: &RenderDevice,
        pipelines: &GpuComputePipelines,
        buffers: &GpuComputeBuffers,
    ) -> wgpu::BindGroup {
        device.wgpu_device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Collision Bind Group"),
            layout: &pipelines.collision_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.read_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.grid_cells.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.grid_counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.write_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: buffers.cell_counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: buffers.modes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: buffers.physics_uniforms.as_entire_binding(),
                },
            ],
        })
    }

    fn create_velocity_bind_group(
        device: &RenderDevice,
        pipelines: &GpuComputePipelines,
        buffers: &GpuComputeBuffers,
    ) -> wgpu::BindGroup {
        device.wgpu_device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Velocity Update Bind Group"),
            layout: &pipelines.velocity_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.read_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.write_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.cell_counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.velocity_uniforms.as_entire_binding(),
                },
            ],
        })
    }

    fn create_position_bind_group(
        device: &RenderDevice,
        pipelines: &GpuComputePipelines,
        buffers: &GpuComputeBuffers,
    ) -> wgpu::BindGroup {
        device.wgpu_device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Position Update Bind Group"),
            layout: &pipelines.position_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.read_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.write_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.cell_counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.position_uniforms.as_entire_binding(),
                },
            ],
        })
    }

    /// Recreate bind groups (needed when buffer index changes)
    pub fn rebuild(
        device: &RenderDevice,
        pipelines: &GpuComputePipelines,
        buffers: &GpuComputeBuffers,
    ) -> Self {
        Self::new(device, pipelines, buffers)
    }
}

/// Dispatch compute shaders for physics simulation using provided command encoder
/// This should be called from a render graph node to use Bevy's managed command encoder
///
/// Buffer swapping strategy (matches C++ implementation):
/// - Pass 1: Grid Clear
/// - Pass 2: Grid Insert
/// - Pass 3: Collision (reads from read_buffer, writes to write_buffer) → swap buffers
/// - Pass 4: Position Update (reads from read_buffer, writes to write_buffer) → swap buffers
/// - Pass 5: Grid Insert (cells moved)
/// - Pass 6: Velocity Update (reads from read_buffer, writes to write_buffer) → swap buffers
pub fn dispatch_gpu_physics_to_encoder(
    encoder: &mut wgpu::CommandEncoder,
    device: &RenderDevice,
    pipelines: &GpuComputePipelines,
    buffers: &mut GpuComputeBuffers,
    cell_count: u32,
) {
    // Calculate workgroup counts
    let cell_workgroups = (cell_count + 255) / 256; // Round up to nearest 256
    let grid_workgroups = (TOTAL_GRID_CELLS + 255) / 256;

    // Pass 1: Clear spatial grid counts
    {
        let bind_group = device.wgpu_device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Clear Bind Group"),
            layout: &pipelines.grid_clear_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.grid_counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.grid_clear_uniforms.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Grid Clear Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipelines.grid_clear_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(grid_workgroups, 1, 1);
        drop(pass);
    }

    // Pass 2: Insert cells into spatial grid
    {
        let bind_group = device.wgpu_device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Insert Bind Group"),
            layout: &pipelines.grid_insert_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.read_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.grid_cells.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.grid_offsets.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.cell_counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: buffers.grid_uniforms.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Grid Insert Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipelines.grid_insert_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(cell_workgroups, 1, 1);
        drop(pass);
    }

    // Pass 3: Compute cell collisions and forces (reads from read_buffer, writes accelerations to write_buffer)
    {
        let bind_group = device.wgpu_device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Collision Bind Group"),
            layout: &pipelines.collision_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.read_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.grid_cells.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.grid_counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.write_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: buffers.cell_counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: buffers.modes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: buffers.physics_uniforms.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Collision Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipelines.collision_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(cell_workgroups, 1, 1);
        drop(pass);
    }

    // Swap read/write buffers (collision wrote to write_buffer, now becomes read_buffer)
    buffers.swap_read_write();

    // Pass 4: Update positions and orientations (reads from read_buffer, writes to write_buffer)
    {
        let bind_group = device.wgpu_device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Position Update Bind Group"),
            layout: &pipelines.position_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.read_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.write_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.cell_counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.position_uniforms.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Position Update Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipelines.position_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(cell_workgroups, 1, 1);
        drop(pass);
    }

    // Swap read/write buffers (position wrote to write_buffer, now becomes read_buffer)
    buffers.swap_read_write();

    // Pass 5: Update velocity (reads from read_buffer, writes to write_buffer)
    {
        let bind_group = device.wgpu_device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Velocity Update Bind Group"),
            layout: &pipelines.velocity_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.read_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.write_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.cell_counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.velocity_uniforms.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Velocity Update Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipelines.velocity_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(cell_workgroups, 1, 1);
        drop(pass);
    }

    // Final swap: velocity wrote to write_buffer
    // After this frame completes, render_buffer will be used for rendering
    buffers.swap_read_write();
}

/// Legacy function - creates own encoder and submits directly
/// DEPRECATED: Use dispatch_gpu_physics_to_encoder with render graph node instead
#[allow(dead_code)]
pub fn dispatch_gpu_physics(
    device: &RenderDevice,
    queue: &RenderQueue,
    pipelines: &GpuComputePipelines,
    buffers: &mut GpuComputeBuffers,
    cell_count: u32,
) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("GPU Physics Command Encoder"),
    });

    dispatch_gpu_physics_to_encoder(&mut encoder, device, pipelines, buffers, cell_count);

    // Submit command buffer through Bevy's render queue
    queue.submit(std::iter::once(encoder.finish()));
}

/// Update uniform buffers with current simulation parameters
pub fn update_compute_uniforms(
    queue: &RenderQueue,
    buffers: &GpuComputeBuffers,
    delta_time: f32,
    cell_count: u32,
    dragged_cell_index: Option<usize>,
) {
    let dragged_index = dragged_cell_index.map(|i| i as i32).unwrap_or(-1);
    
    // Update physics uniforms
    let physics_uniforms = PhysicsUniforms {
        dragged_cell_index: dragged_index,
        acceleration_damping: 0.5,
        grid_resolution: GRID_RESOLUTION,
        grid_cell_size: GRID_CELL_SIZE,
        world_size: WORLD_SIZE,
        max_cells_per_grid: MAX_CELLS_PER_GRID,
        enable_thrust_force: 0, // Disable flagellocyte thrust for now
        _padding: 0,
    };
    queue.write_buffer(&buffers.physics_uniforms, 0, bytemuck::bytes_of(&physics_uniforms));

    // Update velocity uniforms
    let velocity_uniforms = VelocityUniforms {
        delta_time,
        damping: 0.98,
        dragged_cell_index: dragged_index,
        sphere_radius: WORLD_SIZE * 0.5,
        sphere_center: [0.0, 0.0, 0.0],
        _padding1: 0.0,  // WGSL vec3 alignment padding
        enable_velocity_barrier: 1,
        barrier_damping: 0.8,
        barrier_push_distance: 2.0,
        _padding2: 0.0,  // 16-byte alignment padding
    };
    queue.write_buffer(&buffers.velocity_uniforms, 0, bytemuck::bytes_of(&velocity_uniforms));

    // Update position uniforms
    let position_uniforms = PositionUniforms {
        delta_time,
        dragged_cell_index: dragged_index,
        _padding: [0.0; 2],
    };
    queue.write_buffer(&buffers.position_uniforms, 0, bytemuck::bytes_of(&position_uniforms));

    // Update grid uniforms
    let grid_uniforms = GridUniforms {
        grid_resolution: GRID_RESOLUTION,
        grid_cell_size: GRID_CELL_SIZE,
        world_size: WORLD_SIZE,
        max_cells_per_grid: MAX_CELLS_PER_GRID,
    };
    queue.write_buffer(&buffers.grid_uniforms, 0, bytemuck::bytes_of(&grid_uniforms));

    // Update grid clear uniforms (32 bytes to match WGSL alignment)
    let grid_clear_data: [u32; 8] = [TOTAL_GRID_CELLS, 0, 0, 0, 0, 0, 0, 0];
    queue.write_buffer(&buffers.grid_clear_uniforms, 0, bytemuck::bytes_of(&grid_clear_data));

    // Update cell count buffer
    let cell_count_data = CellCountBuffer {
        total_cell_count: cell_count,
        live_cell_count: cell_count,
        total_adhesion_count: 0,
        free_adhesion_top: 0,
    };
    queue.write_buffer(&buffers.cell_counts, 0, bytemuck::bytes_of(&cell_count_data));
}
