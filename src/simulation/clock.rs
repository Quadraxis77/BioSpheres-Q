use bevy::prelude::*;

/// Unified simulation clock controlling both Main and Preview modes
/// 
/// This resource tracks simulation time and controls how time progresses.
/// It supports:
/// - Fixed timestep physics (deterministic)
/// - Variable speed multiplier (slow motion / fast forward)
/// - Pause functionality
/// - Time accumulation for fixed timestep
/// 
/// The clock ensures that physics always runs at a fixed timestep
/// (e.g., 1/64 second) regardless of frame rate, which is essential
/// for deterministic behavior.
#[derive(Resource, Debug, Clone)]
pub struct SimulationClock {
    /// Current simulation time (in simulation seconds)
    /// This is the logical time in the simulation, not wall-clock time
    pub current_time: f32,
    
    /// Fixed physics timestep (in seconds)
    /// Default: 1/64 second (~15.6ms) - Bevy's default fixed timestep
    /// This value should match PhysicsConfig.fixed_timestep
    pub fixed_dt: f32,
    
    /// Speed multiplier for time progression
    /// - 1.0 = real-time
    /// - 0.5 = half-speed (slow motion)
    /// - 2.0 = double-speed (fast forward)
    /// - 0.0 = paused (alternative to pause flag)
    pub speed_multiplier: f32,
    
    /// Pause flag
    /// When true, time does not advance regardless of frame delta
    pub paused: bool,
    
    /// Accumulated time for fixed timestep
    /// This tracks fractional timesteps that haven't been executed yet
    /// When this exceeds fixed_dt, a physics step is executed
    pub time_accumulator: f32,
}

impl SimulationClock {
    /// Create a new simulation clock with the given fixed timestep
    /// 
    /// # Arguments
    /// * `fixed_dt` - Fixed physics timestep in seconds (e.g., 1.0/64.0)
    /// 
    /// # Returns
    /// A new SimulationClock starting at time 0
    pub fn new(fixed_dt: f32) -> Self {
        Self {
            current_time: 0.0,
            fixed_dt,
            speed_multiplier: 1.0,
            paused: false,
            time_accumulator: 0.0,
        }
    }
    
    /// Calculate how many physics steps to execute this frame
    /// 
    /// This method implements fixed timestep accumulation:
    /// 1. If paused, return 0 steps
    /// 2. Accumulate time scaled by speed multiplier
    /// 3. Calculate how many fixed timesteps fit in accumulated time
    /// 4. Subtract executed time from accumulator
    /// 
    /// # Arguments
    /// * `real_dt` - Real-time delta since last frame (in seconds)
    /// 
    /// # Returns
    /// Number of physics steps to execute this frame
    pub fn steps_this_frame(&mut self, real_dt: f32) -> u32 {
        if self.paused {
            return 0;
        }
        
        // Accumulate time scaled by speed multiplier
        self.time_accumulator += real_dt * self.speed_multiplier;
        
        // Calculate number of fixed steps
        let steps = (self.time_accumulator / self.fixed_dt).floor() as u32;
        
        // Subtract executed time from accumulator
        self.time_accumulator -= steps as f32 * self.fixed_dt;
        
        steps
    }
    
    /// Advance clock by one physics step
    /// 
    /// This should be called after each physics step is executed.
    /// It advances the simulation time by exactly one fixed timestep.
    pub fn advance_step(&mut self) {
        self.current_time += self.fixed_dt;
    }
    
    /// Reset the clock to time zero
    /// 
    /// This is useful when restarting the simulation or switching modes.
    /// The accumulator is also cleared.
    pub fn reset(&mut self) {
        self.current_time = 0.0;
        self.time_accumulator = 0.0;
    }
    
    /// Set the simulation time directly
    /// 
    /// This is primarily used by Preview mode to set the target time.
    /// The accumulator is cleared when setting time directly.
    /// 
    /// # Arguments
    /// * `time` - New simulation time in seconds
    pub fn set_time(&mut self, time: f32) {
        self.current_time = time;
        self.time_accumulator = 0.0;
    }
    
    /// Pause the simulation
    pub fn pause(&mut self) {
        self.paused = true;
    }
    
    /// Resume the simulation
    pub fn resume(&mut self) {
        self.paused = false;
    }
    
    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
    
    /// Set the speed multiplier
    /// 
    /// # Arguments
    /// * `multiplier` - New speed multiplier (0.0 = paused, 1.0 = real-time, 2.0 = double-speed)
    pub fn set_speed(&mut self, multiplier: f32) {
        self.speed_multiplier = multiplier.max(0.0); // Clamp to non-negative
    }
    
    /// Get the current simulation time
    pub fn time(&self) -> f32 {
        self.current_time
    }
    
    /// Check if the simulation is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }
}

impl Default for SimulationClock {
    fn default() -> Self {
        // Default to Bevy's standard fixed timestep: 64 Hz
        Self::new(1.0 / 64.0)
    }
}

