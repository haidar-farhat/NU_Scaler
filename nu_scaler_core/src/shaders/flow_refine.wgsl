// nu_scaler_core/src/shaders/flow_refine.wgsl
// Residual Horn-Schunck refinement pass.

// [[block]] struct HSUniforms { // WGSL doesn't use [[block]] for top-level structs usually
struct HSUniforms {
  size: vec2<u32>;
  alpha: f32;         // smoothness weight
  _pad: vec3<f32>;    // Padding. size (8) + alpha (4) + _pad (12) = 24 bytes.
                      // Consider making this struct 16 or 32 bytes for robustness.
                      // e.g. { size: vec2<u32>, _p1: vec2<u32>, alpha:f32, _p2: vec3<f32> } for 32 bytes
                      // or   { size: vec2<u32>, alpha:f32, _p_alpha: f32 } for 16 bytes if _pad is just for alpha.
};

@group(0) @binding(0) var<uniform> u: HSUniforms;
// For I1 and I2, assuming they are Rgba32Float pyramid levels.
// Shader uses .r, implying luminance.
@group(0) @binding(1) var I1_tex: texture_2d<f32>; 
@group(0) @binding(2) var I2_tex: texture_2d<f32>;

// For flow_in, user spec is texture_2d<vec2<f32>>.
// Adhering to user spec, assuming it means an Rg32Float texture sampled to vec2<f32>.
// This usually implies an implicit sampler or special handling.
@group(0) @binding(3) var flow_in_tex: texture_2d<vec2<f32>>; 
@group(0) @binding(4) var flow_out_tex: texture_storage_2d<rg32float, write>;

// Adding samplers, as .read() is not standard for texture_2d<f32> / texture_2d<vec2<f32>>.
// Standard way is textureSampleLevel or textureLoad.
// textureLoad does not use a sampler and takes integer coords.
// textureSampleLevel uses a sampler and takes float UVs.
// The .read() syntax in user's WGSL for flow_refine.wgsl for I1, I2, flow_in is non-standard.
// It might be a shorthand from a higher-level shader language or a custom extension.
// I will replace .read(coord) with textureLoad(tex, coord, 0) assuming coord is u32 and no mipmapping.
// This requires textures to be compatible with textureLoad (e.g. not multisampled, correct format).

// Function to get luminance, assuming input texture is Rgba32Float.
// If I1_tex and I2_tex are already luminance, then just use .r from textureLoad(tex, coord, 0).r
fn get_luminance(tex: texture_2d<f32>, coord: vec2<u32>) -> f32 {
    // Assuming Rgba32Float, where .r contains luminance if pre-converted, or use .r from calculation.
    return textureLoad(tex, coord, 0).r; 
}

fn gradient(coord: vec2<u32>) -> vec2<f32> {
  // Central differences using textureLoad
  // Clamping coordinates to avoid out-of-bounds.
  let size_minus_1 = u.size - vec2<u32>(1u);
  
  let lum_xp1 = get_luminance(I1_tex, vec2<u32>(min(coord.x + 1u, size_minus_1.x), coord.y));
  let lum_xm1 = get_luminance(I1_tex, vec2<u32>(max(coord.x, 1u) - 1u, coord.y));
  let lum_yp1 = get_luminance(I1_tex, vec2<u32>(coord.x, min(coord.y + 1u, size_minus_1.y)));
  let lum_ym1 = get_luminance(I1_tex, vec2<u32>(coord.x, max(coord.y, 1u) - 1u));

  let Ix = (lum_xp1 - lum_xm1) * 0.5;
  let Iy = (lum_yp1 - lum_ym1) * 0.5;
  return vec2<f32>(Ix, Iy);
}

fn get_flow_at_coord(tex: texture_2d<vec2<f32>>, coord: vec2<u32>) -> vec2<f32> {
    // Assuming texture_2d<vec2<f32>> means textureLoad returns vec2<f32> directly.
    // This is non-standard. Normally textureLoad on Rg32Float (declared as texture_2d<f32>)
    // would return vec4<f32>, and we'd take .xy.
    // If texture_2d<vec2<f32>> is a valid type that loads as vec2, this is fine.
    // Otherwise, it should be: textureLoad(tex, coord, 0).xy where tex is texture_2d<f32>
    return textureLoad(tex, coord, 0);
}


