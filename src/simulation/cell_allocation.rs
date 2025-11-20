//! Deterministic Cell Allocation System
//!
//! This module provides a deterministic cell division and adhesion allocation system
//! using a reservation-based approach that ensures deterministic behavior.
//!
//! The core allocation pipeline consists of five sequential stages:
//! 1. **Identify Free Slots** - Scan cell and adhesion arrays to find available indices
//! 2. **Generate Reservations** - Create reservation requests for child cells and inherited adhesions
//! 3. **Compact** - Sort and consolidate free slots and reservations
//! 4. **Assign** - Match reservations to available free slots
//! 5. **Write** - Initialize new cells and adhesions at assigned indices

use bevy::prelude::*;

/// Sentinel value used to mark invalid or unallocated slots
const BIG: u32 = u32::MAX;

/// Maximum number of adhesion slots per cell (20 for child A + 20 for child B)
const MAX_ADH_PER_CELL: usize = 40;

/// Cell component representing a simulated entity in the system
///
/// Cells have properties including age, adhesion creation flags, and inheritance arrays
/// that determine how adhesions are passed to child cells during division.
#[derive(Component, Clone, Default)]
pub struct Cell {
    /// Cell age in simulation steps; -1 indicates a free slot
    pub age: i32,
    /// Flag indicating whether to create adhesion between child cells
    pub make_adhesion: bool,
    /// Array of 20 booleans indicating which parent adhesions child A inherits
    pub child_a_inherits: [bool; 20],
    /// Array of 20 booleans indicating which parent adhesions child B inherits
    pub child_b_inherits: [bool; 20],
}

/// Adhesion component representing a connection between two cells
///
/// Currently a marker component that can be extended with connection data
/// in future iterations.
#[derive(Component, Clone)]
pub struct Adhesion;

/// Simulation resource managing cell and adhesion allocation
///
/// This resource contains all the data structures needed for the 5-stage
/// allocation pipeline: identify free slots, generate reservations, compact,
/// assign, and write.
#[derive(Resource)]
pub struct Simulation {
    // Core data
    pub(crate) cells: Vec<Cell>,
    #[allow(dead_code)]
    pub(crate) adhesions: Vec<Option<Adhesion>>,
    
    // Allocation tracking
    pub(crate) free_cell_slots: Vec<u32>,
    pub(crate) free_adh_slots: Vec<u32>,
    pub(crate) cell_reservations: Vec<u32>,
    pub(crate) adhesion_reservations: Vec<u32>,
    pub(crate) assignments_cells: Vec<u32>,
    pub(crate) assignments_adhesions: Vec<u32>,
}

impl Simulation {
    /// Create a new Simulation with specified capacities
    ///
    /// Pre-allocates all vectors to avoid runtime allocations during simulation.
    ///
    /// # Arguments
    ///
    /// * `cell_capacity` - Maximum number of cells the simulation can hold
    /// * `adh_capacity` - Maximum number of adhesions the simulation can hold
    ///
    /// # Returns
    ///
    /// A new Simulation instance with all vectors pre-allocated
    pub fn new(cell_capacity: usize, adh_capacity: usize) -> Self {
        // Pre-allocate core data vectors
        let mut cells = Vec::with_capacity(cell_capacity);
        cells.resize(cell_capacity, Cell::default());
        
        let mut adhesions = Vec::with_capacity(adh_capacity);
        adhesions.resize(adh_capacity, None);
        
        // Pre-allocate allocation tracking vectors
        // These are sized for worst-case scenarios
        let free_cell_slots = Vec::with_capacity(cell_capacity);
        let free_adh_slots = Vec::with_capacity(adh_capacity);
        
        // Reservation arrays: worst case is all cells divide
        let cell_reservations = Vec::with_capacity(cell_capacity * 2);
        let adhesion_reservations = Vec::with_capacity(cell_capacity * MAX_ADH_PER_CELL);
        
        // Assignment arrays: sized to match reservation arrays
        let assignments_cells = Vec::with_capacity(cell_capacity * 2);
        let assignments_adhesions = Vec::with_capacity(cell_capacity * MAX_ADH_PER_CELL);
        
        Self {
            cells,
            adhesions,
            free_cell_slots,
            free_adh_slots,
            cell_reservations,
            adhesion_reservations,
            assignments_cells,
            assignments_adhesions,
        }
    }

