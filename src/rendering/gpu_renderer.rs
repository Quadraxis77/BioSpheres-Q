//! WebGPU renderer with composite rendering.
//!
//! Rendering strategy:
//! 1. GPU scene renders to its own offscreen texture (clears with orange, draws cells)
//! 2. Composite pass blits the GPU scene texture onto the swap chain
//! 3. ImGui renders on top with LoadOp::Load
//!
//! This allows GPU scene and ImGui to coexist without overwriting each other.

use bevy::prelude::*;
use bevy::render::{
    render_graph::{Node, NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel},
    renderer::{RenderContext, RenderDevice, RenderQueue},
    view::ExtractedWindows,
    Extract, Render, RenderApp, RenderSystems,
};
use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};

use super::gpu_camera::{CameraUniform, GpuCamera};
use super::gpu_icosphere::{IcosphereMesh, IcosphereMeshBuffers};
use super::gpu_shaders::ShaderSystem;
use super::gpu_triple_buffer::DEFAULT_MAX_INSTANCES;
use super::gpu_types::{CellInstanceData, CellPhysicsData, WebGpuError};
use super::gpu_compute::*;
use super::gpu_compute_pipeline::GpuComputePipelines;
use super::gpu_compute_dispatcher::*;
use crate::genome::CurrentGenome;
use crate::simulation::GpuSceneState;
use crate::ui::camera::MainCamera;

pub const DEFAULT_ICOSPHERE_SUBDIVISIONS: u32 = 2;

pub struct WebGpuRendererPlugin;

impl Plugin for WebGpuRendererPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpuDragState>()
            .add_systems(OnEnter(GpuSceneState::Active), on_enter_gpu_scene)
            .add_systems(OnExit(GpuSceneState::Active), on_exit_gpu_scene)
            .add_systems(
                Update,
                (
                    sync_gpu_camera_from_main,
                    handle_gpu_drag_start,
                    handle_gpu_drag_update,
                    handle_gpu_drag_end,
                ).run_if(in_state(GpuSceneState::Active)),
            )
            // Run cell division in Last schedule so it happens right before extraction
            .add_systems(
                Last,
                check_cell_division_system.run_if(in_state(GpuSceneState::Active)),
            );
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(ExtractSchedule, extract_gpu_scene_data)
                .add_systems(Render, (
                    prepare_gpu_resources.in_set(RenderSystems::Prepare),
                    prepare_compute_resources.in_set(RenderSystems::Prepare),
                ));

            render_app.add_render_graph_node::<GpuSceneDirectNode>(Core3d, GpuSceneNodeLabel);
            render_app.add_render_graph_edges(Core3d, (Node3d::Upscaling, GpuSceneNodeLabel));
        }
    }
}

/// Plugin to add render graph edge after ImGui node exists
/// This MUST be added after UiPlugin (which contains ImguiPlugin)
pub struct GpuSceneImguiEdgePlugin;

impl Plugin for GpuSceneImguiEdgePlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        // This plugin runs AFTER ImguiPlugin, so the ImGui node now exists
        // Add edge: GpuSceneNode -> ImGuiNode to ensure ImGui renders on top
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_render_graph_edges(
                Core3d,
                (GpuSceneNodeLabel, bevy_mod_imgui::ImGuiNodeLabel),
            );
            info!("Added render graph edge: GpuSceneNode -> ImGuiNode");
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct GpuSceneNodeLabel;

/// Fullscreen blit shader for compositing
const BLIT_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Fullscreen triangle that covers the entire screen
    // Vertices: 0 -> (-1, -1), 1 -> (3, -1), 2 -> (-1, 3)
    var out: VertexOutput;
    let x = f32(i32(vertex_index) * 2 - 1);
    let y = f32(i32(vertex_index / 2u) * 4 - 1);
    
    // Use standard fullscreen triangle positions
    var pos: vec2<f32>;
    var uv: vec2<f32>;
    switch vertex_index {
        case 0u: {
            pos = vec2<f32>(-1.0, -1.0);
            uv = vec2<f32>(0.0, 1.0);
        }
        case 1u: {
            pos = vec2<f32>(3.0, -1.0);
            uv = vec2<f32>(2.0, 1.0);
        }
        case 2u, default: {
            pos = vec2<f32>(-1.0, 3.0);
            uv = vec2<f32>(0.0, -1.0);
        }
    }
    
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@group(0) @binding(0) var t_scene: texture_2d<f32>;
@group(0) @binding(1) var s_scene: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_scene, s_scene, in.uv);
}
"#;


/// Resources for composite rendering
#[derive(Resource)]
pub struct CompositeResources {
    /// Offscreen texture for GPU scene rendering
    pub scene_texture: wgpu::Texture,
    pub scene_view: wgpu::TextureView,
    /// Depth texture
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    /// Blit pipeline and bind group
    pub blit_pipeline: wgpu::RenderPipeline,
    pub blit_bind_group_layout: wgpu::BindGroupLayout,
    pub blit_bind_group: wgpu::BindGroup,
    pub sampler: wgpu::Sampler,
    /// Current dimensions
    pub dimensions: (u32, u32),
}

