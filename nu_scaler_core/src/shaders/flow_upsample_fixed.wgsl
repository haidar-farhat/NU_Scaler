// nu_scaler_core/src/shaders/flow_upsample_fixed.wgsl
// Bilinearly upsamples a flow field. (Corrected version)

struct UpsampleUniforms {
  src_size: vec2<u32>;
  dst_size: vec2<u32>;
} // No semicolon here

@group(0) @binding(0) var<uniform> u: UpsampleUniforms;

// Need sampler defined for textureSampleLevel
@group(0) @binding(1) var<uniform> src_flow_tex: texture_2d<f32>;
@group(0) @binding(2) var<uniform> bilinear_sampler: sampler; // Sampler definition
@group(0) @binding(3) var<storage> dst_flow_tex: texture_storage_2d<rg32float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  // Use dst_size for boundary check
  if (id.x >= u.dst_size.x || id.y >= u.dst_size.y) {
    return;
  }

  // Use dst_size for UV calculation (maps normalized coords 1:1)
  let dst_size_f32 = vec2<f32>(u.dst_size);
  let normalized_uv = (vec2<f32>(id.xy) + 0.5) / dst_size_f32;

  // Correct call to textureSampleLevel with the sampler
  let sampled_flow_vec4 = textureSampleLevel(src_flow_tex, bilinear_sampler, normalized_uv, 0.0);
  let flow_vec2 = sampled_flow_vec4.xy;

  textureStore(dst_flow_tex, vec2<i32>(id.xy), vec4<f32>(flow_vec2.x, flow_vec2.y, 0.0, 1.0));
} 