    /// Stage 1: Identify free slots in cell and adhesion buffers
    ///
    /// Scans the cell array and writes indices of free slots (age == -1) to free_cell_slots.
    /// Occupied slots are marked with the sentinel value BIG.
    pub(crate) fn identify_free_cell_slots(&mut self) {
        self.free_cell_slots.clear();
        self.free_cell_slots.resize(self.cells.len(), BIG);
        
        for (i, cell) in self.cells.iter().enumerate() {
            if cell.age == -1 {
                // Free slot: write the index
                self.free_cell_slots[i] = i as u32;
            } else {
                // Occupied slot: write sentinel value
                self.free_cell_slots[i] = BIG;
            }
        }
    }

    /// Stage 1: Identify free slots in adhesion buffers
    ///
    /// Scans the adhesion array and writes indices of free slots (None) to free_adh_slots.
    /// Occupied slots are marked with the sentinel value BIG.
    #[allow(dead_code)]
    pub(crate) fn identify_free_adhesion_slots(&mut self) {
        self.free_adh_slots.clear();
        self.free_adh_slots.resize(self.adhesions.len(), BIG);
        
        for (i, adhesion) in self.adhesions.iter().enumerate() {
            if adhesion.is_none() {
                // Free slot: write the index
                self.free_adh_slots[i] = i as u32;
            } else {
                // Occupied slot: write sentinel value
                self.free_adh_slots[i] = BIG;
            }
        }
    }

    /// Stage 2: Generate reservations for child cells and adhesions
    ///
    /// For each dividing cell at index N:
    /// - Reserves cell slots 2*N (child A) and 2*N+1 (child B)
    /// - Reserves adhesion slots based on inheritance arrays
    /// - Handles make_adhesion flag for new adhesions between children
    pub(crate) fn generate_reservations(&mut self) {
        // Clear and resize reservation arrays with sentinel values
        self.cell_reservations.clear();
        self.cell_reservations.resize(self.cells.len() * 2, BIG);
        
        self.adhesion_reservations.clear();
        self.adhesion_reservations.resize(self.cells.len() * MAX_ADH_PER_CELL, BIG);
        
        // Process each cell
        for (i, cell) in self.cells.iter().enumerate() {
            // Only process cells that are dividing (age != -1)
            if cell.age == -1 {
                continue;
            }
            
            // Reserve cell slots for children
            // Child A at 2*i, Child B at 2*i+1
            let child_a_idx = 2 * i;
            let child_b_idx = 2 * i + 1;
            
            if child_a_idx < self.cell_reservations.len() {
                self.cell_reservations[child_a_idx] = child_a_idx as u32;
            }
            
            if child_b_idx < self.cell_reservations.len() {
                self.cell_reservations[child_b_idx] = child_b_idx as u32;
            }
            
            // Mark parent cell as free (will be freed after division)
            if i < self.free_cell_slots.len() {
                self.free_cell_slots[i] = i as u32;
            }
            
            // Process adhesion inheritance
            // Child A adhesion slots: 40*i + 0 through 40*i + 19
            // Child B adhesion slots: 40*i + 20 through 40*i + 39
            let base_adh_idx = i * MAX_ADH_PER_CELL;
            
            for adh_slot in 0..20 {
                // Child A inherits adhesion at position adh_slot
                if cell.child_a_inherits[adh_slot] {
                    let reservation_idx = base_adh_idx + adh_slot;
                    if reservation_idx < self.adhesion_reservations.len() {
                        self.adhesion_reservations[reservation_idx] = reservation_idx as u32;
                    }
                }
                
                // Child B inherits adhesion at position adh_slot
                if cell.child_b_inherits[adh_slot] {
                    let reservation_idx = base_adh_idx + 20 + adh_slot;
                    if reservation_idx < self.adhesion_reservations.len() {
                        self.adhesion_reservations[reservation_idx] = reservation_idx as u32;
                    }
                }
            }
            
            // Handle make_adhesion flag for new adhesion between children
            if cell.make_adhesion {
                // Try to reserve slot 40*i+19 (last slot for child A)
                let slot_a = base_adh_idx + 19;
                if slot_a < self.adhesion_reservations.len() && self.adhesion_reservations[slot_a] == BIG {
                    self.adhesion_reservations[slot_a] = slot_a as u32;
                } else {
                    // If slot 19 is taken, try slot 40*i+39 (last slot for child B)
                    let slot_b = base_adh_idx + 39;
                    if slot_b < self.adhesion_reservations.len() {
                        self.adhesion_reservations[slot_b] = slot_b as u32;
                    }
                }
            }
        }
    }

