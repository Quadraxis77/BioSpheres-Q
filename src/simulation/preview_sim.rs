use bevy::prelude::*;
use crate::cell::{Cell, CellPosition, CellOrientation, CellSignaling};
use crate::genome::CurrentGenome;
use crate::ui::camera::MainCamera;
use crate::simulation::cpu_physics::CanonicalState;
use crate::simulation::initial_state::InitialState;
use crate::simulation::PhysicsConfig;

/// Preview simulation plugin for genome testing
/// Uses deterministic replay from time 0 with canonical physics
pub struct PreviewSimPlugin;

impl Plugin for PreviewSimPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PreviewSimState>()
            .init_resource::<PreviewRequest>()
            .add_systems(OnEnter(PreviewSceneState::Active), setup_preview_scene)
            .add_systems(OnExit(PreviewSceneState::Active), cleanup_preview_scene)
            .add_systems(
                Update,
                (
                    run_preview_resimulation,
                    respawn_preview_cells_after_resimulation,
                )
                    .chain()
                    .run_if(in_state(PreviewSceneState::Active))
                    .run_if(|state: Res<crate::simulation::SimulationState>| {
                        state.mode == crate::simulation::SimulationMode::Preview
                    }),
            )
            .add_systems(
                Update,
                (
                    sync_preview_visuals,
                    crate::rendering::sync_transforms,
                )
                    .chain()
                    .after(crate::input::CellDraggingSet)
                    .run_if(in_state(PreviewSceneState::Active))
                    .run_if(|state: Res<crate::simulation::SimulationState>| {
                        state.mode == crate::simulation::SimulationMode::Preview
                    }),
            )
            .init_state::<PreviewSceneState>();
    }
}

/// State for Preview simulation scene
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PreviewSceneState {
    #[default]
    Inactive,
    Active,
}

/// Marker component for Preview scene entities
#[derive(Component)]
pub struct PreviewSceneEntity;

/// Preview simulation state resource
#[derive(Resource)]
pub struct PreviewSimState {
    /// Canonical state for preview (separate from main)
    pub canonical_state: CanonicalState,
    
    /// Initial state (shared with main)
    pub initial_state: InitialState,
    
    /// Current preview time
    pub current_time: f32,
    
    /// Mapping from cell index to ECS entity (1D array for cache efficiency)
    /// Index matches canonical_state cell indices
    pub index_to_entity: Vec<Option<Entity>>,
}

impl Default for PreviewSimState {
    fn default() -> Self {
        // Preview simulation uses a fixed capacity of 256 cells for optimal performance
        let initial_state = InitialState::new(PhysicsConfig::default(), 256, 0);
        let canonical_state = initial_state.to_canonical_state();
        
        Self {
            canonical_state,
            initial_state,
            current_time: 0.0,
            index_to_entity: vec![None; 256],
        }
    }
}

/// Preview request resource
#[derive(Resource, Default)]
pub struct PreviewRequest {
    /// Target time for preview (None = no pending request)
    pub target_time: Option<f32>,
    
    /// Whether genome changed (requires full resimulation)
    pub genome_changed: bool,
}

