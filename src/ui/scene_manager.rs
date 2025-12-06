use bevy::prelude::*;
use bevy_mod_imgui::prelude::*;
use crate::simulation::{SimulationState, SimulationMode, CpuSceneState, PreviewSceneState, CpuSceneEntity};

/// Event to trigger scene reset
#[derive(Message)]
pub struct ResetSceneEvent;

/// Resource to track Scene Manager window state
#[derive(Resource)]
pub struct SceneManagerState {
    pub window_open: bool,
    pub show_exit_confirmation: bool,
}

impl Default for SceneManagerState {
    fn default() -> Self {
        Self {
            window_open: true,
            show_exit_confirmation: false,
        }
    }
}

/// Resource to store CPU scene cell capacity setting
#[derive(Resource)]
pub struct CpuCellCapacity {
    pub capacity: usize,
}

impl Default for CpuCellCapacity {
    fn default() -> Self {
        Self {
            capacity: 4096,
        }
    }
}

/// Scene Manager plugin for managing scene transitions and time controls
pub struct SceneManagerPlugin;

impl Plugin for SceneManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneManagerState>()
            .init_resource::<CpuCellCapacity>()
            .add_message::<ResetSceneEvent>()
            .add_systems(Update, render_scene_manager_window)
            .add_systems(Update, handle_reset_scene_event);
    }
}