impl CompositeResources {
    pub fn new(device: &wgpu::Device, width: u32, height: u32, surface_format: wgpu::TextureFormat) -> Self {
        let width = width.max(1);
        let height = height.max(1);

        // Create offscreen scene texture
        let scene_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GPU Scene Offscreen Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let scene_view = scene_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create depth texture
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GPU Scene Depth Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("GPU Scene Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create blit shader
        let blit_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blit Shader"),
            source: wgpu::ShaderSource::Wgsl(BLIT_SHADER.into()),
        });

        // Create bind group layout
        let blit_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blit Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create blit pipeline
        let blit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit Pipeline Layout"),
            bind_group_layouts: &[&blit_bind_group_layout],
            push_constant_ranges: &[],
        });

        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blit Pipeline"),
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None, // Opaque blit
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create bind group
        let blit_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blit Bind Group"),
            layout: &blit_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            scene_texture,
            scene_view,
            depth_texture,
            depth_view,
            blit_pipeline,
            blit_bind_group_layout,
            blit_bind_group,
            sampler,
            dimensions: (width, height),
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32, surface_format: wgpu::TextureFormat) {
        if (width, height) == self.dimensions || width == 0 || height == 0 {
            return;
        }

        // Recreate textures
        self.scene_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GPU Scene Offscreen Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.scene_view = self.scene_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GPU Scene Depth Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Recreate bind group with new texture view
        self.blit_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blit Bind Group"),
            layout: &self.blit_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.scene_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        self.dimensions = (width, height);
    }
}


/// Simplified direct render node - accesses main world resources directly
pub struct GpuSceneDirectNode;

impl FromWorld for GpuSceneDirectNode {
    fn from_world(_world: &mut World) -> Self {
        GpuSceneDirectNode
    }
}

impl Node for GpuSceneDirectNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // Use extracted resources (standard Bevy approach)
        let Some(state) = world.get_resource::<ExtractedGpuSceneState>() else {
            return Ok(());
        };
        if !state.is_active {
            return Ok(());
        }

        let Some(physics) = world.get_resource::<ExtractedCellPhysicsData>() else {
            return Ok(());
        };
        
        let cell_count = physics.cells.len() as u32;
        if cell_count == 0 {
            return Ok(());
        }
        
        let Some(_device) = world.get_resource::<RenderDevice>() else {
            return Ok(());
        };
        let Some(queue) = world.get_resource::<RenderQueue>() else {
            return Ok(());
        };
        
        let Some(composite) = world.get_resource::<CompositeResources>() else {
            return Ok(());
        };

        let Some(extracted_windows) = world.get_resource::<ExtractedWindows>() else {
            return Ok(());
        };
        let Some(primary) = extracted_windows.primary else {
            return Ok(());
        };
        let Some(extracted_window) = extracted_windows.windows.get(&primary) else {
            return Ok(());
        };
        let Some(swap_chain_texture_view) = extracted_window.swap_chain_texture_view.as_ref() else {
            return Ok(());
        };

        let compute_ready = world.get_resource::<GpuComputeReady>()
            .map(|r| r.ready)
            .unwrap_or(false);

        if !compute_ready {
            return Ok(());
        }

        let Some(_pipelines) = world.get_resource::<GpuComputePipelines>() else {
            return Ok(());
        };
        
        let dragged_cell_index = world.get_resource::<ExtractedGpuDragState>()
            .and_then(|drag| drag.dragged_cell_index);
        
        // SAFETY: Get mutable access to buffers
        unsafe {
            let world_mut = world as *const World as *mut World;
            if let Some(buffers) = (*world_mut).get_resource_mut::<GpuComputeBuffers>() {
                // Upload extracted cell data to ALL three buffers to ensure consistency
                let compute_cells: Vec<ComputeCell> = physics.cells
                    .iter()
                    .map(|cell| cell.to_compute_cell())
                    .collect();
                let data: &[u8] = bytemuck::cast_slice(&compute_cells);
                
                // Upload to all buffers so they all have the current cell data
                queue.write_buffer(&buffers.cell_buffers[0], 0, data);
                queue.write_buffer(&buffers.cell_buffers[1], 0, data);
                queue.write_buffer(&buffers.cell_buffers[2], 0, data);
                
                // Debug: Log cell positions when count changes
                static mut LAST_UPLOAD_COUNT: u32 = 0;
                if cell_count != LAST_UPLOAD_COUNT {
                    info!("[UPLOAD] {} cells uploaded to GPU buffers", cell_count);
                    info!("  ComputeCell size: {} bytes, buffer size: {} bytes", 
                        std::mem::size_of::<ComputeCell>(), data.len());
                    for (i, cell) in physics.cells.iter().enumerate().take(5) {
                        info!("  Cell {}: pos=({:.2}, {:.2}, {:.2}), radius={:.2}", 
                            i, cell.position.x, cell.position.y, cell.position.z, cell.radius);
                    }
                    LAST_UPLOAD_COUNT = cell_count;
                }
                
                // Update uniforms
                if let Some(time) = world.get_resource::<Time>() {
                    update_compute_uniforms(&queue, &buffers, time.delta_secs(), cell_count, dragged_cell_index);
                }

                // SKIP COMPUTE FOR NOW - just test rendering
                // Don't advance buffers - render from buffer[0] which we just uploaded to
                // buffers.advance_frame();
            }
        }

        let command_encoder = render_context.command_encoder();

        // Pass 1: Render GPU scene to offscreen texture
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("GPU Scene Offscreen Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &composite.scene_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.6,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &composite.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Draw cells using extracted data
            if let Some(resources) = world.get_resource::<WebGpuRenderResources>() {
                if let Some(compute_buffers) = world.get_resource::<GpuComputeBuffers>() {
                    let instance_count = cell_count;
                    if instance_count > 0 {
                        static mut LAST_RENDER_COUNT: u32 = 0;
                        if instance_count != unsafe { LAST_RENDER_COUNT } {
                            info!("[RENDER] Drawing {} instances (was {})", instance_count, unsafe { LAST_RENDER_COUNT });
                            unsafe { LAST_RENDER_COUNT = instance_count; }
                        }
                        
                        render_pass.set_pipeline(&resources.shader_system.render_pipeline);
                        render_pass.set_bind_group(0, &resources.camera_bind_group, &[]);
                        render_pass.set_vertex_buffer(0, resources.icosphere_mesh.vertex_buffer.slice(..));
                        // Use compute buffer as instance buffer (contains GPU-updated cell data)
                        let wgpu_buffer: &wgpu::Buffer = compute_buffers.render_buffer();
                        render_pass.set_vertex_buffer(1, wgpu_buffer.slice(..));
                        render_pass.set_index_buffer(
                            resources.icosphere_mesh.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        // Draw all instances at once
                        render_pass.draw_indexed(
                            0..resources.icosphere_mesh.index_count(),
                            0,
                            0..instance_count,
                        );
                    }
                }
            }
        }

        // Pass 2: Blit offscreen texture to swap chain
        // Use LoadOp::Load to preserve anything already rendered, then draw our scene on top
        // ImGui will then draw on top of us with its own LoadOp::Load
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("GPU Scene Blit Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: swap_chain_texture_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Preserve existing content
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&composite.blit_pipeline);
            render_pass.set_bind_group(0, &composite.blit_bind_group, &[]);
            render_pass.draw(0..3, 0..1); // Fullscreen triangle
        }

        Ok(())
    }
}

