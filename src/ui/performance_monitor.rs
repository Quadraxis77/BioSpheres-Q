use bevy::prelude::*;
use bevy_mod_imgui::prelude::*;
use std::collections::VecDeque;

/// Plugin for the Advanced Performance Monitor window
pub struct PerformanceMonitorPlugin;

impl Plugin for PerformanceMonitorPlugin {
    fn build(&self, app: &mut App) {
        // Note: We don't add FrameTimeDiagnosticsPlugin here because it can create
        // unwanted overlays. We calculate our own metrics directly from Time resource.
        app.init_resource::<PerformanceMonitor>()
            .add_systems(Update, (
                update_performance_metrics,
                render_performance_window,
            ));
    }
}

/// Performance monitoring data matching desktop implementation
#[derive(Resource)]
pub struct PerformanceMonitor {
    // Update control
    pub last_perf_update: f32,
    pub perf_update_interval: f32,
    pub frame_count: u32,
    pub frame_time_accumulator: f32,

    // Display values (updated every interval)
    pub display_fps: f32,
    pub display_frame_time: f32,

    // Advanced metrics
    pub min_frame_time: f32,
    pub max_frame_time: f32,
    pub avg_frame_time: f32,

    // History buffers (120 samples = 2 seconds at 60fps)
    pub frame_time_history: VecDeque<f32>,
    pub fps_history: VecDeque<f32>,

    // Reset timer for min/max
    pub reset_timer: f32,

    // Window state
    pub window_open: bool,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self {
            last_perf_update: -1.0, // Start at -1 to force immediate first update
            perf_update_interval: 0.25, // Update every 250ms
            frame_count: 0,
            frame_time_accumulator: 0.0,

            display_fps: 60.0, // Start with reasonable default
            display_frame_time: 16.67, // ~60 FPS

            min_frame_time: 1000.0,
            max_frame_time: 0.0,
            avg_frame_time: 16.67,

            frame_time_history: VecDeque::with_capacity(120),
            fps_history: VecDeque::with_capacity(120),

            reset_timer: 0.0,

            window_open: true,
        }
    }
}

const HISTORY_SIZE: usize = 120;

/// Update performance metrics matching desktop implementation
fn update_performance_metrics(
    time: Res<Time>,
    mut perf_monitor: ResMut<PerformanceMonitor>,
) {
    let delta_time = time.delta_secs();
    let current_time = time.elapsed_secs();
    let frame_time_ms = delta_time * 1000.0;

    // Update frame counting
    perf_monitor.frame_count += 1;
    perf_monitor.frame_time_accumulator += delta_time;

    // Update min/max
    if frame_time_ms < perf_monitor.min_frame_time {
        perf_monitor.min_frame_time = frame_time_ms;
    }
    if frame_time_ms > perf_monitor.max_frame_time {
        perf_monitor.max_frame_time = frame_time_ms;
    }

    // Update history (circular buffer)
    perf_monitor.frame_time_history.push_back(frame_time_ms);
    if perf_monitor.frame_time_history.len() > HISTORY_SIZE {
        perf_monitor.frame_time_history.pop_front();
    }

    let current_fps = if delta_time > 0.0 { 1.0 / delta_time } else { 0.0 };
    perf_monitor.fps_history.push_back(current_fps);
    if perf_monitor.fps_history.len() > HISTORY_SIZE {
        perf_monitor.fps_history.pop_front();
    }

    // Calculate average
    if !perf_monitor.frame_time_history.is_empty() {
        let sum: f32 = perf_monitor.frame_time_history.iter().sum();
        perf_monitor.avg_frame_time = sum / perf_monitor.frame_time_history.len() as f32;
    }

    // Reset min/max every 5 seconds
    perf_monitor.reset_timer += delta_time;
    if perf_monitor.reset_timer >= 5.0 {
        perf_monitor.min_frame_time = 1000.0;
        perf_monitor.max_frame_time = 0.0;
        perf_monitor.reset_timer = 0.0;
    }

    // Update display values every interval (250ms)
    // Handle first frame case where last_perf_update is -1
    let should_update = if perf_monitor.last_perf_update < 0.0 {
        true // Force first update
    } else {
        current_time - perf_monitor.last_perf_update >= perf_monitor.perf_update_interval
    };
    
    if should_update {
        let elapsed = if perf_monitor.last_perf_update < 0.0 {
            delta_time // Use single frame time for first update
        } else {
            current_time - perf_monitor.last_perf_update
        };
        
        if elapsed > 0.0 && perf_monitor.frame_count > 0 {
            perf_monitor.display_fps = perf_monitor.frame_count as f32 / elapsed;
            perf_monitor.display_frame_time = (perf_monitor.frame_time_accumulator / perf_monitor.frame_count as f32) * 1000.0;
        }

        perf_monitor.frame_count = 0;
        perf_monitor.frame_time_accumulator = 0.0;
        perf_monitor.last_perf_update = current_time;
    }
}