    /// Stage 3: Compact arrays by sorting and removing sentinel values
    ///
    /// This helper function sorts an array in ascending order and removes all
    /// sentinel values (BIG) by truncating the array. After compaction, all
    /// valid indices appear at the beginning in sorted order.
    ///
    /// # Arguments
    ///
    /// * `array` - Mutable reference to the vector to compact
    fn compact(array: &mut Vec<u32>) {
        // Sort the array in ascending order (sentinel values BIG will be at the end)
        array.sort_unstable();
        
        // Find the first occurrence of BIG (sentinel value)
        // All valid indices are before this point
        let valid_count = array.iter().position(|&x| x == BIG).unwrap_or(array.len());
        
        // Truncate to remove all sentinel values
        array.truncate(valid_count);
    }

    /// Stage 3: Compact all allocation arrays
    ///
    /// Applies the compact operation to free slot arrays and reservation arrays.
    /// After compaction, all arrays contain only valid indices in sorted order.
    pub(crate) fn compact_arrays(&mut self) {
        Self::compact(&mut self.free_cell_slots);
        Self::compact(&mut self.free_adh_slots);
        Self::compact(&mut self.cell_reservations);
        Self::compact(&mut self.adhesion_reservations);
    }

    /// Stage 4: Assign reservations to free slots
    ///
    /// Maps reservation IDs to available free slot indices. This stage creates
    /// assignment arrays that map each reservation ID to the actual slot index
    /// where the cell or adhesion will be written.
    ///
    /// The assignment process:
    /// 1. Calculate how many assignments can be made (min of free slots and reservations)
    /// 2. Find the maximum reservation ID to size the assignment array
    /// 3. Initialize assignment arrays with sentinel values
    /// 4. Map each reservation ID to a free slot index
    pub(crate) fn assign_reservations(&mut self) {
        // Assign cell reservations to free cell slots
        self.assign_cell_reservations();
        
        // Assign adhesion reservations to free adhesion slots
        self.assign_adhesion_reservations();
    }

    /// Helper function to assign cell reservations to free cell slots
    fn assign_cell_reservations(&mut self) {
        // Calculate how many assignments we can make
        let num_assignments = self.free_cell_slots.len().min(self.cell_reservations.len());
        
        // Find the maximum reservation ID to determine assignment array size
        let max_reservation_id = self.cell_reservations
            .iter()
            .max()
            .copied()
            .unwrap_or(0) as usize;
        
        // Resize assignment array to accommodate all possible reservation IDs
        // Initialize with sentinel values
        self.assignments_cells.clear();
        self.assignments_cells.resize(max_reservation_id + 1, BIG);
        
        // Map each reservation ID to a free slot index
        for i in 0..num_assignments {
            let reservation_id = self.cell_reservations[i] as usize;
            let free_slot_idx = self.free_cell_slots[i];
            
            // Bounds check before assignment
            if reservation_id < self.assignments_cells.len() {
                self.assignments_cells[reservation_id] = free_slot_idx;
            }
        }
    }

    /// Helper function to assign adhesion reservations to free adhesion slots
    fn assign_adhesion_reservations(&mut self) {
        // Calculate how many assignments we can make
        let num_assignments = self.free_adh_slots.len().min(self.adhesion_reservations.len());
        
        // Find the maximum reservation ID to determine assignment array size
        let max_reservation_id = self.adhesion_reservations
            .iter()
            .max()
            .copied()
            .unwrap_or(0) as usize;
        
        // Resize assignment array to accommodate all possible reservation IDs
        // Initialize with sentinel values
        self.assignments_adhesions.clear();
        self.assignments_adhesions.resize(max_reservation_id + 1, BIG);
        
        // Map each reservation ID to a free slot index
        for i in 0..num_assignments {
            let reservation_id = self.adhesion_reservations[i] as usize;
            let free_slot_idx = self.free_adh_slots[i];
            
            // Bounds check before assignment
            if reservation_id < self.assignments_adhesions.len() {
                self.assignments_adhesions[reservation_id] = free_slot_idx;
            }
        }
    }

    /// Stage 5: Write to assigned slots
    ///
    /// Iterates through assignment arrays and writes default cells and adhesions
    /// to the assigned indices. This stage performs bounds checking to ensure
    /// safe writes and skips invalid assignments (marked with BIG sentinel).
    ///
    /// For cells: writes a default-initialized Cell to each assigned index
    /// For adhesions: writes Some(Adhesion) to each assigned index
    #[allow(dead_code)]
    pub(crate) fn write_to_assigned_slots(&mut self) {
        // Write cells to assigned slots
        self.write_assigned_cells();
        
        // Write adhesions to assigned slots
        self.write_assigned_adhesions();
    }

