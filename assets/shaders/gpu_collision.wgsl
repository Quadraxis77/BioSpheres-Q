// GPU Collision Detection and Force Computation Shader
// Uses brute-force O(n²) collision detection for reliability

// Cell data structure - uses vec4 for proper alignment
// Rust layout: [f32; 3] position, f32 radius, [f32; 3] velocity, f32 mass, f32 stiffness, [f32; 3] padding
// Total: 48 bytes (12 floats)
struct CellData {
    // position.xyz = position, position.w = radius
    position_radius: vec4<f32>,
    // velocity.xyz = velocity, velocity.w = mass  
    velocity_mass: vec4<f32>,
    // stiffness_pad.x = stiffness, stiffness_pad.yzw = padding
    stiffness_pad: vec4<f32>,
}

// Output force/torque for each cell
// Rust layout: [f32; 3] force, f32 pad, [f32; 3] torque, f32 pad
// Total: 32 bytes (8 floats)
struct ForceOutput {
    force_pad: vec4<f32>,
    torque_pad: vec4<f32>,
}

// Collision pair for debugging/verification
struct CollisionPair {
    index_a: u32,
    index_b: u32,
    overlap: f32,
    normal: vec3<f32>,
}

// Physics configuration
struct PhysicsParams {
    cell_count: u32,
    grid_size: u32,
    world_size: f32,
    sphere_radius: f32,
    default_stiffness: f32,
    damping: f32,
    friction_coefficient: f32,
    max_force: f32,
}

// Spatial grid cell (stores start index and count in sorted cell list)
struct GridCell {
    start: u32,
    count: u32,
}

@group(0) @binding(0) var<storage, read> cells: array<CellData>;
@group(0) @binding(1) var<storage, read_write> forces: array<ForceOutput>;
@group(0) @binding(2) var<uniform> params: PhysicsParams;
@group(0) @binding(3) var<storage, read> grid_cells: array<GridCell>;
@group(0) @binding(4) var<storage, read> cell_indices: array<u32>;  // Sorted cell indices by grid cell
@group(0) @binding(5) var<storage, read> cell_grid_indices: array<u32>;  // Grid cell index for each cell

// Convert world position to grid coordinates
fn world_to_grid(position: vec3<f32>) -> vec3<i32> {
    let cell_size = params.world_size / f32(params.grid_size);
    let offset_pos = position + vec3<f32>(params.world_size / 2.0);
    let grid_pos = offset_pos / cell_size;
    let max_coord = i32(params.grid_size) - 1;
    return vec3<i32>(
        clamp(i32(grid_pos.x), 0, max_coord),
        clamp(i32(grid_pos.y), 0, max_coord),
        clamp(i32(grid_pos.z), 0, max_coord)
    );
}

// Convert grid coordinates to linear index
fn grid_to_index(grid_coord: vec3<i32>) -> u32 {
    let g = params.grid_size;
    return u32(grid_coord.x) + u32(grid_coord.y) * g + u32(grid_coord.z) * g * g;
}

// Check if grid coordinate is valid
fn is_valid_grid_coord(coord: vec3<i32>) -> bool {
    let max_coord = i32(params.grid_size) - 1;
    return coord.x >= 0 && coord.x <= max_coord &&
           coord.y >= 0 && coord.y <= max_coord &&
           coord.z >= 0 && coord.z <= max_coord;
}

// Compute collision force on cell_idx FROM other_idx
// Returns force that should be applied to cell_idx
fn compute_force_on_cell(cell_idx: u32, other_idx: u32) -> vec3<f32> {
    let cell_a = cells[cell_idx];
    let cell_b = cells[other_idx];
    
    // Extract data from packed format
    let pos_a = cell_a.position_radius.xyz;
    let radius_a = cell_a.position_radius.w;
    let vel_a = cell_a.velocity_mass.xyz;
    let stiffness_a = cell_a.stiffness_pad.x;
    
    let pos_b = cell_b.position_radius.xyz;
    let radius_b = cell_b.position_radius.w;
    let vel_b = cell_b.velocity_mass.xyz;
    let stiffness_b = cell_b.stiffness_pad.x;
    
    let delta = pos_b - pos_a;
    let distance = length(delta);
    let combined_radius = radius_a + radius_b;
    
    // No collision
    if distance >= combined_radius || distance < 0.0001 {
        return vec3<f32>(0.0);
    }
    
    let overlap = combined_radius - distance;
    let normal = delta / distance; // Points from cell_idx toward other_idx
    
    // Compute combined stiffness using harmonic mean
    var combined_stiffness: f32;
    if stiffness_a > 0.0 && stiffness_b > 0.0 {
        combined_stiffness = (stiffness_a * stiffness_b) / (stiffness_a + stiffness_b);
    } else if stiffness_a > 0.0 {
        combined_stiffness = stiffness_a;
    } else if stiffness_b > 0.0 {
        combined_stiffness = stiffness_b;
    } else {
        combined_stiffness = params.default_stiffness;
    }
    
    // Spring force magnitude (positive = repulsion)
    let spring_force = combined_stiffness * overlap;
    
    // Damping force
    let relative_velocity = vel_b - vel_a;
    let relative_vel_normal = dot(relative_velocity, normal);
    let damping_force = -params.damping * relative_vel_normal;
    
    // Total force magnitude (clamped)
    let total_force = clamp(spring_force + damping_force, -params.max_force, params.max_force);
    
    // Force on cell_idx points AWAY from other_idx (repulsion)
    return -total_force * normal;
}

// Main collision detection and force computation kernel
// Each thread handles one cell and checks collisions with ALL other cells (brute force O(n²))
// This is simpler and more reliable than spatial grid for small cell counts
@compute @workgroup_size(64)
fn collision_detect(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cell_idx = global_id.x;
    
    if cell_idx >= params.cell_count {
        return;
    }
    
    var total_force = vec3<f32>(0.0);
    var total_torque = vec3<f32>(0.0);
    
    // Brute force: check against all other cells
    for (var other_idx: u32 = 0u; other_idx < params.cell_count; other_idx++) {
        // Skip self-collision
        if other_idx == cell_idx {
            continue;
        }
        
        // Compute force on this cell from the other cell
        total_force += compute_force_on_cell(cell_idx, other_idx);
    }
    
    // Write output (packed format)
    forces[cell_idx].force_pad = vec4<f32>(total_force, 0.0);
    forces[cell_idx].torque_pad = vec4<f32>(total_torque, 0.0);
}

// Note: Spatial grid building is done on CPU for simplicity
// GPU-based grid building would require atomic types and prefix sum compute passes
