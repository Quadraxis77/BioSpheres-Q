use bevy::prelude::*;

pub mod cell_dragging;

pub use cell_dragging::{CellDraggingPlugin, DragState, CellDraggingSet};

/// Plugin for input handling
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CellDraggingPlugin);
    }
}

/// Currently selected tool
#[derive(Resource, Default)]
pub struct SelectedTool {
    pub tool: Tool,
}

/// Available interaction tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    Select,
    Drag,
    Add,
    Remove,
    SampleGenome,
    EditCell,
}

/// Currently selected cell
#[derive(Resource, Default)]
pub struct SelectedCell {
    pub entity: Option<Entity>,
}
