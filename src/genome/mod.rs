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
    /// Whether to highlight cells of the selected mode with a pulsing glow
    pub show_mode_glow: bool,
}

impl Default for CurrentGenome {
    fn default() -> Self {
        Self {
            genome: GenomeData::default(),
            selected_mode_index: 0,
            show_mode_glow: true,
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
    pub opacity: f32, // Cell transparency (0.0 = fully transparent, 1.0 = fully opaque)
    #[serde(default)]
    pub emissive: f32, // Emissive glow intensity (0.0 = no glow, 1.0+ = bright glow)

    // Cell type
    pub cell_type: i32,

    // Parent settings
    pub parent_make_adhesion: bool,
    pub split_mass: f32,
    #[serde(default)]
    pub split_mass_min: Option<f32>, // If Some, split_mass is the max and this is the min for random range
    pub split_interval: f32,
    #[serde(default)]
    pub split_interval_min: Option<f32>, // If Some, split_interval is the max and this is the min for random range
    pub nutrient_gain_rate: f32, // Mass gained per second (for Test cells)
    pub max_cell_size: f32, // Maximum visual size (1.0 to 2.0 units)
    pub split_ratio: f32, // Ratio of parent mass going to Child A (0.0 to 1.0, default 0.5 for 50/50 split)
    pub nutrient_priority: f32, // Priority for nutrient transport (0.1 to 10.0, default 1.0)
    pub prioritize_when_low: bool, // When enabled, priority increases when nutrients are low to prevent death
    pub parent_split_direction: Vec2, // pitch, yaw in degrees
    pub max_adhesions: i32,
    pub min_adhesions: i32, // Minimum number of connections required before cell can split
    pub enable_parent_angle_snapping: bool,
    pub max_splits: i32, // Maximum number of times a cell can split (1-20, or -1 for infinite). Split count resets to 0 when switching modes
    pub mode_a_after_splits: i32, // Mode that Child A transitions to when max_splits is reached (-1 = use normal child_a mode)
    pub mode_b_after_splits: i32, // Mode that Child B transitions to when max_splits is reached (-1 = use normal child_b mode)
    
    // Flagellocyte settings
    pub swim_force: f32, // Forward thrust force (0.0 to 1.0, for Flagellocyte cells)

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
            opacity: 1.0, // Default: fully opaque
            emissive: 0.0, // Default: no glow
            cell_type: 0,
            parent_make_adhesion: false,
            split_mass: 1.5,
            split_mass_min: None,
            split_interval: 5.0,
            split_interval_min: None,
            nutrient_gain_rate: 0.2, // Default: gain 0.2 mass per second
            max_cell_size: 2.0, // Default: max size of 2.0 units
            split_ratio: 0.5, // Default: 50/50 split
            nutrient_priority: 1.0, // Default: neutral priority
            prioritize_when_low: true, // Default: protect cells from death
            parent_split_direction: Vec2::ZERO,
            max_adhesions: 20,
            min_adhesions: 0, // No minimum by default
            enable_parent_angle_snapping: true,
            max_splits: -1, // Infinite by default
            mode_a_after_splits: -1, // Use normal child_a mode by default
            mode_b_after_splits: -1, // Use normal child_b mode by default
            swim_force: 0.5, // Default swim force for flagellocytes
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

    /// Get the effective split mass for a cell, using deterministic randomness if a range is set
    /// 
    /// # Arguments
    /// * `cell_id` - Unique cell identifier
    /// * `tick` - Current simulation tick
    /// * `seed` - Global random seed
    pub fn get_split_mass(&self, cell_id: u32, tick: u64, seed: u64) -> f32 {
        match self.split_mass_min {
            Some(min) => {
                let t = crate::simulation::deterministic_random(cell_id, tick, seed, 0);
                min + t * (self.split_mass - min)
            }
            None => self.split_mass,
        }
    }

    /// Get the effective split interval for a cell, using deterministic randomness if a range is set
    /// 
    /// # Arguments
    /// * `cell_id` - Unique cell identifier
    /// * `tick` - Current simulation tick
    /// * `seed` - Global random seed
    pub fn get_split_interval(&self, cell_id: u32, tick: u64, seed: u64) -> f32 {
        match self.split_interval_min {
            Some(min) => {
                let t = crate::simulation::deterministic_random(cell_id, tick, seed, 1);
                min + t * (self.split_interval - min)
            }
            None => self.split_interval,
        }
    }
}

impl Default for ModeSettings {
    fn default() -> Self {
        Self {
            name: "Untitled Mode".to_string(),
            default_name: "Untitled Mode".to_string(),
            color: Vec3::new(1.0, 1.0, 1.0),
            opacity: 1.0, // Default: fully opaque
            emissive: 0.0, // Default: no glow
            cell_type: 0,
            parent_make_adhesion: false,
            split_mass: 1.5,
            split_mass_min: None,
            split_interval: 5.0,
            split_interval_min: None,
            nutrient_gain_rate: 0.2, // Default: gain 0.2 mass per second
            max_cell_size: 2.0, // Default: max size of 2.0 units
            split_ratio: 0.5, // Default: 50/50 split
            nutrient_priority: 1.0, // Default: neutral priority
            prioritize_when_low: true, // Default: protect cells from death
            parent_split_direction: Vec2::ZERO,
            max_adhesions: 20,
            min_adhesions: 0, // No minimum by default
            enable_parent_angle_snapping: true,
            max_splits: -1, // Infinite by default
            mode_a_after_splits: -1, // Use normal child_a mode by default
            mode_b_after_splits: -1, // Use normal child_b mode by default
            swim_force: 0.5, // Default swim force for flagellocytes
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
            "Mode 0".to_string(),
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