/// Setup the Preview simulation scene with camera and initial state
fn setup_preview_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut preview_state: ResMut<PreviewSimState>,
    genome: Res<CurrentGenome>,
    config: Res<PhysicsConfig>,
) {
    // Spawn camera
    commands.spawn((
        Camera3d::default(),
        MainCamera{
            center: Vec3::new(0.0, 0.0, 10.0), // Orbit center offset from world origin
            distance: 0.0, // No orbit offset (camera at orbit center)
            rotation: Quat::IDENTITY,
        },
        PreviewSceneEntity,
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
        PreviewSceneEntity,
    ));

    // Spawn lights
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
        PreviewSceneEntity,
    ));

    commands.spawn((
        AmbientLight {
            color: Color::WHITE,
            brightness: 500.0,
            ..default()
        },
        PreviewSceneEntity,
    ));
    
    // Initialize preview state with single cell at origin
    let initial_mode_index = genome.genome.initial_mode.max(0) as usize;
    let mode = genome.genome.modes.get(initial_mode_index)
        .or_else(|| genome.genome.modes.first());
    
    let (split_mass, split_interval) = if let Some(mode) = mode {
        (mode.split_mass, mode.split_interval)
    } else {
        (1.0, 5.0)
    };
    
    let cell_radius = 1.0;
    let stiffness = 10.0;
    
    // Create initial state with preview-specific capacity limit (256 cells)
    // Preview simulation is optimized for low cell counts with real-time genome updates
    let mut initial_state = InitialState::new(
        config.clone(),
        256, // Preview capacity limit
        0, // RNG seed
    );
    
    initial_state.add_cell(crate::simulation::InitialCell {
        id: 0,
        position: Vec3::ZERO,
        velocity: Vec3::ZERO,
        rotation: genome.genome.initial_orientation,
        angular_velocity: Vec3::ZERO,
        mass: split_mass,
        radius: cell_radius,
        genome_id: 0,
        mode_index: initial_mode_index,
        birth_time: 0.0,
        split_interval,
        stiffness,
    });
    
    // Convert to canonical state
    let canonical_state = initial_state.to_canonical_state();
    
    // Update preview state
    preview_state.initial_state = initial_state;
    preview_state.canonical_state = canonical_state;
    preview_state.current_time = 0.0;
    preview_state.index_to_entity.clear();
    preview_state.index_to_entity.resize(256, None);
    
    // Spawn ECS entity for the initial cell
    let mode = genome.genome.modes.get(initial_mode_index)
        .or_else(|| genome.genome.modes.first());
    let color = if let Some(mode) = mode {
        mode.color
    } else {
        Vec3::ONE
    };
    
    // Check if this is a flagellocyte
    let is_flagellocyte = mode.map(|m| m.cell_type == 1).unwrap_or(false);
    let swim_force = mode.map(|m| m.swim_force).unwrap_or(0.0);
    
    // Choose mesh based on cell type
    let cell_mesh = if is_flagellocyte {
        meshes.add(crate::rendering::flagellocyte_mesh::generate_flagellocyte_mesh(cell_radius, swim_force, 5))
    } else {
        meshes.add(Sphere::new(cell_radius).mesh().ico(5).unwrap())
    };
    
    let entity = commands.spawn((
        Cell {
            mass: split_mass,
            radius: cell_radius,
            genome_id: 0,
            mode_index: initial_mode_index,
        },
        CellPosition {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
        },
        CellOrientation {
            rotation: genome.genome.initial_orientation,
            angular_velocity: Vec3::ZERO,
        },
        CellSignaling::default(),
        crate::cell::division::DivisionTimer {
            birth_time: 0.0,
            split_interval,
        },
        crate::cell::physics::CellForces::default(),
        crate::cell::physics::Cytoskeleton {
            stiffness,
        },
        Mesh3d(cell_mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(color.x, color.y, color.z),
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO)
            .with_rotation(genome.genome.initial_orientation),
        Visibility::default(),
        PreviewSceneEntity,
    )).id();
    
    // Map cell index to entity
    preview_state.index_to_entity[0] = Some(entity);
}

