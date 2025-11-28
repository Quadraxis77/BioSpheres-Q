// Spatial Grid Insert Shader
// Inserts cells into spatial grid for efficient collision detection

struct ComputeCell {
    position_and_mass: vec4<f32>,
    velocity: vec4<f32>,
    acceleration: vec4<f32>,
    prev_acceleration: vec4<f32>,
    orientation: vec4<f32>,
    genome_orientation: vec4<f32>,
    angular_velocity: vec4<f32>,
    angular_acceleration: vec4<f32>,
    prev_angular_acceleration: vec4<f32>,
    signalling_substances: vec4<f32>,
    mode_index: i32,
    age: f32,
    toxins: f32,
    nitrates: f32,
    adhesion_indices: array<i32, 20>,
    _padding: array<u32, 4>,
}

struct Uniforms {
    grid_resolution: i32,
    grid_cell_size: f32,
    world_size: f32,
    max_cells_per_grid: i32,
}

@group(0) @binding(0) var<storage, read> cells: array<ComputeCell>;
@group(0) @binding(1) var<storage, read_write> grid_cells: array<u32>;
@group(0) @binding(2) var<storage, read_write> grid_offsets: array<atomic<u32>>;
@group(0) @binding(3) var<storage, read> cell_counts: vec4<u32>;
@group(0) @binding(4) var<uniform> uniforms: Uniforms;

// Convert world position to grid coordinates
fn world_to_grid(world_pos: vec3<f32>) -> vec3<i32> {
    let half_world = uniforms.world_size * 0.5;
    let clamped_pos = clamp(world_pos, vec3<f32>(-half_world), vec3<f32>(half_world));
    let normalized_pos = (clamped_pos + half_world) / uniforms.world_size;
    let grid_pos = vec3<i32>(normalized_pos * f32(uniforms.grid_resolution));
    let max_coord = uniforms.grid_resolution - 1;
    return clamp(grid_pos, vec3<i32>(0), vec3<i32>(max_coord));
}

// Convert 3D grid coordinates to 1D index
fn grid_to_index(grid_pos: vec3<i32>) -> u32 {
    let res = uniforms.grid_resolution;
    return u32(grid_pos.x + grid_pos.y * res + grid_pos.z * res * res);
}

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cell_index = global_id.x;
    let allocated_cell_count = cell_counts[0];

    if (cell_index >= allocated_cell_count) {
        return;
    }

    // Get cell position
    let cell_pos = cells[cell_index].position_and_mass.xyz;

    // Convert to grid coordinates
    let grid_pos = world_to_grid(cell_pos);
    let grid_index = grid_to_index(grid_pos);

    // Atomically claim a slot in this grid cell
    let slot_index = atomicAdd(&grid_offsets[grid_index], 1u);

    // Calculate the actual index in the grid buffer
    let grid_buffer_index = grid_index * u32(uniforms.max_cells_per_grid) + slot_index;

    // Make sure we don't exceed the maximum cells per grid cell
    if (slot_index < u32(uniforms.max_cells_per_grid)) {
        grid_cells[grid_buffer_index] = cell_index;
    }
}