/// Render the performance monitor window matching desktop layout
fn render_performance_window(
    mut perf_monitor: ResMut<PerformanceMonitor>,
    mut imgui_context: NonSendMut<ImguiContext>,
    sim_state: Option<Res<crate::simulation::SimulationState>>,
    main_sim_state: Option<Res<crate::simulation::cpu_sim::MainSimState>>,
    preview_sim_state: Option<Res<crate::simulation::preview_sim::PreviewSimState>>,
    time: Res<Time<Fixed>>,
    global_ui_state: Res<super::GlobalUiState>,
) {
    if !perf_monitor.window_open {
        return;
    }

    // Copy out all the data we need before building the window
    // Ensure all values are valid (not NaN or infinity)
    let fps = if perf_monitor.display_fps.is_finite() { perf_monitor.display_fps } else { 0.0 };
    let frame_time = if perf_monitor.display_frame_time.is_finite() { perf_monitor.display_frame_time } else { 0.0 };
    let min_frame_time = if perf_monitor.min_frame_time.is_finite() && perf_monitor.min_frame_time < 1000.0 { 
        perf_monitor.min_frame_time 
    } else { 
        0.0 
    };
    let max_frame_time = if perf_monitor.max_frame_time.is_finite() { perf_monitor.max_frame_time } else { 0.0 };
    let avg_frame_time = if perf_monitor.avg_frame_time.is_finite() { perf_monitor.avg_frame_time } else { 0.0 };
    let frame_time_history: Vec<f32> = perf_monitor.frame_time_history.iter().copied().collect();
    let fps_history: Vec<f32> = perf_monitor.fps_history.iter().copied().collect();

    let ui = imgui_context.ui();

    // Build flags based on lock state
    use imgui::WindowFlags;
    let flags = if global_ui_state.windows_locked {
        WindowFlags::NO_MOVE | WindowFlags::NO_RESIZE
    } else {
        WindowFlags::empty()
    };

    ui.window("Advanced Performance Monitor")
        .position([1615.0, 280.0], Condition::FirstUseEver)
        .size([300.0, 565.0], Condition::FirstUseEver)
        .opened(&mut perf_monitor.window_open)
        .flags(flags)
        .build(|| {
            // Performance Overview Section
            ui.text_colored([1.0, 1.0, 1.0, 1.0], "Performance Overview");
            ui.separator();

            // FPS with color coding
            let fps_color = get_fps_color(fps);
            ui.text("FPS:");
            ui.same_line();
            ui.text_colored(fps_color, format!("{:.1}", fps));

            // Frame Time with color coding
            let frame_time_color = get_frame_time_color(frame_time);
            ui.text("Frame Time:");
            ui.same_line();
            ui.text_colored(frame_time_color, format!("{:.2} ms", frame_time));

            // Min/Avg/Max Frame Time
            ui.text(format!("Min: {:.2} ms  Avg: {:.2} ms  Max: {:.2} ms",
                min_frame_time,
                avg_frame_time,
                max_frame_time
            ));

            ui.spacing();

            // Frame Time History Graph
            if !frame_time_history.is_empty() {
                ui.plot_lines("##FrameTime", &frame_time_history)
                    .scale_min(0.0)
                    .scale_max(50.0)
                    .graph_size([0.0, 80.0])
                    .build();
            }

            ui.spacing();

            // FPS History Graph
            if !fps_history.is_empty() {
                ui.plot_lines("##FPS", &fps_history)
                    .scale_min(0.0)
                    .scale_max(120.0)
                    .graph_size([0.0, 80.0])
                    .build();
            }

            ui.spacing();

            // Performance Indicators Section
            ui.text_colored([1.0, 1.0, 1.0, 1.0], "Performance Indicators");
            ui.separator();

            // FPS Performance Bar with badges
            let (fps_status, _) = get_fps_status(fps);
            ui.text(format!("FPS: {}", fps_status));
            ui.same_line();

            // Badges
            {
                let _spacing = ui.push_style_var(StyleVar::ItemSpacing([2.0, 2.0]));

                let badge_60_color = if fps >= 60.0 {
                    [0.0, 1.0, 0.0, 1.0]
                } else {
                    [0.5, 0.5, 0.5, 1.0]
                };
                ui.text_colored(badge_60_color, "60+");
                ui.same_line();

                let badge_30_color = if fps >= 30.0 {
                    [1.0, 1.0, 0.0, 1.0]
                } else {
                    [0.5, 0.5, 0.5, 1.0]
                };
                ui.text_colored(badge_30_color, "30+");
            }

            // FPS Progress Bar (text-based indicator)
            let fps_ratio = (fps / 120.0).min(1.0);
            let fps_bar_color = get_fps_bar_color(fps);
            let bar_chars = (fps_ratio * 40.0) as usize;
            let bar_str = "#".repeat(bar_chars) + &"-".repeat(40 - bar_chars);
            ui.text_colored(fps_bar_color, bar_str);

            ui.spacing();

            // Frame Time Performance Bar
            let (ft_status, _) = get_frame_time_status(frame_time);
            ui.text(format!("Frame Time: {}", ft_status));

            // Frame Time Progress Bar (text-based indicator)
            let ft_ratio = (1.0 - (frame_time / 50.0)).max(0.0).min(1.0);
            let ft_bar_color = get_frame_time_bar_color(frame_time);
            let bar_chars = (ft_ratio * 40.0) as usize;
            let bar_str = "#".repeat(bar_chars) + &"-".repeat(40 - bar_chars);
            ui.text_colored(ft_bar_color, bar_str);

            ui.spacing();

            // Simulation Metrics Section
            ui.text_colored([1.0, 1.0, 1.0, 1.0], "Simulation Metrics");
            ui.separator();

            // Get accurate cell count and capacity based on current simulation mode
            let (cell_count, max_capacity) = if let Some(state) = sim_state.as_ref() {
                match state.mode {
                    crate::simulation::SimulationMode::Preview => {
                        if let Some(preview_state) = preview_sim_state.as_ref() {
                            (preview_state.canonical_state.cell_count, preview_state.initial_state.max_cells)
                        } else {
                            (0, 256)
                        }
                    }
                    crate::simulation::SimulationMode::Gpu => {
                        // GPU simulation targets 100K cells
                        (0, 100_000)
                    }
                    crate::simulation::SimulationMode::Cpu => {
                        if let Some(main_state) = main_sim_state.as_ref() {
                            (main_state.canonical_state.cell_count, main_state.initial_state.max_cells)
                        } else {
                            (0, 4096)
                        }
                    }
                }
            } else if let Some(main_state) = main_sim_state.as_ref() {
                (main_state.canonical_state.cell_count, main_state.initial_state.max_cells)
            } else {
                (0, 4096)
            };

            ui.text(format!("Cells: {} / {}", cell_count, max_capacity));
            
            // Show capacity percentage
            let capacity_percent = (cell_count as f32 / max_capacity as f32) * 100.0;
            let capacity_color = if capacity_percent > 90.0 {
                [1.0, 0.0, 0.0, 1.0] // Red when near capacity
            } else if capacity_percent > 75.0 {
                [1.0, 1.0, 0.0, 1.0] // Yellow
            } else {
                [0.0, 1.0, 0.0, 1.0] // Green
            };
            ui.text_colored(capacity_color, format!("Capacity: {:.1}%", capacity_percent));

            if let Some(sim) = sim_state {
                ui.text(format!("Mode: {}",
                    if sim.mode == crate::simulation::SimulationMode::Cpu {
                        "CPU Simulation"
                    } else {
                        "GPU Compute Shaders"
                    }
                ));
                
                ui.text(format!("Status: {}",
                    if sim.paused { "Paused" } else { "Running" }
                ));
            }
            
            // Show simulation time (from main sim state if available)
            let sim_time = if let Some(main_state) = main_sim_state.as_ref() {
                main_state.simulation_time
            } else {
                time.elapsed_secs()
            };
            ui.text(format!("Sim Time: {:.2}s", sim_time));

            // Estimate memory usage for canonical state
            let canonical_memory = if let Some(main_state) = main_sim_state.as_ref() {
                let state = &main_state.canonical_state;
                let vec_memory = (
                    state.positions.capacity() * std::mem::size_of::<bevy::prelude::Vec3>() +
                    state.velocities.capacity() * std::mem::size_of::<bevy::prelude::Vec3>() +
                    state.masses.capacity() * std::mem::size_of::<f32>() +
                    state.radii.capacity() * std::mem::size_of::<f32>() +
                    state.forces.capacity() * std::mem::size_of::<bevy::prelude::Vec3>()
                ) as f32 / (1024.0 * 1024.0);
                vec_memory
            } else {
                0.0
            };
            ui.text(format!("Memory: {:.2} MB", canonical_memory));

            ui.spacing();

            // Performance Warnings
            if fps < 30.0 {
                ui.text_colored([1.0, 0.0, 0.0, 1.0],
                    "⚠ Low FPS detected! Performance is below acceptable levels.");
            }
            if frame_time > 33.33 {
                ui.text_colored([1.0, 0.5, 0.0, 1.0],
                    "⚠ High frame time detected! Frame rendering is taking too long.");
            }
        });
}

