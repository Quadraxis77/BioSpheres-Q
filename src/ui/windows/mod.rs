// Window modules for genome editor panels

pub mod modes;
pub mod adhesion_settings;
pub mod name_type_editor;
pub mod parent_settings;
pub mod scene_manager;

// Re-export rendering functions with consistent naming
pub use modes::render_modes_panel;
pub use adhesion_settings::render as render_adhesion_settings;
pub use name_type_editor::render as render_name_type_editor;
pub use parent_settings::render as render_parent_settings;
pub use scene_manager::render as render_scene_manager;
