use bevy::prelude::*;
use crate::simulation::PhysicsConfig;
use crate::simulation::cpu_physics::CanonicalState;

/// Immutable initial state for deterministic replay
/// 
/// This structure captures everything needed to replay the simulation
/// from time zero. It is created once at the start of a simulation
/// and remains immutable throughout the simulation lifetime.
/// 
/// Both Main and Preview simulation modes initialize from this state,
/// ensuring they start from identical conditions.
#[derive(Clone)]
pub struct InitialState {
    /// Physics configuration (timestep, damping, boundaries, etc.)
    pub config: PhysicsConfig,
    
    /// Maximum number of cells allowed in the simulation
    pub max_cells: usize,
    
    /// Initial cell state (positions, velocities, properties)
    pub initial_cells: Vec<InitialCell>,
    
    /// Deterministic RNG seed for reproducible randomness
    pub rng_seed: u64,
    
    /// Timestamp when this state was created (for debugging/logging)
    pub created_at: f64,
    
    /// Spatial grid density (NxNxN cells). Valid range: 16-128
    pub grid_density: u32,
}

/// Initial state for a single cell
/// 
/// Contains all properties needed to recreate a cell in the canonical state.
/// This is a snapshot of the cell at time zero.
#[derive(Clone, Debug)]
pub struct InitialCell {
    /// Unique cell identifier (determines iteration order)
    pub id: u32,
    
    /// Initial position in world space
    pub position: Vec3,
    
    /// Initial velocity
    pub velocity: Vec3,
    
    /// Initial rotation (orientation)
    pub rotation: Quat,
    
    /// Initial angular velocity
    pub angular_velocity: Vec3,
    
    /// Cell mass (affects physics)
    pub mass: f32,
    
    /// Cell radius (affects collisions)
    pub radius: f32,
    
    /// Genome identifier (which genome this cell uses)
    pub genome_id: usize,
    
    /// Current mode index within the genome
    pub mode_index: usize,
    
    /// Time when this cell was born (0.0 for initial cells)
    pub birth_time: f32,
    
    /// Time interval until cell division
    pub split_interval: f32,
    
    /// Mass threshold for cell division (may be randomized from range)
    pub split_mass: f32,
    
    /// Cytoskeleton stiffness (affects collision response)
    pub stiffness: f32,
}

impl InitialState {
    /// Create a new initial state with the given configuration
    /// 
    /// # Arguments
    /// * `config` - Physics configuration
    /// * `max_cells` - Maximum number of cells allowed
    /// * `rng_seed` - Seed for deterministic random number generation
    /// 
    /// # Returns
    /// A new InitialState with no cells
    pub fn new(config: PhysicsConfig, max_cells: usize, rng_seed: u64) -> Self {
        Self {
            config,
            max_cells,
            initial_cells: Vec::new(),
            rng_seed,
            created_at: 0.0, // Will be set when actually created
            grid_density: 64, // Default grid density
        }
    }
    
    /// Create a new initial state with custom grid density
    pub fn with_grid_density(config: PhysicsConfig, max_cells: usize, rng_seed: u64, grid_density: u32) -> Self {
        Self {
            config,
            max_cells,
            initial_cells: Vec::new(),
            rng_seed,
            created_at: 0.0,
            grid_density,
        }
    }
    
    /// Add a cell to the initial state
    /// 
    /// # Arguments
    /// * `cell` - The initial cell to add
    pub fn add_cell(&mut self, cell: InitialCell) {
        self.initial_cells.push(cell);
    }
    
    /// Convert this initial state to a canonical state
    /// 
    /// This creates a new CanonicalState with all cells from the initial state.
    /// The canonical state is ready to be simulated forward in time.
    /// 
    /// # Returns
    /// A new CanonicalState initialized from this initial state
    pub fn to_canonical_state(&self) -> CanonicalState {
        let mut state = CanonicalState::with_grid_density(self.max_cells, self.grid_density);
        
        // Add all initial cells to the canonical state
        for cell in &self.initial_cells {
            state.add_cell(
                cell.position,
                cell.velocity,
                cell.rotation,
                cell.angular_velocity,
                cell.mass,
                cell.radius,
                cell.genome_id,
                cell.mode_index,
                cell.birth_time,
                cell.split_interval,
                cell.split_mass,
                cell.stiffness,
                cell.rotation, // genome_orientation = rotation initially
                0, // Initial cells start with split_count = 0
            );
        }
        
        state
    }
    
    /// Get the number of initial cells
    pub fn cell_count(&self) -> usize {
        self.initial_cells.len()
    }
}

impl Default for InitialState {
    fn default() -> Self {
        Self::with_grid_density(PhysicsConfig::default(), 10_000, 0, 64)
    }
}

