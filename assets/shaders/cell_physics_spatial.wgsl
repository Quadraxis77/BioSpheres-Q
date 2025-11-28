// GPU Cell Physics - Collision Detection and Force Calculation
// Port from cell_physics_spatial.comp (GLSL) to WGSL
// Workgroup size optimized for 100K cells

// Cell data structure matching C++ ComputeCell
struct ComputeCell {
    position_and_mass: vec4<f32>,      // xyz = position, w = mass
    velocity: vec4<f32>,                // xyz = velocity, w = padding
    acceleration: vec4<f32>,            // xyz = acceleration, w = padding
    prev_acceleration: vec4<f32>,       // xyz = previous acceleration, w = padding
    orientation: vec4<f32>,             // quaternion (w, x, y, z)
    genome_orientation: vec4<f32>,      // quaternion for genome-derived orientation
    angular_velocity: vec4<f32>,        // xyz = angular velocity, w = padding
    angular_acceleration: vec4<f32>,    // xyz = angular acceleration, w = padding
    prev_angular_acceleration: vec4<f32>, // xyz = previous angular accel, w = padding
    signalling_substances: vec4<f32>,   // 4 substances
    mode_index: i32,                    // absolute index of cell's mode
    age: f32,                           // also used for split timer
    toxins: f32,
    nitrates: f32,
    adhesion_indices: array<i32, 20>,   // indices of adhesion connections
    // Padding to maintain alignment
    _padding: array<u32, 4>,
}

// GPU Mode structure
struct GPUMode {
    color: vec4<f32>,
    orientation_a: vec4<f32>,           // quaternion for child A
    orientation_b: vec4<f32>,           // quaternion for child B
    split_direction: vec4<f32>,         // direction cells split apart
    child_modes: vec2<i32>,             // mode indices for children
    split_interval: f32,                // time between divisions
    genome_offset: i32,
    // Adhesion settings (48 bytes)
    adhesion_can_break: i32,
    adhesion_break_force: f32,
    adhesion_rest_length: f32,
    adhesion_linear_stiffness: f32,
    adhesion_linear_damping: f32,
    adhesion_orientation_stiffness: f32,
    adhesion_orientation_damping: f32,
    adhesion_max_angular_deviation: f32,
    adhesion_twist_stiffness: f32,
    adhesion_twist_damping: f32,
    adhesion_enable_twist: i32,
    adhesion_padding: i32,
    // Parent/child settings
    parent_make_adhesion: i32,
    child_a_keep_adhesion: i32,
    child_b_keep_adhesion: i32,
    max_adhesions: i32,
    flagellocyte_thrust_force: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
}

// Uniforms
struct Uniforms {
    dragged_cell_index: i32,
    acceleration_damping: f32,
    grid_resolution: i32,
    grid_cell_size: f32,
    world_size: f32,
    max_cells_per_grid: i32,
    enable_thrust_force: i32,
    _padding: i32,
}

// Storage buffers
@group(0) @binding(0) var<storage, read> input_cells: array<ComputeCell>;
@group(0) @binding(1) var<storage, read> grid_cells: array<u32>;
@group(0) @binding(2) var<storage, read> grid_counts: array<u32>;
@group(0) @binding(3) var<storage, read_write> output_cells: array<ComputeCell>;
@group(0) @binding(4) var<storage, read> cell_counts: vec4<u32>; // total, live, adhesion, free_adhesion
@group(0) @binding(5) var<storage, read> modes: array<GPUMode>;
@group(0) @binding(6) var<uniform> uniforms: Uniforms;

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

