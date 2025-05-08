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

// Same weights as horizontal pass
const KERNEL_RADIUS: i32 = 2;
const KERNEL_WEIGHTS = array<f32, 5>(1.0, 4.0, 6.0, 4.0, 1.0);
const KERNEL_SUM = 16.0;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= params.out_size.x || global_id.y >= params.out_size.y) {
        return;
    }

    let output_coord = vec2<i32>(i32(global_id.x), i32(global_id.y));

    var color_sum: vec4<f32> = vec4<f32>(0.0);

    // Apply vertical kernel
    for (var i: i32 = -KERNEL_RADIUS; i <= KERNEL_RADIUS; i = i + 1) {
        // Clamp coordinates to stay within source texture bounds
        // Reading input size from params.in_size
        let sample_coord_y = clamp(output_coord.y + i, 0, i32(params.in_size.y) - 1);
        let sample_coord = vec2<i32>(output_coord.x, sample_coord_y);

        // Fetch color from source texture
        let neighbor_color = textureLoad(src_tex, vec2<u32>(u32(sample_coord.x), u32(sample_coord.y)), 0);

        // Get weight from kernel
        let weight = KERNEL_WEIGHTS[i + KERNEL_RADIUS];
        color_sum += neighbor_color * weight;
    }

    // Normalize and write to destination texture
    textureStore(dst_tex, output_coord, color_sum / KERNEL_SUM);
} 