//! Triple buffer system for GPU instance data.
//!
//! This module implements a lock-free triple buffering system that allows
//! concurrent simulation updates and rendering without CPU-GPU synchronization stalls.
//! The design follows the Biospheres.cpp pattern for buffer management.

use std::sync::atomic::{AtomicUsize, Ordering};
use bytemuck::cast_slice;

use super::gpu_types::CellInstanceData;

/// Default maximum number of cell instances to pre-allocate.
pub const DEFAULT_MAX_INSTANCES: usize = 100_000;

/// A single instance buffer with CPU and GPU storage.
struct InstanceBuffer {
    /// CPU-side data for this buffer
    cpu_data: Vec<CellInstanceData>,
    /// GPU buffer for rendering
    gpu_buffer: wgpu::Buffer,
    /// Number of active instances in this buffer
    instance_count: usize,
}

impl InstanceBuffer {
    /// Create a new instance buffer with the given capacity.
    fn new(device: &wgpu::Device, capacity: usize, label: &str) -> Self {
        let cpu_data = Vec::with_capacity(capacity);
        
        // Create GPU buffer with enough space for max instances
        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (capacity * CellInstanceData::SIZE) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            cpu_data,
            gpu_buffer,
            instance_count: 0,
        }
    }
}

/// Lock-free triple buffering system for cell instance data.
///
/// The system maintains three buffers that rotate through roles:
/// - **Write buffer**: Simulation writes new data here
/// - **Standby buffer**: Recently written data waiting to be read
/// - **Read buffer**: Renderer reads from here for GPU upload
///
/// Buffer rotation follows: write → standby → read → write
pub struct TripleBufferSystem {
    /// The three instance buffers
    buffers: [InstanceBuffer; 3],
    /// Index of the buffer currently being written to
    write_index: AtomicUsize,
    /// Index of the buffer ready for reading
    read_index: AtomicUsize,
    /// Index of the standby buffer
    standby_index: AtomicUsize,
    /// Maximum capacity per buffer
    capacity: usize,
}

impl TripleBufferSystem {
    /// Create a new triple buffer system with pre-allocated capacity.
    ///
    /// # Arguments
    /// * `device` - The wgpu device for creating GPU buffers
    /// * `max_instances` - Maximum number of cell instances to support
    pub fn new(device: &wgpu::Device, max_instances: usize) -> Self {
        let buffers = [
            InstanceBuffer::new(device, max_instances, "Instance Buffer 0"),
            InstanceBuffer::new(device, max_instances, "Instance Buffer 1"),
            InstanceBuffer::new(device, max_instances, "Instance Buffer 2"),
        ];

        Self {
            buffers,
            write_index: AtomicUsize::new(0),
            read_index: AtomicUsize::new(1),
            standby_index: AtomicUsize::new(2),
            capacity: max_instances,
        }
    }

    /// Get the maximum capacity of each buffer.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the current write buffer index.
    pub fn write_index(&self) -> usize {
        self.write_index.load(Ordering::Acquire)
    }

    /// Get the current read buffer index.
    pub fn read_index(&self) -> usize {
        self.read_index.load(Ordering::Acquire)
    }

    /// Get the current standby buffer index.
    pub fn standby_index(&self) -> usize {
        self.standby_index.load(Ordering::Acquire)
    }

    /// Get mutable access to the write buffer's CPU data.
    ///
    /// # Safety
    /// This should only be called from the simulation thread.
    /// The caller must ensure no concurrent access to the write buffer.
    pub fn write_buffer_mut(&mut self) -> &mut Vec<CellInstanceData> {
        let idx = self.write_index.load(Ordering::Acquire);
        &mut self.buffers[idx].cpu_data
    }

    /// Get read-only access to the write buffer's CPU data.
    pub fn write_buffer(&self) -> &[CellInstanceData] {
        let idx = self.write_index.load(Ordering::Acquire);
        &self.buffers[idx].cpu_data
    }

    /// Set the instance count for the write buffer.
    pub fn set_write_instance_count(&mut self, count: usize) {
        let idx = self.write_index.load(Ordering::Acquire);
        self.buffers[idx].instance_count = count.min(self.capacity);
    }

