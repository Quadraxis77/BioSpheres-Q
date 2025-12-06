use bevy::prelude::*;
use crate::cell::{Cell, CellPosition, CellOrientation, CellSignaling};
use crate::genome::CurrentGenome;
use crate::simulation::CpuSceneEntity;

/// Resource tracking pending cell divisions
///
/// This resource is used to track which cells are ready to divide,
/// allowing the cell allocation system to run conditionally only when
/// divisions are pending (Phase 1 optimization P1.2).
#[derive(Resource, Default)]
pub struct DivisionQueue {
    /// List of entities that are pending division
    pub pending_divisions: Vec<Entity>,
}

/// Condition function for running systems only when divisions are pending
///
/// This is used with Bevy's `.run_if()` to conditionally execute the cell
/// allocation pipeline only when there are pending divisions (Phase 1 optimization P1.2).
///
/// # Returns
/// `true` if there are pending divisions, `false` otherwise
pub fn has_pending_divisions(queue: Res<DivisionQueue>) -> bool {
    !queue.pending_divisions.is_empty()
}

/// Plugin for deterministic cell division
pub struct DivisionPlugin;

impl Plugin for DivisionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DivisionQueue>();
            // NOTE: check_and_divide_cells is disabled because CPU simulation
            // uses the canonical physics division system (handle_divisions in cpu_sim.rs)
            // This prevents duplicate division logic from running.
            // If GPU simulation needs ECS-based division, re-enable with proper run conditions.
    }
}

/// Division timer component tracking when a cell should divide
#[derive(Component, Clone, Copy)]
pub struct DivisionTimer {
    pub birth_time: f32,
    pub split_interval: f32,
}

