// Boundary Crossing Post-Processing Effect
// Full-screen wavy distortion that fades out

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct BoundaryCrossingSettings {
    intensity: f32,
    time: f32,
    direction: f32,
    aspect_ratio: f32,
}

@group(0) @binding(2) var<uniform> settings: BoundaryCrossingSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    
    if (settings.intensity < 0.001) {
        return textureSample(screen_texture, texture_sampler, uv);
    }
    
    // === FULL SCREEN WAVY RIPPLES ===
    // Multiple wave frequencies for organic water-like distortion
    let wave1 = sin(uv.y * 25.0 - settings.time * 10.0);
    let wave2 = sin(uv.y * 15.0 - settings.time * 6.0 + 2.0);
    let wave3 = sin(uv.y * 40.0 - settings.time * 14.0 + 1.0);
    
    // Combine waves
    let combined_wave = wave1 * 0.5 + wave2 * 0.35 + wave3 * 0.15;
    
    // Horizontal displacement - intensity controls strength
    let displacement = combined_wave * settings.intensity * 0.05;
    
    // Vertical wobble for extra waviness
    let v_wave = sin(uv.x * 18.0 + uv.y * 12.0 - settings.time * 8.0);
    let v_wobble = v_wave * settings.intensity * 0.012;
    
    // Apply distortion
    uv.x = uv.x + displacement;
    uv.y = uv.y + v_wobble;
    
    // Clamp UV
    uv = clamp(uv, vec2<f32>(0.002), vec2<f32>(0.998));
    
    // === CHROMATIC ABERRATION ===
    let chroma = settings.intensity * 0.01;
    let r = textureSample(screen_texture, texture_sampler, uv + vec2<f32>(chroma, 0.0)).r;
    let g = textureSample(screen_texture, texture_sampler, uv).g;
    let b = textureSample(screen_texture, texture_sampler, uv - vec2<f32>(chroma, 0.0)).b;
    var color = vec3<f32>(r, g, b);
    
    // === COLOR TINT ===
    let tint_strength = settings.intensity * 0.1;
    if (settings.direction > 0.0) {
        color = mix(color, color * vec3<f32>(0.92, 0.96, 1.08), tint_strength);
    } else {
        color = mix(color, color * vec3<f32>(1.06, 1.0, 0.94), tint_strength);
    }
    
    return vec4<f32>(color, 1.0);
}
