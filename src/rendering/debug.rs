use bevy::prelude::*;
use crate::cell::{Cell, CellPosition, CellOrientation};
use crate::genome::{CurrentGenome, GenomeLibrary};
use super::RenderingConfig;

/// Plugin for debug rendering
pub struct DebugRenderingPlugin;

impl Plugin for DebugRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_orientation_gizmos)
            .add_systems(Update, update_split_plane_gizmos)
            .add_systems(Update, update_split_plane_transforms)
            .add_systems(Update, update_anchor_gizmos)
            .add_systems(Update, update_anchor_transforms);
    }
}

/// Marker component for anchor gizmo spheres
#[derive(Component)]
pub struct AnchorGizmo {
    pub cell_entity: Entity,
    pub adhesion_index: usize,
    pub is_side_a: bool,  // true if this is cell A's anchor, false if cell B's
}

/// Marker component for split plane ring entities
#[derive(Component)]
pub struct SplitPlaneRing {
    pub cell_entity: Entity,
}

/// Render orientation gizmos for all cells
fn render_orientation_gizmos(
    mut gizmos: Gizmos,
    config: Res<RenderingConfig>,
    cells_query: Query<(&Cell, &CellPosition, &CellOrientation, &Visibility)>,
    focal_plane: Res<crate::ui::camera::FocalPlaneSettings>,
    camera_query: Query<(&Transform, &crate::ui::camera::MainCamera)>,
) {
    if !config.show_orientation_gizmos {
        return;
    }

    // Get focal plane info for visibility check
    let focal_plane_check = if focal_plane.enabled {
        if let Ok((camera_transform, cam)) = camera_query.single() {
            if cam.mode == crate::ui::camera::CameraMode::FreeFly {
                let camera_pos = camera_transform.translation;
                let camera_forward = camera_transform.rotation * Vec3::NEG_Z;
                let plane_center = camera_pos + camera_forward * focal_plane.distance;
                Some((plane_center, camera_forward))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Render orientation axes for each cell
    for (cell, position, orientation, visibility) in cells_query.iter() {
        // Skip hidden cells (respects focal plane visibility set by camera system)
        if *visibility == Visibility::Hidden {
            continue;
        }
        
        // Double-check with focal plane (in case visibility hasn't updated yet)
        if let Some((plane_center, camera_forward)) = focal_plane_check {
            let to_cell = position.position - plane_center;
            let signed_distance = to_cell.dot(camera_forward);
            if signed_distance + cell.radius <= 0.0 {
                continue;
            }
        }
        
        let gizmo_length = cell.radius * 1.8;
        
        // Create three axis lines: forward (blue), right (green), up (red)
        // Matching C++ implementation colors
        let axes = [
            (Vec3::X, Color::srgb(0.0, 0.0, 1.0)), // Forward (X) - Blue
            (Vec3::Y, Color::srgb(0.0, 1.0, 0.0)), // Right (Y) - Green
            (Vec3::Z, Color::srgb(1.0, 0.0, 0.0)), // Up (Z) - Red
        ];

        for (axis, color) in axes.iter() {
            let world_axis = orientation.rotation * *axis;
            let end_pos = position.position + world_axis * gizmo_length;
            gizmos.line(position.position, end_pos, *color);
        }
    }
}

/// Update split plane gizmos for all cells
fn update_split_plane_gizmos(
    mut commands: Commands,
    config: Res<RenderingConfig>,
    genome_library: Res<GenomeLibrary>,
    current_genome: Res<CurrentGenome>,
    cells_query: Query<(Entity, &Cell, &CellPosition, &CellOrientation)>,
    ring_query: Query<(Entity, &SplitPlaneRing)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !config.show_split_plane_gizmos {
        // Remove all rings if disabled
        for (entity, _) in ring_query.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Track which cells have rings
    let mut cells_with_rings: std::collections::HashSet<Entity> = 
        ring_query.iter().map(|(_, r)| r.cell_entity).collect();

    // Get the genome to use
    let genome = if !genome_library.genomes.is_empty() {
        &genome_library.genomes[0]
    } else {
        &current_genome.genome
    };

    // Create rings for cells that don't have them
    for (cell_entity, cell, position, orientation) in cells_query.iter() {
        if cells_with_rings.contains(&cell_entity) {
            cells_with_rings.remove(&cell_entity);
            continue; // Ring already exists
        }

        // Get mode settings
        if cell.mode_index >= genome.modes.len() {
            continue;
        }
        let mode = &genome.modes[cell.mode_index];

        // Calculate split direction using the same method as cell division
        let pitch = mode.parent_split_direction.x.to_radians();
        let yaw = mode.parent_split_direction.y.to_radians();
        
        // Use Euler rotation to match the actual division code
        let split_direction_local = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0) * Vec3::Z;

        // Create two rings perpendicular to the split direction IN LOCAL SPACE
        let outer_radius = cell.radius * 1.2;
        let inner_radius = cell.radius * 1.4;
        let offset_distance = 0.001;
        let segments = 32;

        // Create blue ring (one side) - mesh with LOCAL offset, will be rotated by Transform
        let blue_local_offset = split_direction_local * offset_distance;
        let blue_mesh = create_filled_ring_mesh(blue_local_offset, split_direction_local, inner_radius, outer_radius, segments);
        commands.spawn((
            Mesh3d(meshes.add(blue_mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 0.0, 1.0), // Opaque blue
                unlit: true,
                cull_mode: None, // Render both sides
                ..default()
            })),
            Transform::from_translation(position.position).with_rotation(orientation.rotation),
            Visibility::default(),
            SplitPlaneRing { cell_entity },
        ));

        // Create green ring (other side) - mesh with LOCAL offset, will be rotated by Transform
        let green_local_offset = -split_direction_local * offset_distance;
        let green_mesh = create_filled_ring_mesh(green_local_offset, split_direction_local, inner_radius, outer_radius, segments);
        commands.spawn((
            Mesh3d(meshes.add(green_mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 1.0, 0.0), // Opaque green
                unlit: true,
                cull_mode: None, // Render both sides
                ..default()
            })),
            Transform::from_translation(position.position).with_rotation(orientation.rotation),
            Visibility::default(),
            SplitPlaneRing { cell_entity },
        ));
    }

    // Remove rings for cells that no longer exist
    for cell_entity in cells_with_rings {
        for (ring_entity, ring) in ring_query.iter() {
            if ring.cell_entity == cell_entity {
                commands.entity(ring_entity).despawn();
            }
        }
    }
}