// Check if grid coordinates are valid
fn is_valid_grid_pos(grid_pos: vec3<i32>) -> bool {
    let res = uniforms.grid_resolution;
    return grid_pos.x >= 0 && grid_pos.x < res &&
           grid_pos.y >= 0 && grid_pos.y < res &&
           grid_pos.z >= 0 && grid_pos.z < res;
}

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let total_cell_count = cell_counts[0];

    // Check bounds
    if (index >= total_cell_count) {
        return;
    }

    // Skip physics for dragged cell - it will be positioned directly
    if (i32(index) == uniforms.dragged_cell_index) {
        output_cells[index] = input_cells[index];
        output_cells[index].velocity = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        output_cells[index].acceleration = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return;
    }

    // Copy input cell data to output and reset acceleration
    output_cells[index] = input_cells[index];
    output_cells[index].acceleration = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    // Calculate forces from nearby cells using spatial partitioning
    var total_force = vec3<f32>(0.0);
    let my_pos = input_cells[index].position_and_mass.xyz;
    let my_mass = input_cells[index].position_and_mass.w;
    let my_radius = pow(my_mass, 1.0 / 3.0);

    // Get the grid cell this cell belongs to
    let my_grid_pos = world_to_grid(my_pos);

    // Search neighboring grid cells
    let search_radius = 1i;

    for (var dx = -search_radius; dx <= search_radius; dx++) {
        for (var dy = -search_radius; dy <= search_radius; dy++) {
            for (var dz = -search_radius; dz <= search_radius; dz++) {
                let neighbor_grid_pos = my_grid_pos + vec3<i32>(dx, dy, dz);

                // Skip if neighbor is outside grid bounds
                if (!is_valid_grid_pos(neighbor_grid_pos)) {
                    continue;
                }

                let neighbor_grid_index = grid_to_index(neighbor_grid_pos);
                let local_cell_count = grid_counts[neighbor_grid_index];

                // Early exit if no cells in this grid
                if (local_cell_count == 0u) {
                    continue;
                }

                // Limit search to reasonable number of cells
                let max_cells_to_check = min(local_cell_count, u32(uniforms.max_cells_per_grid));

                // Check all cells in this neighboring grid cell
                for (var i = 0u; i < max_cells_to_check; i++) {
                    let grid_buffer_index = neighbor_grid_index * u32(uniforms.max_cells_per_grid) + i;
                    let other_index = grid_cells[grid_buffer_index];

                    // Skip self and invalid indices
                    if (other_index == index || other_index >= total_cell_count) {
                        continue;
                    }

                    let other_pos = input_cells[other_index].position_and_mass.xyz;
                    let delta = my_pos - other_pos;
                    let distance = length(delta);

                    // Early distance check before radius calculation
                    if (distance > 4.0) {
                        continue;
                    }

                    let other_radius = pow(input_cells[other_index].position_and_mass.w, 1.0 / 3.0);
                    let min_distance = my_radius + other_radius;

                    if (distance < min_distance && distance > 0.001) {
                        // Collision detected - apply repulsion force
                        let direction = normalize(delta);
                        let overlap = min_distance - distance;
                        let hardness = 10.0;
                        total_force += direction * overlap * hardness;
                    }
                }
            }
        }
    }

    // Calculate acceleration (F = ma, so a = F/m)
    var acceleration = total_force / my_mass;

    // Add flagellocyte thrust force (continuous forward movement)
    if (uniforms.enable_thrust_force != 0) {
        let mode_index = input_cells[index].mode_index;
        if (mode_index >= 0 && mode_index < i32(arrayLength(&modes))) {
            let thrust_force = modes[mode_index].flagellocyte_thrust_force;
            if (thrust_force > 0.0) {
                // Apply thrust in the forward direction (convert quaternion to forward vector)
                let orientation = input_cells[index].orientation;
                let forward = vec3<f32>(
                    2.0 * (orientation.y * orientation.w + orientation.x * orientation.z),
                    2.0 * (orientation.z * orientation.w - orientation.x * orientation.y),
                    1.0 - 2.0 * (orientation.x * orientation.x + orientation.y * orientation.y)
                );
                acceleration += forward * thrust_force / my_mass;
            }
        }
    }

    let acc_magnitude = length(acceleration);

    // Multi-tier acceleration damping to prevent drift
    if (acc_magnitude < 0.001) {
        acceleration = vec3<f32>(0.0);
    } else if (acc_magnitude < 0.01) {
        acceleration *= 0.1;
    } else if (acc_magnitude < 0.05) {
        acceleration *= uniforms.acceleration_damping;
    }

    // Store acceleration in output
    output_cells[index].acceleration = vec4<f32>(acceleration, 0.0);
}
