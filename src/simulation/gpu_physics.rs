//! GPU-accelerated collision physics using wgpu compute shaders
//!
//! This module provides GPU-accelerated collision detection and force computation
//! using spatial hashing for broad-phase and direct distance checks for narrow-phase.
//!
//! The GPU physics pipeline:
//! 1. Upload cell data (positions, velocities, radii, etc.) to GPU buffers
//! 2. Build spatial grid on GPU (count cells per grid cell, compute prefix sums)
//! 3. Run collision detection compute shader
//! 4. Download computed forces back to CPU
//! 5. Apply forces in the existing physics integration step

use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use std::sync::Arc;

use crate::simulation::{CanonicalState, PhysicsConfig};

/// Maximum number of cells supported by GPU physics
pub const GPU_MAX_CELLS: usize = 65536;

/// Grid size for spatial hashing (NxNxN)
pub const GPU_GRID_SIZE: u32 = 64;

/// Cell data structure for GPU (must match WGSL layout)
/// Uses vec4 packing for proper GPU alignment
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuCellData {
    /// position.xyz = position, position.w = radius
    pub position_radius: [f32; 4],
    /// velocity.xyz = velocity, velocity.w = mass
    pub velocity_mass: [f32; 4],
    /// stiffness_pad.x = stiffness, stiffness_pad.yzw = padding
    pub stiffness_pad: [f32; 4],
}

/// Force output structure from GPU (must match WGSL layout)
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuForceOutput {
    /// force.xyz = force, force.w = padding
    pub force_pad: [f32; 4],
    /// torque.xyz = torque, torque.w = padding
    pub torque_pad: [f32; 4],
}

/// Physics parameters uniform (must match WGSL layout)
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuPhysicsParams {
    pub cell_count: u32,
    pub grid_size: u32,
    pub world_size: f32,
    pub sphere_radius: f32,
    pub default_stiffness: f32,
    pub damping: f32,
    pub friction_coefficient: f32,
    pub max_force: f32,
}

/// Spatial grid cell structure (must match WGSL layout)
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, Default)]
pub struct GpuGridCell {
    pub start: u32,
    pub count: u32,
}

/// GPU Physics compute pipeline and resources
pub struct GpuPhysicsContext {
    // Compute pipelines
    pub collision_pipeline: wgpu::ComputePipeline,
    
    // Buffers
    pub cell_buffer: wgpu::Buffer,
    pub force_buffer: wgpu::Buffer,
    pub params_buffer: wgpu::Buffer,
    pub grid_cells_buffer: wgpu::Buffer,
    pub cell_indices_buffer: wgpu::Buffer,
    pub cell_grid_indices_buffer: wgpu::Buffer,
    pub staging_buffer: wgpu::Buffer,
    
    // Bind group
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    
    // CPU-side buffers for upload/download
    pub cpu_cells: Vec<GpuCellData>,
    pub cpu_forces: Vec<GpuForceOutput>,
    pub cpu_grid_cells: Vec<GpuGridCell>,
    pub cpu_cell_indices: Vec<u32>,
    pub cpu_cell_grid_indices: Vec<u32>,
}

impl GpuPhysicsContext {
    /// Create a new GPU physics context using Bevy's existing device
    pub fn new_from_bevy(
        device: &bevy::render::renderer::RenderDevice,
    ) -> Result<Self, GpuPhysicsError> {
        // Use Bevy's existing device to avoid conflicts
        let wgpu_device = device.wgpu_device();
        
        // Load shader
        let shader_source = include_str!("../../assets/shaders/gpu_collision.wgsl");
        let shader_module = wgpu_device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GPU Collision Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        
        // Create bind group layout
        let bind_group_layout = wgpu_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("GPU Physics Bind Group Layout"),
            entries: &[
                // cells: storage buffer (read)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // forces: storage buffer (read_write)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // params: uniform buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // grid_cells: storage buffer (read)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // cell_indices: storage buffer (read)
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // cell_grid_indices: storage buffer (read)
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // Create pipeline layout
        let pipeline_layout = wgpu_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GPU Physics Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create compute pipeline
        let collision_pipeline = wgpu_device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Collision Detection Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("collision_detect"),
            compilation_options: Default::default(),
            cache: None,
        });
        