/// Cleanup Preview scene entities
fn cleanup_preview_scene(
    mut commands: Commands,
    query: Query<Entity, With<PreviewSceneEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Run preview re-simulation using canonical physics
/// This replaces the old headless simulation with deterministic replay
fn run_preview_resimulation(
    mut preview_state: ResMut<PreviewSimState>,
    mut sim_state: ResMut<crate::simulation::SimulationState>,
    config: Res<PhysicsConfig>,
    genome: Res<CurrentGenome>,
) {
    let Some(target_time) = sim_state.target_time else {
        sim_state.is_resimulating = false;
        return;
    };

    sim_state.is_resimulating = true;

    let start_time = std::time::Instant::now();

    // Update initial state with current genome values before any simulation
    // This ensures that when genome changes trigger resimulation, the initial cell
    // has the updated split_interval and other genome properties
    if let Some(initial_cell) = preview_state.initial_state.initial_cells.first_mut() {
        let initial_mode_index = genome.genome.initial_mode.max(0) as usize;
        let mode = genome.genome.modes.get(initial_mode_index)
            .or_else(|| genome.genome.modes.first());

        if let Some(mode) = mode {
            initial_cell.split_interval = mode.split_interval;
            initial_cell.mode_index = initial_mode_index;
            initial_cell.rotation = genome.genome.initial_orientation;
        }
    }

    // Determine if we can simulate forward incrementally
    let (start_step, steps) = if target_time > preview_state.current_time {
        // Moving forward: simulate only the additional steps
        let start_step = (preview_state.current_time / config.fixed_timestep).ceil() as u32;
        let end_step = (target_time / config.fixed_timestep).ceil() as u32;
        (start_step, end_step - start_step)
    } else {
        // Moving backward or genome changed (target_time <= current_time):
        // reset to initial state and simulate from beginning
        preview_state.canonical_state = preview_state.initial_state.to_canonical_state();
        preview_state.current_time = 0.0;
        (0, (target_time / config.fixed_timestep).ceil() as u32)
    };

    // Extract values we need before the loop
    let max_cells = preview_state.initial_state.max_cells;
    let rng_seed = preview_state.initial_state.rng_seed;

    // Run physics steps (no ECS overhead)
    let mut total_physics_time = std::time::Duration::ZERO;
    let mut total_division_time = std::time::Duration::ZERO;

    for step in 0..steps {
        let current_time = (start_step + step) as f32 * config.fixed_timestep;

        // Run CPU physics step (single-threaded for preview)
        let physics_start = std::time::Instant::now();
        crate::simulation::cpu_physics::physics_step_st_with_genome(
            &mut preview_state.canonical_state, 
            &config,
            &genome.genome,
        );
        total_physics_time += physics_start.elapsed();

        // Run division step using CPU physics
        let division_start = std::time::Instant::now();
        crate::simulation::cpu_physics::division_step(
            &mut preview_state.canonical_state,
            &genome.genome,
            current_time,
            max_cells,
            rng_seed,
        );
        total_division_time += division_start.elapsed();
    }

    let _sim_duration = start_time.elapsed();

    preview_state.current_time = target_time;
    sim_state.target_time = None;
    sim_state.is_resimulating = false;
    sim_state.needs_respawn = true;
}

/// Respawn all preview cells after resimulation completes
/// This runs after run_preview_resimulation and rebuilds the ECS entities
/// 
/// NOTE: This is a temporary implementation that spawns individual entities.
/// For proper performance with many cells, this should be replaced with
/// instanced rendering using Bevy's WGPU rendering pipeline.
fn respawn_preview_cells_after_resimulation(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut preview_state: ResMut<PreviewSimState>,
    mut sim_state: ResMut<crate::simulation::SimulationState>,
    genome: Res<CurrentGenome>,
    cells_query: Query<Entity, (With<Cell>, With<PreviewSceneEntity>)>,
    drag_state: Res<crate::input::DragState>,
) {
    // Don't respawn if currently dragging
    if drag_state.dragged_entity.is_some() {
        return;
    }
    
    // Only respawn if explicitly flagged
    if !sim_state.needs_respawn {
        return;
    }
    
    {
        // Despawn all existing cell entities
        for entity in cells_query.iter() {
            commands.entity(entity).despawn();
        }
        
        // Clear entity mapping
        for slot in preview_state.index_to_entity.iter_mut() {
            *slot = None;
        }
        
        // Pre-create shared mesh (reuse for all cells)
        let sphere_mesh = meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap());
        
        // Pre-create materials for each unique color (cache by mode as 1D array)
        let max_modes = genome.genome.modes.len();
        let mut material_cache: Vec<Option<Handle<StandardMaterial>>> = vec![None; max_modes];
        
        // Batch spawn all cells
        for i in 0..preview_state.canonical_state.cell_count {
            let mode_index = preview_state.canonical_state.mode_indices[i];
            let mass = preview_state.canonical_state.masses[i];
            let radius = preview_state.canonical_state.radii[i];
            let genome_id = preview_state.canonical_state.genome_ids[i];
            let position = preview_state.canonical_state.positions[i];
            let velocity = preview_state.canonical_state.velocities[i];
            let rotation = preview_state.canonical_state.rotations[i];
            let angular_velocity = preview_state.canonical_state.angular_velocities[i];
            let birth_time = preview_state.canonical_state.birth_times[i];
            let split_interval = preview_state.canonical_state.split_intervals[i];
            let stiffness = preview_state.canonical_state.stiffnesses[i];
            
            // Get mode to check cell type and swim force
            let mode = genome.genome.modes.get(mode_index);
            let is_flagellocyte = mode.map(|m| m.cell_type == 1).unwrap_or(false);
            let swim_force = mode.map(|m| m.swim_force).unwrap_or(0.0);
            
            // Get or create material for this mode
            let material = if mode_index < material_cache.len() {
                if let Some(mat) = &material_cache[mode_index] {
                    mat.clone()
                } else {
                    let color = if let Some(mode) = mode {
                        mode.color
                    } else {
                        Vec3::ONE
                    };
                    let mat = materials.add(StandardMaterial {
                        base_color: Color::srgb(color.x, color.y, color.z),
                        ..default()
                    });
                    material_cache[mode_index] = Some(mat.clone());
                    mat
                }
            } else {
                // Fallback for invalid mode index
                materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 1.0, 1.0),
                    ..default()
                })
            };
            
            // Choose mesh based on cell type
            let cell_mesh = if is_flagellocyte {
                meshes.add(crate::rendering::flagellocyte_mesh::generate_flagellocyte_mesh(1.0, swim_force, 5))
            } else {
                sphere_mesh.clone()
            };
            
            let entity = commands.spawn((
                Cell {
                    mass,
                    radius,
                    genome_id,
                    mode_index,
                },
                CellPosition {
                    position,
                    velocity,
                },
                CellOrientation {
                    rotation,
                    angular_velocity,
                },
                CellSignaling::default(),
                crate::cell::division::DivisionTimer {
                    birth_time,
                    split_interval,
                },
                crate::cell::physics::CellForces::default(),
                crate::cell::physics::Cytoskeleton {
                    stiffness,
                },
                Mesh3d(cell_mesh),
                MeshMaterial3d(material),
                Transform::from_translation(position)
                    .with_rotation(rotation)
                    .with_scale(Vec3::splat(radius)),
                Visibility::default(),
                PreviewSceneEntity,
            )).id();
            
            preview_state.index_to_entity[i] = Some(entity);
        }
        
        // Clear the respawn flag
        sim_state.needs_respawn = false;
    }
}

