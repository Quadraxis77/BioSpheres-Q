use bevy::prelude::*;
use bevy::{input::mouse::AccumulatedMouseMotion, input::mouse::AccumulatedMouseScroll};
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

/// Plugin for camera control - Space Engineers style 6DOF camera
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraConfig>()
            .init_resource::<CameraState>()
            .init_resource::<ImGuiWantCapture>()
            .init_resource::<OrbitReferenceState>()
            .add_systems(Update, (
                detect_double_click_and_snap,
                camera_mouse_grab,
                camera_update,
                follow_entity_system,
                update_orbit_reference_ball,
            ).chain());
    }
}

/// Camera mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    Orbit,
    FreeFly,
}

/// Component marking our orbital camera
#[derive(Component)]
pub struct MainCamera {
    pub center: Vec3,
    pub distance: f32,
    pub rotation: Quat,
    pub mode: CameraMode,
    pub followed_entity: Option<Entity>,
}

/// Component marking the orbit reference ball
#[derive(Component)]
pub struct OrbitReferenceBall;

/// Resource to track orbit reference ball fade state
#[derive(Resource)]
pub struct OrbitReferenceState {
    pub alpha: f32,
    pub time_since_change: f32,
    pub fade_delay: f32,
    pub fade_duration: f32,
}

impl Default for OrbitReferenceState {
    fn default() -> Self {
        Self {
            alpha: 0.0,
            time_since_change: 999.0, // Start faded out
            fade_delay: 1.0,           // Wait 1 second before fading
            fade_duration: 1.0,        // Fade over 1 second
        }
    }
}

/// Spawn the orbit camera
pub fn setup_camera(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        Camera3d::default(),
        MainCamera {
            center: Vec3::ZERO, // Orbit center is always world origin
            distance: 50.0, // Start with some distance from origin
            rotation: Quat::from_rotation_x(-0.5) * Quat::from_rotation_y(0.5), // Start with a nice angle
            mode: CameraMode::Orbit, // Start in orbit mode
            followed_entity: None, // Not following any entity initially
        },
        Transform::IDENTITY,
    ));
    
    // Spawn orbit reference ball at origin
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.5, 0.8, 1.0, 0.3), // Start visible in orbit mode
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO),
        OrbitReferenceBall,
    ));
}


/// Camera configuration (matches original Biospheres settings)
#[derive(Resource)]
pub struct CameraConfig {
    pub move_speed: f32,
    pub sprint_multiplier: f32,
    pub mouse_sensitivity: f32,
    pub roll_speed: f32,
    pub invert_look: bool,
    pub zoom_speed: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            move_speed: 15.0,           // Base movement speed
            sprint_multiplier: 3.0,     // Sprint speed multiplier
            mouse_sensitivity: 0.003,     // Mouse look sensitivity
            roll_speed: 1.5,           // Roll radians per second
            invert_look: false,         // Invert mouse Y axis
            zoom_speed: 0.2,
        }
    }
}

/// Camera runtime state
#[derive(Resource, Default)]
pub struct CameraState {
    pub is_dragging: bool,
}

/// Resource to track whether ImGui wants to capture input
#[derive(Resource, Default)]
pub struct ImGuiWantCapture {
    pub want_capture_mouse: bool,
    pub want_capture_keyboard: bool,
}

/// System to handle mouse grab for camera rotation (middle-click for orbit, right-click for free fly)
fn camera_mouse_grab(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut camera_state: ResMut<CameraState>,
    mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    camera_query: Query<&MainCamera>,
) {
    if let Ok(cam) = camera_query.single() {
        let control_button = match cam.mode {
            CameraMode::Orbit => MouseButton::Middle,
            CameraMode::FreeFly => MouseButton::Right,
        };
        
        if mouse_button.just_pressed(control_button) {
            camera_state.is_dragging = true;
            cursor_options.grab_mode = CursorGrabMode::Locked;
            cursor_options.visible = false;
        }

        if mouse_button.just_released(control_button) {
            camera_state.is_dragging = false;
            cursor_options.grab_mode = CursorGrabMode::None;
            cursor_options.visible = true;
        }
    }
}

