use bevy::prelude::*;
use crate::cell::{Cell, CellPosition, CellOrientation, CellSignaling};
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

/// Setup the GPU simulation scene with camera and single cell at origin
fn setup_gpu_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn 3D camera
    commands.spawn((
        Camera3d::default(),
        MainCamera{
            center: Vec3::ZERO,
            distance: 10.0,
            rotation: Quat::IDENTITY,
        },
        GpuSceneEntity,
    ));

    // Spawn a single cell at the origin
    let cell_radius = 1.0;
    commands.spawn((
        // Cell data components
        Cell {
            mass: 1.0,
            radius: cell_radius,
            genome_id: 0,
            mode_index: 0,
        },
        CellPosition {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
        },
        CellOrientation::default(),
        CellSignaling::default(),
        // Visual representation
        Mesh3d(meshes.add(Sphere::new(cell_radius).mesh().ico(5).unwrap())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.3, 0.5),
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO),
        Visibility::default(),
        GpuSceneEntity,
    ));

    // Add basic lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
        GpuSceneEntity,
    ));

    // Add ambient light as an entity
    commands.spawn((
        AmbientLight {
            color: Color::WHITE,
            brightness: 500.0,
            ..default()
        },
        GpuSceneEntity,
    ));
}

/// Cleanup GPU scene entities
fn cleanup_gpu_scene(
    mut commands: Commands,
    query: Query<Entity, With<GpuSceneEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