/// Main Scene Manager window rendering system
fn render_scene_manager_window(
    mut imgui_context: NonSendMut<ImguiContext>,
    mut scene_manager_state: ResMut<SceneManagerState>,
    mut simulation_state: ResMut<SimulationState>,
    mut app_exit_events: MessageWriter<AppExit>,
    cpu_scene_state: Option<Res<State<CpuSceneState>>>,
    preview_scene_state: Option<Res<State<PreviewSceneState>>>,
    next_cpu_scene_state: Option<ResMut<NextState<CpuSceneState>>>,
    next_preview_scene_state: Option<ResMut<NextState<PreviewSceneState>>>,
    mut reset_scene_events: MessageWriter<ResetSceneEvent>,
    global_ui_state: Res<super::GlobalUiState>,
) {
    // Early return if states aren't initialized yet
    let Some(cpu_scene_state) = cpu_scene_state else { return };
    let Some(preview_scene_state) = preview_scene_state else { return };
    let Some(mut next_cpu_scene_state) = next_cpu_scene_state else { return };
    let Some(mut next_preview_scene_state) = next_preview_scene_state else { return };

    let ui = imgui_context.ui();

    // Only render if window is open
    if !scene_manager_state.window_open {
        return;
    }

    // Only show if visibility is enabled
    if !global_ui_state.show_scene_manager {
        return;
    }

    // Build flags based on lock state
    use imgui::WindowFlags;
    let flags = if global_ui_state.windows_locked {
        WindowFlags::NO_MOVE | WindowFlags::NO_RESIZE
    } else {
        WindowFlags::empty()
    };

    ui.window("Scene Manager")
        .position([1704.0, 26.0], Condition::FirstUseEver)
        .size([212.0, 321.0], Condition::FirstUseEver)
        .size_constraints([250.0, 150.0], [f32::MAX, f32::MAX])
        .collapsible(true)
        .flags(flags)
        .build(|| {
            // Exit button at the top in red
            let red = [0.8, 0.2, 0.2, 1.0];
            let red_hovered = [1.0, 0.3, 0.3, 1.0];
            let red_active = [0.6, 0.1, 0.1, 1.0];
            
            let _button_color = ui.push_style_color(StyleColor::Button, red);
            let _button_hovered = ui.push_style_color(StyleColor::ButtonHovered, red_hovered);
            let _button_active = ui.push_style_color(StyleColor::ButtonActive, red_active);
            
            if ui.button("Exit Application") {
                scene_manager_state.show_exit_confirmation = true;
            }
            
            ui.separator();
            
            ui.text("Scene Selection");
            ui.separator();
            
            // Scene selection using selectable items (radio button behavior)
            let mut selected_mode = simulation_state.mode;
            
            if ui.selectable_config("Genome Editor")
                .selected(selected_mode == SimulationMode::Preview)
                .build()
            {
                selected_mode = SimulationMode::Preview;
            }
            
            if ui.selectable_config("CPU Scene")
                .selected(selected_mode == SimulationMode::Cpu)
                .build()
            {
                selected_mode = SimulationMode::Cpu;
            }
            
            // Handle scene transition if mode changed
            if selected_mode != simulation_state.mode {
                handle_scene_transition(
                    &mut simulation_state,
                    selected_mode,
                    &cpu_scene_state,
                    &preview_scene_state,
                    &mut next_cpu_scene_state,
                    &mut next_preview_scene_state,
                );
            }
            
            ui.separator();
            
            // Reset scene button (only for CPU scene)
            if simulation_state.mode != SimulationMode::Preview {
                if ui.button("Reset Scene") {
                    reset_scene_events.write(ResetSceneEvent);
                }
                
                ui.separator();
            }
            
            // Time controls section
            ui.text("Time Controls");
            ui.separator();
            
            // Show pause/play toggle for CPU mode
            // Show message for Preview mode
            match simulation_state.mode {
                SimulationMode::Cpu => {
                    // Toggle pause/play button
                    let button_label = if simulation_state.paused { "Play" } else { "Pause" };
                    if ui.button(button_label) {
                        simulation_state.paused = !simulation_state.paused;
                    }
                    
                    ui.spacing();
                    
                    // Simulation speed control
                    ui.text("Simulation Speed");
                    
                    // Speed preset buttons
                    let speed_presets = [
                        ("0.5x", 0.5),
                        ("1x", 1.0),
                        ("2x", 2.0),
                    ];
                    
                    for (i, (label, speed)) in speed_presets.iter().enumerate() {
                        if i > 0 {
                            ui.same_line();
                        }
                        
                        let is_current = (simulation_state.speed_multiplier - speed).abs() < 0.01;
                        if is_current {
                            let _style = ui.push_style_color(StyleColor::Button, [0.0, 0.5, 0.8, 1.0]);
                            ui.button(label);
                        } else if ui.button(label) {
                            simulation_state.speed_multiplier = *speed;
                        }
                    }
                }
                SimulationMode::Preview => {
                    // Display message for Preview mode
                    ui.text("Time control handled by Time Scrubber");
                }
            }
            
            ui.separator();
            
            // Display current simulation statistics
            // Note: Time tracking now handled by Bevy's Time<Fixed> resource
        });
    
    // Exit confirmation modal
    if scene_manager_state.show_exit_confirmation {
        // Get display size to center the dialog
        let display_size = ui.io().display_size;
        let center_x = display_size[0] * 0.5;
        let center_y = display_size[1] * 0.5;
        
        ui.window("Exit Confirmation")
            .position([center_x, center_y], Condition::Always)
            .position_pivot([0.5, 0.5])
            .size([300.0, 120.0], Condition::Always)
            .collapsible(false)
            .resizable(false)
            .flags(WindowFlags::NO_MOVE | WindowFlags::NO_COLLAPSE)
            .build(|| {
                ui.text("Are you sure you want to exit?");
                ui.spacing();
                ui.separator();
                ui.spacing();
                
                // Center the buttons
                let button_width = 120.0;
                let spacing = 10.0;
                let total_width = button_width * 2.0 + spacing;
                let window_width = 300.0;
                let offset = (window_width - total_width) * 0.5;
                
                ui.set_cursor_pos([offset, ui.cursor_pos()[1]]);
                
                // Yes button (red)
                let red = [0.8, 0.2, 0.2, 1.0];
                let red_hovered = [1.0, 0.3, 0.3, 1.0];
                let red_active = [0.6, 0.1, 0.1, 1.0];
                
                let _button_color = ui.push_style_color(StyleColor::Button, red);
                let _button_hovered = ui.push_style_color(StyleColor::ButtonHovered, red_hovered);
                let _button_active = ui.push_style_color(StyleColor::ButtonActive, red_active);
                
                if ui.button_with_size("Yes", [button_width, 0.0]) {
                    app_exit_events.write(AppExit::Success);
                    scene_manager_state.show_exit_confirmation = false;
                }
                
                drop(_button_color);
                drop(_button_hovered);
                drop(_button_active);
                
                ui.same_line();
                
                // No button (default style)
                if ui.button_with_size("No", [button_width, 0.0]) {
                    scene_manager_state.show_exit_confirmation = false;
                }
            });
    }
}

