//! GPU-based simulation plugin.
//!
//! This module provides the GPU simulation scene that uses WebGPU for rendering
//! instead of Bevy's standard rendering system. Cell rendering is handled by
//! WebGpuRenderer, but we spawn a Camera3d to keep Bevy's Core3d render graph
//! active (required for ImGui overlay).

use bevy::prelude::*;
use crate::ui::camera::MainCamera;

/// GPU-based simulation plugin
/// Target: 100K cells at >50tps
pub struct GpuSimPlugin;

impl Plugin for GpuSimPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GpuSceneState>()
            .add_systems(OnEnter(GpuSceneState::Active), setup_gpu_scene)
            .add_systems(OnExit(GpuSceneState::Active), cleanup_gpu_scene);
    }
}

/// State for GPU simulation scene
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GpuSceneState {
    #[default]
    Inactive,
    Active,
}

/// Marker component for GPU scene entities
#[derive(Component)]
pub struct GpuSceneEntity;

/// Setup the GPU simulation scene.
///
/// Spawns a Camera3d with MainCamera to:
/// 1. Keep Bevy's Core3d render graph active (required for ImGui overlay)
/// 2. Enable existing camera controls (WASD, mouse, scroll)
///
/// Cell rendering is handled by WebGpuRenderer, not Bevy's standard pipeline.
/// No Mesh3d, lights, or other Bevy rendering components are spawned.
fn setup_gpu_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Spawn camera to keep Core3d render graph active for ImGui
    // WebGPU pass clears with its own color, Bevy camera doesn't need to clear
    commands.spawn((
        Camera {
            // Don't clear - WebGPU render pass handles background
            clear_color: Color::NONE.into(),
            ..default()
        },
        Camera3d::default(),
        MainCamera {
            center: Vec3::ZERO, // Orbit around world origin
            distance: 50.0, // Start with some distance from origin
            rotation: Quat::from_rotation_x(-0.5) * Quat::from_rotation_y(0.5),
            mode: crate::ui::camera::CameraMode::Orbit,
            followed_entity: None,
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 50.0)),
        GpuSceneEntity,
    ));

    // Spawn orbit reference ball
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.5, 0.8, 1.0, 0.0), // Start invisible
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO),
        crate::ui::camera::OrbitReferenceBall,
        GpuSceneEntity,
    ));
    
    info!("GPU scene activated - WebGPU rendering enabled");
}

/// Cleanup GPU scene entities
fn cleanup_gpu_scene(
    mut commands: Commands,
    query: Query<Entity, With<GpuSceneEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    info!("GPU scene deactivated");
}