#[derive(Resource, Default)]
pub struct ExtractedGpuSceneState {
    pub is_active: bool,
}

/// Tracks when GPU compute resources are fully initialized and ready for use
#[derive(Resource, Default)]
pub struct GpuComputeReady {
    pub ready: bool,
}

#[derive(Resource, Default)]
pub struct ExtractedGpuDragState {
    pub dragged_cell_index: Option<usize>,
}

#[derive(Resource, Default)]
pub struct ExtractedGpuCamera {
    pub uniform: CameraUniform,
}

#[derive(Resource, Default)]
pub struct ExtractedInstanceData {
    pub instances: Vec<CellInstanceData>,
}

#[derive(Resource, Default)]
pub struct ExtractedCellPhysicsData {
    pub cells: Vec<CellPhysicsData>,
}

#[derive(Resource)]
pub struct GpuSceneData {
    /// Physics state for all cells
    pub cells: Vec<CellPhysicsData>,
    /// Rendering data (synced from physics each frame)
    pub triple_buffer_data: Vec<CellInstanceData>,
    /// Maximum number of cells allowed (GPU limit)
    pub max_cells: usize,
}

impl Default for GpuSceneData {
    fn default() -> Self {
        Self {
            cells: Vec::new(),
            triple_buffer_data: Vec::new(),
            max_cells: 100_000, // GPU target: 100K cells
        }
    }
}

/// Resource tracking GPU scene cell dragging state
#[derive(Resource, Default)]
pub struct GpuDragState {
    pub dragged_cell_index: Option<usize>,
    pub drag_offset: Vec3,
    pub drag_plane_normal: Vec3,
    /// Fixed distance from camera to drag plane (not the cell center)
    pub camera_to_plane_distance: f32,
}


/// Render world resource holding GPU resources for cell rendering
#[derive(Resource)]
pub struct WebGpuRenderResources {
    pub shader_system: ShaderSystem,
    pub camera_uniform_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub icosphere_mesh: IcosphereMeshBuffers,
    pub instance_buffer: wgpu::Buffer,
    pub instance_count: u32,
}

impl WebGpuRenderResources {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Result<Self, WebGpuError> {
        let shader_system = ShaderSystem::new(device, surface_format)?;

        let camera_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Scene Camera Uniform Buffer"),
            size: CameraUniform::SIZE as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = shader_system.create_camera_bind_group(device, &camera_uniform_buffer);

        let icosphere_mesh = IcosphereMeshBuffers::from_mesh(
            device,
            &IcosphereMesh::generate(DEFAULT_ICOSPHERE_SUBDIVISIONS),
        );

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Scene Instance Buffer"),
            size: (DEFAULT_MAX_INSTANCES * CellInstanceData::SIZE) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            shader_system,
            camera_uniform_buffer,
            camera_bind_group,
            icosphere_mesh,
            instance_buffer,
            instance_count: 0,
        })
    }
}

