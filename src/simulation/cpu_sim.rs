use bevy::prelude::*;
use bevy::light::NotShadowCaster;
use crate::cell::{Cell, CellPosition, CellOrientation, CellSignaling};
use crate::ui::camera::MainCamera;
use crate::simulation::{CanonicalState, InitialState, InitialCell};
use crate::simulation::PhysicsConfig;
use std::collections::HashMap;

/// CPU-based simulation plugin
/// Target: 4K cells at >50tps
/// 
/// Uses multithreaded physics via Rayon for improved performance:
/// - Parallel Verlet integration (position and velocity updates)
/// - Parallel collision detection across spatial grid cells
/// - Parallel force computation with deterministic accumulation
/// - Parallel boundary condition application
/// 
/// All parallel operations maintain deterministic ordering to ensure
/// bit-identical results across runs.
pub struct CpuSimPlugin;

/// Fixed timestep simulation plugin for CPU mode
pub struct CpuSimTimestepPlugin;

impl Plugin for CpuSimPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CpuSimTimestepPlugin)
            .init_resource::<MainSimState>()
            .add_systems(OnEnter(CpuSceneState::Active), (setup_cpu_scene, spawn_cpu_skybox))
            .add_systems(OnExit(CpuSceneState::Active), cleanup_cpu_scene)
            .init_state::<CpuSceneState>();
    }
}

impl Plugin for CpuSimTimestepPlugin {
    fn build(&self, app: &mut App) {
        // Use Bevy's default fixed timestep: 64 Hz
        // Note: Time<Fixed> defaults to 64 Hz, so we don't need to set it explicitly
        app
            // Add physics systems to FixedUpdate schedule (runs at fixed rate)
            .add_systems(
                FixedUpdate,
                (
                    // Update fixed timestep based on speed multiplier
                    update_fixed_timestep,
                    // Run canonical physics step
                    run_main_simulation,
                )
                    .chain()
                    .run_if(in_state(CpuSceneState::Active))
                    .run_if(|state: Res<crate::simulation::SimulationState>| {
                        state.mode == crate::simulation::SimulationMode::Cpu && !state.paused
                    }),
            )
            // Add rendering/UI systems to Update schedule (runs every frame)
            .add_systems(
                Update,
                (
                    sync_ecs_from_canonical,
                    crate::cell::physics::sync_transforms,
                )
                    .chain()
                    .run_if(in_state(CpuSceneState::Active))
                    .run_if(|state: Res<crate::simulation::SimulationState>| {
                        state.mode == crate::simulation::SimulationMode::Cpu
                    }),
            );
    }
}

/// Main simulation state resource
/// Contains the canonical state and mapping from cell IDs to ECS entities
#[derive(Resource)]
pub struct MainSimState {
    /// Canonical state for main simulation
    pub canonical_state: CanonicalState,
    
    /// Initial state for potential reset
    pub initial_state: InitialState,
    
    /// Mapping from cell ID to ECS entity
    pub id_to_entity: HashMap<u32, Entity>,
    
    /// Mapping from ECS entity to cell index in canonical state
    pub entity_to_index: HashMap<Entity, usize>,
    
    /// OPTIMIZATION: Direct array mapping from cell index to entity
    /// This avoids HashMap lookups in the hot sync path
    pub index_to_entity: Vec<Option<Entity>>,
    
    /// OPTIMIZATION: Entity pool for reusing despawned entities
    /// Instead of despawning and spawning, we hide/show and update components
    pub entity_pool: Vec<Entity>,
    
    /// OPTIMIZATION: Cached sphere mesh handle (reused for all cells)
    /// Creating meshes is VERY expensive - reuse the same mesh for all cells
    pub sphere_mesh: Handle<Mesh>,
    
    /// OPTIMIZATION: Material cache by color
    /// Creating materials is expensive - cache and reuse by color, opacity, and emissive
    /// Key is (r, g, b, a, emissive) as u8 values for fast lookup
    pub material_cache: HashMap<(u8, u8, u8, u8, u8), Handle<StandardMaterial>>,
    
