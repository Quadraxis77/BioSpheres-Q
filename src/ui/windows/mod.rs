// Window modules for genome editor panels

pub mod modes;
pub mod adhesion_settings;
pub mod circle_sliders;
pub mod name_type_editor;
pub mod parent_settings;

// Re-export rendering functions with consistent naming
pub use modes::render_modes_panel;
pub use adhesion_settings::render as render_adhesion_settings;
pub use circle_sliders::render as render_circle_sliders;
pub use name_type_editor::render as render_name_type_editor;
pub use parent_settings::render as render_parent_settings;