    /// Get the instance count for the read buffer.
    pub fn read_instance_count(&self) -> usize {
        let idx = self.read_index.load(Ordering::Acquire);
        self.buffers[idx].instance_count
    }

    /// Rotate buffers: write → standby, standby → read, read → write.
    ///
    /// This operation is lock-free and uses atomic operations to ensure
    /// consistency across threads.
    pub fn rotate(&self) {
        // Load current indices
        let write = self.write_index.load(Ordering::Acquire);
        let read = self.read_index.load(Ordering::Acquire);
        let standby = self.standby_index.load(Ordering::Acquire);

        // Rotate: write → standby → read → write
        // New write = old read
        // New read = old standby  
        // New standby = old write
        self.write_index.store(read, Ordering::Release);
        self.read_index.store(standby, Ordering::Release);
        self.standby_index.store(write, Ordering::Release);
    }

    /// Upload the read buffer data to the GPU.
    ///
    /// This performs a non-blocking upload using `queue.write_buffer`.
    pub fn upload_to_gpu(&self, queue: &wgpu::Queue) {
        let idx = self.read_index.load(Ordering::Acquire);
        let buffer = &self.buffers[idx];
        
        if buffer.instance_count > 0 && !buffer.cpu_data.is_empty() {
            let data_to_upload = &buffer.cpu_data[..buffer.instance_count.min(buffer.cpu_data.len())];
            queue.write_buffer(&buffer.gpu_buffer, 0, cast_slice(data_to_upload));
        }
    }

    /// Get the GPU buffer for rendering (the read buffer).
    pub fn render_buffer(&self) -> &wgpu::Buffer {
        let idx = self.read_index.load(Ordering::Acquire);
        &self.buffers[idx].gpu_buffer
    }

    /// Clear all buffers and reset instance counts.
    pub fn clear(&mut self) {
        for buffer in &mut self.buffers {
            buffer.cpu_data.clear();
            buffer.instance_count = 0;
        }
    }

    /// Add a single instance to the write buffer.
    ///
    /// Returns `true` if the instance was added, `false` if the buffer is full.
    pub fn push_instance(&mut self, instance: CellInstanceData) -> bool {
        let idx = self.write_index.load(Ordering::Acquire);
        let buffer = &mut self.buffers[idx];
        
        if buffer.cpu_data.len() < self.capacity {
            buffer.cpu_data.push(instance);
            buffer.instance_count = buffer.cpu_data.len();
            true
        } else {
            false
        }
    }

