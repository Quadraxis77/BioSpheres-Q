// Spatial Grid Clear Shader
// Clears grid cell counts before inserting cells

struct Uniforms {
    total_grid_cells: u32,
    _padding: vec3<u32>,
}

@group(0) @binding(0) var<storage, read_write> grid_counts: array<u32>;
@group(0) @binding(1) var<uniform> uniforms: Uniforms;

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;

    if (index >= uniforms.total_grid_cells) {
        return;
    }

    grid_counts[index] = 0u;
}
