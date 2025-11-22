use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::cell::{Cell, CellPosition};
use crate::ui::camera::MainCamera;

/// Plugin for cell dragging interaction
pub struct CellDraggingPlugin;

/// System set for cell dragging systems
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CellDraggingSet;

impl Plugin for CellDraggingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
            .add_systems(Update, (
                handle_drag_start,
                handle_drag_update,
                handle_drag_end,
            ).chain().in_set(CellDraggingSet));
    }
}

/// State tracking for cell dragging
#[derive(Resource, Default)]
pub struct DragState {
    pub dragged_entity: Option<Entity>,
    pub drag_offset: Vec3,
    pub drag_plane_normal: Vec3,
    pub drag_plane_distance: f32,
}

/// System to handle starting a drag operation
fn handle_drag_start(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<DragState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    cell_query: Query<(Entity, &CellPosition, &Cell)>,
) {
    // Only start drag on left mouse button press
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Skip if already dragging
    if drag_state.dragged_entity.is_some() {
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
    let mut closest_hit: Option<(Entity, f32, Vec3)> = None;

    for (entity, cell_pos, cell) in cell_query.iter() {
        // Ray-sphere intersection test
        if let Some(hit_distance) = ray_sphere_intersection(
            ray.origin,
            *ray.direction,
            cell_pos.position,
            cell.radius,
        ) {
            // Keep track of closest hit
            if closest_hit.is_none() || hit_distance < closest_hit.unwrap().1 {
                let hit_point = ray.origin + *ray.direction * hit_distance;
                closest_hit = Some((entity, hit_distance, hit_point));
            }
        }
    }

    // If we hit a cell, start dragging it
    if let Some((entity, _, _)) = closest_hit {
        let cell_pos = cell_query.get(entity).unwrap().1;
        
        // Calculate drag plane perpendicular to camera forward
        let camera_forward = camera_transform.forward();
        let drag_plane_normal = Vec3::from(*camera_forward);
        let drag_plane_distance = cell_pos.position.dot(drag_plane_normal);
        
        // Project the cell center onto the drag plane to find where we should track from
        // This ensures the cell center stays under the cursor, not the surface hit point
        let ray_to_plane = ray_plane_intersection(
            ray.origin,
            *ray.direction,
            drag_plane_normal,
            drag_plane_distance,
        );
        
        let drag_offset = if let Some(plane_point) = ray_to_plane {
            plane_point - cell_pos.position
        } else {
            Vec3::ZERO
        };
        
        drag_state.dragged_entity = Some(entity);
        drag_state.drag_offset = drag_offset;
        drag_state.drag_plane_normal = drag_plane_normal;
        drag_state.drag_plane_distance = drag_plane_distance;
    }
}

/// System to update dragged cell position
fn handle_drag_update(
    drag_state: Res<DragState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut cell_query: Query<&mut CellPosition>,
    sim_state: Res<crate::simulation::SimulationState>,
    mut main_sim_state: Option<ResMut<crate::simulation::cpu_sim::MainSimState>>,
    mut preview_sim_state: Option<ResMut<crate::simulation::preview_sim::PreviewSimState>>,
) {
    // Skip if not dragging
    let Some(dragged_entity) = drag_state.dragged_entity else {
        return;
    };

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

    // Intersect ray with drag plane
    let Some(plane_hit) = ray_plane_intersection(
        ray.origin,
        *ray.direction,
        drag_state.drag_plane_normal,
        drag_state.drag_plane_distance,
    ) else {
        return;
    };

    // Calculate new position
    let new_position = plane_hit - drag_state.drag_offset;

    // Update cell position in ECS
    if let Ok(mut cell_pos) = cell_query.get_mut(dragged_entity) {
        cell_pos.position = new_position;
        cell_pos.velocity = Vec3::ZERO;
    }

    // Update the canonical state based on current simulation mode
    match sim_state.mode {
        crate::simulation::SimulationMode::Cpu => {
            if let Some(ref mut main_state) = main_sim_state {
                if let Some(&cell_index) = main_state.entity_to_index.get(&dragged_entity) {
                    if cell_index < main_state.canonical_state.cell_count {
                        main_state.canonical_state.positions[cell_index] = new_position;
                        main_state.canonical_state.velocities[cell_index] = Vec3::ZERO;
                        main_state.canonical_state.prev_positions[cell_index] = new_position;
                    }
                }
            }
        }
        crate::simulation::SimulationMode::Preview => {
            if let Some(ref mut preview_state) = preview_sim_state {
                // Find the cell index in preview canonical state by matching entity
                for i in 0..preview_state.canonical_state.cell_count {
                    if let Some(entity) = preview_state.index_to_entity[i] {
                        if entity == dragged_entity {
                            preview_state.canonical_state.positions[i] = new_position;
                            preview_state.canonical_state.velocities[i] = Vec3::ZERO;
                            preview_state.canonical_state.prev_positions[i] = new_position;
                            break;
                        }
                    }
                }
            }
        }
        crate::simulation::SimulationMode::Gpu => {
            // GPU mode not yet implemented
        }
    }
}

/// System to handle ending a drag operation
fn handle_drag_end(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<DragState>,
) {
    // End drag on left mouse button release
    if mouse_button.just_released(MouseButton::Left) {
        drag_state.dragged_entity = None;
    }
}

/// Ray-sphere intersection test
/// Returns the distance along the ray to the intersection point, or None if no hit
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
/// Returns the point of intersection, or None if ray is parallel to plane
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
