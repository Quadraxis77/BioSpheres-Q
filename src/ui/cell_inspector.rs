use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_mod_imgui::prelude::*;
use crate::cell::{Cell, CellPosition};
use crate::genome::CurrentGenome;
use crate::simulation::{SimulationState, SimulationMode};
use crate::simulation::cpu_sim::MainSimState;
use crate::simulation::preview_sim::PreviewSimState;
use crate::ui::camera::MainCamera;

/// Resource to track the inspected cell
#[derive(Resource, Default)]
pub struct CellInspectorState {
    pub window_open: bool,
    /// The cell index being inspected (in canonical state)
    pub inspected_cell_index: Option<usize>,
    /// Entity of the inspected cell (for ECS queries)
    pub inspected_entity: Option<Entity>,
}

/// Plugin for cell inspection window
pub struct CellInspectorPlugin;

impl Plugin for CellInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CellInspectorState {
            window_open: true,
            inspected_cell_index: None,
            inspected_entity: None,
        })
        .add_systems(Update, (
            handle_cell_click,
            render_cell_inspector_window,
        ).chain());
    }
}

/// System to handle cell clicking/selection for inspection
fn handle_cell_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    cell_query: Query<(Entity, &CellPosition, &Cell)>,
    ui_capture: Res<crate::ui::camera::UiWantCapture>,
    simulation_state: Res<SimulationState>,
    main_sim_state: Option<Res<MainSimState>>,
    preview_sim_state: Option<Res<PreviewSimState>>,
    mut inspector_state: ResMut<CellInspectorState>,
) {
    // Don't process if UI wants to capture mouse
    if ui_capture.want_capture_mouse {
        return;
    }

    // Only respond to left click (pressed or held for drag)
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = window_query.single() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };

    // Find closest cell intersected by ray
    let mut closest_hit: Option<(Entity, f32)> = None;

    for (entity, cell_pos, cell) in cell_query.iter() {
        if let Some(hit_distance) = ray_sphere_intersection(
            ray.origin,
            *ray.direction,
            cell_pos.position,
            cell.radius,
        ) {
            if closest_hit.is_none() || hit_distance < closest_hit.unwrap().1 {
                closest_hit = Some((entity, hit_distance));
            }
        }
    }

    // Update inspector state with clicked cell
    if let Some((entity, _)) = closest_hit {
        inspector_state.inspected_entity = Some(entity);
        
        // Find the cell index in canonical state
        match simulation_state.mode {
            SimulationMode::Cpu => {
                if let Some(ref main_state) = main_sim_state {
                    if let Some(&index) = main_state.entity_to_index.get(&entity) {
                        inspector_state.inspected_cell_index = Some(index);
                    }
                }
            }
            SimulationMode::Preview => {
                if let Some(ref preview_state) = preview_sim_state {
                    for (i, slot) in preview_state.index_to_entity.iter().enumerate() {
                        if *slot == Some(entity) {
                            inspector_state.inspected_cell_index = Some(i);
                            break;
                        }
                    }
                }
            }
        }
    }
}


/// Ray-sphere intersection test
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
        if t > 0.0 { Some(t) } else { None }
    }
}