fn extract_gpu_scene_data(
    mut commands: Commands,
    gpu_scene_state: Extract<Res<State<GpuSceneState>>>,
    gpu_camera: Extract<Option<Res<GpuCamera>>>,
    gpu_scene_data: Extract<Option<Res<GpuSceneData>>>,
    gpu_drag_state: Extract<Option<Res<GpuDragState>>>,
) {
    let is_active = **gpu_scene_state == GpuSceneState::Active;
    commands.insert_resource(ExtractedGpuSceneState { is_active });

    if let Some(camera) = gpu_camera.as_ref() {
        commands.insert_resource(ExtractedGpuCamera {
            uniform: camera.to_uniform(),
        });
    }

    // Extract drag state
    if let Some(drag_state) = gpu_drag_state.as_ref() {
        commands.insert_resource(ExtractedGpuDragState {
            dragged_cell_index: drag_state.dragged_cell_index,
        });
    } else {
        commands.insert_resource(ExtractedGpuDragState::default());
    }

    if let Some(scene_data) = gpu_scene_data.as_ref() {
        commands.insert_resource(ExtractedInstanceData {
            instances: scene_data.triple_buffer_data.clone(),
        });

        // Also extract physics data for GPU compute
        commands.insert_resource(ExtractedCellPhysicsData {
            cells: scene_data.cells.clone(),
        });
    }
}

fn prepare_gpu_resources(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    extracted_windows: Res<ExtractedWindows>,
    scene_state: Option<Res<ExtractedGpuSceneState>>,
    extracted_camera: Option<Res<ExtractedGpuCamera>>,
    extracted_instances: Option<Res<ExtractedInstanceData>>,
    mut resources: Option<ResMut<WebGpuRenderResources>>,
    mut composite: Option<ResMut<CompositeResources>>,
) {
    let Some(state) = scene_state else { return };
    if !state.is_active {
        return;
    }

    let device = render_device.wgpu_device();

    // Get window dimensions and format
    let (width, height, format) = if let Some(primary) = extracted_windows.primary {
        if let Some(window) = extracted_windows.windows.get(&primary) {
            let fmt = window.swap_chain_texture_format.unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);
            (window.physical_width, window.physical_height, fmt)
        } else {
            (1920, 1080, wgpu::TextureFormat::Bgra8UnormSrgb)
        }
    } else {
        (1920, 1080, wgpu::TextureFormat::Bgra8UnormSrgb)
    };

    // Create or resize composite resources
    if composite.is_none() {
        info!("Creating composite resources {}x{}", width, height);
        commands.insert_resource(CompositeResources::new(device, width, height, format));
    } else if let Some(comp) = composite.as_mut() {
        comp.resize(device, width, height, format);
    }

    // Create or update render resources
    if resources.is_none() {
        match WebGpuRenderResources::new(device, format) {
            Ok(new_resources) => {
                info!("WebGPU render resources created");
                commands.insert_resource(new_resources);
            }
            Err(e) => {
                error!("Failed to create WebGPU render resources: {}", e);
                return;
            }
        }
        return;
    }

    let resources = resources.as_mut().unwrap();

    if let Some(camera) = extracted_camera {
        render_queue.write_buffer(&resources.camera_uniform_buffer, 0, camera.uniform.as_bytes());
    }

    if let Some(instances) = extracted_instances {
        if !instances.instances.is_empty() {
            let data: &[u8] = bytemuck::cast_slice(&instances.instances);
            render_queue.write_buffer(&resources.instance_buffer, 0, data);
            resources.instance_count = instances.instances.len() as u32;
        } else {
            resources.instance_count = 0;
        }
    }
}


// ============================================================================
// Main App Systems
// ============================================================================

fn on_enter_gpu_scene(
    mut commands: Commands,
    current_genome: Option<Res<CurrentGenome>>,
    mut main_camera_query: Query<(&mut MainCamera, &mut Transform)>,
) {
    info!("GPU scene activated - WebGPU composite rendering enabled");

    // Set MainCamera to match other scenes: center at (0, 0, 10) with 0 orbit distance
    if let Ok((mut main_camera, mut transform)) = main_camera_query.single_mut() {
        main_camera.center = Vec3::new(0.0, 0.0, 10.0);
        main_camera.distance = 0.0;
        // Update transform to match
        let offset = main_camera.rotation * Vec3::new(0.0, 0.0, main_camera.distance);
        transform.translation = main_camera.center + offset;
    }

    let gpu_camera = GpuCamera::default();
    commands.insert_resource(gpu_camera);

    let mut scene_data = GpuSceneData::default();
    create_initial_cell_from_genome(&mut scene_data, current_genome.as_deref());

    // Sync physics to rendering data for initial display
    sync_physics_to_rendering(&mut scene_data);

    commands.insert_resource(scene_data);
}