pub fn camera_update(
    time: Res<Time>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<CameraConfig>,
    imgui_capture: Res<ImGuiWantCapture>,
    mut orbit_ref_state: ResMut<OrbitReferenceState>,
    mut query: Query<(&mut Transform, &mut MainCamera)>,
) {
    let dt = time.delta_secs();

    // Gracefully handle case when no MainCamera exists (e.g., GPU scene)
    let Ok((mut transform, mut cam)) = query.single_mut() else {
        return;
    };

    // -------------------------------
    // 0. MODE SWITCHING (Tab key)
    // -------------------------------
    // Allow Tab to work even when ImGui wants keyboard (camera mode is critical)
    if keyboard.just_pressed(KeyCode::Tab) {
        match cam.mode {
            CameraMode::Orbit => {
                // Switch to FreeFly: move orbit center to current camera position
                cam.center = transform.translation;
                cam.distance = 0.0;
                cam.mode = CameraMode::FreeFly;
                cam.followed_entity = None; // Stop following when switching to free fly
                orbit_ref_state.alpha = 0.0; // Hide reference ball
            }
            CameraMode::FreeFly => {
                // Switch to Orbit: set orbit center to world origin, calculate distance
                let current_pos = transform.translation;
                cam.center = Vec3::ZERO; // Always orbit around world origin
                cam.distance = current_pos.distance(Vec3::ZERO);
                cam.mode = CameraMode::Orbit;
                orbit_ref_state.time_since_change = 0.0;
                orbit_ref_state.alpha = 0.3; // Show reference ball
            }
        }
    }

    // -------------------------------
    // 1. ZOOM (scroll) - Only in Orbit mode
    // -------------------------------
    if cam.mode == CameraMode::Orbit && !imgui_capture.want_capture_mouse && mouse_scroll.delta.y.abs() > 0.001 {
        // Additive zoom - constant speed regardless of distance
        cam.distance -= mouse_scroll.delta.y * config.zoom_speed * 15.0;
        cam.distance = cam.distance.max(0.1); // Don't allow too close to origin
        
        // Reset fade timer when orbit distance changes
        orbit_ref_state.time_since_change = 0.0;
        orbit_ref_state.alpha = 0.3; // Set to visible
    }

    // -------------------------------
    // 2. ROTATION (mouse)
    // -------------------------------
    let control_button = match cam.mode {
        CameraMode::Orbit => MouseButton::Middle,
        CameraMode::FreeFly => MouseButton::Right,
    };
    
    if mouse_buttons.pressed(control_button) {
        let delta = mouse_motion.delta * config.mouse_sensitivity;

        if cam.mode == CameraMode::Orbit {
            // Orbit mode: rotate around world origin
            // Horizontal rotation (yaw) around world Y axis
            let yaw = Quat::from_axis_angle(Vec3::Y, -delta.x);
            
            // Vertical rotation (pitch) around camera's local right axis
            let right = cam.rotation * Vec3::X;
            let pitch = Quat::from_axis_angle(right, -delta.y);
            
            // Apply rotations
            cam.rotation = yaw * pitch * cam.rotation;
            cam.rotation = cam.rotation.normalize();
            
            // Show reference ball when orbiting
            orbit_ref_state.time_since_change = 0.0;
            orbit_ref_state.alpha = 0.3;
        } else {
            // FreeFly mode: free rotation
            let pitch = Quat::from_axis_angle(cam.rotation * Vec3::X, -delta.y);
            let local_up = cam.rotation * Vec3::Y;
            let free_yaw = Quat::from_axis_angle(local_up, -delta.x);
            
            cam.rotation = (free_yaw * pitch) * cam.rotation;
            cam.rotation = cam.rotation.normalize();
        }
    }

    // -------------------------------
    // 3. ROLL (Q/E) - Only in FreeFly mode
    // -------------------------------
    if cam.mode == CameraMode::FreeFly && !imgui_capture.want_capture_keyboard {
        let mut roll_amount = 0.0;
        if keyboard.pressed(KeyCode::KeyQ) {
            roll_amount += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyE) {
            roll_amount -= 1.0;
        }

        if roll_amount != 0.0 {
            let roll_axis = cam.rotation * Vec3::Z; // viewing direction local Z
            let roll = Quat::from_axis_angle(roll_axis, roll_amount * config.roll_speed * dt);
            cam.rotation = (roll * cam.rotation).normalize();
        }
    }

    // -------------------------------
    // 4. MOVEMENT (WASD + Space + C) - Only in FreeFly mode
    // -------------------------------
    if cam.mode == CameraMode::FreeFly && !imgui_capture.want_capture_keyboard {
        let mut speed = config.move_speed * dt;
        if keyboard.pressed(KeyCode::ShiftLeft) {
            speed *= config.sprint_multiplier;
        }

        let mut move_vec = Vec3::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            move_vec += cam.rotation * Vec3::Z * -1.0; // forward
        }
        if keyboard.pressed(KeyCode::KeyS) {
            move_vec += cam.rotation * Vec3::Z; // backward
        }
        if keyboard.pressed(KeyCode::KeyA) {
            move_vec += cam.rotation * Vec3::X * -1.0; // left
        }
        if keyboard.pressed(KeyCode::KeyD) {
            move_vec += cam.rotation * Vec3::X; // right
        }
        if keyboard.pressed(KeyCode::Space) {
            move_vec += cam.rotation * Vec3::Y; // up
        }
        if keyboard.pressed(KeyCode::KeyC) {
            move_vec += cam.rotation * Vec3::Y * -1.0; // down
        }

        if move_vec.length_squared() > 0.0 {
            cam.center += move_vec.normalize() * speed;
        }
    }

    // -------------------------------
    // 5. Write camera transform
    // -------------------------------
    let offset = cam.rotation * Vec3::new(0.0, 0.0, cam.distance);
    transform.translation = cam.center + offset;
    transform.rotation = cam.rotation;
}

