#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_view_bindings::view,
}

struct VolumetricFogMaterial {
    fog_params: vec4<f32>,  // density, sphere_radius, absorption, padding
    fog_color: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> material_ext: VolumetricFogMaterial;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // Generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    
    // Alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
    
    var out: FragmentOutput;
    // Apply PBR lighting (this gives us proper lighting and shadows)
    out.color = apply_pbr_lighting(pbr_input);
    
    // Now apply volumetric fog effect on top of the lit color
    let world_pos = in.world_position.xyz;
    let camera_pos = view.world_position;
    
    // Calculate ray direction from camera to fragment
    let ray_dir = normalize(world_pos - camera_pos);
    let ray_origin = camera_pos;
    
    // Calculate distance from camera to fragment
    let fragment_dist = length(world_pos - camera_pos);
    
    // Calculate distance from center of sphere (origin)
    let dist_from_center = length(world_pos);
    
    // Extract fog parameters
    let density = material_ext.fog_params.x;
    let sphere_radius = material_ext.fog_params.y;
    let absorption = material_ext.fog_params.z;
    
    // Ray-sphere intersection for volumetric fog
    // We're inside the sphere, so calculate the path length through the fog
    let sphere_center = vec3<f32>(0.0, 0.0, 0.0);
    let oc = ray_origin - sphere_center;
    let a = dot(ray_dir, ray_dir);
    let b = 2.0 * dot(oc, ray_dir);
    let c = dot(oc, oc) - sphere_radius * sphere_radius;
    let discriminant = b * b - 4.0 * a * c;
    
    var fog_depth = fragment_dist;
    
    if (discriminant >= 0.0) {
        let sqrt_disc = sqrt(discriminant);
        let t1 = (-b - sqrt_disc) / (2.0 * a);
        let t2 = (-b + sqrt_disc) / (2.0 * a);
        
        // We're inside the sphere, so use the distance to the far intersection
        if (t2 > 0.0) {
            fog_depth = min(fragment_dist, t2);
        }
    }
    
    // Exponential fog falloff based on distance traveled through fog
    let fog_amount = 1.0 - exp(-density * fog_depth);
    
    // Add depth-based density variation (denser toward center)
    let center_factor = 1.0 - (dist_from_center / sphere_radius);
    let density_variation = 1.0 + center_factor * 0.5;
    
    // Calculate final fog with absorption
    let final_fog_amount = fog_amount * density_variation * absorption;
    
    // Mix the lit color with fog color based on fog amount
    let fog_color = material_ext.fog_color;
    out.color = vec4<f32>(
        mix(out.color.rgb, fog_color.rgb, final_fog_amount * 0.3), // Blend fog with lit surface
        out.color.a
    );
    
    // Apply in-shader post processing (fog, alpha-premultiply, tonemapping, debanding)
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    
    return out;
}