    /// Simulation time (advances based on speed multiplier)
    pub simulation_time: f32,
}

impl Default for MainSimState {
    fn default() -> Self {
        // Create a minimal initial state with smaller spatial grid
        let config = PhysicsConfig::default();
        let initial_state = InitialState::new(config, 4_096, 0);
        
        // Use 64x64x64 spatial grid for optimal performance with large cell counts
        let mut canonical_state = initial_state.to_canonical_state();
        canonical_state.spatial_grid = crate::simulation::DeterministicSpatialGrid::new(64, 200.0, 100.0);
        
        let capacity = canonical_state.capacity;
        
        Self {
            canonical_state,
            initial_state,
            id_to_entity: HashMap::new(),
            entity_to_index: HashMap::new(),
            index_to_entity: vec![None; capacity],
            entity_pool: Vec::with_capacity(capacity),
            sphere_mesh: Handle::default(), // Will be initialized in setup
            material_cache: HashMap::new(),
            simulation_time: 0.0,
        }
    }
}

/// Update the fixed timestep based on simulation speed multiplier
fn update_fixed_timestep(
    sim_state: Res<crate::simulation::SimulationState>,
    mut time: ResMut<Time<Fixed>>,
) {
    // Base timestep is 64 Hz (1/64 seconds)
    let base_timestep = 1.0 / 64.0;
    
    // Clamp speed multiplier between 0.1x and 10x
    let speed = sim_state.speed_multiplier.clamp(0.1, 10.0);
    
    // Calculate new timestep (higher speed = SMALLER timestep = more ticks per frame)
    // At 1x: 1/64 seconds per tick (64 Hz)
    // At 10x: 1/640 seconds per tick (640 Hz)
    let new_timestep = base_timestep / speed;
    
    // Update the fixed timestep
    let current_timestep = time.timestep().as_secs_f32();
    if (current_timestep - new_timestep).abs() > 0.0001 {
        time.set_timestep(std::time::Duration::from_secs_f32(new_timestep));
    }
}