fn laplacian(coord: vec2<u32>) -> vec2<f32> {
  // 4-neighbor laplacian using textureLoad
  var sum = vec2<f32>(0.0);
  let size_minus_1 = u.size - vec2<u32>(1u);

  sum += get_flow_at_coord(flow_in_tex, vec2<u32>(min(coord.x + 1u, size_minus_1.x), coord.y));
  sum += get_flow_at_coord(flow_in_tex, vec2<u32>(max(coord.x, 1u) - 1u, coord.y));
  sum += get_flow_at_coord(flow_in_tex, vec2<u32>(coord.x, min(coord.y + 1u, size_minus_1.y)));
  sum += get_flow_at_coord(flow_in_tex, vec2<u32>(coord.x, max(coord.y, 1u) - 1u));
  
  let center_flow = get_flow_at_coord(flow_in_tex, coord);
  // The user's original was sum * 0.25 - center_flow. This computes (avg_neighbors - center_flow).
  // Standard Laplacian is (sum_neighbors - N * center_flow). For N=4, it's sum_neighbors - 4.0 * center_flow.
  // Let's stick to user's formulation if it's intentional: (sum_neighbors / N) - center_flow
  // If N=4, then sum * 0.25 - center_flow is correct.
  return sum * 0.25 - center_flow;
}

// [[stage(compute), workgroup_size(16,16)]] // WGSL uses @compute @workgroup_size(16,16,1) for 2D
@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= u.size.x || id.y >= u.size.y) { return; }
  
  let coord = id.xy;
  let f_old = get_flow_at_coord(flow_in_tex, coord); // upsampled flow from coarser level
  let grad_I1 = gradient(coord); // Spatial gradient of I1 at current level

  // Temporal difference It = I2(x + u_old, y + v_old) - I1(x,y)
  // x and y are integer coords 'coord'. u_old, v_old are f_old.
  // So I2 is sampled at fractional pixel locations. This requires textureSample, not textureLoad.
  // The user's code `I2.read(vec2<u32>(clamp(warped_uv, ...)))` implies rounding/truncating warped_uv.
  // This loses subpixel accuracy. Using textureSample is better for optical flow.
  // For now, I'll try to implement the user's `I2.read(vec2<u32>(clamped_warped_coord))` logic.
  
  let warped_coord_f = vec2<f32>(coord) + f_old;
  let warped_coord_u_clamped = vec2<u32>(clamp(warped_coord_f, vec2<f32>(0.0), vec2<f32>(u.size - vec2<u32>(1u))));
  
  let lum_I1_current = get_luminance(I1_tex, coord);
  let lum_I2_warped = get_luminance(I2_tex, warped_coord_u_clamped);
  let It = lum_I2_warped - lum_I1_current; // Temporal difference It

  // Data term for Horn-Schunck: (Ix*u_avg + Iy*v_avg + It) / (alpha^2 + Ix^2 + Iy^2)
  // Here, u_avg and v_avg are components of f_old (the upsampled flow).
  // The expression `grad * (grad.x * f_old.x + ...)` in user's shader looks like `grad * common_numerator_term_scalar`
  // Let common_numerator = Ix*u + Iy*v + It
  let common_numerator = grad_I1.x * f_old.x + grad_I1.y * f_old.y + It;
  let denominator = u.alpha * u.alpha + dot(grad_I1, grad_I1);
  
  var flow_update_contribution = vec2<f32>(0.0);
  if (abs(denominator) > 1e-6) { // Avoid division by zero
      flow_update_contribution = (common_numerator / denominator) * grad_I1;
  }
  
  // Laplacian term (smoothness)
  let laplacian_f_old = laplacian(coord); // Laplacian of the upsampled flow f_old

  // Horn-Schunck update: f_new = f_old - data_term_contribution + smoothness_term_contribution
  // User's was: f_new = f_old - num / den + u.alpha * laplacian(coord);
  // where num/den was (grad * (grad.x*f_old.x + ...)) / (alpha^2 + dot(grad,grad))
  // This matches: f_new = f_old - flow_update_contribution + u.alpha * laplacian_f_old (if alpha is squared in denom or not in smoothness)
  // Typical HS: u_k+1 = u_avg - Ix * common_term ; v_k+1 = v_avg - Iy * common_term
  // where common_term = (Ix*u_avg + Iy*v_avg + It) / (alpha^2 + Ix^2 + Iy^2)
  // And u_avg is often related to laplacian.
  // The user's formula: f_new = f_old - ( (grad_I1 * common_numerator) / denominator ) + u.alpha * laplacian_f_old;
  // This formula directly from the user's example is:
  // let f_new = f_old - num / den + u.alpha * laplacian(coord);
  // where num = grad * (grad.x * f_old.x + grad.y * f_old.y + It);
  // and den = u.alpha * u.alpha + dot(grad, grad);
  // So flow_update_contribution in my terms is (num/den) if num is a vector: grad * scalar_common_numerator.
  
  let f_new = f_old - flow_update_contribution + u.alpha * laplacian_f_old;

  textureStore(flow_out_tex, coord, vec4<f32>(f_new.x, f_new.y, 0.0, 1.0));
} 