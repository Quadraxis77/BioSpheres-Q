//! Boundary Crossing Post-Processing Effect
//! 
//! Creates a water-like ripple/warp effect when the camera crosses the world boundary sphere.
//! Uses a full-screen post-processing shader with:
//! - Expanding ripple waves
//! - Chromatic aberration
//! - Barrel/pincushion lens distortion
//! - Color tinting based on direction
//! - Vignette pulse

use bevy::{
    core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
        FullscreenShader,
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroupEntries, BindGroupLayout, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, FragmentState, Operations, PipelineCache, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, ShaderType, TextureFormat, TextureSampleType,
            BindGroupLayoutEntries, SpecializedRenderPipeline, SpecializedRenderPipelines,
        },
        renderer::{RenderContext, RenderDevice},
        view::{ExtractedView, ViewTarget},
        Render, RenderApp, RenderStartup, RenderSystems,
    },
};

use crate::ui::camera::MainCamera;

/// World boundary radius (must match the sphere spawned in preview_sim.rs)
pub const WORLD_BOUNDARY_RADIUS: f32 = 100.0;

/// Shader asset path
const SHADER_ASSET_PATH: &str = "shaders/boundary_crossing.wgsl";

/// Plugin for the boundary crossing post-processing effect
pub struct BoundaryCrossingPlugin;

impl Plugin for BoundaryCrossingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoundaryCrossingState>()
            .add_plugins((
                ExtractComponentPlugin::<BoundaryCrossingSettings>::default(),
                UniformComponentPlugin::<BoundaryCrossingSettings>::default(),
            ))
            .add_systems(Update, (
                detect_boundary_crossing,
                update_boundary_effect,
            ).chain());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedRenderPipelines<BoundaryCrossingPipeline>>()
            .add_systems(RenderStartup, init_boundary_crossing_pipeline)
            .add_systems(Render, prepare_boundary_crossing_pipelines.in_set(RenderSystems::Prepare));

        render_app
            .add_render_graph_node::<ViewNodeRunner<BoundaryCrossingNode>>(
                Core3d,
                BoundaryCrossingLabel,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::Tonemapping,
                    BoundaryCrossingLabel,
                    Node3d::EndMainPassPostProcessing,
                ),
            );
    }
}

/// Label for the boundary crossing render graph node
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct BoundaryCrossingLabel;

/// Settings component for the boundary crossing effect (attached to cameras)
/// This gets extracted to the render world and uploaded to the GPU
#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
pub struct BoundaryCrossingSettings {
    /// Effect intensity (0.0 = no effect, 1.0 = full effect)
    pub intensity: f32,
    /// Time since crossing started
    pub time: f32,
    /// Direction: 1.0 = entering, -1.0 = exiting
    pub direction: f32,
    /// Screen aspect ratio
    pub aspect_ratio: f32,
}

/// State resource for tracking boundary crossing
#[derive(Resource)]
pub struct BoundaryCrossingState {
    /// Whether camera was inside the boundary last frame
    pub was_inside: bool,
    /// Whether an effect is currently playing
    pub effect_active: bool,
    /// Time when the effect started
    pub effect_start_time: f32,
    /// Direction of the crossing (1.0 = entering, -1.0 = exiting)
    pub crossing_direction: f32,
    /// Effect duration in seconds
    pub effect_duration: f32,
    /// Whether the effect is enabled
    pub enabled: bool,
}

impl Default for BoundaryCrossingState {
    fn default() -> Self {
        Self {
            was_inside: true, // Assume starting inside
            effect_active: false,
            effect_start_time: 0.0,
            crossing_direction: 1.0,
            effect_duration: 1.0, // Quick fade
            enabled: true,
        }
    }
}

/// System to detect when camera crosses the world boundary
fn detect_boundary_crossing(
    time: Res<Time>,
    mut state: ResMut<BoundaryCrossingState>,
    camera_query: Query<&Transform, With<MainCamera>>,
) {
    if !state.enabled {
        return;
    }

    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let camera_distance = camera_transform.translation.length();
    let is_inside = camera_distance < WORLD_BOUNDARY_RADIUS;

    // Detect crossing
    if is_inside != state.was_inside {
        state.effect_active = true;
        state.effect_start_time = time.elapsed_secs();
        state.crossing_direction = if is_inside { 1.0 } else { -1.0 };
    }

    state.was_inside = is_inside;
}

