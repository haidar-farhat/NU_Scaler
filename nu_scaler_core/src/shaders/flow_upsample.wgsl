// nu_scaler_core/src/shaders/flow_upsample.wgsl
// Bilinearly upsamples a flow field.

struct UpsampleUniforms {
  src_width: u32;
  src_height: u32;
  dst_width: u32;
  dst_height: u32;
}

@group(0) @binding(0) var<uniform> u: UpsampleUniforms;

// src_flow: user spec texture_2d<vec2<f32>>. This implies Rg32Float.
// For textureSampleLevel, the texture type is usually texture_2d<f32> and a sampler is used.
// If texture_2d<vec2<f32>> is a special type that allows sampling as vec2, it may work.
// Assuming standard WGSL, this should be texture_2d<f32> and a sampler is also needed.
@group(0) @binding(1) var src_flow_tex: texture_2d<f32>; // Changed from texture_2d<vec2<f32>>
@group(0) @binding(2) var bilinear_sampler: sampler;

@group(0) @binding(3) var dst_flow_tex: texture_storage_2d<rg32float, write>; // Renamed from dst_flow for clarity

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= u.dst_width || id.y >= u.dst_height) { 
    return; 
  }
  
  let dst_size_f32 = vec2<f32>(f32(u.dst_width), f32(u.dst_height));

  // Calculate UVs based on destination dimensions
  let normalized_uv = (vec2<f32>(id.xy) + 0.5) / dst_size_f32;

  // Sampling from src_flow_tex using these normalized UVs
  let sampled_flow_vec4 = textureSampleLevel(src_flow_tex, bilinear_sampler, normalized_uv, 0.0);
  let flow_vec2 = sampled_flow_vec4.xy;

  textureStore(dst_flow_tex, vec2<i32>(id.xy), vec4<f32>(flow_vec2.x, flow_vec2.y, 0.0, 1.0));
} 