/// Update split plane ring transforms to follow their parent cells
fn update_split_plane_transforms(
    config: Res<RenderingConfig>,
    cells_query: Query<(Entity, &CellPosition, &CellOrientation, &Visibility), Without<SplitPlaneRing>>,
    mut ring_query: Query<(&SplitPlaneRing, &mut Transform, &mut Visibility), Without<Cell>>,
) {
    if !config.show_split_plane_gizmos {
        return;
    }

    // Update each ring's transform to match its parent cell
    for (ring, mut transform, mut ring_visibility) in ring_query.iter_mut() {
        // Find the parent cell and update position and rotation
        if let Ok((_, position, orientation, cell_visibility)) = cells_query.get(ring.cell_entity) {
            // Update position and rotation to follow the cell
            transform.translation = position.position;
            transform.rotation = orientation.rotation;
            
            // Match parent cell's visibility (for focal plane)
            let new_visibility = if *cell_visibility == Visibility::Hidden {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
            if *ring_visibility != new_visibility {
                *ring_visibility = new_visibility;
            }
        }
    }
}

/// Create a flat filled ring mesh manually
fn create_filled_ring_mesh(
    center: Vec3,
    normal: Vec3,
    inner_radius: f32,
    outer_radius: f32,
    segments: usize,
) -> Mesh {
    // Calculate perpendicular vectors to create the ring plane
    // The ring lies in the plane perpendicular to 'normal'
    let temp_up = if normal.y.abs() < 0.9 {
        Vec3::Y
    } else {
        Vec3::X
    };
    let right = normal.cross(temp_up).normalize();
    let up = right.cross(normal).normalize();

    // Build vertices and indices manually
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    // Generate ring vertices
    for i in 0..segments {
        let angle1 = (2.0 * std::f32::consts::PI * i as f32) / segments as f32;
        let angle2 = (2.0 * std::f32::consts::PI * ((i + 1) % segments) as f32) / segments as f32;

        let cos1 = angle1.cos();
        let sin1 = angle1.sin();
        let cos2 = angle2.cos();
        let sin2 = angle2.sin();

        // Calculate positions in the ring plane
        let inner1 = center + (right * cos1 + up * sin1) * inner_radius;
        let outer1 = center + (right * cos1 + up * sin1) * outer_radius;
        let inner2 = center + (right * cos2 + up * sin2) * inner_radius;
        let outer2 = center + (right * cos2 + up * sin2) * outer_radius;

        let base_idx = positions.len() as u32;

        // Add vertices
        positions.push(inner1.to_array());
        positions.push(outer1.to_array());
        positions.push(inner2.to_array());
        positions.push(outer2.to_array());

        // Add normals (all pointing in the normal direction)
        for _ in 0..4 {
            normals.push(normal.to_array());
        }

        // Triangle 1: inner1, outer1, inner2
        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);

        // Triangle 2: outer1, outer2, inner2
        indices.push(base_idx + 1);
        indices.push(base_idx + 3);
        indices.push(base_idx + 2);
    }

    // Create a new mesh from scratch
    let mut mesh = Mesh::new(
        bevy_mesh::PrimitiveTopology::TriangleList,
        bevy_asset::RenderAssetUsages::RENDER_WORLD,
    );
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(bevy_mesh::Indices::U32(indices));
    
    mesh
}

