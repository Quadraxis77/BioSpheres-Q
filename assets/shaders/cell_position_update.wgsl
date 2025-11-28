// GPU Cell Physics - Position and Orientation Update (Verlet Integration)
// Port from cell_position_update.comp (GLSL) to WGSL

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
    dragged_cell_index: i32,
    _padding: vec2<f32>,
}

// Storage buffers
@group(0) @binding(0) var<storage, read> input_cells: array<ComputeCell>;
@group(0) @binding(1) var<storage, read_write> output_cells: array<ComputeCell>;
@group(0) @binding(2) var<storage, read> cell_counts: vec4<u32>;
@group(0) @binding(3) var<uniform> uniforms: Uniforms;

// Quaternion multiplication
fn quat_multiply(q1: vec4<f32>, q2: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        q1.w * q2.x + q1.x * q2.w + q1.y * q2.z - q1.z * q2.y,
        q1.w * q2.y - q1.x * q2.z + q1.y * q2.w + q1.z * q2.x,
        q1.w * q2.z + q1.x * q2.y - q1.y * q2.x + q1.z * q2.w,
        q1.w * q2.w - q1.x * q2.x - q1.y * q2.y - q1.z * q2.z
    );
}

// Rotate quaternion by angular velocity pseudovector
fn quat_rotate(q: vec4<f32>, v: vec3<f32>) -> vec4<f32> {
    let mag = length(v);

    if (mag > 0.001) {
        // Normalize angular velocity to get rotation axis
        let rotation_axis = v / mag;

        // Create quaternion for this rotation
        let half_angle = mag * 0.5;
        let rotation_quat = vec4<f32>(
            rotation_axis * sin(half_angle),
            cos(half_angle)
        );

        // Apply rotation to current orientation
        let result = quat_multiply(rotation_quat, q);
        return normalize(result);
    }

    return q;
}

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let total_cell_count = cell_counts[0];

    if (index >= total_cell_count) {
        return;
    }

    var cell = input_cells[index];
    // 0.5x aging in position pass (0.5 here + 0.5 in velocity pass = 1.0 total)
    cell.age += uniforms.delta_time * 0.5;

    // If this is the dragged cell, only update age and skip physics
    if (i32(index) == uniforms.dragged_cell_index) {
        output_cells[index] = cell;
        return;
    }

    // Linear Verlet position
    var pos = cell.position_and_mass.xyz;
    let vel = cell.velocity.xyz;
    let acc = cell.acceleration.xyz;

    pos += vel * uniforms.delta_time + 0.5 * acc * (uniforms.delta_time * uniforms.delta_time);

    cell.position_and_mass = vec4<f32>(pos, cell.position_and_mass.w);
    cell.prev_acceleration = vec4<f32>(acc, 0.0);
    cell.acceleration = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    // Angular Verlet orientation
    var q = cell.orientation;
    let w = cell.angular_velocity.xyz;
    let alpha = cell.angular_acceleration.xyz;

    // Predict angular velocity
    let delta_orientation = w * uniforms.delta_time;

    q = quat_rotate(q, delta_orientation);

    cell.orientation = q;
    cell.prev_angular_acceleration = vec4<f32>(alpha, 0.0);
    cell.angular_acceleration = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    output_cells[index] = cell;
}
