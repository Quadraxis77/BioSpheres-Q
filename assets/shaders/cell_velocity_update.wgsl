// GPU Cell Physics - Velocity Update (Verlet Integration)
// Port from cell_velocity_update.comp (GLSL) to WGSL

// Cell data structure matching C++ ComputeCell
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

// Uniforms
struct Uniforms {
    delta_time: f32,
    damping: f32,
    dragged_cell_index: i32,
    sphere_radius: f32,
    sphere_center: vec3<f32>,
    enable_velocity_barrier: u32,
    barrier_damping: f32,
    barrier_push_distance: f32,
}

// Storage buffers
@group(0) @binding(0) var<storage, read> input_cells: array<ComputeCell>;
@group(0) @binding(1) var<storage, read_write> output_cells: array<ComputeCell>;
@group(0) @binding(2) var<storage, read> cell_counts: vec4<u32>;
@group(0) @binding(3) var<uniform> uniforms: Uniforms;

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let total_cell_count = cell_counts[0];

    if (index >= total_cell_count) {
        return;
    }

    var cell = input_cells[index];
    cell.age += uniforms.delta_time * 0.5;

    // If this is the dragged cell, only update age and skip physics
    if (i32(index) == uniforms.dragged_cell_index) {
        output_cells[index] = cell;
        return;
    }

    // Linear Verlet velocity
    var vel = cell.velocity.xyz;
    let acc_old = cell.prev_acceleration.xyz;
    let acc_new = cell.acceleration.xyz;

    vel += 0.5 * (acc_old + acc_new) * uniforms.delta_time;
    vel *= pow(uniforms.damping, uniforms.delta_time * 100.0);

    // Velocity Barrier Logic
    if (uniforms.enable_velocity_barrier != 0u) {
        let cell_pos = cell.position_and_mass.xyz;
        let distance_from_center = length(cell_pos - uniforms.sphere_center);

        // Check if cell is outside or very close to the sphere boundary
        if (distance_from_center > uniforms.sphere_radius - uniforms.barrier_push_distance) {
            // Calculate direction from sphere center to cell
            let outward_direction = normalize(cell_pos - uniforms.sphere_center);

            // Check if velocity is pointing outward
            let outward_velocity = dot(vel, outward_direction);

            if (outward_velocity > 0.0) {
                // Reverse the outward component of velocity
                vel -= outward_direction * outward_velocity * 2.0;

                // Apply additional damping
                vel *= uniforms.barrier_damping;

                // Add small inward push to prevent getting stuck
                vel -= outward_direction * 0.5;
            }

            // If cell is already outside, push it back inside
            if (distance_from_center > uniforms.sphere_radius) {
                let push_direction = -outward_direction;
                let push_strength = (distance_from_center - uniforms.sphere_radius) * 2.0;
                vel += push_direction * push_strength;
            }
        }
    }

    cell.velocity = vec4<f32>(vel, 0.0);

    // Angular Verlet velocity
    var w = cell.angular_velocity.xyz;
    let alpha_old = cell.prev_angular_acceleration.xyz;
    let alpha_new = cell.angular_acceleration.xyz;

    w += 0.5 * (alpha_old + alpha_new) * uniforms.delta_time;
    w *= pow(uniforms.damping, uniforms.delta_time * 100.0);

    cell.angular_velocity = vec4<f32>(w, 0.0);

    output_cells[index] = cell;
}