/// Update anchor gizmos for all adhesion connections
fn update_anchor_gizmos(
    mut commands: Commands,
    config: Res<RenderingConfig>,
    main_state: Option<Res<crate::simulation::cpu_sim::MainSimState>>,
    preview_state: Option<Res<crate::simulation::preview_sim::PreviewSimState>>,
    sim_state: Res<crate::simulation::SimulationState>,
    anchor_query: Query<(Entity, &AnchorGizmo)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !config.show_orientation_gizmos {
        // Remove all anchor gizmos if disabled
        for (entity, _) in anchor_query.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Get the appropriate state based on simulation mode
    let (state, connections) = match sim_state.mode {
        crate::simulation::SimulationMode::Cpu => {
            if let Some(main) = main_state.as_ref() {
                (&main.canonical_state, &main.canonical_state.adhesion_connections)
            } else {
                return;
            }
        }
        crate::simulation::SimulationMode::Preview => {
            if let Some(preview) = preview_state.as_ref() {
                (&preview.canonical_state, &preview.canonical_state.adhesion_connections)
            } else {
                return;
            }
        }
    };

    // Track which anchors exist (entity, adhesion_index, is_side_a)
    let mut existing_anchors: std::collections::HashSet<(Entity, usize, bool)> = 
        anchor_query.iter().map(|(_, g)| (g.cell_entity, g.adhesion_index, g.is_side_a)).collect();

    // Build index-to-entity mapping based on simulation mode
    let mut index_to_entity: std::collections::HashMap<usize, Entity> = std::collections::HashMap::new();
    
    match sim_state.mode {
        crate::simulation::SimulationMode::Cpu => {
            if let Some(main) = main_state.as_ref() {
                // Use the entity_to_index mapping to build the reverse mapping
                for (&entity, &index) in main.entity_to_index.iter() {
                    index_to_entity.insert(index, entity);
                }
            }
        }
        crate::simulation::SimulationMode::Preview => {
            if let Some(preview) = preview_state.as_ref() {
                // For preview, use index_to_entity directly
                for i in 0..preview.canonical_state.cell_count {
                    if let Some(entity) = preview.index_to_entity[i] {
                        index_to_entity.insert(i, entity);
                    }
                }
            }
        }
    }

    // Create anchor spheres for each active adhesion
    for i in 0..connections.active_count {
        if connections.is_active[i] == 0 {
            continue;
        }

        let cell_a_idx = connections.cell_a_index[i];
        let cell_b_idx = connections.cell_b_index[i];

        // Validate indices
        if cell_a_idx >= state.cell_count || cell_b_idx >= state.cell_count {
            continue;
        }

        // Find the cell entities using proper index mapping
        let cell_a_entity = index_to_entity.get(&cell_a_idx).copied();
        let cell_b_entity = index_to_entity.get(&cell_b_idx).copied();

        if let (Some(entity_a), Some(entity_b)) = (cell_a_entity, cell_b_entity) {
            // Create anchor for cell A
            if !existing_anchors.contains(&(entity_a, i, true)) {
                let pos_a = state.positions[cell_a_idx];
                let rot_a = state.rotations[cell_a_idx];  // Use physics rotation so anchors rotate with cell
                let anchor_dir_a = connections.anchor_direction_a[i];
                let cell_radius_a = state.radii[cell_a_idx];
                let radius_a = cell_radius_a * 0.1; // Small sphere for visualization

                // Transform anchor direction to world space
                let world_anchor_a = rot_a * anchor_dir_a;
                let anchor_pos_a = pos_a + world_anchor_a * cell_radius_a;

                // Get zone color
                let zone_a = connections.zone_a[i];
                let color_a = crate::cell::get_zone_color(match zone_a {
                    0 => crate::cell::AdhesionZone::ZoneA,
                    1 => crate::cell::AdhesionZone::ZoneB,
                    _ => crate::cell::AdhesionZone::ZoneC,
                });

                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(radius_a * 0.5))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: color_a,
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    })),
                    Transform::from_translation(anchor_pos_a),
                    Visibility::default(),
                    AnchorGizmo {
                        cell_entity: entity_a,
                        adhesion_index: i,
                        is_side_a: true,
                    },
                ));
            }

            // Create anchor for cell B
            if !existing_anchors.contains(&(entity_b, i, false)) {
                let pos_b = state.positions[cell_b_idx];
                let rot_b = state.rotations[cell_b_idx];  // Use physics rotation so anchors rotate with cell
                let anchor_dir_b = connections.anchor_direction_b[i];
                let cell_radius_b = state.radii[cell_b_idx];
                let radius_b = cell_radius_b * 0.1; // Small sphere for visualization

                // Transform anchor direction to world space
                let world_anchor_b = rot_b * anchor_dir_b;
                let anchor_pos_b = pos_b + world_anchor_b * cell_radius_b;

                // Get zone color
                let zone_b = connections.zone_b[i];
                let color_b = crate::cell::get_zone_color(match zone_b {
                    0 => crate::cell::AdhesionZone::ZoneA,
                    1 => crate::cell::AdhesionZone::ZoneB,
                    _ => crate::cell::AdhesionZone::ZoneC,
                });

                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(radius_b * 0.5))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: color_b,
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    })),
                    Transform::from_translation(anchor_pos_b),
                    Visibility::default(),
                    AnchorGizmo {
                        cell_entity: entity_b,
                        adhesion_index: i,
                        is_side_a: false,
                    },
                ));
            }

            existing_anchors.remove(&(entity_a, i, true));
            existing_anchors.remove(&(entity_b, i, false));
        }
    }

    // Remove anchors that no longer exist
    for (cell_entity, adhesion_idx, is_side_a) in existing_anchors {
        for (anchor_entity, anchor) in anchor_query.iter() {
            if anchor.cell_entity == cell_entity 
                && anchor.adhesion_index == adhesion_idx 
                && anchor.is_side_a == is_side_a {
                commands.entity(anchor_entity).despawn();
            }
        }
    }
}

