use bevy::prelude::*;

pub mod cpu_physics;
pub mod cell_allocation;
pub mod clock;
pub mod cpu_sim;
pub mod double_buffer;
pub mod gpu_sim;
pub mod initial_state;
pub mod physics_config;
pub mod preview_sim;
pub mod adhesion_inheritance;

pub use cpu_physics::{CanonicalState, DeterministicSpatialGrid, physics_step, deterministic_random};
pub use physics_config::PhysicsConfig;
pub use cell_allocation::{Cell, Adhesion};
pub use clock::SimulationClock;
pub use cpu_sim::{CpuSimPlugin, CpuSimTimestepPlugin, CpuSceneState, CpuSceneEntity};
pub use double_buffer::DoubleBufferedState;
pub use gpu_sim::{GpuSimPlugin, GpuSceneState, GpuSceneEntity};
pub use initial_state::{InitialState, InitialCell};
pub use preview_sim::{PreviewSimPlugin, PreviewSceneState, PreviewSceneEntity};
pub use adhesion_inheritance::inherit_adhesions_on_division;

/// Configuration for simulation threading
#[derive(Resource, Clone, Copy, Debug)]
pub struct SimulationThreadingConfig {
    /// Enable multithreading for CPU simulation
    pub cpu_multithreaded: bool,
    /// Enable multithreading for preview simulation
    pub preview_multithreaded: bool,
}

impl Default for SimulationThreadingConfig {
    fn default() -> Self {
        Self {
            cpu_multithreaded: false,
            preview_multithreaded: false,
        }
    }
}

/// Main simulation plugin that coordinates all simulation modes
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add shared cell allocation plugin once
            .add_plugins(
                cell_allocation::CellSimulationPlugin::builder()
                    .with_cell_capacity(2000)
                    .with_adhesion_capacity(2000 * 40)
                    .build()
            )
            // Add mode-specific plugins
            .add_plugins(CpuSimPlugin)
            .add_plugins(GpuSimPlugin)
            .add_plugins(PreviewSimPlugin)
            .init_resource::<PhysicsConfig>()
            .init_resource::<SimulationState>()
            .init_resource::<SimulationConfig>()
            .init_resource::<SimulationThreadingConfig>()
            .add_systems(Startup, initialize_default_scene);
    }
}

/// Initialize the default scene on startup
/// Since SimulationMode defaults to Preview, we need to activate PreviewSceneState
fn initialize_default_scene(
    mut next_preview_state: ResMut<NextState<PreviewSceneState>>,
) {
    next_preview_state.set(PreviewSceneState::Active);
}

/// Current simulation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SimulationMode {
    Cpu,
    Gpu,
    #[default]
    Preview,
}

/// Global simulation state
#[derive(Resource)]
pub struct SimulationState {
    pub mode: SimulationMode,
    pub paused: bool,
    pub target_time: Option<f32>,
    pub is_resimulating: bool,
    pub needs_respawn: bool,
    /// Simulation speed multiplier (1.0 = real-time, 10.0 = 10x speed)
    pub speed_multiplier: f32,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            mode: SimulationMode::default(),
            paused: false,
            target_time: None,
            is_resimulating: false,
            needs_respawn: false,
            speed_multiplier: 1.0,
        }
    }
}

/// Simulation configuration
#[derive(Resource)]
pub struct SimulationConfig {
    pub cell_count_limit: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            cell_count_limit: 100_000,
        }
    }
}