    /// Helper function to write cells to assigned slots
    #[allow(dead_code)]
    fn write_assigned_cells(&mut self) {
        // Iterate through all cell assignments
        for &assigned_slot_idx in &self.assignments_cells {
            // Skip sentinel values (unassigned reservations)
            if assigned_slot_idx == BIG {
                continue;
            }
            
            // Bounds check before writing
            let slot_idx = assigned_slot_idx as usize;
            if slot_idx < self.cells.len() {
                // Write default-initialized Cell to the assigned slot
                self.cells[slot_idx] = Cell::default();
            }
        }
    }

    /// Helper function to write adhesions to assigned slots
    #[allow(dead_code)]
    fn write_assigned_adhesions(&mut self) {
        // Iterate through all adhesion assignments
        for &assigned_slot_idx in &self.assignments_adhesions {
            // Skip sentinel values (unassigned reservations)
            if assigned_slot_idx == BIG {
                continue;
            }
            
            // Bounds check before writing
            let slot_idx = assigned_slot_idx as usize;
            if slot_idx < self.adhesions.len() {
                // Write Some(Adhesion) to the assigned slot
                self.adhesions[slot_idx] = Some(Adhesion);
            }
        }
    }
}

/// Builder for configuring CellSimulationPlugin
///
/// Provides a fluent API for setting custom capacities before building the plugin.
pub struct CellSimulationPluginBuilder {
    cell_capacity: usize,
    adhesion_capacity: usize,
}

impl Default for CellSimulationPluginBuilder {
    fn default() -> Self {
        Self {
            cell_capacity: 64,
            adhesion_capacity: 256,
        }
    }
}

impl CellSimulationPluginBuilder {
    pub fn with_cell_capacity(mut self, capacity: usize) -> Self {
        self.cell_capacity = capacity;
        self
    }

    pub fn with_adhesion_capacity(mut self, capacity: usize) -> Self {
        self.adhesion_capacity = capacity;
        self
    }

    pub fn build(self) -> CellSimulationPlugin {
        CellSimulationPlugin {
            cell_capacity: self.cell_capacity,
            adhesion_capacity: self.adhesion_capacity,
        }
    }
}

/// Bevy plugin for deterministic cell division and adhesion allocation
pub struct CellSimulationPlugin {
    cell_capacity: usize,
    adhesion_capacity: usize,
}

impl Default for CellSimulationPlugin {
    fn default() -> Self {
        Self {
            cell_capacity: 64,
            adhesion_capacity: 256,
        }
    }
}

impl CellSimulationPlugin {
    pub fn builder() -> CellSimulationPluginBuilder {
        CellSimulationPluginBuilder::default()
    }
}

impl Plugin for CellSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Simulation::new(self.cell_capacity, self.adhesion_capacity));
        // Note: setup and simulation_step systems removed - they were demo/test systems
        // The Simulation resource is used directly by division_step in cpu_physics
    }
}

/// Bevy system that initializes demo cells for testing the allocation pipeline
#[allow(dead_code)]
pub(crate) fn setup(mut sim: ResMut<Simulation>) {
    if sim.cells.len() < 3 {
        return;
    }
    
    sim.cells[0].age = -1;
    sim.cells[0].make_adhesion = false;
    sim.cells[0].child_a_inherits = [false; 20];
    sim.cells[0].child_b_inherits = [false; 20];
    
    sim.cells[1].age = 5;
    sim.cells[1].make_adhesion = true;
    sim.cells[1].child_a_inherits[0] = true;
    sim.cells[1].child_b_inherits[1] = true;
    
    sim.cells[2].age = -1;
    sim.cells[2].make_adhesion = false;
    sim.cells[2].child_a_inherits = [false; 20];
    sim.cells[2].child_b_inherits = [false; 20];
}

/// Bevy system that executes the complete 5-stage allocation pipeline
#[allow(dead_code)]
pub(crate) fn simulation_step(mut sim: ResMut<Simulation>) {
    sim.identify_free_cell_slots();
    sim.identify_free_adhesion_slots();
    sim.generate_reservations();
    sim.compact_arrays();
    sim.assign_reservations();
    sim.write_to_assigned_slots();
}