        // Calculate buffer sizes
        let cell_buffer_size = (GPU_MAX_CELLS * std::mem::size_of::<GpuCellData>()) as u64;
        let force_buffer_size = (GPU_MAX_CELLS * std::mem::size_of::<GpuForceOutput>()) as u64;
        let params_buffer_size = std::mem::size_of::<GpuPhysicsParams>() as u64;
        let grid_size_total = (GPU_GRID_SIZE * GPU_GRID_SIZE * GPU_GRID_SIZE) as usize;
        let grid_cells_buffer_size = (grid_size_total * std::mem::size_of::<GpuGridCell>()) as u64;
        let cell_indices_buffer_size = (GPU_MAX_CELLS * std::mem::size_of::<u32>()) as u64;
        
        // Create buffers
        let cell_buffer = wgpu_device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cell Data Buffer"),
            size: cell_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let force_buffer = wgpu_device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Force Output Buffer"),
            size: force_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        let params_buffer = wgpu_device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Physics Params Buffer"),
            size: params_buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let grid_cells_buffer = wgpu_device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Cells Buffer"),
            size: grid_cells_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let cell_indices_buffer = wgpu_device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cell Indices Buffer"),
            size: cell_indices_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let cell_grid_indices_buffer = wgpu_device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cell Grid Indices Buffer"),
            size: cell_indices_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let staging_buffer = wgpu_device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: force_buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create bind group
        let bind_group = wgpu_device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GPU Physics Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: cell_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: force_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: grid_cells_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: cell_indices_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: cell_grid_indices_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Initialize CPU-side buffers
        let cpu_cells = vec![GpuCellData::zeroed(); GPU_MAX_CELLS];
        let cpu_forces = vec![GpuForceOutput::zeroed(); GPU_MAX_CELLS];
        let cpu_grid_cells = vec![GpuGridCell::default(); grid_size_total];
        let cpu_cell_indices = vec![0u32; GPU_MAX_CELLS];
        let cpu_cell_grid_indices = vec![0u32; GPU_MAX_CELLS];
        
