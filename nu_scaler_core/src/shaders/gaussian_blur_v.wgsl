// nu_scaler_core/src/shaders/gaussian_blur_v.wgsl
// Separable Gaussian Blur - Vertical Pass

struct Params {
    in_size: vec2<u32>, // Size of input texture being read (output of horizontal pass)
    out_size: vec2<u32>, // Size of output texture being written (final blurred output for this level)
    radius: u32,        // Kernel radius (e.g., 2 for 5x5)
    _pad0: u32,
    _pad1: vec2<u32>,   // Padding to meet WGSL uniform buffer rules
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var src_tex: texture_2d<f32>; // Input texture (e.g., Rgba32Float - result of H blur)
@group(0) @binding(2) var dst_tex: texture_storage_2d<rgba32float, write>; // Output texture
@group(0) @binding(3) var image_sampler: sampler; // Use a sampler for potential boundary handling

// Use hardcoded weights for 5x5 kernel (radius 2)
const W0: f32 = 1.0 / 16.0;
const W1: f32 = 4.0 / 16.0;
const W2: f32 = 6.0 / 16.0;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= params.out_size.x || global_id.y >= params.out_size.y) {
        return;
    }

    let output_coord = vec2<i32>(i32(global_id.x), i32(global_id.y));
    let in_coord_x = u32(output_coord.x);

    // Apply vertical kernel (Unrolled loop)
    let coord_m2 = clamp(output_coord.y - 2, 0, i32(params.in_size.y) - 1);
    let coord_m1 = clamp(output_coord.y - 1, 0, i32(params.in_size.y) - 1);
    let coord_00 = clamp(output_coord.y,     0, i32(params.in_size.y) - 1);
    let coord_p1 = clamp(output_coord.y + 1, 0, i32(params.in_size.y) - 1);
    let coord_p2 = clamp(output_coord.y + 2, 0, i32(params.in_size.y) - 1);

    let color_m2 = textureLoad(src_tex, vec2<u32>(in_coord_x, u32(coord_m2)), 0);
    let color_m1 = textureLoad(src_tex, vec2<u32>(in_coord_x, u32(coord_m1)), 0);
    let color_00 = textureLoad(src_tex, vec2<u32>(in_coord_x, u32(coord_00)), 0);
    let color_p1 = textureLoad(src_tex, vec2<u32>(in_coord_x, u32(coord_p1)), 0);
    let color_p2 = textureLoad(src_tex, vec2<u32>(in_coord_x, u32(coord_p2)), 0);

    let color_sum = color_m2 * W0 +
                    color_m1 * W1 +
                    color_00 * W2 +
                    color_p1 * W1 +
                    color_p2 * W0;

    // Write to destination texture (already normalized by weights)
    textureStore(dst_tex, output_coord, color_sum);
} 