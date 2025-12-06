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
            .init_resource::<FocalPlaneSettings>()
            .init_resource::<ModeNotification>()
            .add_systems(Update, (
                detect_double_click_and_snap,
                camera_mouse_grab,
                camera_update,
                focal_plane_input,
                update_focal_plane_visibility,
                follow_entity_system,
                update_camera_fov,
                render_mode_notification,
            ).chain());
    }
}

/// Resource for displaying centered fading notifications
#[derive(Resource, Default)]
pub struct ModeNotification {
    /// The message to display
    pub message: String,
    /// Time remaining to display (seconds)
    pub time_remaining: f32,
    /// Total display duration for calculating fade
    pub total_duration: f32,
}

impl ModeNotification {
    /// Show a notification message
    pub fn show(&mut self, message: impl Into<String>, duration: f32) {
        self.message = message.into();
        self.time_remaining = duration;
        self.total_duration = duration;
    }
}

/// Focal plane settings for cross-section viewing
/// Hides cells between the camera and the plane, showing only what's beyond
#[derive(Resource)]
pub struct FocalPlaneSettings {
    /// Whether the focal plane is enabled
    pub enabled: bool,
    /// Distance from camera to the focal plane
    pub distance: f32,
    /// Speed of distance adjustment with scroll wheel
    pub scroll_speed: f32,
    /// Minimum distance from camera
    pub min_distance: f32,
    /// Maximum distance from camera
    pub max_distance: f32,
}

impl Default for FocalPlaneSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            distance: 20.0,
            scroll_speed: 2.0,
            min_distance: 1.0,
            max_distance: 200.0,
        }
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
    pub target_distance: f32,
    pub rotation: Quat,
    pub target_rotation: Quat,
    pub mode: CameraMode,
    pub followed_entity: Option<Entity>,
}