/// Handle scene transitions when user selects a different scene
fn handle_scene_transition(
    simulation_state: &mut SimulationState,
    new_mode: SimulationMode,
    cpu_scene_state: &State<CpuSceneState>,
    preview_scene_state: &State<PreviewSceneState>,
    next_cpu_scene_state: &mut NextState<CpuSceneState>,
    next_preview_scene_state: &mut NextState<PreviewSceneState>,
) {
    
    let old_mode = simulation_state.mode;
    
    // Deactivate old scene
    match old_mode {
        SimulationMode::Cpu => {
            if **cpu_scene_state == CpuSceneState::Active {
                next_cpu_scene_state.set(CpuSceneState::Inactive);
            }
        }
        SimulationMode::Preview => {
            if **preview_scene_state == PreviewSceneState::Active {
                next_preview_scene_state.set(PreviewSceneState::Inactive);
            }
        }
    }
    
    // Activate new scene
    match new_mode {
        SimulationMode::Cpu => {
            next_cpu_scene_state.set(CpuSceneState::Active);
        }
        SimulationMode::Preview => {
            next_preview_scene_state.set(PreviewSceneState::Active);
        }
    }
    
    // Update simulation mode
    simulation_state.mode = new_mode;
}

/// Handle reset scene events by despawning and respawning only cells
fn handle_reset_scene_event(
    mut reset_events: MessageReader<ResetSceneEvent>,
    mut commands: Commands,
    simulation_state: ResMut<SimulationState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    genome: Res<crate::genome::CurrentGenome>,
    config: Res<crate::cell::physics::PhysicsConfig>,
    cpu_cells: Query<Entity, (With<crate::cell::Cell>, With<CpuSceneEntity>)>,
    preview_cells: Query<Entity, (With<crate::cell::Cell>, With<crate::simulation::preview_sim::PreviewSceneEntity>)>,
    mut main_sim_state: Option<ResMut<crate::simulation::cpu_sim::MainSimState>>,
    mut preview_sim_state: Option<ResMut<crate::simulation::preview_sim::PreviewSimState>>,
    cpu_cell_capacity: Res<CpuCellCapacity>,
) {
    for _ in reset_events.read() {
        // Only despawn and respawn cells, leaving lights, fog, and world sphere intact
        match simulation_state.mode {
            SimulationMode::Cpu => {
                // Despawn only CPU cells
                for entity in cpu_cells.iter() {
                    commands.entity(entity).despawn();
                }
                
                // Reset MainSimState and spawn initial cell
                if let Some(ref mut main_state) = main_sim_state {
                    spawn_cpu_cells_only(&mut commands, &mut meshes, &mut materials, &genome, &config, main_state, &cpu_cell_capacity);
                }
            }
            SimulationMode::Preview => {
                // Despawn only Preview cells
                for entity in preview_cells.iter() {
                    commands.entity(entity).despawn();
                }
                
                // Reset PreviewSimState and spawn initial cell
                if let Some(ref mut preview_state) = preview_sim_state {
                    spawn_preview_cells_only(&mut commands, &mut meshes, &mut materials, &genome, &config, preview_state);
                }
            }
        }
    }
}

