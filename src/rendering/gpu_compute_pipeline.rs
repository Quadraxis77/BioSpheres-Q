//! GPU Compute Pipeline Manager
//!
//! Manages compute shader pipelines for GPU physics simulation

use bevy::prelude::*;
use bevy::render::{
    render_resource::*,
    renderer::RenderDevice,
};

/// Compute pipeline resources
#[derive(Resource)]
pub struct GpuComputePipelines {
    /// Grid clear pipeline
    pub grid_clear_pipeline: wgpu::ComputePipeline,
    pub grid_clear_bind_group_layout: wgpu::BindGroupLayout,

    /// Grid insert pipeline
    pub grid_insert_pipeline: wgpu::ComputePipeline,
    pub grid_insert_bind_group_layout: wgpu::BindGroupLayout,

    /// Collision physics pipeline
    pub collision_pipeline: wgpu::ComputePipeline,
    pub collision_bind_group_layout: wgpu::BindGroupLayout,

    /// Velocity update pipeline
    pub velocity_pipeline: wgpu::ComputePipeline,
    pub velocity_bind_group_layout: wgpu::BindGroupLayout,

    /// Position update pipeline
    pub position_pipeline: wgpu::ComputePipeline,
    pub position_bind_group_layout: wgpu::BindGroupLayout,
}

impl GpuComputePipelines {
    pub fn new(device: &RenderDevice) -> Self {
        // Create bind group layouts
        let grid_clear_layout = Self::create_grid_clear_layout(device);
        let grid_insert_layout = Self::create_grid_insert_layout(device);
        let collision_layout = Self::create_collision_layout(device);
        let velocity_layout = Self::create_velocity_layout(device);
        let position_layout = Self::create_position_layout(device);

        // Load and compile shaders
        let grid_clear_shader = Self::load_shader(device, include_str!("../../assets/shaders/grid_clear.wgsl"), "grid_clear.wgsl");
        let grid_insert_shader = Self::load_shader(device, include_str!("../../assets/shaders/grid_insert.wgsl"), "grid_insert.wgsl");
        let collision_shader = Self::load_shader(device, include_str!("../../assets/shaders/cell_physics_spatial.wgsl"), "cell_physics_spatial.wgsl");
        let velocity_shader = Self::load_shader(device, include_str!("../../assets/shaders/cell_velocity_update.wgsl"), "cell_velocity_update.wgsl");
        let position_shader = Self::load_shader(device, include_str!("../../assets/shaders/cell_position_update.wgsl"), "cell_position_update.wgsl");

        // Create compute pipelines
        let grid_clear_pipeline = Self::create_compute_pipeline(device, &grid_clear_shader, &grid_clear_layout, "Grid Clear Pipeline");
        let grid_insert_pipeline = Self::create_compute_pipeline(device, &grid_insert_shader, &grid_insert_layout, "Grid Insert Pipeline");
        let collision_pipeline = Self::create_compute_pipeline(device, &collision_shader, &collision_layout, "Collision Pipeline");
        let velocity_pipeline = Self::create_compute_pipeline(device, &velocity_shader, &velocity_layout, "Velocity Pipeline");
        let position_pipeline = Self::create_compute_pipeline(device, &position_shader, &position_layout, "Position Pipeline");

        Self {
            grid_clear_pipeline,
            grid_clear_bind_group_layout: grid_clear_layout,
            grid_insert_pipeline,
            grid_insert_bind_group_layout: grid_insert_layout,
            collision_pipeline,
            collision_bind_group_layout: collision_layout,
            velocity_pipeline,
            velocity_bind_group_layout: velocity_layout,
            position_pipeline,
            position_bind_group_layout: position_layout,
        }
    }

    fn load_shader(device: &RenderDevice, source: &str, label: &str) -> ShaderModule {
        device.wgpu_device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        })
    }

    fn create_compute_pipeline(
        device: &RenderDevice,
        shader: &ShaderModule,
        layout: &wgpu::BindGroupLayout,
        label: &str,
    ) -> wgpu::ComputePipeline {
        let pipeline_layout = device.wgpu_device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} Layout", label)),
            bind_group_layouts: &[layout],
            push_constant_ranges: &[],
        });

        device.wgpu_device().create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),
            module: shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        })
    }

    fn create_grid_clear_layout(device: &RenderDevice) -> wgpu::BindGroupLayout {
        device.wgpu_device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Grid Clear Bind Group Layout"),
            entries: &[
                // grid_counts
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_grid_insert_layout(device: &RenderDevice) -> wgpu::BindGroupLayout {
        device.wgpu_device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Grid Insert Bind Group Layout"),
            entries: &[
                // cells (read)
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
                // grid_cells
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
                // grid_offsets (atomic)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // cell_counts
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
                // uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_collision_layout(device: &RenderDevice) -> wgpu::BindGroupLayout {
        device.wgpu_device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Collision Bind Group Layout"),
            entries: &[
                // input_cells
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
                // grid_cells
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // grid_counts
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // output_cells
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // cell_counts
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
                // modes
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
                // uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_velocity_layout(device: &RenderDevice) -> wgpu::BindGroupLayout {
        device.wgpu_device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Velocity Update Bind Group Layout"),
            entries: &[
                // input_cells
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
                // output_cells
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
                // cell_counts
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_position_layout(device: &RenderDevice) -> wgpu::BindGroupLayout {
        device.wgpu_device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Position Update Bind Group Layout"),
            entries: &[
                // input_cells
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
                // output_cells
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
                // cell_counts
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }
}