/// System to update the boundary crossing effect settings on cameras
fn update_boundary_effect(
    time: Res<Time>,
    mut state: ResMut<BoundaryCrossingState>,
    windows: Query<&Window>,
    mut camera_query: Query<&mut BoundaryCrossingSettings, With<MainCamera>>,
) {
    let Ok(mut settings) = camera_query.single_mut() else {
        return;
    };

    // Get aspect ratio from window
    let aspect_ratio = windows
        .iter()
        .next()
        .map(|w| w.width() / w.height())
        .unwrap_or(16.0 / 9.0);

    settings.aspect_ratio = aspect_ratio;

    if !state.effect_active || !state.enabled {
        settings.intensity = 0.0;
        settings.time = 0.0;
        return;
    }

    let elapsed = time.elapsed_secs() - state.effect_start_time;
    
    if elapsed > state.effect_duration {
        state.effect_active = false;
        settings.intensity = 0.0;
        settings.time = 0.0;
        return;
    }

    // Quick ramp up, smooth fade out
    let normalized_time = elapsed / state.effect_duration;
    let intensity = if normalized_time < 0.08 {
        // Quick ramp up
        normalized_time / 0.08
    } else {
        // Smooth exponential fade out
        let fade_t = (normalized_time - 0.08) / 0.92;
        (1.0 - fade_t).powf(1.5)
    };

    settings.intensity = intensity;
    settings.time = elapsed;
    settings.direction = state.crossing_direction;
}

/// Component storing the per-view pipeline ID for boundary crossing
#[derive(Component)]
pub struct ViewBoundaryCrossingPipeline {
    pub pipeline_id: CachedRenderPipelineId,
}

/// Render node for the boundary crossing effect
#[derive(Default)]
struct BoundaryCrossingNode;

impl ViewNode for BoundaryCrossingNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static BoundaryCrossingSettings,
        &'static DynamicUniformIndex<BoundaryCrossingSettings>,
        &'static ViewBoundaryCrossingPipeline,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, settings, settings_index, view_pipeline): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // Skip if effect is not active (intensity near zero)
        // This prevents any render pass from being created when not needed
        if settings.intensity < 0.001 {
            return Ok(());
        }

        let Some(pipeline) = world.get_resource::<BoundaryCrossingPipeline>() else {
            return Ok(());
        };

        let pipeline_cache = world.resource::<PipelineCache>();

        // Use the per-view specialized pipeline
        let Some(render_pipeline) = pipeline_cache.get_render_pipeline(view_pipeline.pipeline_id) else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<BoundaryCrossingSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        // Get post process write - this handles the ping-pong buffer swap
        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "boundary_crossing_bind_group",
            &pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("boundary_crossing_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(render_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

/// Pipeline key for specializing the boundary crossing pipeline based on texture format
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct BoundaryCrossingPipelineKey {
    texture_format: TextureFormat,
}

/// Pipeline resource for the boundary crossing effect
#[derive(Resource)]
struct BoundaryCrossingPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    fullscreen_shader: FullscreenShader,
    fragment_shader: Handle<Shader>,
}

impl SpecializedRenderPipeline for BoundaryCrossingPipeline {
    type Key = BoundaryCrossingPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("boundary_crossing_pipeline".into()),
            layout: vec![self.layout.clone()],
            vertex: self.fullscreen_shader.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: self.fragment_shader.clone(),
                shader_defs: vec![],
                targets: vec![Some(ColorTargetState {
                    format: key.texture_format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        }
    }
}

fn init_boundary_crossing_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
) {
    let layout = render_device.create_bind_group_layout(
        "boundary_crossing_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<BoundaryCrossingSettings>(true),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());
    let fragment_shader = asset_server.load(SHADER_ASSET_PATH);

    commands.insert_resource(BoundaryCrossingPipeline {
        layout,
        sampler,
        fullscreen_shader: fullscreen_shader.clone(),
        fragment_shader,
    });
}

/// System to prepare specialized pipelines for each view based on HDR setting
fn prepare_boundary_crossing_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<BoundaryCrossingPipeline>>,
    boundary_pipeline: Res<BoundaryCrossingPipeline>,
    views: Query<(Entity, &ExtractedView), With<BoundaryCrossingSettings>>,
) {
    for (entity, view) in &views {
        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &boundary_pipeline,
            BoundaryCrossingPipelineKey {
                texture_format: if view.hdr {
                    ViewTarget::TEXTURE_FORMAT_HDR
                } else {
                    TextureFormat::bevy_default()
                },
            },
        );

        commands
            .entity(entity)
            .insert(ViewBoundaryCrossingPipeline { pipeline_id });
    }
}
