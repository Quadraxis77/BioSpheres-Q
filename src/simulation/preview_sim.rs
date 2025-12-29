use bevy::prelude::*;
use bevy::light::NotShadowCaster;
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
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
            .add_systems(OnEnter(PreviewSceneState::Active), (setup_preview_scene, spawn_preview_skybox))
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
                    highlight_selected_mode_cells,
                )
                    .chain()
                    .after(respawn_preview_cells_after_resimulation)
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
    
    /// Checkpoints for fast backward scrubbing (time, state)
    pub checkpoints: Vec<(f32, CanonicalState)>,
    
    /// Checkpoint interval in seconds
    pub checkpoint_interval: f32,
    
    /// Hash of genome to detect changes
    pub genome_hash: u64,
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
            checkpoints: Vec::new(),
            checkpoint_interval: 5.0, // Checkpoint every 5 seconds
            genome_hash: 0,
        }
    }
}

impl PreviewSimState {
    /// Compute a comprehensive hash of the genome for change detection
    /// This hashes ALL fields that affect simulation behavior
    fn compute_genome_hash(genome: &crate::genome::GenomeData) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash genome-level properties
        genome.name.hash(&mut hasher);
        genome.initial_mode.hash(&mut hasher);
        genome.initial_orientation.to_array().iter().for_each(|v| {
            v.to_bits().hash(&mut hasher);
        });

        // Hash ALL mode fields that affect simulation
        for mode in &genome.modes {
            // Identity
            mode.name.hash(&mut hasher);
            mode.default_name.hash(&mut hasher);
            mode.color.x.to_bits().hash(&mut hasher);
            mode.color.y.to_bits().hash(&mut hasher);
            mode.color.z.to_bits().hash(&mut hasher);
            mode.opacity.to_bits().hash(&mut hasher);
            mode.emissive.to_bits().hash(&mut hasher);

            // Cell type
            mode.cell_type.hash(&mut hasher);
            mode.swim_force.to_bits().hash(&mut hasher);

            // Parent settings
            mode.split_mass.to_bits().hash(&mut hasher);
            mode.split_interval.to_bits().hash(&mut hasher);
            mode.nutrient_gain_rate.to_bits().hash(&mut hasher);
            mode.max_cell_size.to_bits().hash(&mut hasher);
            mode.split_ratio.to_bits().hash(&mut hasher);
            mode.nutrient_priority.to_bits().hash(&mut hasher);
            mode.parent_split_direction.x.to_bits().hash(&mut hasher);
            mode.parent_split_direction.y.to_bits().hash(&mut hasher);
            mode.max_adhesions.hash(&mut hasher);
            mode.min_adhesions.hash(&mut hasher);
            mode.max_splits.hash(&mut hasher);
            mode.parent_make_adhesion.hash(&mut hasher);

            // Child settings
            mode.child_a.mode_number.hash(&mut hasher);
            mode.child_a.orientation.x.to_bits().hash(&mut hasher);
            mode.child_a.orientation.y.to_bits().hash(&mut hasher);
            mode.child_a.orientation.z.to_bits().hash(&mut hasher);
            mode.child_a.orientation.w.to_bits().hash(&mut hasher);
            mode.child_a.keep_adhesion.hash(&mut hasher);
            mode.child_b.mode_number.hash(&mut hasher);
            mode.child_b.orientation.x.to_bits().hash(&mut hasher);
            mode.child_b.orientation.y.to_bits().hash(&mut hasher);
            mode.child_b.orientation.z.to_bits().hash(&mut hasher);
            mode.child_b.orientation.w.to_bits().hash(&mut hasher);
            mode.child_b.keep_adhesion.hash(&mut hasher);

            // Adhesion settings
            mode.adhesion_settings.can_break.hash(&mut hasher);
            mode.adhesion_settings.break_force.to_bits().hash(&mut hasher);
            mode.adhesion_settings.rest_length.to_bits().hash(&mut hasher);
            mode.adhesion_settings.linear_spring_stiffness.to_bits().hash(&mut hasher);
            mode.adhesion_settings.linear_spring_damping.to_bits().hash(&mut hasher);
            mode.adhesion_settings.orientation_spring_stiffness.to_bits().hash(&mut hasher);
            mode.adhesion_settings.orientation_spring_damping.to_bits().hash(&mut hasher);
            mode.adhesion_settings.max_angular_deviation.to_bits().hash(&mut hasher);
            mode.adhesion_settings.enable_twist_constraint.hash(&mut hasher);
            mode.adhesion_settings.twist_constraint_stiffness.to_bits().hash(&mut hasher);
            mode.adhesion_settings.twist_constraint_damping.to_bits().hash(&mut hasher);
        }