        Ok(Self {
            collision_pipeline,
            cell_buffer,
            force_buffer,
            params_buffer,
            grid_cells_buffer,
            cell_indices_buffer,
            cell_grid_indices_buffer,
            staging_buffer,
            bind_group_layout,
            bind_group,
            cpu_cells,
            cpu_forces,
            cpu_grid_cells,
            cpu_cell_indices,
            cpu_cell_grid_indices,
        })
    }
    
    /// Build spatial grid on CPU (simpler than GPU prefix sum for now)
    fn build_spatial_grid(&mut self, state: &CanonicalState, config: &PhysicsConfig) {
        let cell_count = state.cell_count;
        let grid_size = GPU_GRID_SIZE as usize;
        let world_size = config.world_bounds.x;
        let cell_size = world_size / GPU_GRID_SIZE as f32;
        
        // Clear grid cells
        for gc in &mut self.cpu_grid_cells {
            gc.start = 0;
            gc.count = 0;
        }
        
        // Count cells per grid cell
        for i in 0..cell_count {
            let pos = state.positions[i];
            let grid_coord = self.world_to_grid(pos, world_size, cell_size);
            let grid_idx = self.grid_to_index(grid_coord, grid_size);
            self.cpu_cell_grid_indices[i] = grid_idx as u32;
            self.cpu_grid_cells[grid_idx].count += 1;
        }
        
        // Compute prefix sums (start indices)
        let mut offset = 0u32;
        for gc in &mut self.cpu_grid_cells {
            gc.start = offset;
            offset += gc.count;
            gc.count = 0; // Reset for insertion phase
        }
        
        // Insert cell indices into sorted order
        for i in 0..cell_count {
            let grid_idx = self.cpu_cell_grid_indices[i] as usize;
            let insert_pos = self.cpu_grid_cells[grid_idx].start + self.cpu_grid_cells[grid_idx].count;
            self.cpu_cell_indices[insert_pos as usize] = i as u32;
            self.cpu_grid_cells[grid_idx].count += 1;
        }
    }
    
    fn world_to_grid(&self, position: Vec3, world_size: f32, cell_size: f32) -> [i32; 3] {
        let offset_pos = position + Vec3::splat(world_size / 2.0);
        let grid_pos = offset_pos / cell_size;
        let max_coord = GPU_GRID_SIZE as i32 - 1;
        [
            (grid_pos.x as i32).clamp(0, max_coord),
            (grid_pos.y as i32).clamp(0, max_coord),
            (grid_pos.z as i32).clamp(0, max_coord),
        ]
    }
    
    fn grid_to_index(&self, coord: [i32; 3], grid_size: usize) -> usize {
        coord[0] as usize + coord[1] as usize * grid_size + coord[2] as usize * grid_size * grid_size
    }
    
    /// Upload cell data to GPU
    fn upload_cell_data(&mut self, state: &CanonicalState, queue: &bevy::render::renderer::RenderQueue) {
        let cell_count = state.cell_count;
        
        for i in 0..cell_count {
            let pos = state.positions[i];
            let vel = state.velocities[i];
            self.cpu_cells[i] = GpuCellData {
                position_radius: [pos.x, pos.y, pos.z, state.radii[i]],
                velocity_mass: [vel.x, vel.y, vel.z, state.masses[i]],
                stiffness_pad: [state.stiffnesses[i], 0.0, 0.0, 0.0],
            };
        }
        
        let cell_data_bytes = bytemuck::cast_slice(&self.cpu_cells[..cell_count]);
        queue.write_buffer(&self.cell_buffer, 0, cell_data_bytes);
    }
    
    /// Upload physics parameters
    fn upload_params(&self, state: &CanonicalState, config: &PhysicsConfig, queue: &bevy::render::renderer::RenderQueue) {
        let params = GpuPhysicsParams {
            cell_count: state.cell_count as u32,
            grid_size: GPU_GRID_SIZE,
            world_size: config.world_bounds.x,
            sphere_radius: config.sphere_radius,
            default_stiffness: config.default_stiffness,
            damping: config.damping,
            friction_coefficient: config.friction_coefficient,
            max_force: 10000.0,
        };
        
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));
    }
    
    /// Upload spatial grid data
    fn upload_grid_data(&self, cell_count: usize, queue: &bevy::render::renderer::RenderQueue) {
        let grid_data_bytes = bytemuck::cast_slice(&self.cpu_grid_cells);
        queue.write_buffer(&self.grid_cells_buffer, 0, grid_data_bytes);
        
        let indices_bytes = bytemuck::cast_slice(&self.cpu_cell_indices[..cell_count]);
        queue.write_buffer(&self.cell_indices_buffer, 0, indices_bytes);
        
        let grid_indices_bytes = bytemuck::cast_slice(&self.cpu_cell_grid_indices[..cell_count]);
        queue.write_buffer(&self.cell_grid_indices_buffer, 0, grid_indices_bytes);
    }
    
    /// Run collision detection on GPU and download results
    pub fn compute_collision_forces(
        &mut self,
        state: &mut CanonicalState,
        config: &PhysicsConfig,
        device: &bevy::render::renderer::RenderDevice,
        queue: &bevy::render::renderer::RenderQueue,
    ) {
        if state.cell_count == 0 {
            return;
        }
        
        let cell_count = state.cell_count;
        
        // Build spatial grid on CPU
        self.build_spatial_grid(state, config);
        
        // Upload data to GPU
        self.upload_cell_data(state, queue);
        self.upload_params(state, config, queue);
        self.upload_grid_data(cell_count, queue);
        
        let wgpu_device = device.wgpu_device();
        
        // Create command encoder
        let mut encoder = wgpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("GPU Physics Encoder"),
        });
        
        // Dispatch collision detection compute shader
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Collision Detection Pass"),
                timestamp_writes: None,
            });
            
            compute_pass.set_pipeline(&self.collision_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            
            // Dispatch with 64 threads per workgroup
            let workgroup_count = (cell_count as u32 + 63) / 64;
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }
        
        // Copy force buffer to staging buffer for readback
        let force_size = (cell_count * std::mem::size_of::<GpuForceOutput>()) as u64;
        encoder.copy_buffer_to_buffer(&self.force_buffer, 0, &self.staging_buffer, 0, force_size);
        
        // Submit commands
        queue.submit(std::iter::once(encoder.finish()));
        
        // Map staging buffer and read results
        let buffer_slice = self.staging_buffer.slice(..force_size);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        
        // Wait for GPU to finish
        let _ = wgpu_device.poll(wgpu::PollType::Wait);
        rx.recv().unwrap().unwrap();
        
        // Read force data
        {
            let data = buffer_slice.get_mapped_range();
            let force_data: &[GpuForceOutput] = bytemuck::cast_slice(&data);
            
            // Apply forces to state (extract xyz from vec4)
            for i in 0..cell_count {
                let f = &force_data[i].force_pad;
                let t = &force_data[i].torque_pad;
                state.forces[i] = Vec3::new(f[0], f[1], f[2]);
                state.torques[i] = Vec3::new(t[0], t[1], t[2]);
            }
        }
        
        // Unmap buffer
        self.staging_buffer.unmap();
    }
}

