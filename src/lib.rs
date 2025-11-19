pub mod cell;
pub mod genome;
pub mod input;
pub mod rendering;
pub mod simulation;
pub mod ui;

// Re-export all plugins for convenient access
pub use cell::CellPlugin;
pub use genome::GenomePlugin;
pub use input::InputPlugin;
pub use rendering::RenderingPlugin;
pub use simulation::SimulationPlugin;
pub use ui::UiPlugin;
