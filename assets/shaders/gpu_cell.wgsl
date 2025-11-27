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

// Per-instance data from storage buffer
struct InstanceInput {
    @location(2) position_and_radius: vec4<f32>,  // xyz = position, w = radius
    @location(3) color: vec4<f32>,                 // rgba
    @location(4) orientation: vec4<f32>,           // quaternion (w, x, y, z)
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
    
    // Rotate vertex position and normal by instance orientation
    let rotated_pos = quat_rotate(instance.orientation, vertex.position);
    let rotated_normal = quat_rotate(instance.orientation, vertex.normal);
    
    // Scale by radius and translate to instance position
    let world_pos = instance.position_and_radius.xyz + rotated_pos * instance.position_and_radius.w;
    
    out.world_position = world_pos;
    out.world_normal = rotated_normal;
    out.color = instance.color.rgb;
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
