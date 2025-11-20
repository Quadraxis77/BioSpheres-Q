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
                camera_mouse_grab,
                camera_update,
                update_orbit_reference_ball,
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
            center: Vec3::ZERO,
            distance: 0.0, // Start at center (no orbit)
            rotation: Quat::IDENTITY,
        },
        Transform::IDENTITY,
    ));
    
    // Spawn orbit reference ball at origin
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.5, 0.8, 1.0, 0.0), // Start invisible
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
    mut orbit_ref_state: ResMut<OrbitReferenceState>,
    mut query: Query<(&mut Transform, &mut MainCamera)>,
) {
    let dt = time.delta_secs();

    let (mut transform, mut cam) = query.single_mut().unwrap();

    // -------------------------------
    // 0. RESET ORBIT (middle click)
    // -------------------------------
    // Only process if ImGui is not capturing the mouse
    if !imgui_capture.want_capture_mouse && mouse_buttons.just_pressed(MouseButton::Middle) {
        cam.distance = 0.0; // Reset to center
        orbit_ref_state.time_since_change = 0.0; // Show reference ball
        orbit_ref_state.alpha = 0.3; // Set to visible
    }

    // -------------------------------
    // 1. ZOOM (scroll)
    // -------------------------------
    // Only process scroll if ImGui is not capturing the mouse
    if !imgui_capture.want_capture_mouse && mouse_scroll.delta.y.abs() > 0.001 {
        // Additive zoom - constant speed regardless of distance
        cam.distance -= mouse_scroll.delta.y * config.zoom_speed * 5.0;
        cam.distance = cam.distance.max(0.0); // Don't allow negative distance
        
        // Reset fade timer when orbit distance changes
        orbit_ref_state.time_since_change = 0.0;
        orbit_ref_state.alpha = 0.3; // Set to visible
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
    // Only process keyboard input if ImGui is not capturing it
    if !imgui_capture.want_capture_keyboard {
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
    
    // Update ball position and material
    if let Ok(cam) = camera_query.single() {
        if let Ok((mut ball_transform, material_handle)) = ball_query.single_mut() {
            // Position ball at camera center (orbit point)
            ball_transform.translation = cam.center;
            
            // Update material alpha
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.base_color = Color::srgba(0.5, 0.8, 1.0, orbit_ref_state.alpha);
            }
        }
    }
}