/// System to update the orbit reference ball visibility and position
fn update_orbit_reference_ball(
    time: Res<Time>,
    mut orbit_ref_state: ResMut<OrbitReferenceState>,
    camera_query: Query<&MainCamera>,
    mut ball_query: Query<(&mut Transform, &MeshMaterial3d<StandardMaterial>), With<OrbitReferenceBall>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();
    
    // Update ball position and material
    if let Ok(cam) = camera_query.single() {
        if let Ok((mut ball_transform, material_handle)) = ball_query.single_mut() {
            // Position ball at orbit center
            ball_transform.translation = cam.center;
            
            // Only show in orbit mode
            if cam.mode == CameraMode::Orbit {
                // Update fade timer
                orbit_ref_state.time_since_change += dt;
                
                // Calculate alpha based on fade state
                if orbit_ref_state.time_since_change < orbit_ref_state.fade_delay {
                    // Still visible, no fade yet
                    orbit_ref_state.alpha = 0.3;
                } else {
                    // Start fading
                    let fade_progress = (orbit_ref_state.time_since_change - orbit_ref_state.fade_delay) / orbit_ref_state.fade_duration;
                    orbit_ref_state.alpha = (0.3 * (1.0 - fade_progress.min(1.0))).max(0.0);
                }
            } else {
                // Hide in free fly mode
                orbit_ref_state.alpha = 0.0;
            }
            
            // Update material alpha
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.base_color = Color::srgba(0.5, 0.8, 1.0, orbit_ref_state.alpha);
            }
        }
    }
}

/// System to detect double-click and snap to cell
fn detect_double_click_and_snap(
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<crate::input::cell_dragging::DragState>,
    mut camera_query: Query<(&Camera, &GlobalTransform, &mut MainCamera)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    cell_query: Query<(Entity, &crate::cell::CellPosition, &crate::cell::Cell)>,
    imgui_capture: Res<ImGuiWantCapture>,
    mut orbit_ref_state: ResMut<OrbitReferenceState>,
) {
    // Don't process if ImGui wants to capture mouse
    if imgui_capture.want_capture_mouse {
        return;
    }
    
    // Only process left mouse button clicks
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    
    let current_time = time.elapsed_secs();
    let time_since_last_click = current_time - drag_state.last_click_time;
    
    // Check if this is a double-click
    let is_double_click = time_since_last_click < drag_state.double_click_threshold;
    
    // Update last click time for next detection
    drag_state.last_click_time = current_time;
    
    if !is_double_click {
        return;
    }
    
    // Get camera and window
    let Ok((camera, camera_transform, mut main_camera)) = camera_query.single_mut() else {
        return;
    };
    
    // Only work in orbit mode
    if main_camera.mode != CameraMode::Orbit {
        return;
    }
    
    let Ok(window) = window_query.single() else {
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
    let mut closest_hit: Option<(Entity, f32)> = None;
    
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
                closest_hit = Some((entity, hit_distance));
            }
        }
    }
    
    // If we hit a cell, snap to it
    if let Some((entity, _)) = closest_hit {
        // Set flag to prevent drag system from starting
        drag_state.skip_next_drag = true;
        
        if main_camera.followed_entity == Some(entity) {
            // Double-clicked the same cell - unfollow
            main_camera.followed_entity = None;
            main_camera.center = Vec3::ZERO;
        } else {
            // Follow this cell
            main_camera.followed_entity = Some(entity);
            
            // Get the cell's position and set it as orbit center
            if let Ok((_, cell_pos, _)) = cell_query.get(entity) {
                main_camera.center = cell_pos.position;
            }
        }
        
        // Show reference ball
        orbit_ref_state.time_since_change = 0.0;
        orbit_ref_state.alpha = 0.3;
    }
}

/// System to update camera center to follow entity
fn follow_entity_system(
    mut camera_query: Query<&mut MainCamera>,
    cell_query: Query<&crate::cell::CellPosition>,
) {
    let Ok(mut cam) = camera_query.single_mut() else {
        return;
    };
    
    // Only follow in orbit mode
    if cam.mode != CameraMode::Orbit {
        return;
    }
    
    // If following an entity, update center to its position
    if let Some(followed_entity) = cam.followed_entity {
        if let Ok(cell_pos) = cell_query.get(followed_entity) {
            cam.center = cell_pos.position;
        } else {
            // Entity no longer exists, stop following
            cam.followed_entity = None;
            cam.center = Vec3::ZERO;
        }
    }
}

/// Ray-sphere intersection test (borrowed from cell_dragging.rs)
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