/// GPU Physics errors
#[derive(Debug, thiserror::Error)]
pub enum GpuPhysicsError {
    #[error("No suitable GPU adapter found")]
    NoAdapter,
    #[error("Failed to create GPU device: {0}")]
    DeviceCreation(String),
    #[error("Shader compilation error: {0}")]
    ShaderCompilation(String),
}

/// Bevy resource for GPU physics
#[derive(Resource, Clone)]
pub struct GpuPhysicsResource {
    pub context: Option<Arc<std::sync::Mutex<GpuPhysicsContext>>>,
    pub device: Option<bevy::render::renderer::RenderDevice>,
    pub queue: Option<bevy::render::renderer::RenderQueue>,
    pub enabled: bool,
    pub initialization_attempted: bool,
}

impl Default for GpuPhysicsResource {
    fn default() -> Self {
        Self {
            context: None,
            device: None,
            queue: None,
            enabled: false,
            initialization_attempted: false,
        }
    }
}

/// Initialize GPU physics context using Bevy's render device
pub fn initialize_gpu_physics_from_bevy(
    gpu_physics: &mut GpuPhysicsResource,
    render_device: &bevy::render::renderer::RenderDevice,
    render_queue: &bevy::render::renderer::RenderQueue,
) {
    if gpu_physics.initialization_attempted {
        return;
    }
    
    gpu_physics.initialization_attempted = true;
    
    // Store device and queue for later use
    gpu_physics.device = Some(render_device.clone());
    gpu_physics.queue = Some(render_queue.clone());
    
    match GpuPhysicsContext::new_from_bevy(render_device) {
        Ok(context) => {
            info!("GPU physics initialized successfully using Bevy's render device");
            gpu_physics.context = Some(Arc::new(std::sync::Mutex::new(context)));
            gpu_physics.enabled = true;
        }
        Err(e) => {
            warn!("Failed to initialize GPU physics: {}. Falling back to CPU.", e);
            gpu_physics.enabled = false;
        }
    }
}

/// Compute collision forces using GPU
pub fn compute_collision_forces_gpu(
    state: &mut CanonicalState,
    config: &PhysicsConfig,
    gpu_physics: &mut GpuPhysicsResource,
) {
    if let (Some(ref context), Some(ref device), Some(ref queue)) = 
        (&gpu_physics.context, &gpu_physics.device, &gpu_physics.queue) {
        if let Ok(mut ctx) = context.lock() {
            ctx.compute_collision_forces(state, config, device, queue);
        }
    }
}

/// GPU Physics plugin for Bevy
pub struct GpuPhysicsPlugin;

impl Plugin for GpuPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpuPhysicsResource>()
            // Delay initialization until first use when render resources are available
            .add_systems(Update, lazy_initialize_gpu_physics.run_if(
                |gpu: Res<GpuPhysicsResource>| !gpu.initialization_attempted
            ));
    }
}

fn lazy_initialize_gpu_physics(
    mut gpu_physics: ResMut<GpuPhysicsResource>,
    render_device: Option<Res<bevy::render::renderer::RenderDevice>>,
    render_queue: Option<Res<bevy::render::renderer::RenderQueue>>,
) {
    if let (Some(device), Some(queue)) = (render_device, render_queue) {
        initialize_gpu_physics_from_bevy(&mut gpu_physics, &device, &queue);
    } else {
        // Render resources not yet available, mark as attempted to avoid spam
        gpu_physics.initialization_attempted = true;
        gpu_physics.enabled = false;
        warn!("Render device not available for GPU physics initialization");
    }
}


