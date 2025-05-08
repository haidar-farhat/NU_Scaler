// nu_scaler_core/src/shaders/horn_schunck.wgsl
// Horn-Schunck Optical Flow - Coarse Level - Single Iteration Step

struct Params {
    size: vec2<u32>,   // Size of this pyramid level texture
    lambda: f32,       // Smoothness weight (α² in some formulations)
    _padding: f32,     // Padding
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var i1_tex: texture_2d<f32>; // Frame A, coarsest level (e.g., Rgba32Float)
@group(0) @binding(2) var i2_tex: texture_2d<f32>; // Frame B, coarsest level (e.g., Rgba32Float)
@group(0) @binding(3) var flow_in_tex: texture_2d<f32>; // Previous iteration flow (Rg32Float)
@group(0) @binding(4) var flow_out_tex: texture_storage_2d<rg32float, write>; // Updated flow output

// Helper to convert RGBA to Luminance (simple average for now)
fn luminance(color: vec4<f32>) -> f32 {
    // return dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114)); // Standard Rec.709
    return (color.r + color.g + color.b) * 0.33333;
}

// Helper to get flow vector from neighbors (simple average)
// Note: textureLoad requires u32 coords
fn get_flow_average(coord: vec2<i32>) -> vec2<f32> {
    var sum = vec2<f32>(0.0);
    var count = 0.0;
    for (var dy: i32 = -1; dy <= 1; dy = dy + 1) {
        for (var dx: i32 = -1; dx <= 1; dx = dx + 1) {
            // Skip the center pixel (dx=0, dy=0) for average calculation if desired
            // if (dx == 0 && dy == 0) { continue; }
            
            let neighbor_coord = clamp(coord + vec2<i32>(dx, dy), 
                                     vec2<i32>(0), 
                                     vec2<i32>(i32(params.size.x) - 1, i32(params.size.y) - 1));
            sum += textureLoad(flow_in_tex, vec2<u32>(neighbor_coord), 0).xy;
            count += 1.0;
        }
    }
    // If count is zero (shouldn't happen with 3x3 unless image is 1x1), return zero or current flow
    if (count > 0.0) {
        return sum / count;
    } else {
        // Fallback for 1x1 texture? Return center pixel flow.
        return textureLoad(flow_in_tex, vec2<u32>(coord), 0).xy;
    }
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= params.size.x || global_id.y >= params.size.y) {
        return;
    }

    let coord = vec2<i32>(i32(global_id.x), i32(global_id.y));
    let coord_u32 = vec2<u32>(coord);

    // Calculate spatial derivatives (Ix, Iy) - using central difference on I1 for simplicity
    // Texture coordinates for +/- 1 pixels, clamped
    let coord_xp1 = vec2<u32>(min(coord_u32.x + 1u, params.size.x - 1u), coord_u32.y);
    let coord_xm1 = vec2<u32>(max(coord_u32.x, 1u) - 1u, coord_u32.y); // Avoid negative u32
    let coord_yp1 = vec2<u32>(coord_u32.x, min(coord_u32.y + 1u, params.size.y - 1u));
    let coord_ym1 = vec2<u32>(coord_u32.x, max(coord_u32.y, 1u) - 1u);

    let lum_xp1 = luminance(textureLoad(i1_tex, coord_xp1, 0));
    let lum_xm1 = luminance(textureLoad(i1_tex, coord_xm1, 0));
    let lum_yp1 = luminance(textureLoad(i1_tex, coord_yp1, 0));
    let lum_ym1 = luminance(textureLoad(i1_tex, coord_ym1, 0));

    let ix = (lum_xp1 - lum_xm1) * 0.5; // Central difference gradient x
    let iy = (lum_yp1 - lum_ym1) * 0.5; // Central difference gradient y

    // Calculate temporal derivative (It)
    let lum1 = luminance(textureLoad(i1_tex, coord_u32, 0));
    let lum2 = luminance(textureLoad(i2_tex, coord_u32, 0));
    let it = lum2 - lum1;

    // Get average flow from neighbors (previous iteration)
    let uv_avg = get_flow_average(coord);

    // Horn-Schunck iteration (Jacobi form)
    let common_term = (ix * uv_avg.x + iy * uv_avg.y + it) / (params.lambda + ix * ix + iy * iy);
    
    // Check for division by zero or near-zero (denominator can be small if lambda is small and gradients are zero)
    // Add a small epsilon to prevent NaN/Inf, or handle degenerate cases.
    // let denominator = params.lambda + ix*ix + iy*iy + 1e-6;
    // let common_term = (ix * uv_avg.x + iy * uv_avg.y + it) / denominator;

    let uv_new = uv_avg - common_term * vec2<f32>(ix, iy);

    // Write updated flow
    textureStore(flow_out_tex, coord, vec4<f32>(uv_new, 0.0, 1.0));
} 