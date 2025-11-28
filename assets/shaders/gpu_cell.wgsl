// GPU Cell Shader - Vertex and Fragment shaders for instanced icosphere rendering
// Feature: webgpu-rendering
// Validates: Requirements 4.3, 4.4, 5.1, 5.3

// Camera uniform buffer containing view and projection matrices
struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

// Vertex input from icosphere mesh
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

// Per-instance data from ComputeCell buffer (272 bytes)
// Must match the layout in gpu_compute.rs
struct InstanceInput {
    // Offset 0: position_and_mass
    @location(2) position_and_mass: vec4<f32>,     // xyz = position, w = mass
    // Offset 16: velocity (skip)
    @location(3) velocity: vec4<f32>,
    // Offset 32: acceleration (skip)
    @location(4) acceleration: vec4<f32>,
    // Offset 48: prev_acceleration (skip)
    @location(5) prev_acceleration: vec4<f32>,
    // Offset 64: orientation
    @location(6) orientation: vec4<f32>,           // quaternion (w, x, y, z)
    // Offset 80: genome_orientation (skip for rendering)
    @location(7) genome_orientation: vec4<f32>,
    // Offset 96: angular_velocity (skip)
    @location(8) angular_velocity: vec4<f32>,
    // Offset 112: angular_acceleration (skip)
    @location(9) angular_acceleration: vec4<f32>,
    // Offset 128: prev_angular_acceleration (skip)
    @location(10) prev_angular_acceleration: vec4<f32>,
    // For now, remaining fields don't need to be declared as we only need position, mass, and orientation
};

// Output from vertex shader to fragment shader
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) color: vec3<f32>,
};

// Bind camera uniform at group 0, binding 0
@group(0) @binding(0) var<uniform> camera: CameraUniform;

// Rotate a vector by a quaternion
// q is in (w, x, y, z) format
fn quat_rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let qv = vec3<f32>(q.y, q.z, q.w);  // x, y, z components
    let qw = q.x;                        // w component
    let uv = cross(qv, v);
    let uuv = cross(qv, uv);
    return v + ((uv * qw) + uuv) * 2.0;
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Calculate radius from mass: radius = mass^(1/3)
    let mass = instance.position_and_mass.w;
    let radius = pow(mass, 1.0 / 3.0);

    // Rotate vertex position and normal by instance orientation
    let rotated_pos = quat_rotate(instance.orientation, vertex.position);
    let rotated_normal = quat_rotate(instance.orientation, vertex.normal);

    // Scale by radius and translate to instance position
    let world_pos = instance.position_and_mass.xyz + rotated_pos * radius;

    out.world_position = world_pos;
    out.world_normal = rotated_normal;
    // Use default color for now (will be replaced with mode color lookup later)
    out.color = vec3<f32>(0.8, 0.3, 0.5);
    out.clip_position = camera.projection * camera.view * vec4<f32>(world_pos, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple directional lighting
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let normal = normalize(in.world_normal);
    
    // Ambient and diffuse lighting
    let ambient = 0.1;
    let diffuse = max(dot(normal, light_dir), 0.0);
    
    let lighting = ambient + diffuse * 0.9;
    let final_color = in.color * lighting;
    
    return vec4<f32>(final_color, 1.0);
}