/// Sync preview visuals from canonical state
/// This updates existing ECS entities to match the canonical state without respawning
fn sync_preview_visuals(
    preview_state: Res<PreviewSimState>,
    mut cells_query: Query<(Entity, &mut CellPosition, &mut CellOrientation), (With<Cell>, With<PreviewSceneEntity>)>,
    drag_state: Res<crate::input::DragState>,
) {
    // Skip sync entirely if currently dragging a cell
    if drag_state.dragged_entity.is_some() {
        return;
    }
    
    // Sync positions and orientations from canonical state to existing entities
    // Only update CellPosition and CellOrientation - Transform will be synced by sync_transforms system
    for i in 0..preview_state.canonical_state.cell_count {
        if let Some(entity) = preview_state.index_to_entity[i] {
            // Skip the dragged entity
            if Some(entity) == drag_state.dragged_entity {
                continue;
            }
            
            // Update entity components directly using index
            if let Ok((_, mut cell_pos, mut cell_orientation)) = cells_query.get_mut(entity) {
                cell_pos.position = preview_state.canonical_state.positions[i];
                cell_pos.velocity = preview_state.canonical_state.velocities[i];
                cell_orientation.rotation = preview_state.canonical_state.rotations[i];
                cell_orientation.angular_velocity = preview_state.canonical_state.angular_velocities[i];
            }
        }
    }
}