/// Main cell inspector window rendering system
fn render_cell_inspector_window(
    mut imgui_context: NonSendMut<ImguiContext>,
    mut inspector_state: ResMut<CellInspectorState>,
    simulation_state: Res<SimulationState>,
    main_sim_state: Option<Res<MainSimState>>,
    preview_sim_state: Option<Res<PreviewSimState>>,
    genome: Res<CurrentGenome>,
    global_ui_state: Res<super::GlobalUiState>,
) {
    let ui = imgui_context.ui();

    if !inspector_state.window_open {
        return;
    }

    // Extract data before building window to avoid borrow issues
    let cell_index = inspector_state.inspected_cell_index;
    
    // Get cell data and simulation time from canonical state based on simulation mode
    let (cell_data, sim_time) = cell_index.map(|idx| {
        match simulation_state.mode {
            SimulationMode::Cpu => {
                main_sim_state.as_ref().and_then(|state| {
                    if idx < state.canonical_state.cell_count {
                        Some((extract_cell_data(&state.canonical_state, idx), state.simulation_time))
                    } else {
                        None
                    }
                })
            }
            SimulationMode::Preview => {
                preview_sim_state.as_ref().and_then(|state| {
                    if idx < state.canonical_state.cell_count {
                        Some((extract_cell_data(&state.canonical_state, idx), state.current_time))
                    } else {
                        None
                    }
                })
            }
        }
    }).flatten().unzip();
    
    // Track if we need to clear selection
    let mut should_clear_selection = false;

    // Only show if visibility is enabled
    if !global_ui_state.show_cell_inspector {
        return;
    }

    use imgui::WindowFlags;
    let flags = if global_ui_state.windows_locked {
        WindowFlags::NO_MOVE | WindowFlags::NO_RESIZE
    } else {
        WindowFlags::empty()
    };

    ui.window("Cell Inspector")
        .position([1704.0, 741.0], Condition::FirstUseEver)
        .size([212.0, 336.0], Condition::FirstUseEver)
        .size_constraints([250.0, 200.0], [f32::MAX, f32::MAX])
        .collapsible(true)
        .flags(flags)
        .build(|| {
            // Check if we have a cell selected
            if cell_index.is_none() {
                ui.text("Click on a cell to inspect it");
                ui.text("(or drag a cell)");
                return;
            }
            
            let Some(data) = cell_data.as_ref() else {
                ui.text("Cell no longer exists");
                should_clear_selection = true;
                return;
            };
            
            // Get mode settings from genome
            let mode = genome.genome.modes.get(data.mode_index);
            let mode_name = mode.map(|m| m.name.as_str()).unwrap_or("Unknown");
            let cell_type_name = mode.map(|m| get_cell_type_name(m.cell_type)).unwrap_or("Unknown");
            
            // Calculate time alive
            let time_alive = sim_time.map(|t| t - data.birth_time).unwrap_or(0.0);
            
            // === Cell Identity (always visible) ===
            ui.text(format!("Cell Index: {}", cell_index.unwrap_or(0)));
            ui.text(format!("Cell ID: {}", data.cell_id));
            ui.text(format!("Mode: {} ({})", mode_name, data.mode_index));
            ui.text(format!("Type: {}", cell_type_name));
            
            ui.separator();
            
            // === Key Stats (always visible) ===
            // Mass with visual bar
            // Range: 0.5 (depleted/minimum) to split_mass * 2.0 (upper range)
            const MIN_CELL_MASS: f32 = 0.5;
            let split_mass = data.split_mass; // Use per-cell split_mass threshold
            let max_display_mass = split_mass * 2.0;
            let mass_ratio = ((data.mass - MIN_CELL_MASS) / (max_display_mass - MIN_CELL_MASS)).clamp(0.0, 1.0);
            let bar_width = 16;
            let filled = (mass_ratio * bar_width as f32) as usize;
            let bar_str = format!("[{}{}]", "#".repeat(filled), "-".repeat(bar_width - filled));
            
            // Color based on mass relative to split threshold
            let split_ratio = (data.mass / split_mass).clamp(0.0, 2.0);
            let bar_color = if split_ratio >= 1.0 {
                [0.0, 1.0, 0.0, 1.0] // Green - ready to split
            } else if split_ratio >= 0.5 {
                [1.0, 1.0, 0.0, 1.0] // Yellow - growing
            } else {
                [1.0, 0.5, 0.0, 1.0] // Orange - low/depleted
            };
            
            ui.text("Mass:");
            ui.same_line();
            ui.text_colored(bar_color, format!("{:.2}", data.mass));
            ui.same_line();
            ui.text_colored(bar_color, bar_str);
            
            ui.text(format!("Time Alive: {:.2}s", time_alive));
            ui.text(format!("Split Count: {}", data.split_count));
            
            ui.separator();
            
            // === Nutrient Details ===
            if ui.collapsing_header("Nutrient Details", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                ui.indent();
                
                // Nutrient Storage (Mass)
                const MIN_CELL_MASS: f32 = 0.5;
                let storage_cap = split_mass * 2.0; // Storage cap is 2x split_mass
                let stored_nutrients = (data.mass - MIN_CELL_MASS).max(0.0);
                let storage_percent = (stored_nutrients / (storage_cap - MIN_CELL_MASS) * 100.0).min(100.0);
                
                ui.text("Nutrient Storage:");
                ui.same_line();
                let storage_color = if data.mass >= storage_cap {
                    [0.0, 1.0, 0.0, 1.0] // Green - at cap
                } else if data.mass >= split_mass {
                    [0.5, 1.0, 0.0, 1.0] // Light green - ready to split
                } else if data.mass >= MIN_CELL_MASS + (storage_cap - MIN_CELL_MASS) * 0.5 {
                    [1.0, 1.0, 0.0, 1.0] // Yellow - half full
                } else if data.mass > MIN_CELL_MASS {
                    [1.0, 0.5, 0.0, 1.0] // Orange - low
                } else {
                    [1.0, 0.0, 0.0, 1.0] // Red - depleted
                };
                ui.text_colored(storage_color, format!("{:.2}/{:.2} ({:.0}%)", stored_nutrients, storage_cap - MIN_CELL_MASS, storage_percent));
                
                ui.spacing();
                ui.text(format!("Current Mass: {:.3}", data.mass));
                ui.text(format!("Split Mass: {:.2}", split_mass));
                ui.text(format!("Storage Cap: {:.2}", storage_cap));
                ui.text(format!("Minimum Mass: {:.2}", MIN_CELL_MASS));
                ui.text(format!("Radius: {:.3}", data.radius));
                
                if let Some(mode) = mode {
                    ui.spacing();
                    if mode.cell_type == 0 {
                        ui.text(format!("Gain Rate: {:.2}/s", mode.nutrient_gain_rate));
                    } else if mode.cell_type == 1 {
                        ui.text(format!("Swim Force: {:.2}", mode.swim_force));
                        ui.text(format!("Consumption: {:.3}/s", mode.swim_force * 0.2));
                    }
                    ui.text(format!("Max Size: {:.2}", mode.max_cell_size));
                    
                    // Show base priority and boosted priority if applicable
                    let danger_threshold = 0.6;
                    let priority_boost = 10.0;
                    let is_boosted = mode.prioritize_when_low && data.mass < danger_threshold;
                    let effective_priority = if is_boosted {
                        mode.nutrient_priority * priority_boost
                    } else {
                        mode.nutrient_priority
                    };
                    
                    ui.text(format!("Base Priority: {:.2}", mode.nutrient_priority));
                    if is_boosted {
                        ui.same_line();
                        ui.text_colored([1.0, 0.0, 0.0, 1.0], format!("→ {:.1} (BOOSTED!)", effective_priority));
                    }
                    
                    ui.text(format!("Protect Low: {}", if mode.prioritize_when_low { "Yes" } else { "No" }));
                    if mode.prioritize_when_low {
                        ui.same_line();
                        if is_boosted {
                            ui.text_colored([1.0, 0.0, 0.0, 1.0], "(ACTIVE)");
                        } else {
                            ui.text_colored([0.5, 0.5, 0.5, 1.0], "(inactive)");
                        }
                    }
                    
                    ui.text(format!("Split Ratio: {:.0}%", mode.split_ratio * 100.0));
                }
                
                ui.unindent();
            }
            
            // === Position & Motion ===
            if ui.collapsing_header("Position & Motion", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                ui.indent();
                
                ui.text(format!("Position: ({:.2}, {:.2}, {:.2})", 
                    data.position.x, data.position.y, data.position.z));
                ui.text(format!("Velocity: ({:.2}, {:.2}, {:.2})", 
                    data.velocity.x, data.velocity.y, data.velocity.z));
                ui.text(format!("Speed: {:.3}", data.velocity.length()));
                
                ui.unindent();
            }
            
            // === Rotation ===
            if ui.collapsing_header("Rotation", imgui::TreeNodeFlags::empty()) {
                ui.indent();
                
                let euler = quat_to_euler_degrees(data.rotation);
                ui.text(format!("Rotation (deg): ({:.1}, {:.1}, {:.1})", 
                    euler.x, euler.y, euler.z));
                ui.text(format!("Angular Vel: ({:.2}, {:.2}, {:.2})", 
                    data.angular_velocity.x, data.angular_velocity.y, data.angular_velocity.z));
                
                ui.unindent();
            }
            
            // === Division Info ===
            if ui.collapsing_header("Division", imgui::TreeNodeFlags::empty()) {
                ui.indent();
                
                ui.text(format!("Birth Time: {:.2}s", data.birth_time));
                ui.text(format!("Time Alive: {:.2}s", time_alive));
                ui.text(format!("Split Interval: {:.2}s", data.split_interval));
                
                // Time until next split
                let time_until_split = (data.split_interval - time_alive).max(0.0);
                if time_until_split > 0.0 {
                    ui.text(format!("Next Split In: {:.2}s", time_until_split));
                } else {
                    ui.text_colored([0.0, 1.0, 0.0, 1.0], "Ready to split!");
                }
                
                ui.text(format!("Split Count: {}", data.split_count));
                
                if let Some(mode) = mode {
                    if mode.max_splits >= 0 {
                        ui.text(format!("Max Splits: {}", mode.max_splits));
                        let remaining = (mode.max_splits - data.split_count).max(0);
                        ui.text(format!("Remaining: {}", remaining));
                    } else {
                        ui.text("Max Splits: Infinite");
                    }
                    ui.text(format!("Min Adhesions: {}", mode.min_adhesions));
                }
                
                ui.unindent();
            }
            
            // === Physics ===
            if ui.collapsing_header("Physics", imgui::TreeNodeFlags::empty()) {
                ui.indent();
                
                ui.text(format!("Stiffness: {:.2}", data.stiffness));
                ui.text(format!("Force: ({:.2}, {:.2}, {:.2})", 
                    data.force.x, data.force.y, data.force.z));
                ui.text(format!("Acceleration: ({:.2}, {:.2}, {:.2})", 
                    data.acceleration.x, data.acceleration.y, data.acceleration.z));
                
                ui.unindent();
            }
            
            // === Adhesions ===
            if ui.collapsing_header("Adhesions", imgui::TreeNodeFlags::empty()) {
                ui.indent();
                
                ui.text(format!("Adhesion Count: {}", data.adhesion_count));
                if let Some(mode) = mode {
                    ui.text(format!("Max Adhesions: {}", mode.max_adhesions));
                }
                
                ui.unindent();
            }
            
            // === Flagellocyte-specific ===
            if let Some(mode) = mode {
                if mode.cell_type == 1 {
                    if ui.collapsing_header("Flagellocyte", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                        ui.indent();
                        ui.text(format!("Swim Force: {:.2}", mode.swim_force));
                        ui.unindent();
                    }
                }
            }
            
            ui.separator();
            
            // Clear selection button
            if ui.button("Clear Selection") {
                should_clear_selection = true;
            }
        });
    
    // Clear selection if needed (after window build to avoid borrow issues)
    if should_clear_selection {
        inspector_state.inspected_cell_index = None;
        inspector_state.inspected_entity = None;
    }
}


