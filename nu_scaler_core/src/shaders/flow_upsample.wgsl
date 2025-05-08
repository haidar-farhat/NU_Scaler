// nu_scaler_core/src/shaders/flow_upsample.wgsl
// Bilinearly upsamples a flow field.

// [[block]] struct UpsampleUniforms { // WGSL doesn't use [[block]] for top-level structs
struct UpsampleUniforms {
  src_size: vec2<u32>;
  dst_size: vec2<u32>;
  // No padding needed as vec2<u32> is 8 bytes, total 16 bytes.
}

@group(0) @binding(0) var<uniform> u: UpsampleUniforms;

// src_flow: user spec texture_2d<vec2<f32>>. This implies Rg32Float.
// For textureSampleLevel, the texture type is usually texture_2d<f32> and a sampler is used.
// If texture_2d<vec2<f32>> is a special type that allows sampling as vec2, it may work.
// Assuming standard WGSL, this should be texture_2d<f32> and a sampler is also needed.
@group(0) @binding(1) var src_flow_tex: texture_2d<f32>; // Changed from texture_2d<vec2<f32>>
@group(0) @binding(2) varbilinear_sampler: sampler;

@group(0) @binding(3) var dst_flow_tex: texture_storage_2d<rg32float, write>; // Renamed from dst_flow for clarity

// [[stage(compute), workgroup_size(16,16)]] // WGSL uses @compute @workgroup_size(16,16,1)
@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= u.dst_size.x || id.y >= u.dst_size.y) { 
    return; 
  }
  
  // Calculate the center of the destination pixel in normalized UV coordinates for sampling.
  let dst_pixel_center_uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(u.dst_size);
  
  // The user's original mapping was:
  // let uv = vec2<f32>(id.xy) + 0.5;
  // let src_uv = uv * vec2<f32>(u.src_size) / vec2<f32>(u.dst_size);
  // let f = textureSampleLevel(src_flow, vec2<f32>(src_uv) / vec2<f32>(u.src_size), 0.0);
  // The src_uv was already scaled to src pixel space (approx), then dividing again by src_size made it normalized.
  // A more direct way to get normalized source UVs for sampling from destination UVs:
  // src_normalized_uv = dst_normalized_uv (which is dst_pixel_center_uv here)
  // The key is that textureSampleLevel expects normalized coordinates (0.0-1.0).

  // The mapping from a destination pixel coord to the corresponding source texel coord (for sampling)
  // should consider that we want to sample at the location in the source texture that corresponds
  // to the center of the current destination pixel.
  // Let dst_coord = id.xy
  // Normalized position in dst: (dst_coord + 0.5) / dst_size
  // This normalized position is the same normalized position in src we want to sample from.
  let src_sample_uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(u.dst_size);

  // Sample bilinearly using the provided sampler and normalized coordinates.
  // textureSampleLevel returns vec4<f32> for texture_2d<f32> (like Rg32Float).
  let sampled_flow_vec4 = textureSampleLevel(src_flow_tex, bilinear_sampler, src_sample_uv, 0.0);
  let flow_vec2 = sampled_flow_vec4.xy; // Assuming flow is in .xy

  textureStore(dst_flow_tex, vec2<i32>(id.xy), vec4<f32>(flow_vec2.x, flow_vec2.y, 0.0, 1.0));
} 