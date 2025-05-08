// src/shaders/warp_blend.wgsl

struct InterpolationUniforms {
  size: vec2<u32>,
  _pad0: vec2<u32>,
  time_t: f32,
  _pad1: vec3<f32>,
};

@group(0) @binding(0) var<uniform> u: InterpolationUniforms;
@group(0) @binding(1) var frame_a_tex: texture_2d<f32>;
@group(0) @binding(2) var frame_b_tex: texture_2d<f32>;
@group(0) @binding(3) var flow_tex: texture_2d<f32>; // Expects Rg32Float
@group(0) @binding(4) var out_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(5) var image_sampler: sampler;
@group(0) @binding(6) var flow_sampler: sampler;

@compute @workgroup_size(8, 32, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= u.size.x || global_id.y >= u.size.y) {
        return;
    }
    let output_coord_i32 = vec2<i32>(i32(global_id.x), i32(global_id.y));
    // Normalized UV coordinate for the center of the current output pixel
    let current_pixel_center_uv = (vec2<f32>(global_id.xy) + 0.5) / vec2<f32>(u.size);

    // Sample the flow vector at the current pixel location
    // Flow is stored as pixel delta (how many pixels frame A moved to become frame B)
    let flow_pixel_delta = textureSampleLevel(flow_tex, flow_sampler, current_pixel_center_uv, 0.0).xy;

    // Calculate sample coordinates in frame A and frame B
    // Frame A sample coord: current_pos - t * flow
    // Frame B sample coord: current_pos + (1-t) * flow
    // Coordinates need to be normalized (0.0 - 1.0)
    let uv0 = ((vec2<f32>(global_id.xy) + 0.5) - u.time_t * flow_pixel_delta) / vec2<f32>(u.size);
    let uv1 = ((vec2<f32>(global_id.xy) + 0.5) + (1.0 - u.time_t) * flow_pixel_delta) / vec2<f32>(u.size);

    // Sample textures
    let c0 = textureSampleLevel(frame_a_tex, image_sampler, uv0, 0.0);
    let c1 = textureSampleLevel(frame_b_tex, image_sampler, uv1, 0.0);

    // Blend colors
    let blended_color = mix(c0, c1, u.time_t);

    // Write result
    textureStore(out_tex, output_coord_i32, blended_color);
} 