fn on_exit_gpu_scene(mut commands: Commands) {
    commands.remove_resource::<GpuCamera>();
    commands.remove_resource::<GpuSceneData>();
    info!("GPU scene deactivated");
}

fn sync_gpu_camera_from_main(
    mut gpu_camera: Option<ResMut<GpuCamera>>,
    main_camera_query: Query<(&MainCamera, &Transform)>,
    windows: Query<&Window>,
) {
    let Some(gpu_camera) = gpu_camera.as_mut() else { return };

    if let Ok((main_camera, transform)) = main_camera_query.single() {
        gpu_camera.sync_from_main_camera(main_camera, transform);
    }

    // Update aspect ratio from window dimensions
    if let Ok(window) = windows.single() {
        gpu_camera.set_aspect_ratio(window.physical_width() as f32, window.physical_height() as f32);
    }
}

fn create_initial_cell_from_genome(
    scene_data: &mut GpuSceneData,
    current_genome: Option<&CurrentGenome>,
) {
    let default_color = Vec3::new(0.8, 0.3, 0.5);
    let default_mass = 1.0;
    let default_orientation = Quat::IDENTITY;

    let (color, mass, orientation, mode_index) = if let Some(genome_res) = current_genome {
        let genome = &genome_res.genome;
        let initial_mode_index = genome.initial_mode.max(0) as usize;

        if let Some(mode) = genome.modes.get(initial_mode_index) {
            (mode.color, mode.split_mass, genome.initial_orientation, initial_mode_index)
        } else if let Some(first_mode) = genome.modes.first() {
            (first_mode.color, first_mode.split_mass, genome.initial_orientation, 0)
        } else {
            (default_color, default_mass, default_orientation, 0)
        }
    } else {
        (default_color, default_mass, default_orientation, 0)
    };

    let radius = mass.powf(1.0 / 3.0);

    let mut initial_cell = CellPhysicsData::new();
    initial_cell.position = Vec3::ZERO; // Start at origin
    initial_cell.velocity = Vec3::ZERO; // No initial velocity
    initial_cell.orientation = orientation;
    initial_cell.genome_orientation = orientation; // Initialize genome orientation to match physical orientation
    initial_cell.mass = mass;
    initial_cell.radius = radius;
    initial_cell.color = color;
    initial_cell.mode_index = mode_index;
    initial_cell.age = 0.0;
    initial_cell.energy = 1.0;
    initial_cell.genome_id = 0;
    initial_cell.cell_type = 0;
    initial_cell.flags = 0;

    scene_data.cells.push(initial_cell);

    info!(
        "Initial cell created: color={:?}, mass={}, radius={}, orientation={:?}",
        color, mass, radius, orientation
    );
}

// ============================================================================
// GPU Compute Physics Systems (Render World)
// ============================================================================

/// Prepare compute resources (buffers and pipelines)
fn prepare_compute_resources(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    scene_state: Option<Res<ExtractedGpuSceneState>>,
    extracted_physics: Option<Res<ExtractedCellPhysicsData>>,
    buffers: Option<Res<GpuComputeBuffers>>,
    pipelines: Option<Res<GpuComputePipelines>>,
    compute_ready: Option<Res<GpuComputeReady>>,
) {
    let Some(state) = scene_state else { return };
    if !state.is_active {
        return;
    }

    // Track if we're creating resources this frame (they won't be ready until next frame)
    let mut creating_resources = false;

    // Create buffers if they don't exist
    if buffers.is_none() {
        info!("Creating GPU compute buffers");
        info!("ComputeCell size: {} bytes (expected 384)", std::mem::size_of::<ComputeCell>());
        commands.insert_resource(GpuComputeBuffers::new(&render_device));
        creating_resources = true;
    }

    // Create pipelines if they don't exist
    if pipelines.is_none() {
        info!("Creating GPU compute pipelines");
        commands.insert_resource(GpuComputePipelines::new(&render_device));
        creating_resources = true;
    }

    // Note: Bind groups are now created dynamically during dispatch_gpu_physics_to_encoder
    // This allows us to rebuild them when buffers are swapped between compute passes

    // Mark compute as ready only if all resources exist and we're not creating any this frame
    if !creating_resources && buffers.is_some() && pipelines.is_some() {
        if compute_ready.is_none() || !compute_ready.as_ref().unwrap().ready {
            info!("GPU compute resources ready");
            commands.insert_resource(GpuComputeReady { ready: true });
        }
    }

    // Upload cell data to GPU buffers in prepare phase
    if let (Some(physics), Some(bufs)) = (extracted_physics.as_ref(), buffers.as_ref()) {
        if !physics.cells.is_empty() {
            let compute_cells: Vec<ComputeCell> = physics.cells
                .iter()
                .map(|cell| cell.to_compute_cell())
                .collect();
            let data: &[u8] = bytemuck::cast_slice(&compute_cells);
            
            // Upload to read_buffer only - compute will process and write to write_buffer
            render_queue.write_buffer(bufs.read_buffer(), 0, data);
            
            static mut LAST_UPLOAD_COUNT: usize = 0;
            if physics.cells.len() != unsafe { LAST_UPLOAD_COUNT } {
                info!("[PREPARE] Uploaded {} cells to read_buffer", physics.cells.len());
                info!("  ComputeCell size: {} bytes, buffer size: {} bytes", 
                    std::mem::size_of::<ComputeCell>(), data.len());
                for (i, cell) in physics.cells.iter().enumerate().take(5) {
                    info!("  Cell {}: pos=({:.2}, {:.2}, {:.2}), radius={:.2}", 
                        i, cell.position.x, cell.position.y, cell.position.z, cell.radius);
                }
                unsafe { LAST_UPLOAD_COUNT = physics.cells.len(); }
            }
        }
    }
}

