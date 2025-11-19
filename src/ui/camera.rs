use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

/// Plugin for camera control - Space Engineers style 6DOF camera
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraConfig>()
            .init_resource::<CameraState>()
            .add_systems(Update, (
                camera_mouse_grab,
                camera_mouse_look,
                camera_movement,
                camera_roll,
            ).chain());
    }
}

/// Marker for the main camera
#[derive(Component)]
pub struct MainCamera;

/// Camera configuration (matches original Biospheres settings)
#[derive(Resource)]
pub struct CameraConfig {
    pub move_speed: f32,
    pub sprint_multiplier: f32,
    pub mouse_sensitivity: f32,
    pub roll_speed: f32,
    pub invert_look: bool,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            move_speed: 15.0,           // Base movement speed
            sprint_multiplier: 3.0,     // Sprint speed multiplier
            mouse_sensitivity: 0.1,     // Mouse look sensitivity
            roll_speed: 90.0,           // Roll degrees per second
            invert_look: false,         // Invert mouse Y axis
        }
    }
}

/// Camera runtime state
#[derive(Resource, Default)]
pub struct CameraState {
    pub is_dragging: bool,
}

/// System to handle mouse grab for camera rotation (right-click to look)
fn camera_mouse_grab(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut camera_state: ResMut<CameraState>,
    mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if mouse_button.just_pressed(MouseButton::Right) {
        camera_state.is_dragging = true;
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
    }

    if mouse_button.just_released(MouseButton::Right) {
        camera_state.is_dragging = false;
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
    }
}

/// System to handle mouse look (Space Engineers style - camera-relative rotations)
fn camera_mouse_look(
    camera_state: Res<CameraState>,
    config: Res<CameraConfig>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    if !camera_state.is_dragging {
        mouse_motion.clear();
        return;
    }

    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };

    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    let x_offset = delta.x * config.mouse_sensitivity;
    let mut y_offset = -delta.y * config.mouse_sensitivity; // Invert for natural look

    if config.invert_look {
        y_offset = -y_offset;
    }

    // Get current camera orientation vectors
    let right = transform.right();
    let up = transform.up();

    // Apply camera-relative rotations (true 6DOF - no world constraints)
    // Rotate around camera's current up axis (horizontal mouse movement)
    let yaw_rotation = Quat::from_axis_angle(*up, -x_offset.to_radians());

    // Rotate around camera's current right axis (vertical mouse movement)
    let pitch_rotation = Quat::from_axis_angle(*right, y_offset.to_radians());

    // Apply rotations
    transform.rotation = pitch_rotation * yaw_rotation * transform.rotation;
}

/// System to handle WASD + Space/C movement (camera-relative)
fn camera_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<CameraConfig>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };

    // Calculate velocity with sprint modifier
    let mut velocity = config.move_speed * time.delta_secs();
    if keyboard.pressed(KeyCode::ShiftLeft) {
        velocity *= config.sprint_multiplier;
    }

    // Build movement direction (all relative to camera orientation)
    let mut move_direction = Vec3::ZERO;

    // Forward/backward (W/S)
    if keyboard.pressed(KeyCode::KeyW) {
        move_direction += *transform.forward();
    }
    if keyboard.pressed(KeyCode::KeyS) {
        move_direction -= *transform.forward();
    }

    // Left/right (A/D)
    if keyboard.pressed(KeyCode::KeyA) {
        move_direction -= *transform.right();
    }
    if keyboard.pressed(KeyCode::KeyD) {
        move_direction += *transform.right();
    }

    // Up/down (Space/C) - relative to camera's up
    if keyboard.pressed(KeyCode::Space) {
        move_direction += *transform.up();
    }
    if keyboard.pressed(KeyCode::KeyC) {
        move_direction -= *transform.up();
    }

    // Apply movement
    if move_direction.length_squared() > 0.0 {
        transform.translation += move_direction.normalize() * velocity;
    }
}

/// System to handle Q/E roll controls
fn camera_roll(
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<CameraConfig>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };

    let roll_amount = config.roll_speed * time.delta_secs();

    // Q rolls counter-clockwise (negative around forward)
    if keyboard.pressed(KeyCode::KeyQ) {
        let forward = transform.forward();
        let roll_rotation = Quat::from_axis_angle(*forward, -roll_amount.to_radians());
        transform.rotation = roll_rotation * transform.rotation;
    }

    // E rolls clockwise (positive around forward)
    if keyboard.pressed(KeyCode::KeyE) {
        let forward = transform.forward();
        let roll_rotation = Quat::from_axis_angle(*forward, roll_amount.to_radians());
        transform.rotation = roll_rotation * transform.rotation;
    }
}
