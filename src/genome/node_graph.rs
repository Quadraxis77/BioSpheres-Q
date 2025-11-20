use bevy::prelude::*;
use std::collections::HashMap;

/// Node graph representation of genome modes
#[derive(Resource)]
pub struct GenomeNodeGraph {
    /// Map from mode index to node ID
    pub mode_to_node: HashMap<usize, i32>,
    /// Map from node ID to mode index
    pub node_to_mode: HashMap<i32, usize>,
    /// Node positions (node_id -> (x, y))
    pub node_positions: HashMap<i32, (f32, f32)>,
    /// Next available node ID
    pub next_node_id: i32,
    /// Links between nodes (parent_node_id, child_node_id, is_child_a)
    pub links: Vec<(i32, i32, bool)>,
    /// Next available link ID
    pub next_link_id: i32,
    /// Track if graph needs rebuild
    pub needs_rebuild: bool,
    /// Track if positions need to be set
    pub needs_layout: bool,
    /// Pending position for newly created node (mode_index, x, y)
    pub pending_position: Option<(usize, f32, f32)>,
}

impl Default for GenomeNodeGraph {
    fn default() -> Self {
        Self {
            mode_to_node: HashMap::new(),
            node_to_mode: HashMap::new(),
            node_positions: HashMap::new(),
            next_node_id: 0,
            links: Vec::new(),
            next_link_id: 0,
            needs_rebuild: false,
            needs_layout: true,
            pending_position: None,
        }
    }
}

impl GenomeNodeGraph {
    /// Create a new node for a mode
    pub fn create_node(&mut self, mode_index: usize) -> i32 {
        let node_id = self.next_node_id;
        self.next_node_id += 1;
        self.mode_to_node.insert(mode_index, node_id);
        self.node_to_mode.insert(node_id, mode_index);
        node_id
    }

    /// Remove a node
    pub fn remove_node(&mut self, node_id: i32) {
        if let Some(mode_index) = self.node_to_mode.remove(&node_id) {
            self.mode_to_node.remove(&mode_index);
        }
        // Remove all links involving this node
        self.links.retain(|(from, to, _)| *from != node_id && *to != node_id);
    }

    /// Add a link between nodes
    pub fn add_link(&mut self, from_node: i32, to_node: i32, is_child_a: bool) {
        // Remove existing link from same parent with same child type
        self.links.retain(|(from, _, is_a)| !(*from == from_node && *is_a == is_child_a));
        self.links.push((from_node, to_node, is_child_a));
    }

    /// Get node ID for a mode index
    pub fn get_node_for_mode(&self, mode_index: usize) -> Option<i32> {
        self.mode_to_node.get(&mode_index).copied()
    }

    /// Get mode index for a node ID
    pub fn get_mode_for_node(&self, node_id: i32) -> Option<usize> {
        self.node_to_mode.get(&node_id).copied()
    }

    /// Clear all nodes and links
    pub fn clear(&mut self) {
        self.mode_to_node.clear();
        self.node_to_mode.clear();
        self.node_positions.clear();
        self.links.clear();
        self.next_node_id = 0;
        self.next_link_id = 0;
    }

    /// Mark graph for rebuild
    pub fn mark_for_rebuild(&mut self) {
        self.needs_rebuild = true;
        self.needs_layout = true;
    }

    /// Calculate automatic layout for nodes in a grid pattern
    pub fn calculate_grid_layout(&mut self) {
        const NODE_SPACING_X: f32 = 250.0;
        const NODE_SPACING_Y: f32 = 200.0;
        const START_X: f32 = 50.0;
        const START_Y: f32 = 50.0;
        const COLUMNS: usize = 4;

        let mut sorted_nodes: Vec<i32> = self.node_to_mode.keys().copied().collect();
        sorted_nodes.sort_by_key(|node_id| self.node_to_mode.get(node_id).unwrap_or(&0));

        for (idx, node_id) in sorted_nodes.iter().enumerate() {
            let col = idx % COLUMNS;
            let row = idx / COLUMNS;
            let x = START_X + (col as f32 * NODE_SPACING_X);
            let y = START_Y + (row as f32 * NODE_SPACING_Y);
            self.node_positions.insert(*node_id, (x, y));
        }

        self.needs_layout = false;
    }

    /// Get position for a node
    pub fn get_node_position(&self, node_id: i32) -> Option<(f32, f32)> {
        self.node_positions.get(&node_id).copied()
    }

    /// Set position for a node
    pub fn set_node_position(&mut self, node_id: i32, x: f32, y: f32) {
        self.node_positions.insert(node_id, (x, y));
    }
}
