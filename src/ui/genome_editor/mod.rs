// Genome Editor Module - Main interface for genome editing UI

pub mod modes_panel;
pub mod settings_panels;
pub mod genome_graph;

// Re-export panel rendering functions
pub use modes_panel::render_modes_panel;
pub use settings_panels::{
    render_name_type_editor,
    render_adhesion_settings,
    render_parent_settings,
    render_circle_sliders,
    render_quaternion_ball,
    render_time_slider,
};
pub use genome_graph::render_genome_graph;