/// GPU-accelerated physics step function
/// Replaces collision detection and force computation with GPU compute shaders
/// while keeping integration on CPU for simplicity
pub fn physics_step_gpu(
    state: &mut CanonicalState,
    config: &PhysicsConfig,
    gpu_physics: &mut GpuPhysicsResource,
) {
    use crate::simulation::cpu_physics::{
        verlet_integrate_positions_soa_st,
        verlet_integrate_velocities_soa_st,
        integrate_rotations_soa_st,
        integrate_angular_velocities_soa_st,
        apply_boundary_forces_soa_st,
    };
    
    // 1. Verlet integration (position update) - CPU
    verlet_integrate_positions_soa_st(
        &mut state.positions[..state.cell_count],
        &state.velocities[..state.cell_count],
        &state.accelerations[..state.cell_count],
        config.fixed_timestep,
    );
    
    // 2. Update rotations from angular velocities - CPU
    integrate_rotations_soa_st(
        &mut state.rotations[..state.cell_count],
        &state.angular_velocities[..state.cell_count],
        config.fixed_timestep,
    );
    
    // 3. Update spatial partitioning (needed for adhesion lookups even with GPU collision)
    state.spatial_grid.rebuild(&state.positions, state.cell_count);
    
    // 4-5. Collision detection and force computation - GPU
    if !config.disable_collisions {
        compute_collision_forces_gpu(state, config, gpu_physics);
    } else {
        // Clear forces if collisions disabled
        for i in 0..state.cell_count {
            state.forces[i] = Vec3::ZERO;
            state.torques[i] = Vec3::ZERO;
        }
    }
    
    // 5.5. Compute adhesion forces (if any connections exist) - CPU
    if state.adhesion_connections.active_count > 0 {
        let default_settings = crate::cell::AdhesionSettings::default();
        let mode_settings = vec![default_settings; 10];
        
        crate::cell::compute_adhesion_forces_batched(
            &state.adhesion_connections,
            &state.positions[..state.cell_count],
            &state.velocities[..state.cell_count],
            &state.rotations[..state.cell_count],
            &state.angular_velocities[..state.cell_count],
            &state.masses[..state.cell_count],
            &mode_settings,
            &mut state.forces[..state.cell_count],
            &mut state.torques[..state.cell_count],
        );
    }
    
    // 6. Apply boundary conditions - CPU
    apply_boundary_forces_soa_st(
        &mut state.positions[..state.cell_count],
        &mut state.velocities[..state.cell_count],
        &state.rotations[..state.cell_count],
        &mut state.torques[..state.cell_count],
        config,
    );
    
    // 7. Verlet integration (velocity update) - CPU
    verlet_integrate_velocities_soa_st(
        &mut state.velocities[..state.cell_count],
        &mut state.accelerations[..state.cell_count],
        &mut state.prev_accelerations[..state.cell_count],
        &state.forces[..state.cell_count],
        &state.masses[..state.cell_count],
        config.fixed_timestep,
        config.velocity_damping,
    );
    
    // 8. Update angular velocities from torques - CPU
    integrate_angular_velocities_soa_st(
        &mut state.angular_velocities[..state.cell_count],
        &state.torques[..state.cell_count],
        &state.radii[..state.cell_count],
        &state.masses[..state.cell_count],
        config.fixed_timestep,
        config.angular_damping,
    );
}