/// Run main simulation physics step using canonical physics
fn run_main_simulation(
    mut main_state: ResMut<MainSimState>,
    config: Res<PhysicsConfig>,
    genome: Res<crate::genome::CurrentGenome>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    threading_config: Res<crate::simulation::SimulationThreadingConfig>,
    mut gpu_physics: ResMut<crate::simulation::GpuPhysicsResource>,
) {
    // Early return if no cells (scene not initialized yet)
    if main_state.canonical_state.cell_count == 0 {
        return;
    }
    
    // Run canonical physics step with genome-aware adhesion settings
    let current_time = main_state.simulation_time;
    
    // Choose physics implementation based on configuration
    if threading_config.gpu_physics_enabled && gpu_physics.enabled {
        crate::simulation::gpu_physics::physics_step_gpu_with_genome(
            &mut main_state.canonical_state,
            &config,
            &genome.genome,
            &mut gpu_physics,
            current_time,
            true, // Enable swim in main simulation mode
        );
    } else {
        crate::simulation::cpu_physics::physics_step_with_genome(
            &mut main_state.canonical_state,
            &config,
            &genome.genome,
            current_time,
            true, // Enable swim in main simulation mode
        );
    }
    
    // Advance simulation time by the base timestep (64 Hz)
    // The speed multiplier is already baked into the fixed timestep rate
    // At 1x: timestep = 1/64, runs 64 times per second
    // At 10x: timestep = 1/640, runs 640 times per second
    // Both advance simulation time by 1/64 per tick
    main_state.simulation_time += 1.0 / 64.0;
    
    // Get current simulation time before borrowing main_state mutably
    let current_sim_time = main_state.simulation_time;
    
    // Handle divisions using simulation time
    handle_divisions(
        &mut main_state,
        &genome,
        current_sim_time,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
}

/// Convert color, opacity, and emissive to cache key
#[inline]
fn material_cache_key(color: Vec3, opacity: f32, emissive: f32) -> (u8, u8, u8, u8, u8) {
    (
        (color.x * 255.0) as u8,
        (color.y * 255.0) as u8,
        (color.z * 255.0) as u8,
        (opacity * 255.0) as u8,
        (emissive.clamp(0.0, 10.0) * 25.0) as u8, // Scale emissive to 0-250 range for 0-10 values
    )
}

/// Get or create a cached material for a given color, opacity, and emissive
/// OPTIMIZATION: Reuses materials to enable GPU instancing
fn get_or_create_material(
    color: Vec3,
    opacity: f32,
    emissive: f32,
    material_cache: &mut HashMap<(u8, u8, u8, u8, u8), Handle<StandardMaterial>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Handle<StandardMaterial> {
    let key = material_cache_key(color, opacity, emissive);
    
    material_cache.entry(key).or_insert_with(|| {
        materials.add(StandardMaterial {
            base_color: Color::srgba(color.x, color.y, color.z, opacity),
            emissive: LinearRgba::rgb(color.x * emissive, color.y * emissive, color.z * emissive),
            cull_mode: Some(bevy::render::render_resource::Face::Back),
            alpha_mode: if opacity < 0.99 {
                // Use AlphaToCoverage with MSAA for order-independent transparency
                // Eliminates view-dependent sorting artifacts without requiring OIT
                bevy::prelude::AlphaMode::AlphaToCoverage
            } else {
                bevy::prelude::AlphaMode::Opaque
            },
            ..default()
        })
    }).clone()
}

/// Handle cell divisions in canonical state using the canonical division_step function
fn handle_divisions(
    main_state: &mut MainSimState,
    genome: &crate::genome::CurrentGenome,
    current_time: f32,
    commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Early exit if at capacity
    // Use the actual max_cells from initial state (respects what was configured)
    let max_cells = main_state.initial_state.max_cells;
    if main_state.canonical_state.cell_count >= max_cells {
        return;
    }

    // Create a mapping from cell index to cell ID BEFORE division
    // This allows us to look up which entities to despawn after division
    let index_to_cell_id: Vec<u32> = main_state.canonical_state.cell_ids[..main_state.canonical_state.cell_count].to_vec();

    // Run canonical division step with cell limit
    // This will return events for cells that actually divided
    let rng_seed = main_state.initial_state.rng_seed;
    let division_events = crate::simulation::cpu_physics::division_step(
        &mut main_state.canonical_state,
        &genome.genome,
        current_time,
        max_cells,
        rng_seed,
    );

    // Collect parent cell IDs that ACTUALLY divided (from division events)
    // Important: Only despawn cells that successfully divided, not cells that
    // wanted to divide but couldn't due to capacity constraints
    let mut parent_cell_ids = Vec::new();
    for event in &division_events {
        // parent_idx refers to the index BEFORE division/compaction
        // Look up the cell_id from our saved mapping
        if event.parent_idx < index_to_cell_id.len() {
            parent_cell_ids.push(index_to_cell_id[event.parent_idx]);
        }
    }

    // Return parent entities to pool (instead of despawning)
    for parent_cell_id in parent_cell_ids {
        if let Some(parent_entity) = main_state.id_to_entity.remove(&parent_cell_id) {
            if let Some(idx) = main_state.entity_to_index.remove(&parent_entity) {
                main_state.index_to_entity[idx] = None;
            }
            // Hide entity and return to pool
            commands.entity(parent_entity).insert(Visibility::Hidden);
            main_state.entity_pool.push(parent_entity);
        }
    }
    
    // Batch spawn child entities for better performance
    // Pre-allocate materials to avoid repeated lookups
    let mut materials_needed: std::collections::HashMap<(u8, u8, u8, u8, u8), Handle<StandardMaterial>> = std::collections::HashMap::new();
    
    for event in &division_events {
        let child_a_idx = event.child_a_idx;
        let child_b_idx = event.child_b_idx;
        
        let child_a_mode_idx = main_state.canonical_state.mode_indices[child_a_idx];
        let child_b_mode_idx = main_state.canonical_state.mode_indices[child_b_idx];
        
        let child_a_mode = genome.genome.modes.get(child_a_mode_idx);
        let child_b_mode = genome.genome.modes.get(child_b_mode_idx);
        
        let child_a_color = child_a_mode.map(|m| m.color).unwrap_or(Vec3::ONE);
        let child_a_opacity = child_a_mode.map(|m| m.opacity).unwrap_or(1.0);
        let child_a_emissive = child_a_mode.map(|m| m.emissive).unwrap_or(0.0);
        let child_b_color = child_b_mode.map(|m| m.color).unwrap_or(Vec3::ONE);
        let child_b_opacity = child_b_mode.map(|m| m.opacity).unwrap_or(1.0);
        let child_b_emissive = child_b_mode.map(|m| m.emissive).unwrap_or(0.0);
        
        // Pre-fetch materials
        let key_a = material_cache_key(child_a_color, child_a_opacity, child_a_emissive);
        if !materials_needed.contains_key(&key_a) {
            let mat = get_or_create_material(child_a_color, child_a_opacity, child_a_emissive, &mut main_state.material_cache, materials);
            materials_needed.insert(key_a, mat);
        }
        let key_b = material_cache_key(child_b_color, child_b_opacity, child_b_emissive);
        if !materials_needed.contains_key(&key_b) {
            let mat = get_or_create_material(child_b_color, child_b_opacity, child_b_emissive, &mut main_state.material_cache, materials);
            materials_needed.insert(key_b, mat);
        }
    }
    
    // Batch spawn all children
    let sphere_mesh = main_state.sphere_mesh.clone();
    
    for event in division_events {
        let child_a_idx = event.child_a_idx;
        let child_b_idx = event.child_b_idx;
        
        // Batch read child A properties
        let (child_a_pos, child_a_vel, child_a_rotation, child_a_mass, child_a_radius, 
             child_a_mode_idx, child_a_split_interval, child_a_birth_time, cell_id_a) = (
            main_state.canonical_state.positions[child_a_idx],
            main_state.canonical_state.velocities[child_a_idx],
            main_state.canonical_state.rotations[child_a_idx],
            main_state.canonical_state.masses[child_a_idx],
            main_state.canonical_state.radii[child_a_idx],
            main_state.canonical_state.mode_indices[child_a_idx],
            main_state.canonical_state.split_intervals[child_a_idx],
            main_state.canonical_state.birth_times[child_a_idx],
            main_state.canonical_state.cell_ids[child_a_idx],
        );
        
        // Batch read child B properties
        let (child_b_pos, child_b_vel, child_b_rotation, child_b_mass, child_b_radius,
             child_b_mode_idx, child_b_split_interval, child_b_birth_time, cell_id_b) = (
            main_state.canonical_state.positions[child_b_idx],
            main_state.canonical_state.velocities[child_b_idx],
            main_state.canonical_state.rotations[child_b_idx],
            main_state.canonical_state.masses[child_b_idx],
            main_state.canonical_state.radii[child_b_idx],
            main_state.canonical_state.mode_indices[child_b_idx],
            main_state.canonical_state.split_intervals[child_b_idx],
            main_state.canonical_state.birth_times[child_b_idx],
            main_state.canonical_state.cell_ids[child_b_idx],
        );
        
        let child_a_mode = genome.genome.modes.get(child_a_mode_idx);
        let child_b_mode = genome.genome.modes.get(child_b_mode_idx);
        
        let child_a_color = child_a_mode.map(|m| m.color).unwrap_or(Vec3::ONE);
        let child_a_opacity = child_a_mode.map(|m| m.opacity).unwrap_or(1.0);
        let child_a_emissive = child_a_mode.map(|m| m.emissive).unwrap_or(0.0);
        let child_b_color = child_b_mode.map(|m| m.color).unwrap_or(Vec3::ONE);
        let child_b_opacity = child_b_mode.map(|m| m.opacity).unwrap_or(1.0);
        let child_b_emissive = child_b_mode.map(|m| m.emissive).unwrap_or(0.0);
        
        let material_a = materials_needed[&material_cache_key(child_a_color, child_a_opacity, child_a_emissive)].clone();
        let material_b = materials_needed[&material_cache_key(child_b_color, child_b_opacity, child_b_emissive)].clone();
        
        // Check if cells are flagellocytes and create appropriate meshes
        let is_flagellocyte_a = child_a_mode.map(|m| m.cell_type == 1).unwrap_or(false);
        let swim_force_a = child_a_mode.map(|m| m.swim_force).unwrap_or(0.0);
        let mesh_a = if is_flagellocyte_a {
            _meshes.add(crate::rendering::flagellocyte_mesh::generate_flagellocyte_mesh(1.0, swim_force_a, 5))
        } else {
            sphere_mesh.clone()
        };
        
        // Get or spawn child A entity (reuse from pool if available)
        let entity_a = if let Some(pooled_entity) = main_state.entity_pool.pop() {
            // Reuse pooled entity - just update components
            commands.entity(pooled_entity).insert((
            Cell { mass: child_a_mass, radius: child_a_radius, genome_id: 0, mode_index: child_a_mode_idx, cell_type: child_a_mode.map(|m| m.cell_type).unwrap_or(0) },
            CellPosition { position: child_a_pos, velocity: child_a_vel },
            CellOrientation { rotation: child_a_rotation, angular_velocity: Vec3::ZERO },
            CellSignaling::default(),
            crate::cell::division::DivisionTimer { birth_time: child_a_birth_time, split_interval: child_a_split_interval },
            crate::cell::physics::CellForces::default(),
            crate::cell::physics::Cytoskeleton::default(),
            Mesh3d(mesh_a.clone()),
            MeshMaterial3d(material_a),
            Transform::from_translation(child_a_pos).with_rotation(child_a_rotation).with_scale(Vec3::splat(child_a_radius)),
            Visibility::Visible,
            ));
            pooled_entity
        } else {
            // No pooled entity available - spawn new one
            commands.spawn((
                Cell { mass: child_a_mass, radius: child_a_radius, genome_id: 0, mode_index: child_a_mode_idx, cell_type: child_a_mode.map(|m| m.cell_type).unwrap_or(0) },
                CellPosition { position: child_a_pos, velocity: child_a_vel },
                CellOrientation { rotation: child_a_rotation, angular_velocity: Vec3::ZERO },
                CellSignaling::default(),
                crate::cell::division::DivisionTimer { birth_time: child_a_birth_time, split_interval: child_a_split_interval },
                crate::cell::physics::CellForces::default(),
                crate::cell::physics::Cytoskeleton::default(),
                Mesh3d(mesh_a.clone()),
                MeshMaterial3d(material_a.clone()),
                Transform::from_translation(child_a_pos).with_rotation(child_a_rotation).with_scale(Vec3::splat(child_a_radius)),
                Visibility::Visible,
                CpuSceneEntity,
            )).id()
        };
        
        main_state.id_to_entity.insert(cell_id_a, entity_a);
        main_state.entity_to_index.insert(entity_a, child_a_idx);
        main_state.index_to_entity[child_a_idx] = Some(entity_a);
        
        // Check if child B is flagellocyte
        let is_flagellocyte_b = child_b_mode.map(|m| m.cell_type == 1).unwrap_or(false);
        let swim_force_b = child_b_mode.map(|m| m.swim_force).unwrap_or(0.0);
        let mesh_b = if is_flagellocyte_b {
            _meshes.add(crate::rendering::flagellocyte_mesh::generate_flagellocyte_mesh(1.0, swim_force_b, 5))
        } else {
            sphere_mesh.clone()
        };
        
        // Get or spawn child B entity (reuse from pool if available)
        let entity_b = if let Some(pooled_entity) = main_state.entity_pool.pop() {
            // Reuse pooled entity - just update components
            commands.entity(pooled_entity).insert((
            Cell { mass: child_b_mass, radius: child_b_radius, genome_id: 0, mode_index: child_b_mode_idx, cell_type: child_b_mode.map(|m| m.cell_type).unwrap_or(0) },
            CellPosition { position: child_b_pos, velocity: child_b_vel },
            CellOrientation { rotation: child_b_rotation, angular_velocity: Vec3::ZERO },
            CellSignaling::default(),
            crate::cell::division::DivisionTimer { birth_time: child_b_birth_time, split_interval: child_b_split_interval },
            crate::cell::physics::CellForces::default(),
            crate::cell::physics::Cytoskeleton::default(),
            Mesh3d(mesh_b.clone()),
            MeshMaterial3d(material_b),
            Transform::from_translation(child_b_pos).with_rotation(child_b_rotation).with_scale(Vec3::splat(child_b_radius)),
            Visibility::Visible,
            ));
            pooled_entity
        } else {
            // No pooled entity available - spawn new one
            commands.spawn((
                Cell { mass: child_b_mass, radius: child_b_radius, genome_id: 0, mode_index: child_b_mode_idx, cell_type: child_b_mode.map(|m| m.cell_type).unwrap_or(0) },
                CellPosition { position: child_b_pos, velocity: child_b_vel },
                CellOrientation { rotation: child_b_rotation, angular_velocity: Vec3::ZERO },
                CellSignaling::default(),
                crate::cell::division::DivisionTimer { birth_time: child_b_birth_time, split_interval: child_b_split_interval },
                crate::cell::physics::CellForces::default(),
                crate::cell::physics::Cytoskeleton::default(),
                Mesh3d(mesh_b.clone()),
                MeshMaterial3d(material_b.clone()),
                Transform::from_translation(child_b_pos).with_rotation(child_b_rotation).with_scale(Vec3::splat(child_b_radius)),
                Visibility::Visible,
                CpuSceneEntity,
            )).id()
        };
        
        main_state.id_to_entity.insert(cell_id_b, entity_b);
        main_state.entity_to_index.insert(entity_b, child_b_idx);
        main_state.index_to_entity[child_b_idx] = Some(entity_b);
    }
    
    // No need to rebuild mappings since we removed compaction
    // Child A reuses parent index, child B gets new index
    // All indices remain stable
}

/// Sync ECS components from canonical state
/// OPTIMIZED: Uses direct array indexing instead of HashMap lookups
fn sync_ecs_from_canonical(
    main_state: Res<MainSimState>,
    drag_state: Res<crate::input::cell_dragging::DragState>,
    mut cells_query: Query<(Entity, &mut CellPosition, &mut CellOrientation, &mut Cell)>,
) {
    // Early return if no cells (scene not initialized yet)
    if main_state.canonical_state.cell_count == 0 {
        return;
    }
    
    // OPTIMIZATION: Direct array access instead of HashMap lookups
    // This is O(1) instead of O(log N) and has much better cache locality
    for i in 0..main_state.canonical_state.cell_count {
        if let Some(entity) = main_state.index_to_entity[i] {
            // Skip syncing if this cell is currently being dragged
            if drag_state.dragged_entity == Some(entity) {
                continue;
            }
            
            if let Ok((_, mut pos, mut orientation, mut cell)) = cells_query.get_mut(entity) {
                // Batch read from canonical state (better cache locality)
                pos.position = main_state.canonical_state.positions[i];
                pos.velocity = main_state.canonical_state.velocities[i];
                orientation.rotation = main_state.canonical_state.rotations[i];
                orientation.angular_velocity = main_state.canonical_state.angular_velocities[i];
                cell.mass = main_state.canonical_state.masses[i];
                cell.radius = main_state.canonical_state.radii[i];
                cell.genome_id = main_state.canonical_state.genome_ids[i];
                cell.mode_index = main_state.canonical_state.mode_indices[i];
            }
        }
    }
}

/// State for CPU simulation scene
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CpuSceneState {
    #[default]
    Inactive,
    Active,
}

/// Marker component for CPU scene entities
#[derive(Component)]
pub struct CpuSceneEntity;

/// Setup the CPU simulation scene with initial state (camera already exists from Preview)
fn setup_cpu_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    _fog_settings: Res<crate::rendering::VolumetricFogSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    genome: Res<crate::genome::CurrentGenome>,
    config: Res<PhysicsConfig>,
    mut main_state: ResMut<MainSimState>,
    cpu_cell_capacity: Res<crate::ui::scene_manager::CpuCellCapacity>,
    lighting_config: Res<crate::ui::lighting_settings::LightingConfig>,
    mut camera_query: Query<&mut MainCamera>,
) {
    // Reset camera to default position (reuse existing camera from Preview scene)
    for mut camera in camera_query.iter_mut() {
        camera.center = Vec3::ZERO;
        camera.distance = 50.0;
        camera.target_distance = 50.0;
        camera.rotation = Quat::from_rotation_x(-0.5) * Quat::from_rotation_y(0.5);
        camera.target_rotation = Quat::from_rotation_x(-0.5) * Quat::from_rotation_y(0.5);
        camera.mode = crate::ui::camera::CameraMode::Orbit;
        camera.followed_entity = None;
    }

    // Get initial mode settings from genome (same as preview scene)
    let initial_mode_index = genome.genome.initial_mode.max(0) as usize;
    let mode = genome.genome.modes.get(initial_mode_index)
        .or_else(|| genome.genome.modes.first());
    
    let (color, opacity, emissive, split_mass, split_interval) = if let Some(mode) = mode {
        // Use get_split_mass/get_split_interval for potentially randomized values
        let sm = mode.get_split_mass(0, 0, 0);
        let si = mode.get_split_interval(0, 0, 0);
        (mode.color, mode.opacity, mode.emissive, sm, si)
    } else {
        (Vec3::new(1.0, 1.0, 1.0), 1.0, 0.0, 1.0, 5.0)
    };
    
    let cell_radius = 1.0;
    
    // Create initial state with capacity from settings
    let mut initial_state = InitialState::new(config.clone(), cpu_cell_capacity.capacity, 0);
    initial_state.add_cell(InitialCell {
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
        split_mass,
        stiffness: 500.0,  // Match preview scene to prevent pass-through
    });
    
    // Initialize canonical state from initial state
    main_state.canonical_state = initial_state.to_canonical_state();
    main_state.initial_state = initial_state;
    main_state.id_to_entity.clear();
    main_state.entity_to_index.clear();
    // Resize index_to_entity to match new capacity
    main_state.index_to_entity = vec![None; cpu_cell_capacity.capacity];
    main_state.simulation_time = 0.0;
    
    // OPTIMIZATION: Create shared sphere mesh once (reused for all cells)
    // This is a MASSIVE performance improvement - mesh generation is very expensive
    let sphere_mesh = meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap());
    main_state.sphere_mesh = sphere_mesh.clone();
    
    // OPTIMIZATION: Clear material cache on scene reset
    main_state.material_cache.clear();
    
    // Get or create cached material for initial cell
    let initial_material = get_or_create_material(color, opacity, emissive, &mut main_state.material_cache, &mut materials);
    
    // Check if initial cell is flagellocyte
    let initial_mode = genome.genome.modes.get(initial_mode_index);
    let is_flagellocyte = initial_mode.map(|m| m.cell_type == 1).unwrap_or(false);
    let swim_force = initial_mode.map(|m| m.swim_force).unwrap_or(0.0);
    let initial_mesh = if is_flagellocyte {
        meshes.add(crate::rendering::flagellocyte_mesh::generate_flagellocyte_mesh(cell_radius, swim_force, 5))
    } else {
        sphere_mesh.clone()
    };
    
    // Spawn ECS entity for the initial cell
    let entity = commands.spawn((
        // Cell data components
        Cell {
            mass: split_mass,
            radius: cell_radius,
            genome_id: 0,
            mode_index: initial_mode_index,
            cell_type: initial_mode.map(|m| m.cell_type).unwrap_or(0),
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
        crate::cell::physics::Cytoskeleton::default(),
        // Visual representation
        Mesh3d(initial_mesh),
        MeshMaterial3d(initial_material),
        Transform::from_translation(Vec3::ZERO)
            .with_rotation(genome.genome.initial_orientation)
            .with_scale(Vec3::splat(cell_radius)),
        Visibility::default(),
        CpuSceneEntity,
    )).id();
    
    // Map cell ID to entity
    main_state.id_to_entity.insert(0, entity);
    main_state.entity_to_index.insert(entity, 0);
    main_state.index_to_entity[0] = Some(entity);

    // Add basic lighting (using saved settings)
    let light_rotation = Quat::from_euler(
        EulerRot::XYZ,
        lighting_config.directional_rotation[0].to_radians(),
        lighting_config.directional_rotation[1].to_radians(),
        lighting_config.directional_rotation[2].to_radians(),
    );
    commands.spawn((
        DirectionalLight {
            illuminance: lighting_config.directional_illuminance,
            color: Color::srgb(
                lighting_config.directional_color[0],
                lighting_config.directional_color[1],
                lighting_config.directional_color[2],
            ),
            shadows_enabled: true, // Enable shadows for volumetric lighting
            ..default()
        },
        Transform::from_rotation(light_rotation),
        bevy::light::VolumetricLight, // Enable volumetric lighting
        CpuSceneEntity,
    ));

    // Add ambient light as an entity
    commands.spawn((
        AmbientLight {
            color: Color::WHITE,
            brightness: lighting_config.ambient_brightness,
            ..default()
        },
        CpuSceneEntity,
    ));

    // Add world boundary sphere (200 unit diameter = 100 unit radius)
    let world_mesh = Sphere::new(100.0).mesh().ico(7).unwrap();
    
    // World sphere with Fresnel edge lighting effect
    commands.spawn((
        Mesh3d(meshes.add(world_mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.2, 0.25, 0.35, 0.35),
            emissive: LinearRgba::rgb(0.05, 0.08, 0.12),
            metallic: 0.0,
            perceptual_roughness: 0.2,
            reflectance: 0.95,
            cull_mode: Some(bevy::render::render_resource::Face::Front),
            alpha_mode: AlphaMode::AlphaToCoverage,
            depth_bias: 0.1, // Push world sphere back slightly in depth to ensure cells render in front
            ..default()
        })),
        Transform::default(),
        crate::rendering::WorldSphere,
        NotShadowCaster,
        CpuSceneEntity,
    ));
    
    // Fog volume is now spawned automatically by VolumetricFogPlugin
}

/// Spawn skybox for CPU scene
fn spawn_cpu_skybox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    skybox_config: Res<crate::rendering::SkyboxConfig>,
) {
    crate::rendering::spawn_skybox(
        &mut commands,
        &asset_server,
        &skybox_config,
        CpuSceneEntity,
    );
}

/// Cleanup CPU scene entities (but keep the camera)
fn cleanup_cpu_scene(
    mut commands: Commands,
    query: Query<Entity, (With<CpuSceneEntity>, Without<MainCamera>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}