    /// Set all instances in the write buffer at once.
    ///
    /// This clears the existing data and copies the new instances.
    /// If `instances` exceeds capacity, only the first `capacity` instances are used.
    pub fn set_instances(&mut self, instances: &[CellInstanceData]) {
        let idx = self.write_index.load(Ordering::Acquire);
        let buffer = &mut self.buffers[idx];
        
        buffer.cpu_data.clear();
        let count = instances.len().min(self.capacity);
        buffer.cpu_data.extend_from_slice(&instances[..count]);
        buffer.instance_count = count;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Create a mock wgpu device for testing.
    /// This uses pollster to block on async device creation.
    fn create_test_device() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: true,
        }))
        .expect("Failed to find adapter");

        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Test Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        }))
        .expect("Failed to create device")
    }

    #[test]
    fn test_triple_buffer_creation() {
        let (device, _queue) = create_test_device();
        let buffer_system = TripleBufferSystem::new(&device, 1000);
        
        assert_eq!(buffer_system.capacity(), 1000);
        assert_eq!(buffer_system.write_index(), 0);
        assert_eq!(buffer_system.read_index(), 1);
        assert_eq!(buffer_system.standby_index(), 2);
    }

    #[test]
    fn test_buffer_rotation_cycle() {
        let (device, _queue) = create_test_device();
        let buffer_system = TripleBufferSystem::new(&device, 100);
        
        // Initial state
        assert_eq!(buffer_system.write_index(), 0);
        assert_eq!(buffer_system.read_index(), 1);
        assert_eq!(buffer_system.standby_index(), 2);
        
        // After first rotation: write=1, read=2, standby=0
        buffer_system.rotate();
        assert_eq!(buffer_system.write_index(), 1);
        assert_eq!(buffer_system.read_index(), 2);
        assert_eq!(buffer_system.standby_index(), 0);
        
        // After second rotation: write=2, read=0, standby=1
        buffer_system.rotate();
        assert_eq!(buffer_system.write_index(), 2);
        assert_eq!(buffer_system.read_index(), 0);
        assert_eq!(buffer_system.standby_index(), 1);
        
        // After third rotation: back to initial state
        buffer_system.rotate();
        assert_eq!(buffer_system.write_index(), 0);
        assert_eq!(buffer_system.read_index(), 1);
        assert_eq!(buffer_system.standby_index(), 2);
    }

    #[test]
    fn test_push_instance() {
        let (device, _queue) = create_test_device();
        let mut buffer_system = TripleBufferSystem::new(&device, 10);
        
        let instance = CellInstanceData::from_components(
            [1.0, 2.0, 3.0],
            0.5,
            [1.0, 0.0, 0.0, 1.0],
            [1.0, 0.0, 0.0, 0.0],
        );
        
        assert!(buffer_system.push_instance(instance));
        assert_eq!(buffer_system.write_buffer().len(), 1);
    }

    #[test]
    fn test_push_instance_capacity_limit() {
        let (device, _queue) = create_test_device();
        let mut buffer_system = TripleBufferSystem::new(&device, 3);
        
        let instance = CellInstanceData::default();
        
        assert!(buffer_system.push_instance(instance));
        assert!(buffer_system.push_instance(instance));
        assert!(buffer_system.push_instance(instance));
        // Fourth push should fail - at capacity
        assert!(!buffer_system.push_instance(instance));
    }

    #[test]
    fn test_set_instances() {
        let (device, _queue) = create_test_device();
        let mut buffer_system = TripleBufferSystem::new(&device, 100);
        
        let instances: Vec<CellInstanceData> = (0..5)
            .map(|i| CellInstanceData::from_components(
                [i as f32, 0.0, 0.0],
                1.0,
                [1.0, 1.0, 1.0, 1.0],
                [1.0, 0.0, 0.0, 0.0],
            ))
            .collect();
        
        buffer_system.set_instances(&instances);
        
        assert_eq!(buffer_system.write_buffer().len(), 5);
        assert_eq!(buffer_system.write_buffer()[0].position_and_radius[0], 0.0);
        assert_eq!(buffer_system.write_buffer()[4].position_and_radius[0], 4.0);
    }

    #[test]
    fn test_clear() {
        let (device, _queue) = create_test_device();
        let mut buffer_system = TripleBufferSystem::new(&device, 100);
        
        // Add some instances
        for _ in 0..10 {
            buffer_system.push_instance(CellInstanceData::default());
        }
        
        buffer_system.clear();
        
        assert_eq!(buffer_system.write_buffer().len(), 0);
        assert_eq!(buffer_system.read_instance_count(), 0);
    }

    #[test]
    fn test_rotation_preserves_data_independence() {
        let (device, _queue) = create_test_device();
        let mut buffer_system = TripleBufferSystem::new(&device, 100);
        
        // Write to buffer 0
        let instance1 = CellInstanceData::from_components(
            [1.0, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0, 1.0], [1.0, 0.0, 0.0, 0.0],
        );
        buffer_system.push_instance(instance1);
        
        // Rotate - now writing to buffer 1
        buffer_system.rotate();
        
        // Write different data to buffer 1
        let instance2 = CellInstanceData::from_components(
            [2.0, 0.0, 0.0], 2.0, [0.0, 1.0, 0.0, 1.0], [1.0, 0.0, 0.0, 0.0],
        );
        buffer_system.push_instance(instance2);
        
        // The write buffer should only have the new instance
        assert_eq!(buffer_system.write_buffer().len(), 1);
        assert_eq!(buffer_system.write_buffer()[0].position_and_radius[0], 2.0);
    }
}