/// System that checks for cells ready to divide and spawns children
/// NOTE: Currently disabled - CPU simulation uses canonical physics division instead
#[allow(dead_code)]
fn check_and_divide_cells(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    genome: Res<CurrentGenome>,
    config: Res<crate::simulation::SimulationConfig>,
    time: Res<Time<Fixed>>,
    simulation: Option<Res<crate::simulation::cell_allocation::Simulation>>,
    cells_query: Query<(Entity, &Cell, &CellPosition, &CellOrientation, &DivisionTimer), With<CpuSceneEntity>>,
    _division_queue: ResMut<DivisionQueue>,
) {
    // Check if we've reached the cell capacity limit
    let current_cell_count = cells_query.iter().count();
    let max_capacity = simulation
        .as_ref()
        .map(|sim| sim.cells.len())
        .unwrap_or(config.cell_count_limit as usize);

    // If at or above capacity, don't allow any new divisions
    if current_cell_count >= max_capacity {
        return;
    }

    let current_time = time.elapsed_secs();

    // Collect cells that need to divide
    let mut divisions = Vec::new();

    for (entity, cell, position, orientation, timer) in cells_query.iter() {
        let cell_age = current_time - timer.birth_time;
        if cell_age >= timer.split_interval {
            divisions.push((entity, cell.clone(), *position, *orientation, *timer));
        }
    }

    // Limit divisions to prevent exceeding capacity
    // Each division: removes 1 parent, adds 2 children = net +1 cell
    let available_slots = max_capacity.saturating_sub(current_cell_count);
    let max_divisions = available_slots;
    divisions.truncate(max_divisions);

    // Populate the division queue with entities that will divide
    // This allows the cell allocation system to run conditionally (Phase 1 optimization P1.2)
    // division_queue.pending_divisions.clear();
    // for (entity, _, _, _) in &divisions {
    //     division_queue.pending_divisions.push(*entity);
    // }

    // Process divisions
    for (parent_entity, cell, position, orientation, parent_timer) in divisions {
        let mode = genome.genome.modes.get(cell.mode_index);
        if mode.is_none() {
            continue;
        }
        let mode = mode.unwrap();
        
        // Calculate split direction
        let pitch = mode.parent_split_direction.x.to_radians();
        let yaw = mode.parent_split_direction.y.to_radians();
        let split_direction = orientation.rotation * Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0) * Vec3::Z;
        
        // Reduced offset for better adhesion overlap
        // Combined diameter = 2 * radius, so offset = 0.1 * 2 * radius = 0.2 * radius
        let offset_distance = cell.radius * 0.1;
        let child_a_pos = position.position - split_direction * offset_distance;
        let child_b_pos = position.position + split_direction * offset_distance;
        
        // Get child settings
        let child_a_mode_idx = mode.child_a.mode_number.max(0) as usize;
        let child_b_mode_idx = mode.child_b.mode_number.max(0) as usize;
        
        let child_a_mode = genome.genome.modes.get(child_a_mode_idx);
        let child_b_mode = genome.genome.modes.get(child_b_mode_idx);
        
        let (child_a_color, child_a_opacity, child_a_split_interval, child_a_split_mass) = if let Some(m) = child_a_mode {
            (m.color, m.opacity, m.split_interval, m.split_mass)
        } else {
            (Vec3::ONE, 1.0, 5.0, 1.0)
        };
        
        let (child_b_color, child_b_opacity, child_b_split_interval, child_b_split_mass) = if let Some(m) = child_b_mode {
            (m.color, m.opacity, m.split_interval, m.split_mass)
        } else {
            (Vec3::ONE, 1.0, 5.0, 1.0)
        };
        

        
        let child_a_orientation = orientation.rotation * mode.child_a.orientation;
        let child_b_orientation = orientation.rotation * mode.child_b.orientation;
        
        // Calculate excess age to inherit (prevents simultaneous splitting)
        let parent_age = current_time - parent_timer.birth_time;
        let excess_age = (parent_age - parent_timer.split_interval).max(0.0);
        
        // Spawn child A (inherits parent velocity)
        commands.spawn((
            Cell {
                mass: child_a_split_mass,
                radius: cell.radius,
                genome_id: cell.genome_id,
                mode_index: child_a_mode_idx,
            },
            CellPosition {
                position: child_a_pos,
                velocity: position.velocity,
            },
            CellOrientation {
                rotation: child_a_orientation,
                angular_velocity: Vec3::ZERO,
            },
            CellSignaling::default(),
            DivisionTimer {
                birth_time: current_time - excess_age,
                split_interval: child_a_split_interval,
            },
            crate::cell::physics::CellForces::default(),
            crate::cell::physics::Cytoskeleton::default(),
            Mesh3d(meshes.add(Sphere::new(cell.radius).mesh().ico(5).unwrap())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(child_a_color.x, child_a_color.y, child_a_color.z, child_a_opacity),
                cull_mode: Some(bevy::render::render_resource::Face::Back),
                alpha_mode: if child_a_opacity < 0.99 {
                    bevy::prelude::AlphaMode::Blend
                } else {
                    bevy::prelude::AlphaMode::Opaque
                },
                ..default()
            })),
            Transform::from_translation(child_a_pos).with_rotation(child_a_orientation),
            Visibility::default(),
            CpuSceneEntity,
        ));
        
        // Spawn child B (inherits parent velocity)
        commands.spawn((
            Cell {
                mass: child_b_split_mass,
                radius: cell.radius,
                genome_id: cell.genome_id,
                mode_index: child_b_mode_idx,
            },
            CellPosition {
                position: child_b_pos,
                velocity: position.velocity,
            },
            CellOrientation {
                rotation: child_b_orientation,
                angular_velocity: Vec3::ZERO,
            },
            CellSignaling::default(),
            DivisionTimer {
                birth_time: current_time - excess_age - 0.001, // Slight offset to desync from child A
                split_interval: child_b_split_interval,
            },
            crate::cell::physics::CellForces::default(),
            crate::cell::physics::Cytoskeleton::default(),
            Mesh3d(meshes.add(Sphere::new(cell.radius).mesh().ico(5).unwrap())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(child_b_color.x, child_b_color.y, child_b_color.z, child_b_opacity),
                cull_mode: Some(bevy::render::render_resource::Face::Back),
                alpha_mode: if child_b_opacity < 0.99 {
                    bevy::prelude::AlphaMode::Blend
                } else {
                    bevy::prelude::AlphaMode::Opaque
                },
                ..default()
            })),
            Transform::from_translation(child_b_pos).with_rotation(child_b_orientation),
            Visibility::default(),
            CpuSceneEntity,
        ));
        
        // Despawn parent
        commands.entity(parent_entity).despawn();
    }
}