// GPU compute physics now runs directly in the render graph node (GpuSceneCompositeNode)
// This was removed because it was trying to use Bevy's render systems
// The GPU compute bypasses Bevy and uses raw wgpu, just like the rendering does

/// Check and perform cell division based on age and division threshold
/// Matches C++ implementation in cpu_simd_physics_engine.cpp:1241-1473
fn check_cell_division(scene_data: &mut GpuSceneData, genome: &crate::genome::GenomeData) {
    // Collect cells that need to split
    let mut cells_to_split = Vec::new();

    for (index, cell) in scene_data.cells.iter().enumerate() {
        // Get division threshold from the cell's current mode
        let division_threshold = if let Some(mode) = genome.modes.get(cell.mode_index) {
            mode.split_interval
        } else {
            2.0 // Default threshold
        };

        if cell.age >= division_threshold {
            cells_to_split.push(index);
        }
    }

    // Process cell divisions
    for cell_index in cells_to_split {
        // Check capacity (GPU limit: 100K cells)
        if scene_data.cells.len() >= scene_data.max_cells {
            warn!("GPU scene at maximum capacity ({}), skipping remaining divisions", scene_data.max_cells);
            break;
        }

        // Get parent cell data (need to clone to avoid borrow issues)
        let parent = scene_data.cells[cell_index].clone();

        // Get mode settings from genome
        let mode = genome.modes.get(parent.mode_index);
        let Some(mode) = mode else {
            warn!("Cell has invalid mode_index {}, skipping division", parent.mode_index);
            continue;
        };

        let division_threshold = mode.split_interval;

        // Initialize genome orientation if uninitialized (first division)
        // C++: lines 1268-1277
        if parent.genome_orientation.w == 0.0 && parent.genome_orientation.x == 0.0 &&
           parent.genome_orientation.y == 0.0 && parent.genome_orientation.z == 0.0 {
            scene_data.cells[cell_index].genome_orientation = Quat::IDENTITY;
        }

        // Get parent's genome orientation before division (needed for inheritance)
        let parent_genome_orientation = scene_data.cells[cell_index].genome_orientation;

        // Apply child orientations from genome (C++: lines 1296-1319)
        let child_orientation_a = mode.child_a.orientation;
        let child_orientation_b = mode.child_b.orientation;

        // New orientation = parentOrientation * childOrientation (matching GPU)
        let new_orientation_a = (parent.orientation * child_orientation_a).normalize();
        let new_orientation_b = (parent.orientation * child_orientation_b).normalize();

        // Update parent cell orientation (becomes child A)
        scene_data.cells[cell_index].orientation = new_orientation_a;

        // GPU behavior: Mass is NOT split - both children keep parent's mass (C++: lines 1321-1324)
        let original_mass = parent.mass;
        let radius = original_mass.powf(1.0 / 3.0); // C++: line 1327

        scene_data.cells[cell_index].mass = original_mass;
        scene_data.cells[cell_index].radius = radius;

        // GPU behavior: Age is reset to excess beyond split interval (C++: lines 1332-1335)
        let start_age = parent.age - division_threshold;
        scene_data.cells[cell_index].age = start_age;

        // Split energy (C++: lines 1338-1339)
        scene_data.cells[cell_index].energy = parent.energy * 0.5;

        // Calculate split direction from pitch/yaw (matching CPU division.rs:108-110)
        let pitch = mode.parent_split_direction.x.to_radians();
        let yaw = mode.parent_split_direction.y.to_radians();
        let split_direction = parent.orientation * Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0) * Vec3::Z;

        // Reduced offset for better adhesion overlap
        // Combined diameter = 2 * radius, so offset = 0.1 * 2 * radius = 0.2 * radius
        // Match C++ convention and CPU implementation (C++: lines 1362-1374)
        let offset_distance = radius * 0.1;
        let offset = split_direction * offset_distance;

        // Child A gets +offset (C++: lines 1368-1370)
        scene_data.cells[cell_index].position = parent.position + offset;

        // Update genome orientations for child cells (C++: lines 1399-1402)
        let child_a_genome_orientation = (parent_genome_orientation * child_orientation_a).normalize();
        let child_b_genome_orientation = (parent_genome_orientation * child_orientation_b).normalize();

        scene_data.cells[cell_index].genome_orientation = child_a_genome_orientation;

        // Create daughter cell (child B) (C++: lines 1279-1281)
        let mut daughter = parent.clone();

        // Apply child B properties
        daughter.position = parent.position - offset; // Child B gets -offset (C++: lines 1372-1374)
        daughter.orientation = new_orientation_b;
        daughter.genome_orientation = child_b_genome_orientation;
        daughter.mass = original_mass;
        daughter.radius = radius;
        daughter.age = start_age + 0.001; // Slight offset like GPU (C++: line 1335)
        daughter.energy = parent.energy * 0.5;

        // Copy other properties (C++: lines 1341-1347)
        daughter.cell_type = parent.cell_type;
        daughter.genome_id = parent.genome_id;
        daughter.flags = parent.flags;
        daughter.color = parent.color;
        daughter.velocity = parent.velocity; // GPU behavior: No velocity separation (C++: line 1376)
        daughter.angular_velocity = parent.angular_velocity;

        // Add daughter cell
        scene_data.cells.push(daughter);

        let parent_pos = scene_data.cells[cell_index].position;
        let daughter_pos = scene_data.cells[scene_data.cells.len() - 1].position;
        info!(
            "[DIVISION] Split complete: {} cells total",
            scene_data.cells.len()
        );
        info!(
            "  Parent[{}]: pos=({:.2}, {:.2}, {:.2})",
            cell_index, parent_pos.x, parent_pos.y, parent_pos.z
        );
        info!(
            "  Daughter[{}]: pos=({:.2}, {:.2}, {:.2})",
            scene_data.cells.len() - 1, daughter_pos.x, daughter_pos.y, daughter_pos.z
        );
        info!(
            "  Split: direction={:?}, offset={:.2}, radius={:.2}",
            split_direction, offset_distance, radius
        );
    }
}

