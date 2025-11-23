use bevy::prelude::*;
use serde::{Serialize, Deserialize};

pub mod node_graph;
pub use node_graph::GenomeNodeGraph;

/// Plugin for genome management
pub struct GenomePlugin;

impl Plugin for GenomePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GenomeLibrary>()
            .init_resource::<CurrentGenome>()
            .init_resource::<GenomeNodeGraph>();
    }
}

/// Storage for all genomes in the simulation
#[derive(Resource, Default)]
pub struct GenomeLibrary {
    pub genomes: Vec<GenomeData>,
}

/// Current genome being edited/used
#[derive(Resource)]
pub struct CurrentGenome {
    pub genome: GenomeData,
    pub selected_mode_index: i32,
}

impl Default for CurrentGenome {
    fn default() -> Self {
        Self {
            genome: GenomeData::default(),
            selected_mode_index: 0,
        }
    }
}

/// Adhesion configuration
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AdhesionSettings {
    pub can_break: bool,
    pub break_force: f32,
    pub rest_length: f32,
    pub linear_spring_stiffness: f32,
    pub linear_spring_damping: f32,
    pub orientation_spring_stiffness: f32,
    pub orientation_spring_damping: f32,
    pub max_angular_deviation: f32,
    pub twist_constraint_stiffness: f32,
    pub twist_constraint_damping: f32,
    pub enable_twist_constraint: bool,
}

impl Default for AdhesionSettings {
    fn default() -> Self {
        Self {
            can_break: true,
            break_force: 10.0,
            rest_length: 1.0,
            linear_spring_stiffness: 150.0,
            linear_spring_damping: 5.0,
            orientation_spring_stiffness: 50.0,  // Increased from 10.0 to make rotation more responsive
            orientation_spring_damping: 5.0,      // Increased from 2.0 to prevent oscillation
            max_angular_deviation: 0.0,
            twist_constraint_stiffness: 2.0,
            twist_constraint_damping: 0.5,
            enable_twist_constraint: false,  // Disabled by default - can cause anchors to appear to "follow" the connection
        }
    }
}

/// Child settings for mode transitions
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ChildSettings {
    pub mode_number: i32,
    pub orientation: Quat,
    pub keep_adhesion: bool,
    pub enable_angle_snapping: bool,
}

impl Default for ChildSettings {
    fn default() -> Self {
        Self {
            mode_number: 0,
            orientation: Quat::IDENTITY,
            keep_adhesion: true,
            enable_angle_snapping: true,
        }
    }
}

/// A single mode within a genome
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ModeSettings {
    pub name: String,
    pub default_name: String, // Original/default name to revert to when user clears the name
    pub color: Vec3,

    // Cell type
    pub cell_type: i32,

    // Parent settings
    pub parent_make_adhesion: bool,
    pub split_mass: f32,
    pub split_interval: f32,
    pub parent_split_direction: Vec2, // pitch, yaw in degrees
    pub max_adhesions: i32,
    pub enable_parent_angle_snapping: bool,

    // Child settings
    pub child_a: ChildSettings,
    pub child_b: ChildSettings,

    // Adhesion settings
    pub adhesion_settings: AdhesionSettings,
}

impl ModeSettings {
    /// Create a new mode that splits back to itself
    pub fn new_self_splitting(mode_index: i32, name: String) -> Self {
        Self {
            default_name: name.clone(),
            name,
            color: Vec3::new(1.0, 1.0, 1.0),
            cell_type: 0,
            parent_make_adhesion: false,
            split_mass: 1.0,
            split_interval: 5.0,
            parent_split_direction: Vec2::ZERO,
            max_adhesions: 20,
            enable_parent_angle_snapping: true,
            child_a: ChildSettings {
                mode_number: mode_index,
                ..Default::default()
            },
            child_b: ChildSettings {
                mode_number: mode_index,
                ..Default::default()
            },
            adhesion_settings: AdhesionSettings::default(),
        }
    }
}

impl Default for ModeSettings {
    fn default() -> Self {
        Self {
            name: "Untitled Mode".to_string(),
            default_name: "Untitled Mode".to_string(),
            color: Vec3::new(1.0, 1.0, 1.0),
            cell_type: 0,
            parent_make_adhesion: false,
            split_mass: 1.0,
            split_interval: 5.0,
            parent_split_direction: Vec2::ZERO,
            max_adhesions: 20,
            enable_parent_angle_snapping: true,
            child_a: ChildSettings::default(),
            child_b: ChildSettings::default(),
            adhesion_settings: AdhesionSettings::default(),
        }
    }
}

/// A complete genome definition
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GenomeData {
    pub name: String,
    pub initial_mode: i32,
    pub initial_orientation: Quat,
    pub modes: Vec<ModeSettings>,
}

impl GenomeData {
    /// Set all modes to split back to themselves
    pub fn set_all_modes_self_splitting(&mut self) {
        for (idx, mode) in self.modes.iter_mut().enumerate() {
            let mode_idx = idx as i32;
            mode.child_a.mode_number = mode_idx;
            mode.child_b.mode_number = mode_idx;
        }
    }
}

impl Default for GenomeData {
    fn default() -> Self {
        let mut genome = Self {
            name: "Untitled Genome".to_string(),
            initial_mode: 0,
            initial_orientation: Quat::IDENTITY,
            modes: Vec::new(),
        };
        // Initialize with one default mode that splits back to itself
        genome.modes.push(ModeSettings::new_self_splitting(
            0,
            "Default Mode".to_string(),
        ));
        genome
    }
}

impl GenomeData {
    /// Save genome to a JSON file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load genome from a JSON file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let genome = serde_json::from_str(&json)?;
        Ok(genome)
    }
}