// Color helper functions matching desktop implementation

fn get_fps_color(fps: f32) -> [f32; 4] {
    if fps >= 59.0 {
        [0.0, 1.0, 0.0, 1.0] // Green
    } else if fps >= 30.0 {
        [1.0, 1.0, 0.0, 1.0] // Yellow
    } else {
        [1.0, 0.0, 0.0, 1.0] // Red
    }
}

fn get_frame_time_color(frame_time: f32) -> [f32; 4] {
    if frame_time <= 17.0 {
        [0.0, 1.0, 0.0, 1.0] // Green
    } else if frame_time <= 33.33 {
        [1.0, 1.0, 0.0, 1.0] // Yellow
    } else {
        [1.0, 0.0, 0.0, 1.0] // Red
    }
}

fn get_fps_status(fps: f32) -> (&'static str, [f32; 4]) {
    if fps >= 59.0 {
        ("Excellent", [0.0, 0.8, 0.0, 1.0])
    } else if fps >= 45.0 {
        ("Good", [0.5, 0.8, 0.0, 1.0])
    } else if fps >= 30.0 {
        ("Fair", [1.0, 0.8, 0.0, 1.0])
    } else {
        ("Poor", [1.0, 0.2, 0.2, 1.0])
    }
}

fn get_frame_time_status(frame_time: f32) -> (&'static str, [f32; 4]) {
    if frame_time <= 17.0 {
        ("Smooth", [0.0, 0.8, 0.0, 1.0])
    } else if frame_time <= 25.0 {
        ("Good", [0.5, 0.8, 0.0, 1.0])
    } else if frame_time <= 33.33 {
        ("Acceptable", [1.0, 0.8, 0.0, 1.0])
    } else {
        ("Laggy", [1.0, 0.2, 0.2, 1.0])
    }
}

fn get_fps_bar_color(fps: f32) -> [f32; 4] {
    if fps >= 59.0 {
        [0.0, 0.8, 0.0, 1.0]       // Green
    } else if fps >= 45.0 {
        [0.5, 0.8, 0.0, 1.0]     // Yellow-Green
    } else if fps >= 30.0 {
        [1.0, 0.8, 0.0, 1.0]     // Orange
    } else {
        [1.0, 0.2, 0.2, 1.0]     // Red
    }
}

fn get_frame_time_bar_color(frame_time: f32) -> [f32; 4] {
    if frame_time <= 17.0 {
        [0.0, 0.8, 0.0, 1.0]       // Green
    } else if frame_time <= 25.0 {
        [0.5, 0.8, 0.0, 1.0]     // Yellow-Green
    } else if frame_time <= 33.33 {
        [1.0, 0.8, 0.0, 1.0]     // Orange
    } else {
        [1.0, 0.2, 0.2, 1.0]     // Red
    }
}
