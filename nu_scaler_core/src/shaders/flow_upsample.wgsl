// nu_scaler_core/src/shaders/flow_upsample.wgsl
// Bilinearly upsamples a flow field.

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
@group(0) @binding(2) var bilinear_sampler: sampler;

@group(0) @binding(3) var dst_flow_tex: texture_storage_2d<rg32float, write>; // Renamed from dst_flow for clarity

// [[stage(compute), workgroup_size(16,16)]] // WGSL uses @compute @workgroup_size(16,16,1)
@compute @workgroup_size(1, 1, 1)
fn main() {
  // Empty
} 