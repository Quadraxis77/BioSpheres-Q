use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use crate::simulation::cpu_physics::CanonicalState;

/// Double-buffered canonical state for lock-free reads during rendering
/// 
/// This structure maintains two copies of the canonical state:
/// - Read buffer: Used by rendering/ECS sync (main thread)
/// - Write buffer: Used by physics computation (worker thread)
/// 
/// After each physics step, buffers are swapped atomically.
#[derive(Resource)]
pub struct DoubleBufferedState {
    /// Current read buffer (safe to read from main thread)
    read_buffer: Arc<Mutex<CanonicalState>>,
    
    /// Current write buffer (used by physics thread)
    write_buffer: Arc<Mutex<CanonicalState>>,
    
    /// Frame counter for debugging
    pub frame_count: u64,
}

impl DoubleBufferedState {
    /// Create a new double-buffered state with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            read_buffer: Arc::new(Mutex::new(CanonicalState::new(capacity))),
            write_buffer: Arc::new(Mutex::new(CanonicalState::new(capacity))),
            frame_count: 0,
        }
    }
    
    /// Get a reference to the read buffer (for rendering/ECS sync)
    /// This is safe to call from the main thread without blocking
    pub fn read(&self) -> Arc<Mutex<CanonicalState>> {
        Arc::clone(&self.read_buffer)
    }
    
    /// Get a reference to the write buffer (for physics computation)
    /// This should only be called from the physics worker thread
    pub fn write(&self) -> Arc<Mutex<CanonicalState>> {
        Arc::clone(&self.write_buffer)
    }
    
    /// Swap read and write buffers
    /// Call this after physics step completes to make new state visible
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.read_buffer, &mut self.write_buffer);
        self.frame_count += 1;
    }
    
    /// Initialize both buffers with the same initial state
    pub fn initialize(&mut self, initial_state: CanonicalState) {
        let cloned_state = initial_state.clone();
        *self.read_buffer.lock().unwrap() = initial_state;
        *self.write_buffer.lock().unwrap() = cloned_state;
        self.frame_count = 0;
    }
}

impl Default for DoubleBufferedState {
    fn default() -> Self {
        Self::new(10_000)
    }
}
