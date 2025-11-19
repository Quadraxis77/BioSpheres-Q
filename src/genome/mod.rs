use bevy::prelude::*;

/// Plugin for genome management
pub struct GenomePlugin;

impl Plugin for GenomePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GenomeLibrary>()
            .init_resource::<CurrentGenome>();
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
#[derive(Clone)]
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
            orientation_spring_stiffness: 10.0,
            orientation_spring_damping: 2.0,
            max_angular_deviation: 0.0,
            twist_constraint_stiffness: 2.0,
            twist_constraint_damping: 0.5,
            enable_twist_constraint: true,
        }
    }
}

/// Child settings for mode transitions
#[derive(Clone)]
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
#[derive(Clone)]
pub struct ModeSettings {
    pub name: String,
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

impl Default for ModeSettings {
    fn default() -> Self {
        Self {
            name: "Untitled Mode".to_string(),
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
#[derive(Clone)]
pub struct GenomeData {
    pub name: String,
    pub initial_mode: i32,
    pub initial_orientation: Quat,
    pub modes: Vec<ModeSettings>,
}

impl Default for GenomeData {
    fn default() -> Self {
        let mut genome = Self {
            name: "Untitled Genome".to_string(),
            initial_mode: 0,
            initial_orientation: Quat::IDENTITY,
            modes: Vec::new(),
        };
        // Initialize with one default mode
        genome.modes.push(ModeSettings {
            name: "Default Mode".to_string(),
            color: Vec3::new(1.0, 1.0, 1.0),
            ..Default::default()
        });
        genome
    }
}