/// Spawn the orbit camera
pub fn setup_camera(mut commands: Commands, config: Res<CameraConfig>) {
    // Initial rotation: looking down at the scene from a 45-degree angle
    // Rotate around X axis by -45 degrees (look down) and Y axis by 0 degrees (straight on)
    let initial_rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4);
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: config.fov.to_radians(),
            ..default()
        }),
        MainCamera {
            center: Vec3::ZERO, // Orbit center is always world origin
            distance: 50.0, // Start with some distance from origin
            target_distance: 50.0, // Target distance for spring interpolation
            rotation: initial_rotation, // Start with a nice angle
            target_rotation: initial_rotation,
            mode: CameraMode::Orbit, // Start in orbit mode
            followed_entity: None, // Not following any entity initially
        },
        Transform::IDENTITY,
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
    // Spring settings for orbit mode
    pub enable_spring: bool,
    pub spring_stiffness: f32,
    pub spring_damping: f32,
    // Field of view in degrees
    pub fov: f32,
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
            enable_spring: true,        // Enable spring smoothing by default
            spring_stiffness: 16.0,     // Spring stiffness for smooth camera movement
            spring_damping: 0.7,        // Spring damping (higher = less oscillation)
            fov: 70.0,                  // Field of view in degrees
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
        
        // Safety: if we're marked as dragging but the control button isn't pressed,
        // reset the state. This handles mode switching while dragging.
        if camera_state.is_dragging && !mouse_button.pressed(control_button) {
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
    mut query: Query<(&mut Transform, &mut MainCamera)>,
    mut notification: ResMut<ModeNotification>,
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
                cam.target_distance = 0.0;
                cam.target_rotation = cam.rotation; // Sync target with current
                cam.mode = CameraMode::FreeFly;
                cam.followed_entity = None; // Stop following when switching to free fly
                notification.show("Free-Fly Mode", 1.5);
            }
            CameraMode::FreeFly => {
                // Switch to Orbit: set orbit center to world origin, calculate distance
                let current_pos = transform.translation;
                cam.center = Vec3::ZERO; // Always orbit around world origin
                let new_distance = current_pos.distance(Vec3::ZERO);
                cam.distance = new_distance;
                cam.target_distance = new_distance;
                cam.target_rotation = cam.rotation; // Sync target with current
                cam.mode = CameraMode::Orbit;
                notification.show("Orbit Mode", 1.5);
            }
        }
    }

    // -------------------------------
    // 1. ZOOM (scroll) - Only in Orbit mode
    // -------------------------------
    if cam.mode == CameraMode::Orbit && !imgui_capture.want_capture_mouse && mouse_scroll.delta.y.abs() > 0.001 {
        // Additive zoom - constant speed regardless of distance (doubled multiplier)
        cam.target_distance -= mouse_scroll.delta.y * config.zoom_speed * 30.0;
        cam.target_distance = cam.target_distance.max(0.1); // Don't allow too close to origin
    }
    
    // Apply spring interpolation to distance and rotation in orbit mode
    if cam.mode == CameraMode::Orbit {
        if config.enable_spring {
            // Spring for distance
            let distance_error = cam.target_distance - cam.distance;
            let velocity = distance_error * config.spring_stiffness * dt;
            cam.distance += velocity * (1.0 - config.spring_damping);
            
            // Spring for rotation (slerp with spring-like behavior)
            cam.rotation = cam.rotation.slerp(cam.target_rotation, config.spring_stiffness * dt * (1.0 - config.spring_damping));
        } else {
            // Instant movement - no spring
            cam.distance = cam.target_distance;
            cam.rotation = cam.target_rotation;
        }
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
            let right = cam.target_rotation * Vec3::X;
            let pitch = Quat::from_axis_angle(right, -delta.y);
            
            // Apply rotations to target
            cam.target_rotation = yaw * pitch * cam.target_rotation;
            cam.target_rotation = cam.target_rotation.normalize();
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



/// System to detect double-click and snap to cell
fn detect_double_click_and_snap(
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<crate::input::cell_dragging::DragState>,
    mut camera_query: Query<(&Camera, &GlobalTransform, &mut MainCamera)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    cell_query: Query<(Entity, &crate::cell::CellPosition, &crate::cell::Cell)>,
    imgui_capture: Res<ImGuiWantCapture>,
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

/// System to update camera FOV when config changes
fn update_camera_fov(
    config: Res<CameraConfig>,
    mut camera_query: Query<&mut Projection, With<MainCamera>>,
) {
    if !config.is_changed() {
        return;
    }
    
    for mut projection in camera_query.iter_mut() {
        if let Projection::Perspective(ref mut perspective) = *projection {
            perspective.fov = config.fov.to_radians();
        }
    }
}

/// System to handle focal plane input (F key toggle, scroll wheel for distance)
fn focal_plane_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut focal_plane: ResMut<FocalPlaneSettings>,
    camera_query: Query<&MainCamera>,
    imgui_capture: Res<ImGuiWantCapture>,
    mut notification: ResMut<ModeNotification>,
) {
    let Ok(cam) = camera_query.single() else {
        return;
    };
    
    // Only allow focal plane in FreeFly mode
    if cam.mode != CameraMode::FreeFly {
        // Auto-disable when switching out of FreeFly mode
        if focal_plane.enabled {
            focal_plane.enabled = false;
        }
        return;
    }
    
    // Toggle focal plane with F key (allow even when ImGui wants keyboard for this critical feature)
    if keyboard.just_pressed(KeyCode::KeyF) && !imgui_capture.want_capture_keyboard {
        focal_plane.enabled = !focal_plane.enabled;
        if focal_plane.enabled {
            notification.show("Focal Slice Enabled", 1.5);
        } else {
            notification.show("Focal Slice Disabled", 1.5);
        }
    }
    
    // Adjust distance with scroll wheel when focal plane is enabled
    if focal_plane.enabled && !imgui_capture.want_capture_mouse && mouse_scroll.delta.y.abs() > 0.001 {
        focal_plane.distance += mouse_scroll.delta.y * focal_plane.scroll_speed;
        focal_plane.distance = focal_plane.distance.clamp(focal_plane.min_distance, focal_plane.max_distance);
    }
}

/// System to update cell visibility based on focal plane
/// Hides cells that are on or behind the plane (between camera and plane)
/// Shows cells that are beyond the plane (away from camera)
fn update_focal_plane_visibility(
    focal_plane: Res<FocalPlaneSettings>,
    camera_query: Query<(&Transform, &MainCamera)>,
    mut cell_query: Query<(&crate::cell::CellPosition, &crate::cell::Cell, &mut Visibility)>,
) {
    let Ok((camera_transform, cam)) = camera_query.single() else {
        return;
    };
    
    // If focal plane is disabled or not in FreeFly mode, make all cells visible
    if !focal_plane.enabled || cam.mode != CameraMode::FreeFly {
        for (_, _, mut visibility) in cell_query.iter_mut() {
            if *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
        }
        return;
    }
    
    // Calculate the focal plane position and normal
    let camera_pos = camera_transform.translation;
    let camera_forward = camera_transform.rotation * Vec3::NEG_Z; // Camera looks down -Z
    let plane_center = camera_pos + camera_forward * focal_plane.distance;
    
    for (cell_pos, cell, mut visibility) in cell_query.iter_mut() {
        // Calculate signed distance from cell center to the plane
        // Positive = in front of plane (away from camera), Negative = behind plane (toward camera)
        let to_cell = cell_pos.position - plane_center;
        let signed_distance = to_cell.dot(camera_forward);
        
        // Cell is visible if its nearest edge is beyond the plane (away from camera)
        // Account for cell radius - cell is visible if any part is past the plane
        let should_be_visible = signed_distance + cell.radius > 0.0;
        
        let new_visibility = if should_be_visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        
        if *visibility != new_visibility {
            *visibility = new_visibility;
        }
    }
}

/// System to render the mode notification as a centered fading overlay
fn render_mode_notification(
    time: Res<Time>,
    mut notification: ResMut<ModeNotification>,
    mut imgui_context: NonSendMut<bevy_mod_imgui::ImguiContext>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Update timer
    if notification.time_remaining > 0.0 {
        notification.time_remaining -= time.delta_secs();
    }
    
    // Don't render if no message or time expired
    if notification.time_remaining <= 0.0 || notification.message.is_empty() {
        return;
    }
    
    let ui = imgui_context.ui();
    
    // Get window size for centering
    let Ok(window) = window_query.single() else {
        return;
    };
    let window_width = window.width();
    let window_height = window.height();
    
    // Calculate fade alpha (fade out in last 0.5 seconds)
    let fade_start = 0.5;
    let alpha = if notification.time_remaining < fade_start {
        notification.time_remaining / fade_start
    } else {
        1.0
    };
    
    // Calculate text size for centering
    let text_size = ui.calc_text_size(&notification.message);
    let padding = 20.0;
    let box_width = text_size[0] + padding * 2.0;
    let box_height = text_size[1] + padding * 2.0;
    
    // Center position
    let pos_x = (window_width - box_width) / 2.0;
    let pos_y = (window_height - box_height) / 2.0;
    
    // Style the window
    let bg_color = [0.1, 0.1, 0.1, 0.8 * alpha];
    let text_color = [1.0, 1.0, 1.0, alpha];
    let border_color = [0.4, 0.4, 0.4, 0.6 * alpha];
    
    let _bg_style = ui.push_style_color(imgui::StyleColor::WindowBg, bg_color);
    let _text_style = ui.push_style_color(imgui::StyleColor::Text, text_color);
    let _border_style = ui.push_style_color(imgui::StyleColor::Border, border_color);
    let _rounding = ui.push_style_var(imgui::StyleVar::WindowRounding(8.0));
    let _padding_style = ui.push_style_var(imgui::StyleVar::WindowPadding([padding, padding]));
    
    ui.window("##ModeNotification")
        .position([pos_x, pos_y], imgui::Condition::Always)
        .size([box_width, box_height], imgui::Condition::Always)
        .flags(
            imgui::WindowFlags::NO_TITLE_BAR
                | imgui::WindowFlags::NO_RESIZE
                | imgui::WindowFlags::NO_MOVE
                | imgui::WindowFlags::NO_SCROLLBAR
                | imgui::WindowFlags::NO_SAVED_SETTINGS
                | imgui::WindowFlags::NO_INPUTS
        )
        .build(|| {
            ui.text(&notification.message);
        });
}