/// System that ages cells and checks for cell division
/// This runs in the main world and operates on GpuSceneData
/// The GPU also ages cells independently - they stay roughly in sync
fn check_cell_division_system(
    mut scene_data: ResMut<GpuSceneData>,
    current_genome: Option<Res<CurrentGenome>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    // Age all cells (GPU does this too: 0.5 in position shader + 0.5 in velocity shader = 1.0 total)
    for cell in scene_data.cells.iter_mut() {
        cell.age += dt;
    }

    // Debug: Log cell count every frame when it changes
    static mut LAST_CELL_COUNT: usize = 0;
    let current_count = scene_data.cells.len();
    if current_count != unsafe { LAST_CELL_COUNT } {
        info!("[MAIN WORLD - Last schedule] Cell count changed: {} -> {}", unsafe { LAST_CELL_COUNT }, current_count);
        unsafe { LAST_CELL_COUNT = current_count; }
    }

    // Check for division
    if let Some(genome) = current_genome.as_deref() {
        check_cell_division(&mut scene_data, &genome.genome);
    } else {
        warn!("No current genome available for division check!");
    }
}

/// Sync physics data to rendering buffer
fn sync_physics_to_rendering(scene_data: &mut GpuSceneData) {
    // Clear rendering buffer and collect new instance data
    let instance_data: Vec<CellInstanceData> = scene_data
        .cells
        .iter()
        .map(|cell| cell.to_instance_data())
        .collect();

    scene_data.triple_buffer_data = instance_data;
}

/// TEMPORARY: Simple CPU physics update for MVP
/// This runs in the main world and updates GpuSceneData.cells
/// Eventually this will be replaced by full GPU compute pipeline with proper feedback
#[allow(dead_code)]
fn simple_cpu_physics_update(
    mut scene_data: ResMut<GpuSceneData>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let damping = 0.98_f32.powf(dt * 100.0);
    let sphere_radius = 50.0;

    for cell in scene_data.cells.iter_mut() {
        // Verlet integration: velocity update
        cell.velocity += cell.acceleration * dt;
        cell.velocity *= damping;

        // Sphere boundary check
        let dist = cell.position.length();
        if dist > sphere_radius - cell.radius {
            let normal = cell.position.normalize();
            let penetration = dist - (sphere_radius - cell.radius);
            cell.position -= normal * penetration;

            // Reflect velocity
            let vel_normal = cell.velocity.dot(normal);
            if vel_normal > 0.0 {
                cell.velocity -= normal * vel_normal * 1.8;
                cell.velocity *= 0.8; // Energy loss on bounce
            }
        }

        // Verlet integration: position update
        cell.position += cell.velocity * dt;

        // Reset acceleration for next frame
        cell.acceleration = Vec3::ZERO;
    }
}

/// Sync rendering data from physics after update
#[allow(dead_code)]
fn sync_rendering_data(mut scene_data: ResMut<GpuSceneData>) {
    sync_physics_to_rendering(&mut scene_data);
}

// ============================================================================
// GPU Scene Cell Dragging Systems
// ============================================================================

use bevy::window::PrimaryWindow;

