use std::mem::size_of;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VelocityUniforms {
    pub delta_time: f32,           // 4 bytes
    pub damping: f32,              // 4 bytes
    pub dragged_cell_index: i32,   // 4 bytes
    pub sphere_radius: f32,        // 4 bytes
    pub sphere_center: [f32; 3],   // 12 bytes
    pub enable_velocity_barrier: u32,    // 4 bytes
    pub barrier_damping: f32,      // 4 bytes
    pub barrier_push_distance: f32, // 4 bytes
}

fn main() {
    println!("VelocityUniforms size: {}", size_of::<VelocityUniforms>());
}
