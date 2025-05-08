// nu_scaler_core/src/shaders/downsample.wgsl
// Simple 2x2 average downsample

struct Params {
    in_size: vec2<u32>, // Size of input texture being read (blurred texture at current level)
    out_size: vec2<u32>, // Size of output texture being written (half size)
    _pad: vec2<u32>,   // Padding
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var src_tex: texture_2d<f32>; // Input texture (e.g., Rgba32Float)
@group(0) @binding(2) var dst_tex: texture_storage_2d<rgba32float, write>; // Output texture

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Check if we are out of bounds for the DESTINATION texture
    if (global_id.x >= params.out_size.x || global_id.y >= params.out_size.y) {
        return;
    }

    // Calculate the top-left coordinate in the SOURCE texture for the 2x2 block
    let src_x = global_id.x * 2u;
    let src_y = global_id.y * 2u;

    // Load the four pixels from the source texture
    // Need to ensure src_x+1 and src_y+1 are within in_size bounds, but textureLoad might handle out-of-bounds implicitly (e.g., return 0) or clamp.
    // Assuming valid coordinates for simplicity here, boundary checks could be added.
    let c00 = textureLoad(src_tex, vec2<u32>(src_x,     src_y),     0);
    let c10 = textureLoad(src_tex, vec2<u32>(src_x + 1u, src_y),     0);
    let c01 = textureLoad(src_tex, vec2<u32>(src_x,     src_y + 1u), 0);
    let c11 = textureLoad(src_tex, vec2<u32>(src_x + 1u, src_y + 1u), 0);

    // Calculate the average color
    let average_color = (c00 + c10 + c01 + c11) * 0.25;

    // Write the average color to the destination texture
    textureStore(dst_tex, vec2<i32>(i32(global_id.x), i32(global_id.y)), average_color);
} 