/// Extracted cell data for display
struct CellData {
    cell_id: u32,
    position: Vec3,
    velocity: Vec3,
    rotation: Quat,
    angular_velocity: Vec3,
    mass: f32,
    radius: f32,
    mode_index: usize,
    stiffness: f32,
    birth_time: f32,
    split_interval: f32,
    split_mass: f32,
    split_count: i32,
    force: Vec3,
    acceleration: Vec3,
    adhesion_count: usize,
}

/// Extract cell data from canonical state
fn extract_cell_data(
    state: &crate::simulation::cpu_physics::CanonicalState,
    index: usize,
) -> CellData {
    // Count adhesions for this cell
    let adhesion_count = count_cell_adhesions(state, index);
    
    CellData {
        cell_id: state.cell_ids[index],
        position: state.positions[index],
        velocity: state.velocities[index],
        rotation: state.rotations[index],
        angular_velocity: state.angular_velocities[index],
        mass: state.masses[index],
        radius: state.radii[index],
        mode_index: state.mode_indices[index],
        stiffness: state.stiffnesses[index],
        birth_time: state.birth_times[index],
        split_interval: state.split_intervals[index],
        split_mass: state.split_masses[index],
        split_count: state.split_counts[index],
        force: state.forces[index],
        acceleration: state.accelerations[index],
        adhesion_count,
    }
}

/// Count active adhesions for a cell
fn count_cell_adhesions(
    state: &crate::simulation::cpu_physics::CanonicalState,
    cell_index: usize,
) -> usize {
    let mut count = 0;
    for i in 0..state.adhesion_connections.is_active.len() {
        if state.adhesion_connections.is_active[i] != 0 {
            if state.adhesion_connections.cell_a_index[i] == cell_index ||
               state.adhesion_connections.cell_b_index[i] == cell_index {
                count += 1;
            }
        }
    }
    count
}

/// Get human-readable cell type name
fn get_cell_type_name(cell_type: i32) -> &'static str {
    match cell_type {
        0 => "Test (Nutrient)",
        1 => "Flagellocyte",
        2 => "Photocyte",
        3 => "Phagocyte",
        _ => "Unknown",
    }
}

/// Convert quaternion to euler angles in degrees
fn quat_to_euler_degrees(quat: Quat) -> Vec3 {
    let (y, x, z) = quat.to_euler(EulerRot::YXZ);
    Vec3::new(
        x.to_degrees(),
        y.to_degrees(),
        z.to_degrees(),
    )
}