/// Update anchor gizmo transforms to follow their parent cells
fn update_anchor_transforms(
    config: Res<RenderingConfig>,
    main_state: Option<Res<crate::simulation::cpu_sim::MainSimState>>,
    preview_state: Option<Res<crate::simulation::preview_sim::PreviewSimState>>,
    sim_state: Res<crate::simulation::SimulationState>,
    cells_query: Query<&Visibility, (With<Cell>, Without<AnchorGizmo>)>,
    mut anchor_query: Query<(&AnchorGizmo, &mut Transform, &mut Visibility), Without<Cell>>,
) {
    if !config.show_orientation_gizmos {
        return;
    }

    // Get the appropriate state based on simulation mode
    let (state, connections) = match sim_state.mode {
        crate::simulation::SimulationMode::Cpu => {
            if let Some(main) = main_state.as_ref() {
                (&main.canonical_state, &main.canonical_state.adhesion_connections)
            } else {
                return;
            }
        }
        crate::simulation::SimulationMode::Preview => {
            if let Some(preview) = preview_state.as_ref() {
                (&preview.canonical_state, &preview.canonical_state.adhesion_connections)
            } else {
                return;
            }
        }
    };

    // Update each anchor's position
    for (anchor, mut transform, mut anchor_visibility) in anchor_query.iter_mut() {
        let i = anchor.adhesion_index;
        
        if i >= connections.active_count || connections.is_active[i] == 0 {
            continue;
        }

        let cell_a_idx = connections.cell_a_index[i];
        let cell_b_idx = connections.cell_b_index[i];

        if cell_a_idx >= state.cell_count || cell_b_idx >= state.cell_count {
            continue;
        }

        // Check parent cell visibility and update anchor visibility
        if let Ok(cell_visibility) = cells_query.get(anchor.cell_entity) {
            let new_visibility = if *cell_visibility == Visibility::Hidden {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
            if *anchor_visibility != new_visibility {
                *anchor_visibility = new_visibility;
            }
        }

        // Use the is_side_a flag to determine which anchor direction to use
        if anchor.is_side_a {
            // This is cell A's anchor
            let pos_a = state.positions[cell_a_idx];
            let rot_a = state.rotations[cell_a_idx];  // Use physics rotation so anchors rotate with cell
            let anchor_dir_a = connections.anchor_direction_a[i];
            let cell_radius_a = state.radii[cell_a_idx];
            let world_anchor_a = rot_a * anchor_dir_a;
            transform.translation = pos_a + world_anchor_a * cell_radius_a;
        } else {
            // This is cell B's anchor
            let pos_b = state.positions[cell_b_idx];
            let rot_b = state.rotations[cell_b_idx];  // Use physics rotation so anchors rotate with cell
            let anchor_dir_b = connections.anchor_direction_b[i];
            let cell_radius_b = state.radii[cell_b_idx];
            let world_anchor_b = rot_b * anchor_dir_b;
            transform.translation = pos_b + world_anchor_b * cell_radius_b;
        }
    }
}