/// Spawn only CPU cells (for reset - doesn't spawn lights, fog, or world sphere)
fn spawn_cpu_cells_only(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    genome: &Res<crate::genome::CurrentGenome>,
    config: &Res<crate::cell::physics::PhysicsConfig>,
    main_state: &mut crate::simulation::cpu_sim::MainSimState,
    cpu_cell_capacity: &Res<CpuCellCapacity>,
) {
    use crate::cell::{Cell, CellPosition, CellOrientation, CellSignaling};
    use crate::simulation::{InitialState, InitialCell};

    // Get initial mode settings from genome
    let initial_mode_index = genome.genome.initial_mode.max(0) as usize;
    let mode = genome.genome.modes.get(initial_mode_index)
        .or_else(|| genome.genome.modes.first());
    
    let (color, opacity, split_mass, split_interval, is_test_cell, max_cell_size) = if let Some(mode) = mode {
        (mode.color, mode.opacity, mode.split_mass, mode.split_interval, mode.cell_type == 0, mode.max_cell_size)
    } else {
        (Vec3::new(1.0, 1.0, 1.0), 1.0, 1.0, 5.0, true, 2.0)
    };
    
    // For Test cells, start with half the split mass so they need to grow before splitting
    let initial_mass = if is_test_cell { split_mass * 0.5 } else { split_mass };
    let cell_radius = if is_test_cell { initial_mass.min(max_cell_size).clamp(0.5, 2.0) } else { 1.0 };
    
    // Create initial state with capacity from settings
    let mut initial_state = InitialState::new((**config).clone(), cpu_cell_capacity.capacity, 0);
    initial_state.add_cell(InitialCell {
        id: 0,
        position: Vec3::ZERO,
        velocity: Vec3::ZERO,
        rotation: genome.genome.initial_orientation,
        angular_velocity: Vec3::ZERO,
        mass: initial_mass,
        radius: cell_radius,
        genome_id: 0,
        mode_index: initial_mode_index,
        birth_time: 0.0,
        split_interval,
        stiffness: 10.0,
    });
    
    // Initialize canonical state from initial state
    main_state.canonical_state = initial_state.to_canonical_state();
    main_state.initial_state = initial_state;
    main_state.id_to_entity.clear();
    main_state.entity_to_index.clear();
    // Resize index_to_entity to match new capacity
    main_state.index_to_entity = vec![None; cpu_cell_capacity.capacity];
    
    // Spawn initial cell
    let entity = commands.spawn((
        Cell {
            mass: initial_mass,
            radius: cell_radius,
            genome_id: 0,
            mode_index: initial_mode_index,
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
        Mesh3d(meshes.add(Sphere::new(cell_radius).mesh().ico(5).unwrap())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(color.x, color.y, color.z, opacity),
            cull_mode: Some(bevy::render::render_resource::Face::Back),
            alpha_mode: if opacity < 0.99 {
                bevy::prelude::AlphaMode::Blend
            } else {
                bevy::prelude::AlphaMode::Opaque
            },
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO)
            .with_rotation(genome.genome.initial_orientation),
        Visibility::default(),
        CpuSceneEntity,
    )).id();
    
    // Map cell ID to entity
    main_state.id_to_entity.insert(0, entity);
    main_state.entity_to_index.insert(entity, 0);
}

/// Spawn only Preview cells (for reset - doesn't spawn lights, fog, or world sphere)
fn spawn_preview_cells_only(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    genome: &Res<crate::genome::CurrentGenome>,
    config: &Res<crate::cell::physics::PhysicsConfig>,
    preview_state: &mut crate::simulation::preview_sim::PreviewSimState,
) {
    use crate::cell::{Cell, CellPosition, CellOrientation, CellSignaling};
    use crate::simulation::{InitialState, InitialCell};

    // Get initial mode settings from genome
    let initial_mode_index = genome.genome.initial_mode.max(0) as usize;
    let mode = genome.genome.modes.get(initial_mode_index)
        .or_else(|| genome.genome.modes.first());
    
    let (color, opacity, split_mass, split_interval, is_test_cell, max_cell_size) = if let Some(mode) = mode {
        (mode.color, mode.opacity, mode.split_mass, mode.split_interval, mode.cell_type == 0, mode.max_cell_size)
    } else {
        (Vec3::new(1.0, 1.0, 1.0), 1.0, 1.0, 5.0, true, 2.0)
    };
    
    // For Test cells, start with half the split mass so they need to grow before splitting
    let initial_mass = if is_test_cell { split_mass * 0.5 } else { split_mass };
    let cell_radius = if is_test_cell { initial_mass.min(max_cell_size).clamp(0.5, 2.0) } else { 1.0 };
    let stiffness = 10.0;
    
    // Create initial state
    let mut initial_state = InitialState::new((**config).clone(), 100_000, 0);
    initial_state.add_cell(InitialCell {
        id: 0,
        position: Vec3::ZERO,
        velocity: Vec3::ZERO,
        rotation: genome.genome.initial_orientation,
        angular_velocity: Vec3::ZERO,
        mass: initial_mass,
        radius: cell_radius,
        genome_id: 0,
        mode_index: initial_mode_index,
        birth_time: 0.0,
        split_interval,
        stiffness,
    });
    
    // Convert to canonical state
    let canonical_state = initial_state.to_canonical_state();
    
    // Update preview state
    preview_state.initial_state = initial_state;
    preview_state.canonical_state = canonical_state;
    preview_state.current_time = 0.0;
    for slot in preview_state.index_to_entity.iter_mut() {
        *slot = None;
    }
    
    // Spawn initial cell
    let entity = commands.spawn((
        Cell {
            mass: initial_mass,
            radius: cell_radius,
            genome_id: 0,
            mode_index: initial_mode_index,
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
        Mesh3d(meshes.add(Sphere::new(cell_radius).mesh().ico(5).unwrap())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(color.x, color.y, color.z, opacity),
            cull_mode: Some(bevy::render::render_resource::Face::Back),
            alpha_mode: if opacity < 0.99 {
                bevy::prelude::AlphaMode::Blend
            } else {
                bevy::prelude::AlphaMode::Opaque
            },
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO)
            .with_rotation(genome.genome.initial_orientation),
        Visibility::default(),
        crate::simulation::preview_sim::PreviewSceneEntity,
    )).id();
    
    // Map cell index to entity
    preview_state.index_to_entity[0] = Some(entity);
}