/// System to handle starting a drag operation in GPU scene
fn handle_gpu_drag_start(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<GpuDragState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    scene_data: Res<GpuSceneData>,
    imgui_capture: Res<crate::ui::camera::ImGuiWantCapture>,
) {
    // Don't process mouse input if ImGui wants to capture it
    if imgui_capture.want_capture_mouse {
        return;
    }
    
    // Only start drag on left mouse button press
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Skip if already dragging
    if drag_state.dragged_cell_index.is_some() {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // Get cursor position
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Raycast from camera through cursor
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };

    // Find closest cell intersected by ray
    let mut closest_hit: Option<(usize, f32)> = None;

    for (index, cell) in scene_data.cells.iter().enumerate() {
        // Ray-sphere intersection test
        if let Some(hit_distance) = ray_sphere_intersection(
            ray.origin,
            *ray.direction,
            cell.position,
            cell.radius,
        ) {
            // Keep track of closest hit
            if closest_hit.is_none() || hit_distance < closest_hit.unwrap().1 {
                closest_hit = Some((index, hit_distance));
            }
        }
    }

    // If we hit a cell, start dragging it
    if let Some((cell_index, hit_distance)) = closest_hit {
        let cell = &scene_data.cells[cell_index];
        
        // Calculate drag plane perpendicular to camera forward
        let camera_forward = camera_transform.forward();
        let drag_plane_normal = Vec3::from(*camera_forward);
        
        // Calculate distance from camera to the hit point on the cell surface
        // This accounts for the cell's radius so the surface stays under cursor
        let camera_to_plane_distance = hit_distance;
        
        // Calculate the plane distance in world space
        // The plane is at a fixed distance from camera along camera forward
        let plane_point_on_axis = ray.origin + *ray.direction * camera_to_plane_distance;
        let drag_plane_distance = plane_point_on_axis.dot(drag_plane_normal);
        
        // Calculate offset from plane intersection to cell center
        let ray_to_plane = ray_plane_intersection(
            ray.origin,
            *ray.direction,
            drag_plane_normal,
            drag_plane_distance,
        );
        
        let drag_offset = if let Some(plane_point) = ray_to_plane {
            plane_point - cell.position
        } else {
            Vec3::ZERO
        };
        
        drag_state.dragged_cell_index = Some(cell_index);
        drag_state.drag_offset = drag_offset;
        drag_state.drag_plane_normal = drag_plane_normal;
        drag_state.camera_to_plane_distance = camera_to_plane_distance;
        
        info!("Started dragging GPU cell {}", cell_index);
    }
}

/// System to update dragged cell position in GPU scene
fn handle_gpu_drag_update(
    drag_state: Res<GpuDragState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut scene_data: ResMut<GpuSceneData>,
) {
    // Skip if not dragging
    let Some(cell_index) = drag_state.dragged_cell_index else {
        return;
    };

    // Validate cell index
    if cell_index >= scene_data.cells.len() {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // Get cursor position
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Raycast from camera through cursor
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };

    // Recalculate drag plane distance based on current camera position
    // The plane stays at a fixed distance from the camera along the ray direction
    let camera_forward = camera_transform.forward();
    let drag_plane_normal = Vec3::from(*camera_forward);
    let plane_point_on_axis = ray.origin + *ray.direction * drag_state.camera_to_plane_distance;
    let drag_plane_distance = plane_point_on_axis.dot(drag_plane_normal);

    // Intersect ray with drag plane
    let Some(plane_hit) = ray_plane_intersection(
        ray.origin,
        *ray.direction,
        drag_plane_normal,
        drag_plane_distance,
    ) else {
        return;
    };

    // Calculate new position
    let new_position = plane_hit - drag_state.drag_offset;

    // Update cell position and zero velocity
    scene_data.cells[cell_index].position = new_position;
    scene_data.cells[cell_index].velocity = Vec3::ZERO;
    scene_data.cells[cell_index].acceleration = Vec3::ZERO;
}

/// System to handle ending a drag operation in GPU scene
fn handle_gpu_drag_end(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<GpuDragState>,
) {
    // End drag on left mouse button release
    if mouse_button.just_released(MouseButton::Left) {
        if drag_state.dragged_cell_index.is_some() {
            info!("Ended dragging GPU cell");
        }
        drag_state.dragged_cell_index = None;
    }
}

/// Ray-sphere intersection test
fn ray_sphere_intersection(
    ray_origin: Vec3,
    ray_direction: Vec3,
    sphere_center: Vec3,
    sphere_radius: f32,
) -> Option<f32> {
    let oc = ray_origin - sphere_center;
    let a = ray_direction.dot(ray_direction);
    let b = 2.0 * oc.dot(ray_direction);
    let c = oc.dot(oc) - sphere_radius * sphere_radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        None
    } else {
        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        if t > 0.0 {
            Some(t)
        } else {
            None
        }
    }
}

/// Ray-plane intersection test
fn ray_plane_intersection(
    ray_origin: Vec3,
    ray_direction: Vec3,
    plane_normal: Vec3,
    plane_distance: f32,
) -> Option<Vec3> {
    let denom = ray_direction.dot(plane_normal);
    
    // Check if ray is parallel to plane
    if denom.abs() < 0.0001 {
        return None;
    }
    
    let t = (plane_distance - ray_origin.dot(plane_normal)) / denom;
    
    // Only return intersection if it's in front of the ray
    if t >= 0.0 {
        Some(ray_origin + ray_direction * t)
    } else {
        None
    }
}
