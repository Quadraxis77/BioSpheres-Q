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
            .add_systems(Update, (
                camera_mouse_grab,
                camera_update
            ).chain());
    }
}

/// Component marking our orbital camera
#[derive(Component)]
pub struct MainCamera {
    pub center: Vec3,
    pub distance: f32,
    pub rotation: Quat,
}

/// Spawn the orbit camera
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        MainCamera {
            center: Vec3::ZERO,
            distance: 8.0,
            rotation: Quat::IDENTITY,
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

/// Resource to track whether ImGui wants to capture mouse input
#[derive(Resource, Default)]
pub struct ImGuiWantCapture {
    pub want_capture_mouse: bool,
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

pub fn camera_update(
    time: Res<Time>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<CameraConfig>,
    imgui_capture: Res<ImGuiWantCapture>,
    mut query: Query<(&mut Transform, &mut MainCamera)>,
) {
    let dt = time.delta_secs();

    let (mut transform, mut cam) = query.single_mut().unwrap();

    // -------------------------------
    // 1. ZOOM (scroll)
    // -------------------------------
    // Only process scroll if ImGui is not capturing the mouse
    if !imgui_capture.want_capture_mouse {
        cam.distance *= 1.0 - mouse_scroll.delta.y * config.zoom_speed;
        cam.distance = cam.distance.max(0.01); // prevent inversion
    }

    // -------------------------------
    // 2. ROTATION (mouse)
    // -------------------------------
    if mouse_buttons.pressed(MouseButton::Right) {
        let delta = mouse_motion.delta * config.mouse_sensitivity;

        // Rotate around camera's local axes:
        // Y movement-> pitch around local right
        let pitch = Quat::from_axis_angle(cam.rotation * Vec3::X, -delta.y);

        // Instead of Y-up yaw, we use free rotation:
        // yaw axis is camera's current local up:
        let local_up = cam.rotation * Vec3::Y;
        let free_yaw = Quat::from_axis_angle(local_up, -delta.x);

        // Apply in free-rotation order
        cam.rotation = (free_yaw * pitch) * cam.rotation;
        cam.rotation = cam.rotation.normalize();
    }

    // -------------------------------
    // 3. ROLL (Q/E)
    // -------------------------------
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

    // -------------------------------
    // 4. MOVE ORBIT CENTER (WASD + Space + C)
    // -------------------------------
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

    // -------------------------------
    // 5. Write camera transform
    // -------------------------------
    let offset = cam.rotation * Vec3::new(0.0, 0.0, cam.distance);
    transform.translation = cam.center + offset;
    transform.rotation = cam.rotation;
}