// GPU Cell Division Shader
// Checks cells for division and marks them for splitting
// Actual division execution happens in a separate pass

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

struct GPUMode {
    color: vec4<f32>,
    orientation_a: vec4<f32>,
    orientation_b: vec4<f32>,
    split_direction: vec4<f32>,
    child_modes: vec2<i32>,
    split_interval: f32,
    genome_offset: i32,
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
    parent_make_adhesion: i32,
    child_a_keep_adhesion: i32,
    child_b_keep_adhesion: i32,
    max_adhesions: i32,
    flagellocyte_thrust_force: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
}

struct DivisionRequest {
    cell_index: u32,
    should_divide: u32,
    _padding: vec2<u32>,
}

struct Uniforms {
    max_cells: u32,
    _padding: vec3<u32>,
}

@group(0) @binding(0) var<storage, read> cells: array<ComputeCell>;
@group(0) @binding(1) var<storage, read_write> division_requests: array<DivisionRequest>;
@group(0) @binding(2) var<storage, read> cell_counts: vec4<u32>;
@group(0) @binding(3) var<storage, read> modes: array<GPUMode>;
@group(0) @binding(4) var<uniform> uniforms: Uniforms;

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let total_cell_count = cell_counts[0];

    if (index >= total_cell_count) {
        return;
    }

    let cell = cells[index];
    let mode_index = cell.mode_index;

    // Initialize division request
    division_requests[index].cell_index = index;
    division_requests[index].should_divide = 0u;

    // Check if mode is valid
    if (mode_index < 0 || mode_index >= i32(arrayLength(&modes))) {
        return;
    }

    let mode = modes[mode_index];
    let division_threshold = mode.split_interval;

    // Check if cell should divide (skip if split_interval > 25, which means never-split)
    if (division_threshold <= 25.0 && cell.age >= division_threshold && total_cell_count < uniforms.max_cells) {
        division_requests[index].should_divide = 1u;
    }
}