        hasher.finish()
    }
    
    /// Clear checkpoints (called when genome changes)
    fn clear_checkpoints(&mut self) {
        self.checkpoints.clear();
    }
    
    /// Find the best checkpoint to start from for a given target time
    fn find_best_checkpoint(&self, target_time: f32) -> Option<(f32, CanonicalState)> {
        // Find the latest checkpoint that's before or at target_time
        self.checkpoints
            .iter()
            .rev() // Search from newest to oldest
            .find(|(time, _)| *time <= target_time)
            .cloned()
    }
    
    /// Add a checkpoint if we've passed a checkpoint interval
    fn maybe_add_checkpoint(&mut self, time: f32, state: &CanonicalState) {
        // Check if we should add a checkpoint at this time
        let checkpoint_index = (time / self.checkpoint_interval).floor() as usize;
        
        // Only add if we don't already have this checkpoint
        if self.checkpoints.len() <= checkpoint_index {
            self.checkpoints.push((time, state.clone()));
        }
    }
}

/// Result from background resimulation task
pub struct ResimulationResult {
    pub canonical_state: CanonicalState,
    pub target_time: f32,
    pub new_checkpoints: Vec<(f32, CanonicalState)>,
}

/// Preview request resource
#[derive(Resource, Default)]
pub struct PreviewRequest {
    /// Background task for async resimulation
    pub background_task: Option<Task<ResimulationResult>>,
}

