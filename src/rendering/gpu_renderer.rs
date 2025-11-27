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
use super::gpu_types::{CellInstanceData, WebGpuError};
use crate::genome::CurrentGenome;
use crate::simulation::GpuSceneState;
use crate::ui::camera::MainCamera;

pub const DEFAULT_ICOSPHERE_SUBDIVISIONS: u32 = 2;

pub struct WebGpuRendererPlugin;

impl Plugin for WebGpuRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GpuSceneState::Active), on_enter_gpu_scene)
            .add_systems(OnExit(GpuSceneState::Active), on_exit_gpu_scene)
            .add_systems(
                Update,
                sync_gpu_camera_from_main.run_if(in_state(GpuSceneState::Active)),
            );
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(ExtractSchedule, extract_gpu_scene_data)
                .add_systems(Render, prepare_gpu_resources.in_set(RenderSystems::Prepare));

            // Add composite node that renders GPU scene to offscreen texture then blits to swap chain
            render_app.add_render_graph_node::<GpuSceneCompositeNode>(Core3d, GpuSceneNodeLabel);

            // Run after Upscaling - this is the final stage before presentation
            render_app.add_render_graph_edges(Core3d, (Node3d::Upscaling, GpuSceneNodeLabel));

            // Note: We can't add edge to ImGui here because ImguiPlugin hasn't run yet
            // The GpuSceneImguiEdgePlugin (added after UiPlugin) will add that edge
        }
    }
}

/// Separate plugin to add the render graph edge after all plugins are initialized.
/// This MUST be added after UiPlugin (which contains ImguiPlugin).
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


/// Composite render node: renders GPU scene to offscreen texture, then blits to swap chain
pub struct GpuSceneCompositeNode;

impl FromWorld for GpuSceneCompositeNode {
    fn from_world(_world: &mut World) -> Self {
        GpuSceneCompositeNode
    }
}

impl Node for GpuSceneCompositeNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // Check if GPU scene is active
        let Some(state) = world.get_resource::<ExtractedGpuSceneState>() else {
            return Ok(());
        };
        if !state.is_active {
            return Ok(());
        }

        // Get composite resources
        let Some(composite) = world.get_resource::<CompositeResources>() else {
            return Ok(());
        };

        // Get swap chain texture
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

        let command_encoder = render_context.command_encoder();

        // Pass 1: Render GPU scene to offscreen texture
        {
            let _render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            // TODO: Draw cells here with cell pipeline
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

#[derive(Resource, Default)]
pub struct ExtractedGpuCamera {
    pub uniform: CameraUniform,
}

#[derive(Resource, Default)]
pub struct ExtractedInstanceData {
    pub instances: Vec<CellInstanceData>,
}

#[derive(Resource)]
pub struct GpuSceneData {
    pub triple_buffer_data: Vec<CellInstanceData>,
}

impl Default for GpuSceneData {
    fn default() -> Self {
        Self { triple_buffer_data: Vec::new() }
    }
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
) {
    let is_active = **gpu_scene_state == GpuSceneState::Active;
    commands.insert_resource(ExtractedGpuSceneState { is_active });

    if let Some(camera) = gpu_camera.as_ref() {
        commands.insert_resource(ExtractedGpuCamera {
            uniform: camera.to_uniform(),
        });
    }

    if let Some(scene_data) = gpu_scene_data.as_ref() {
        commands.insert_resource(ExtractedInstanceData {
            instances: scene_data.triple_buffer_data.clone(),
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

fn on_enter_gpu_scene(mut commands: Commands, current_genome: Option<Res<CurrentGenome>>) {
    info!("GPU scene activated - WebGPU composite rendering enabled");

    let gpu_camera = GpuCamera::default();
    commands.insert_resource(gpu_camera);

    let mut scene_data = GpuSceneData::default();
    create_initial_cell_from_genome(&mut scene_data, current_genome.as_deref());
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
) {
    let Some(gpu_camera) = gpu_camera.as_mut() else { return };

    if let Ok((main_camera, transform)) = main_camera_query.single() {
        gpu_camera.sync_from_main_camera(main_camera, transform);
    }
}

fn create_initial_cell_from_genome(
    scene_data: &mut GpuSceneData,
    current_genome: Option<&CurrentGenome>,
) {
    let default_color = [0.8, 0.3, 0.5, 1.0];
    let default_radius = 1.0;
    let default_orientation = [1.0, 0.0, 0.0, 0.0];

    let (color, radius, orientation) = if let Some(genome_res) = current_genome {
        let genome = &genome_res.genome;
        let initial_mode_index = genome.initial_mode as usize;

        if let Some(mode) = genome.modes.get(initial_mode_index) {
            let color = [mode.color.x, mode.color.y, mode.color.z, 1.0];
            let radius = mode.split_mass.powf(1.0 / 3.0);
            let quat = genome.initial_orientation;
            let orientation = [quat.w, quat.x, quat.y, quat.z];
            (color, radius, orientation)
        } else if let Some(first_mode) = genome.modes.first() {
            let color = [first_mode.color.x, first_mode.color.y, first_mode.color.z, 1.0];
            let radius = first_mode.split_mass.powf(1.0 / 3.0);
            let quat = genome.initial_orientation;
            let orientation = [quat.w, quat.x, quat.y, quat.z];
            (color, radius, orientation)
        } else {
            (default_color, default_radius, default_orientation)
        }
    } else {
        (default_color, default_radius, default_orientation)
    };

    let initial_cell = CellInstanceData::from_components(
        [0.0, 0.0, 0.0],
        radius,
        color,
        orientation,
    );

    scene_data.triple_buffer_data.push(initial_cell);

    info!(
        "Initial cell created: color={:?}, radius={}, orientation={:?}",
        color, radius, orientation
    );
}