/// GPU-accelerated physics step with genome-aware adhesion settings
pub fn physics_step_gpu_with_genome(
    state: &mut CanonicalState,
    config: &PhysicsConfig,
    genome: &crate::genome::GenomeData,
    gpu_physics: &mut GpuPhysicsResource,
    _current_time: f32,
    enable_swim: bool,
) {
    use crate::simulation::cpu_physics::{
        verlet_integrate_positions_soa_st,
        verlet_integrate_velocities_soa_st,
        integrate_rotations_soa_st,
        integrate_angular_velocities_soa_st,
        apply_boundary_forces_soa_st,
        apply_swim_forces_st,
    };
    
    // 1. Verlet integration (position update) - CPU
    verlet_integrate_positions_soa_st(
        &mut state.positions[..state.cell_count],
        &state.velocities[..state.cell_count],
        &state.accelerations[..state.cell_count],
        config.fixed_timestep,
    );
    
    // 2. Update rotations from angular velocities - CPU
    integrate_rotations_soa_st(
        &mut state.rotations[..state.cell_count],
        &state.angular_velocities[..state.cell_count],
        config.fixed_timestep,
    );
    
    // 3. Update spatial partitioning
    state.spatial_grid.rebuild(&state.positions, state.cell_count);
    
    // 4-5. Collision detection and force computation - GPU
    if !config.disable_collisions {
        compute_collision_forces_gpu(state, config, gpu_physics);
    } else {
        for i in 0..state.cell_count {
            state.forces[i] = Vec3::ZERO;
            state.torques[i] = Vec3::ZERO;
        }
    }
    
    // 5.5. Compute adhesion forces with genome settings - CPU
    if state.adhesion_connections.active_count > 0 {
        let mode_settings: Vec<crate::cell::AdhesionSettings> = genome.modes.iter()
            .map(|mode| crate::cell::AdhesionSettings {
                can_break: mode.adhesion_settings.can_break,
                break_force: mode.adhesion_settings.break_force,
                rest_length: mode.adhesion_settings.rest_length,
                linear_spring_stiffness: mode.adhesion_settings.linear_spring_stiffness,
                linear_spring_damping: mode.adhesion_settings.linear_spring_damping,
                orientation_spring_stiffness: mode.adhesion_settings.orientation_spring_stiffness,
                orientation_spring_damping: mode.adhesion_settings.orientation_spring_damping,
                max_angular_deviation: mode.adhesion_settings.max_angular_deviation,
                twist_constraint_stiffness: mode.adhesion_settings.twist_constraint_stiffness,
                twist_constraint_damping: mode.adhesion_settings.twist_constraint_damping,
                enable_twist_constraint: mode.adhesion_settings.enable_twist_constraint,
            })
            .collect();
        
        crate::cell::compute_adhesion_forces_batched(
            &state.adhesion_connections,
            &state.positions[..state.cell_count],
            &state.velocities[..state.cell_count],
            &state.rotations[..state.cell_count],
            &state.angular_velocities[..state.cell_count],
            &state.masses[..state.cell_count],
            &mode_settings,
            &mut state.forces[..state.cell_count],
            &mut state.torques[..state.cell_count],
        );
    }
    
    // 5.6. Apply swim forces for Flagellocyte cells
    apply_swim_forces_st(
        &mut state.forces[..state.cell_count],
        &state.rotations[..state.cell_count],
        &state.mode_indices[..state.cell_count],
        genome,
        enable_swim,
    );
    
    // 6. Apply boundary conditions - CPU
    apply_boundary_forces_soa_st(
        &mut state.positions[..state.cell_count],
        &mut state.velocities[..state.cell_count],
        &state.rotations[..state.cell_count],
        &mut state.torques[..state.cell_count],
        config,
    );
    
    // 7. Verlet integration (velocity update) - CPU
    verlet_integrate_velocities_soa_st(
        &mut state.velocities[..state.cell_count],
        &mut state.accelerations[..state.cell_count],
        &mut state.prev_accelerations[..state.cell_count],
        &state.forces[..state.cell_count],
        &state.masses[..state.cell_count],
        config.fixed_timestep,
        config.velocity_damping,
    );
    
    // 8. Update angular velocities from torques - CPU
    integrate_angular_velocities_soa_st(
        &mut state.angular_velocities[..state.cell_count],
        &state.torques[..state.cell_count],
        &state.radii[..state.cell_count],
        &state.masses[..state.cell_count],
        config.fixed_timestep,
        config.angular_damping,
    );
    
    // 9. Update nutrient growth for cells - CPU
    crate::simulation::nutrient_system::update_nutrient_growth_st(
        &mut state.masses[..state.cell_count],
        &mut state.radii[..state.cell_count],
        &state.mode_indices[..state.cell_count],
        genome,
        config.fixed_timestep,
    );
    
    // 10. Synchronized nutrient transport - CPU
    crate::simulation::synchronized_nutrients::transport_nutrients_synchronized(state, genome, config.fixed_timestep);
}