/// Setup the Preview simulation scene with camera and initial state
fn setup_preview_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    fog_settings: Res<crate::rendering::VolumetricFogSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut preview_state: ResMut<PreviewSimState>,
    genome: Res<CurrentGenome>,
    config: Res<PhysicsConfig>,
    lighting_config: Res<crate::ui::lighting_settings::LightingConfig>,
) {
    // Spawn camera with volumetric fog and boundary crossing effect
    commands.spawn((
        Camera3d::default(),
        MainCamera{
            center: Vec3::ZERO, // Orbit around world origin
            distance: 50.0, // Start with some distance from origin
            target_distance: 50.0, // Target distance for spring interpolation
            rotation: Quat::from_rotation_x(-0.5) * Quat::from_rotation_y(0.5),
            target_rotation: Quat::from_rotation_x(-0.5) * Quat::from_rotation_y(0.5),
            mode: crate::ui::camera::CameraMode::Orbit,
            followed_entity: None,
        },
        bevy::light::VolumetricFog {
            ambient_intensity: fog_settings.ambient_intensity,
            step_count: fog_settings.step_count,
            ..default()
        },
        // Boundary crossing post-processing effect
        crate::rendering::BoundaryCrossingSettings::default(),
        // OIT (Order-Independent Transparency) - currently disabled
        // bevy::core_pipeline::oit::OrderIndependentTransparencySettings::default(),
        Msaa::Sample4, // Enable MSAA for AlphaToCoverage transparency
        PreviewSceneEntity,
    ));

    // Spawn lights (using saved settings)
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
        PreviewSceneEntity,
    ));

    commands.spawn((
        AmbientLight {
            color: Color::WHITE,
            brightness: lighting_config.ambient_brightness,
            ..default()
        },
        PreviewSceneEntity,
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
        PreviewSceneEntity,
    ));
    
    // Fog volume is now spawned automatically by VolumetricFogPlugin
    
    // Initialize preview state with single cell at origin
    let initial_mode_index = genome.genome.initial_mode.max(0) as usize;
    let mode = genome.genome.modes.get(initial_mode_index)
        .or_else(|| genome.genome.modes.first());
    
    let (split_mass, split_interval) = if let Some(mode) = mode {
        // Use get_split_mass/get_split_interval for potentially randomized values
        (mode.get_split_mass(0, 0, 0), mode.get_split_interval(0, 0, 0))
    } else {
        (1.0, 5.0)
    };
    
    let cell_radius = 1.0;
    let stiffness = 500.0;  // Increased from 10.0 to prevent pass-through
    
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
        split_mass,
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
    preview_state.checkpoints.clear();
    preview_state.genome_hash = PreviewSimState::compute_genome_hash(&genome.genome);
    
    // Spawn ECS entity for the initial cell
    let mode = genome.genome.modes.get(initial_mode_index)
        .or_else(|| genome.genome.modes.first());
    let (color, opacity, emissive) = if let Some(mode) = mode {
        (mode.color, mode.opacity, mode.emissive)
    } else {
        (Vec3::ONE, 1.0, 0.0)
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
            cell_type: mode.map(|m| m.cell_type).unwrap_or(0),
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
            base_color: Color::srgba(color.x, color.y, color.z, opacity),
            emissive: LinearRgba::rgb(color.x * emissive, color.y * emissive, color.z * emissive),
            cull_mode: Some(bevy::render::render_resource::Face::Back),
            alpha_mode: if opacity < 0.99 {
                bevy::prelude::AlphaMode::AlphaToCoverage
            } else {
                bevy::prelude::AlphaMode::Opaque
            },
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

/// Run preview re-simulation using canonical physics in a background task
/// This runs the simulation asynchronously to keep the UI responsive
fn run_preview_resimulation(
    mut preview_state: ResMut<PreviewSimState>,
    mut sim_state: ResMut<crate::simulation::SimulationState>,
    config: Res<PhysicsConfig>,
    genome: Res<CurrentGenome>,
    mut preview_request: ResMut<PreviewRequest>,
) {
    // Check if there's a completed background task
    if let Some(mut task) = preview_request.background_task.take() {
        if let Some(result) = block_on(poll_once(&mut task)) {
            // Task completed - apply results
            let _old_cell_count = preview_state.canonical_state.cell_count;
            preview_state.canonical_state = result.canonical_state;
            preview_state.current_time = result.target_time;
            let _new_cell_count = preview_state.canonical_state.cell_count;
            
            // Add new checkpoints created during simulation
            for (time, state) in result.new_checkpoints {
                preview_state.maybe_add_checkpoint(time, &state);
            }
            
            sim_state.target_time = None;
            sim_state.is_resimulating = false;
            
            // Always trigger respawn after resimulation to ensure meshes are updated
            // This handles cell type changes, mode changes, etc.
            sim_state.needs_respawn = true;
        } else {
            // Task still running - put it back and keep waiting
            preview_request.background_task = Some(task);
            sim_state.is_resimulating = true;
            return;
        }
    }

    // Check if genome actually changed by comparing hash
    // Note: We can't use genome.is_changed() because the UI system uses ResMut
    // which marks it as changed every frame even with no actual edits
    let current_genome_hash = PreviewSimState::compute_genome_hash(&genome.genome);
    let genome_changed = current_genome_hash != preview_state.genome_hash;

    if genome_changed {
        // Genome changed - clear checkpoints and trigger resimulation from current time
        preview_state.clear_checkpoints();
        preview_state.genome_hash = current_genome_hash;
        // DON'T reset time - keep current time and resimulate from there
        // preview_state.current_time = 0.0;  // REMOVED

        // Trigger resimulation from current time with new genome
        sim_state.target_time = Some(preview_state.current_time);

        // Update initial state with new genome values
        if let Some(initial_cell) = preview_state.initial_state.initial_cells.first_mut() {
            let initial_mode_index = genome.genome.initial_mode.max(0) as usize;
            let mode = genome.genome.modes.get(initial_mode_index)
                .or_else(|| genome.genome.modes.first());

            if let Some(mode) = mode {
                initial_cell.split_interval = mode.get_split_interval(0, 0, 0);
                initial_cell.split_mass = mode.get_split_mass(0, 0, 0);
                initial_cell.mode_index = initial_mode_index;
                initial_cell.rotation = genome.genome.initial_orientation;
            }
        }

        // DON'T reset canonical state here - keep the old state visible until resimulation completes
        // This prevents cells from disappearing during resimulation
        // preview_state.canonical_state = preview_state.initial_state.to_canonical_state();
    }

    // Check if we need to start a new resimulation
    let Some(target_time) = sim_state.target_time else {
        sim_state.is_resimulating = false;
        return;
    };

    // Determine best starting point using checkpoints
    let (start_time, start_step, mut canonical_state) = if target_time > preview_state.current_time && !genome_changed {
        // Moving forward: simulate from current state
        let start_step = (preview_state.current_time / config.fixed_timestep).ceil() as u32;
        (preview_state.current_time, start_step, preview_state.canonical_state.clone())
    } else if let Some((checkpoint_time, checkpoint_state)) = preview_state.find_best_checkpoint(target_time) {
        // Moving backward: use nearest checkpoint
        let start_step = (checkpoint_time / config.fixed_timestep).ceil() as u32;
        (checkpoint_time, start_step, checkpoint_state)
    } else {
        // No suitable checkpoint: start from initial state
        let initial_canonical = preview_state.initial_state.to_canonical_state();
        (0.0, 0, initial_canonical)
    };
    
    let end_step = (target_time / config.fixed_timestep).ceil() as u32;
    let steps = end_step.saturating_sub(start_step);

    // Clone other data needed for background task
    let config = config.clone();
    let genome_data = genome.genome.clone();
    let max_cells = preview_state.initial_state.max_cells;
    let rng_seed = preview_state.initial_state.rng_seed;
    let fixed_timestep = config.fixed_timestep;
    let checkpoint_interval = preview_state.checkpoint_interval;

    // Spawn background task
    let task_pool = AsyncComputeTaskPool::get();
    let task = task_pool.spawn(async move {
        // Track checkpoints to create during simulation
        let mut new_checkpoints = Vec::new();
        let mut last_checkpoint_index = (start_time / checkpoint_interval).floor() as usize;
        
        // Run physics steps in background thread with multithreading
        for step in 0..steps {
            let current_time = (start_step + step) as f32 * fixed_timestep;

            // Run CPU physics step (multithreaded via Rayon, swim disabled for preview)
            // Preview mode disables swim to keep flagellocytes from swimming away
            // 
            // NOTE: GPU physics is not used here because:
            // 1. GPU operations must run on the main thread with GPU context access
            // 2. This background task runs on a separate async compute thread
            // 3. Multithreaded CPU physics via Rayon is already very fast for preview (<256 cells)
            // 
            // For GPU acceleration in preview, we would need to:
            // - Run resimulation synchronously on the main thread, OR
            // - Implement a command queue system to schedule GPU work from background threads
            crate::simulation::cpu_physics::physics_step_with_genome(
                &mut canonical_state,
                &config,
                &genome_data,
                current_time,
                false, // Disable swim in preview mode
            );

            // Run division step
            crate::simulation::cpu_physics::division_step(
                &mut canonical_state,
                &genome_data,
                current_time,
                max_cells,
                rng_seed,
            );
            
            // Check if we should create a checkpoint
            let current_checkpoint_index = (current_time / checkpoint_interval).floor() as usize;
            if current_checkpoint_index > last_checkpoint_index {
                // Create checkpoint at this interval
                new_checkpoints.push((current_time, canonical_state.clone()));
                last_checkpoint_index = current_checkpoint_index;
            }
        }

        ResimulationResult {
            canonical_state,
            target_time,
            new_checkpoints,
        }
    });

    preview_request.background_task = Some(task);
    sim_state.is_resimulating = true;
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
    mut cells_query: Query<(Entity, &mut Cell, &mut CellPosition, &mut CellOrientation, &MeshMaterial3d<StandardMaterial>, &mut Mesh3d), With<PreviewSceneEntity>>,
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
        let new_cell_count = preview_state.canonical_state.cell_count;
        
        // Count existing entities
        let mut existing_count = 0;
        for i in 0..preview_state.index_to_entity.len() {
            if preview_state.index_to_entity[i].is_some() {
                existing_count += 1;
            }
        }
        
        // Update existing cells using index_to_entity mapping
        for i in 0..existing_count.max(new_cell_count) {
            if i < new_cell_count {
                // This cell should exist
                if let Some(entity) = preview_state.index_to_entity[i] {
                    // Update existing entity
                    if let Ok((_, mut cell, mut cell_pos, mut cell_orient, material_handle, mut mesh_handle)) = cells_query.get_mut(entity) {
                        let mode_index = preview_state.canonical_state.mode_indices[i];

                        // Get old cell type from cached value
                        let old_cell_type = cell.cell_type;

                        // Get new cell type from genome
                        let new_mode = genome.genome.modes.get(mode_index);
                        let new_cell_type = new_mode.map(|m| m.cell_type).unwrap_or(0);

                        // Now update cell data
                        cell.mass = preview_state.canonical_state.masses[i];
                        cell.radius = preview_state.canonical_state.radii[i];
                        cell.genome_id = preview_state.canonical_state.genome_ids[i];
                        cell.mode_index = mode_index;
                        cell.cell_type = new_cell_type;
                        
                        cell_pos.position = preview_state.canonical_state.positions[i];
                        cell_pos.velocity = preview_state.canonical_state.velocities[i];
                        cell_orient.rotation = preview_state.canonical_state.rotations[i];
                        cell_orient.angular_velocity = preview_state.canonical_state.angular_velocities[i];
                        
                        // Update material
                        let (color, opacity, emissive) = if let Some(mode) = new_mode {
                            (mode.color, mode.opacity, mode.emissive)
                        } else {
                            (Vec3::ONE, 1.0, 0.0)
                        };
                        
                        if let Some(material) = materials.get_mut(&material_handle.0) {
                            material.base_color = Color::srgba(color.x, color.y, color.z, opacity);
                            material.emissive = LinearRgba::rgb(color.x * emissive, color.y * emissive, color.z * emissive);
                        }
                        
                        // Check if cell type changed (need to update mesh)
                        if old_cell_type != new_cell_type {
                            // Cell type changed - update mesh
                            let is_flagellocyte = new_cell_type == 1;
                            let swim_force = new_mode.map(|m| m.swim_force).unwrap_or(0.0);
                            
                            let new_mesh = if is_flagellocyte {
                                meshes.add(crate::rendering::flagellocyte_mesh::generate_flagellocyte_mesh(1.0, swim_force, 5))
                            } else {
                                meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap())
                            };
                            
                            mesh_handle.0 = new_mesh;
                        }
                    }
                }
                // If entity doesn't exist, it will be spawned below
            } else {
                // This cell should NOT exist - despawn if it does
                if let Some(entity) = preview_state.index_to_entity[i] {
                    commands.entity(entity).despawn();
                    preview_state.index_to_entity[i] = None;
                }
            }
        }
        
        // Spawn cells that don't have entities yet
        let sphere_mesh = meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap());
        let max_modes = genome.genome.modes.len();
        let mut material_cache: Vec<Option<Handle<StandardMaterial>>> = vec![None; max_modes];
        
        for i in 0..new_cell_count {
            // Skip if entity already exists
            if preview_state.index_to_entity[i].is_some() {
                continue;
            }
            
            // Spawn new cell entity
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
                    let (color, opacity, emissive) = if let Some(mode) = mode {
                        (mode.color, mode.opacity, mode.emissive)
                    } else {
                        (Vec3::ONE, 1.0, 0.0)
                    };
                    let mat = materials.add(StandardMaterial {
                        base_color: Color::srgba(color.x, color.y, color.z, opacity),
                        emissive: LinearRgba::rgb(color.x * emissive, color.y * emissive, color.z * emissive),
                        cull_mode: Some(bevy::render::render_resource::Face::Back),
                        alpha_mode: if opacity < 0.99 {
                            bevy::prelude::AlphaMode::AlphaToCoverage
                        } else {
                            bevy::prelude::AlphaMode::Opaque
                        },
                        ..default()
                    });
                    material_cache[mode_index] = Some(mat.clone());
                    mat
                }
            } else {
                // Fallback for invalid mode index
                materials.add(StandardMaterial {
                    base_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                    cull_mode: Some(bevy::render::render_resource::Face::Back),
                    alpha_mode: bevy::prelude::AlphaMode::Opaque,
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
                    cell_type: mode.map(|m| m.cell_type).unwrap_or(0),
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
    mut cells_query: Query<(Entity, &mut Cell, &mut CellPosition, &mut CellOrientation), (With<Cell>, With<PreviewSceneEntity>)>,
    drag_state: Res<crate::input::DragState>,
    sim_state: Res<crate::simulation::SimulationState>,
) {
    // Skip sync entirely if currently dragging a cell
    if drag_state.dragged_entity.is_some() {
        return;
    }
    
    // Skip if we're about to respawn (cell count changed)
    if sim_state.needs_respawn {
        return;
    }
    
    // Sync all cell data from canonical state to existing entities
    for i in 0..preview_state.canonical_state.cell_count {
        if let Some(entity) = preview_state.index_to_entity[i] {
            // Skip the dragged entity
            if Some(entity) == drag_state.dragged_entity {
                continue;
            }
            
            // Update entity components directly using index
            if let Ok((_, mut cell, mut cell_pos, mut cell_orientation)) = cells_query.get_mut(entity) {
                // Update all cell data
                cell.mass = preview_state.canonical_state.masses[i];
                cell.radius = preview_state.canonical_state.radii[i];
                cell.genome_id = preview_state.canonical_state.genome_ids[i];
                cell.mode_index = preview_state.canonical_state.mode_indices[i];
                
                cell_pos.position = preview_state.canonical_state.positions[i];
                cell_pos.velocity = preview_state.canonical_state.velocities[i];
                cell_orientation.rotation = preview_state.canonical_state.rotations[i];
                cell_orientation.angular_velocity = preview_state.canonical_state.angular_velocities[i];
            }
        }
    }
}



/// Spawn skybox for Preview scene
fn spawn_preview_skybox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    skybox_config: Res<crate::rendering::SkyboxConfig>,
) {
    crate::rendering::spawn_skybox(
        &mut commands,
        &asset_server,
        &skybox_config,
        PreviewSceneEntity,
    );
}




/// Highlight cells of the selected mode with a pulsing yellow emissive glow
fn highlight_selected_mode_cells(
    time: Res<Time>,
    genome: Res<CurrentGenome>,
    cells_query: Query<(&Cell, &MeshMaterial3d<StandardMaterial>), With<PreviewSceneEntity>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let selected_mode = genome.selected_mode_index as usize;
    let glow_enabled = genome.show_mode_glow;
    
    // Calculate pulse intensity using sine wave (0.0 to 1.0 range)
    // Pulse at ~1.5 Hz for a gentle effect
    let pulse = (time.elapsed_secs() * 1.5 * std::f32::consts::PI).sin() * 0.5 + 0.5;
    
    // Yellow highlight color with subtle intensity
    let highlight_intensity = 0.15 + pulse * 0.1; // Range: 0.15 to 0.25
    let highlight_color = LinearRgba::rgb(
        highlight_intensity * 1.0,  // Yellow-ish
        highlight_intensity * 0.85,
        highlight_intensity * 0.1,
    );
    
    for (cell, material_handle) in cells_query.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            if glow_enabled && cell.mode_index == selected_mode {
                // Apply pulsing yellow highlight
                let mode = genome.genome.modes.get(cell.mode_index);
                let (color, base_emissive) = if let Some(mode) = mode {
                    (mode.color, mode.emissive)
                } else {
                    (Vec3::ONE, 0.0)
                };
                
                // Combine base emissive with highlight
                material.emissive = LinearRgba::rgb(
                    color.x * base_emissive + highlight_color.red,
                    color.y * base_emissive + highlight_color.green,
                    color.z * base_emissive + highlight_color.blue,
                );
            } else {
                // Reset to normal emissive for non-selected modes (or when glow disabled)
                let mode = genome.genome.modes.get(cell.mode_index);
                let (color, emissive) = if let Some(mode) = mode {
                    (mode.color, mode.emissive)
                } else {
                    (Vec3::ONE, 0.0)
                };
                material.emissive = LinearRgba::rgb(
                    color.x * emissive,
                    color.y * emissive,
                    color.z * emissive,
                );
            }
        }
    }
}
