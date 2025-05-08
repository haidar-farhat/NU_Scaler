// nu_scaler_core/src/wgpu_interpolator.rs
// GPU-based frame interpolation logic

use std::sync::Arc;
use anyhow::Result;
use wgpu::util::DeviceExt; // For create_buffer_init
use wgpu::{
    Device, Queue, ShaderModule, ComputePipeline, BindGroupLayout, Sampler,
    TextureView, TextureFormat, ShaderStages, BindingType, StorageTextureAccess,
    TextureViewDimension, SamplerBindingType, BufferBindingType, PipelineLayoutDescriptor,
    ComputePipelineDescriptor, ShaderModuleDescriptor, ShaderSource, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BufferUsages, BindGroupDescriptor, BindGroupEntry, BindingResource,
    CommandEncoderDescriptor, ComputePassDescriptor, Texture, TextureUsages, Extent3d,
    ImageCopyTexture, ImageDataLayout, Origin3d, Buffer,
    include_wgsl, TextureDescriptor, TextureDimension,
    SamplerDescriptor, AddressMode, FilterMode, TextureViewDescriptor,
};

// Uniform structure for the warp/blend shader - MATCHING ORIGINAL SPEC (48 Bytes)
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InterpolationUniforms {
    size: [u32; 2],       // offset 0, size 8
    _pad0: [u32; 2],      // offset 8, size 8 -> next at 16
    time_t: f32,          // offset 16, size 4 -> next at 20
    // WGSL's _pad1: vec3<f32> will start at offset 32 due to align(16).
    // So, Rust struct needs 12 bytes of padding here.
    _rust_pad_to_align_vec3: [f32; 3], // offset 20, size 12 -> next at 32
    _pad1_wgsl_equivalent: [f32; 3],      // offset 32, size 12. Current total 44.
    // Final padding to make Rust struct 48 bytes, matching WGSL struct total size
    _final_struct_padding: [f32; 1], // offset 44, size 4 -> Total 48 bytes.
}

impl InterpolationUniforms {
    fn new(width: u32, height: u32, time_t: f32) -> Self {
        Self {
            size: [width, height],
            _pad0: [0; 2],
            time_t,
            _rust_pad_to_align_vec3: [0.0; 3],
            _pad1_wgsl_equivalent: [0.0; 3], // Renamed for clarity
            _final_struct_padding: [0.0; 1], // Added final padding
        }
    }
}

// Uniform struct for Blur/Downsample shaders
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PyramidPassParams {
    in_size: [u32; 2],
    out_size: [u32; 2],
    radius: u32, // Only used by blur, ignored by downsample
    _pad0: u32,
    _pad1: [u32; 2], // Padding
}

pub struct WgpuFrameInterpolator {
    device: Arc<Device>,
    warp_blend_pipeline: ComputePipeline,
    warp_blend_bind_group_layout: BindGroupLayout,
    blur_h_pipeline: ComputePipeline,
    blur_v_pipeline: ComputePipeline,
    downsample_pipeline: ComputePipeline,
    pyramid_pass_bind_group_layout: BindGroupLayout,
    shared_sampler: Sampler,
    blur_temp_texture: Option<Texture>,
    blur_temp_texture_view: Option<TextureView>,
    pyramid_a_textures: Vec<Texture>,
    pyramid_a_views: Vec<TextureView>,
    pyramid_b_textures: Vec<Texture>,
    pyramid_b_views: Vec<TextureView>,
    downsample_a_textures: Vec<Texture>,
    downsample_a_views: Vec<TextureView>,
    downsample_b_textures: Vec<Texture>,
    downsample_b_views: Vec<TextureView>,
}

impl WgpuFrameInterpolator {
    pub fn new(device: Arc<Device>) -> Result<Self> {
        // WGSL Shader source as per original user spec (Phase 1.1)
        let warp_blend_shader_source = r#"
            struct InterpolationUniforms {
              size: vec2<u32>,
              _pad0: vec2<u32>,
              time_t: f32,
              _pad1: vec3<f32>,
            };

            @group(0) @binding(0) var<uniform> u: InterpolationUniforms;
            @group(0) @binding(1) var frame_a_tex: texture_2d<f32>;
            @group(0) @binding(2) var frame_b_tex: texture_2d<f32>;
            @group(0) @binding(3) var flow_tex: texture_2d<f32>;
            @group(0) @binding(4) var out_tex: texture_storage_2d<rgba8unorm, write>;
            @group(0) @binding(5) var image_sampler: sampler;
            @group(0) @binding(6) var flow_sampler: sampler;

            @compute @workgroup_size(16, 16, 1)
            fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
                if (global_id.x >= u.size.x || global_id.y >= u.size.y) {
                    return;
                }
                let output_coord_i32 = vec2<i32>(i32(global_id.x), i32(global_id.y));
                let current_pixel_center_uv = (vec2<f32>(global_id.xy) + 0.5) / vec2<f32>(u.size);
                let flow_pixel_delta = textureSampleLevel(flow_tex, flow_sampler, current_pixel_center_uv, 0.0).xy;
                let uv0 = ((vec2<f32>(global_id.xy) + 0.5) - u.time_t * flow_pixel_delta) / vec2<f32>(u.size);
                let uv1 = ((vec2<f32>(global_id.xy) + 0.5) + (1.0 - u.time_t) * flow_pixel_delta) / vec2<f32>(u.size);
                let c0 = textureSampleLevel(frame_a_tex, image_sampler, uv0, 0.0);
                let c1 = textureSampleLevel(frame_b_tex, image_sampler, uv1, 0.0);
                let blended_color = mix(c0, c1, u.time_t);
                textureStore(out_tex, output_coord_i32, blended_color);
            }
        "#;

        let warp_blend_shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Warp/Blend Shader Module (Phase 1)"),
            source: ShaderSource::Wgsl(warp_blend_shader_source.into()),
        });

        let warp_blend_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Warp/Blend BGL (Phase 1)"),
            entries: &[
                // u: InterpolationUniforms
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                // frame_a_tex (Revert to Float { filterable: true })
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                // frame_b_tex (Revert to Float { filterable: true })
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                // flow_tex (Revert to Float { filterable: true })
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                // out_tex (storage texture)
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture { access: StorageTextureAccess::WriteOnly, format: TextureFormat::Rgba8Unorm, view_dimension: TextureViewDimension::D2 },
                    count: None,
                },
                // image_sampler (Filtering)
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // flow_sampler (Filtering or NonFiltering)
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let warp_blend_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Warp/Blend Pipeline Layout (Phase 1)"),
            bind_group_layouts: &[&warp_blend_bind_group_layout],
            push_constant_ranges: &[],
        });

        let warp_blend_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Warp/Blend Pipeline (Phase 1)"),
            layout: Some(&warp_blend_pipeline_layout),
            module: &warp_blend_shader_module,
            entry_point: "main",
        });

        // --- Phase 2.1 Setup: Image Pyramid --- 
        let blur_h_shader_module = device.create_shader_module(include_wgsl!("shaders/gaussian_blur_h.wgsl"));
        let blur_v_shader_module = device.create_shader_module(include_wgsl!("shaders/gaussian_blur_v.wgsl"));
        let downsample_shader_module = device.create_shader_module(include_wgsl!("shaders/downsample.wgsl"));

        // Shared Bind Group Layout for blur and downsample passes
        let pyramid_pass_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Pyramid Pass BGL (Blur/Downsample)"),
            entries: &[
                // params: PyramidPassParams (uniform buffer)
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                // src_tex: Input Texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: TextureViewDimension::D2, multisampled: false }, // Assuming Rgba32Float input
                    count: None,
                },
                // dst_tex: Output Texture (Storage)
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    // IMPORTANT: Output format must match texture being written
                    ty: BindingType::StorageTexture { access: StorageTextureAccess::WriteOnly, format: TextureFormat::Rgba32Float, view_dimension: TextureViewDimension::D2 },
                    count: None,
                },
                // image_sampler (optional, might not be needed if using textureLoad)
                 BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering), // Sampler might be useful for boundary clamp/mirror
                    count: None,
                },
            ],
        });

        let pyramid_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pyramid Pipeline Layout"),
            bind_group_layouts: &[&pyramid_pass_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create compute pipelines
        let blur_h_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Blur Horizontal Pipeline"),
            layout: Some(&pyramid_pipeline_layout),
            module: &blur_h_shader_module,
            entry_point: "main",
        });

        let blur_v_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Blur Vertical Pipeline"),
            layout: Some(&pyramid_pipeline_layout),
            module: &blur_v_shader_module,
            entry_point: "main",
        });

        let downsample_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Downsample Pipeline"),
            layout: Some(&pyramid_pipeline_layout),
            module: &downsample_shader_module,
            entry_point: "main",
        });

        // Create shared sampler
        let shared_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Pyramid Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest, // No mipmaps used here
            ..Default::default()
        });

        Ok(Self {
            device,
            warp_blend_pipeline,
            warp_blend_bind_group_layout,
            blur_h_pipeline,
            blur_v_pipeline,
            downsample_pipeline,
            pyramid_pass_bind_group_layout,
            shared_sampler,
            blur_temp_texture: None,
            blur_temp_texture_view: None,
            pyramid_a_textures: Vec::new(),
            pyramid_a_views: Vec::new(),
            pyramid_b_textures: Vec::new(),
            pyramid_b_views: Vec::new(),
            downsample_a_textures: Vec::new(),
            downsample_a_views: Vec::new(),
            downsample_b_textures: Vec::new(),
            downsample_b_views: Vec::new(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn interpolate(
        &self,
        queue: &Queue,
        frame_a_view: &TextureView,
        frame_b_view: &TextureView,
        flow_texture_view: &TextureView,
        output_texture_view: &TextureView,
        image_sampler: &Sampler, // Sampler for frame_a and frame_b
        flow_sampler: &Sampler,  // Sampler for flow_texture
        width: u32,
        height: u32,
        time_t: f32,
    ) -> Result<()> {
        let uniforms_data = InterpolationUniforms::new(width, height, time_t);
        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Interpolation Uniform Buffer (Phase 1)"),
            contents: bytemuck::bytes_of(&uniforms_data),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST, // COPY_DST only if we update it later, else just UNIFORM
        });

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Warp/Blend Bind Group (Phase 1)"),
            layout: &self.warp_blend_bind_group_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                BindGroupEntry { binding: 1, resource: BindingResource::TextureView(frame_a_view) },
                BindGroupEntry { binding: 2, resource: BindingResource::TextureView(frame_b_view) },
                BindGroupEntry { binding: 3, resource: BindingResource::TextureView(flow_texture_view) },
                BindGroupEntry { binding: 4, resource: BindingResource::TextureView(output_texture_view) },
                BindGroupEntry { binding: 5, resource: BindingResource::Sampler(image_sampler) },
                BindGroupEntry { binding: 6, resource: BindingResource::Sampler(flow_sampler) },
            ],
        });

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Interpolate Command Encoder (Phase 1)"),
        });
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Warp/Blend Compute Pass (Phase 1)"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.warp_blend_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            
            let workgroup_size_x = 16;
            let workgroup_size_y = 16;
            let dispatch_x = (width + workgroup_size_x - 1) / workgroup_size_x;
            let dispatch_y = (height + workgroup_size_y - 1) / workgroup_size_y;
            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
        }
        
        queue.submit(Some(encoder.finish()));
        Ok(())
    }

    // --- Phase 2.1: Image Pyramid Generation --- 

    // Placeholder - implementation details needed
    pub fn build_pyramid(
        &mut self, 
        queue: &Queue, 
        frame_texture: &Texture, // Full res Rgba32Float texture
        levels: u32,
        is_frame_a: bool, // true for pyramid A, false for pyramid B
    ) -> Result<()> {
        let (width, height) = (frame_texture.width(), frame_texture.height());
        let pyramid_textures = if is_frame_a { &mut self.pyramid_a_textures } else { &mut self.pyramid_b_textures };
        let pyramid_views = if is_frame_a { &mut self.pyramid_a_views } else { &mut self.pyramid_b_views };
        
        // TODO: Resize/recreate pyramid textures if needed (size change or first run)
        // TODO: Create/recreate blur_temp_texture if needed
        // TODO: Create default sampler

        pyramid_textures.clear();
        pyramid_views.clear();

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some(if is_frame_a { "Build Pyramid A Encoder" } else { "Build Pyramid B Encoder" }),
        });

        let mut current_width = width;
        let mut current_height = height;
        let mut last_view = frame_texture.create_view(&wgpu::TextureViewDescriptor::default()); // Start with full-res input

        for level in 0..levels {
            let next_width = (current_width + 1) / 2;
            let next_height = (current_height + 1) / 2;
            
            println!("Building pyramid level {}: {}x{} -> {}x{}", level, current_width, current_height, next_width, next_height);

            // TODO: Get/create textures for this level:
            //  - blur_temp (current_width x current_height)
            //  - pyramid_output (current_width x current_height) - this level's final blurred result
            //  - downsample_output (next_width x next_height) - used as input for next level

            // TODO: Create views for the above textures

            // TODO: Create uniform buffers & bind groups for each pass
            // let params_h = PyramidPassParams { ... };
            // let uniform_buffer_h = self.device.create_buffer_init(...);
            // let bind_group_h = self.device.create_bind_group(...);
            // let params_v = PyramidPassParams { ... };
            // let uniform_buffer_v = self.device.create_buffer_init(...);
            // let bind_group_v = self.device.create_bind_group(...);
            // let params_ds = PyramidPassParams { ... };
            // let uniform_buffer_ds = self.device.create_buffer_init(...);
            // let bind_group_ds = self.device.create_bind_group(...);

            // TODO: Begin compute pass
            // TODO: Dispatch blur_h_pipeline (input: last_view, output: blur_temp_view)
            // TODO: Dispatch blur_v_pipeline (input: blur_temp_view, output: pyramid_output_view)
            // TODO: Dispatch downsample_pipeline (input: pyramid_output_view, output: downsample_output_view)
            // TODO: End compute pass

            // TODO: Store the view for this level's blurred output (pyramid_output_view)
            // pyramid_views.push(pyramid_output_view);
            
            // Prepare for next level
            // last_view = downsample_output_view; // The downsampled output is the input for the next level
            current_width = next_width;
            current_height = next_height;
            
            // Break if dimensions get too small
            if current_width <= 1 || current_height <= 1 {
                break;
            }
        }
        
        // We need the final lowest-res blurred texture as well for the coarsest optical flow
        // TODO: Decide if the last `last_view` (the downsampled output of the last iteration) should be stored.
        // Or maybe the loop should run one more time for blur only on the smallest level?

        queue.submit(Some(encoder.finish()));
        Ok(())
    }
}

// Helper to create a texture with specific data (RGBA8 for test images)
pub fn create_texture_with_data(
    device: &Device,
    queue: &Queue,
    width: u32,
    height: u32,
    data: &[u8],
    label: Option<&str>,
    format: TextureFormat,
    usage: TextureUsages,
) -> Texture {
    assert_eq!(data.len() as u32, width * height * format.block_copy_size(None).unwrap());
    let texture_size = Extent3d { width, height, depth_or_array_layers: 1 };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label,
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: usage | TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        ImageCopyTexture { texture: &texture, mip_level: 0, origin: Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        data,
        ImageDataLayout { offset: 0, bytes_per_row: Some(width * format.block_copy_size(None).unwrap()), rows_per_image: Some(height) },
        texture_size,
    );
    texture
}

// Helper to create a flow texture (RG32Float for dx, dy)
// For testing, this can create zero flow or a constant flow.
pub fn create_flow_texture_with_data(
    device: &Device,
    queue: &Queue,
    width: u32,
    height: u32,
    flow_data: &[(f32, f32)], // Array of (dx, dy) pairs, one per pixel
    label: Option<&str>,
) -> Texture {
    assert_eq!(flow_data.len() as u32, width * height);
    let byte_data: Vec<u8> = flow_data.iter().flat_map(|(dx, dy)| [dx.to_ne_bytes(), dy.to_ne_bytes()].concat()).collect();
    create_texture_with_data(device, queue, width, height, &byte_data, label, TextureFormat::Rg32Float, TextureUsages::TEXTURE_BINDING)
}

// Helper to create a generic output texture (e.g. Rgba8Unorm for display, Rg32Float for flow)
pub fn create_output_texture(
    device: &Device, 
    width: u32, 
    height: u32, 
    format: TextureFormat, 
    label: Option<&str>
) -> Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label,
        size: Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

// Helper for reading texture data back to CPU via a buffer
// This is async due to buffer mapping.
// For tests, it's often called with pollster::block_on.
pub async fn read_texture_to_cpu(
    device: &Device,
    queue: &Queue,
    texture: &Texture,
    width: u32,
    height: u32,
    bytes_per_pixel: u32,
) -> Result<Vec<u8>> {
    let buffer_size = (width * height * bytes_per_pixel) as wgpu::BufferAddress;
    let buffer_desc = wgpu::BufferDescriptor {
        label: Some("Texture Readback Buffer"),
        size: buffer_size,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    };
    let readback_buffer = device.create_buffer(&buffer_desc);

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Texture Readback Encoder"),
    });
    encoder.copy_texture_to_buffer(
        ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &readback_buffer,
            layout: ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * bytes_per_pixel),
                rows_per_image: Some(height),
            },
        },
        Extent3d { width, height, depth_or_array_layers: 1 },
    );
    queue.submit(Some(encoder.finish()));

    let buffer_slice = readback_buffer.slice(..);
    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
    
    device.poll(wgpu::Maintain::Wait); // Important to wait for GPU to finish processing, including the map_async callback

    if let Some(Ok(())) = receiver.receive().await {
        let data = buffer_slice.get_mapped_range().to_vec();
        readback_buffer.unmap();
        Ok(data)
    } else {
        Err(anyhow::anyhow!("Failed to map texture readback buffer or mapping failed."))
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import items from parent module
    use wgpu::{Instance, RequestAdapterOptions, PowerPreference, SamplerDescriptor, AddressMode, FilterMode, TextureUsages};
    use pollster; // For blocking on async calls in tests

    async fn setup_wgpu() -> (Arc<Device>, Queue) {
        let instance = Instance::default();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Define required features - IMPORTANT: Request FLOAT32_FILTERABLE
        let required_features = wgpu::Features::FLOAT32_FILTERABLE;
        // Check if the adapter supports the feature (optional but good practice)
        let supported_features = adapter.features();
        if !supported_features.contains(required_features) {
            panic!("Adapter does not support FLOAT32_FILTERABLE feature, required for tests.");
            // Or skip the test: return Err("FLOAT32_FILTERABLE not supported")... but test needs to return Result
        }

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Test Device"),
                    // required_features: wgpu::Features::empty(), // OLD
                    required_features, // NEW: Request the feature
                    required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");
        (Arc::new(device), queue)
    }

    #[test]
    fn test_warp_blend_zero_flow() {
        let (device, queue) = pollster::block_on(setup_wgpu());
        let interpolator = WgpuFrameInterpolator::new(device.clone()).expect("Failed to create interpolator");

        const WIDTH: u32 = 64;
        const HEIGHT: u32 = 64;
        const TIME_T: f32 = 0.5;

        // Create frame_a (all red: 255,0,0,255 as Rgba32Float)
        let mut frame_a_data_f32: Vec<f32> = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
        for _ in 0..(WIDTH * HEIGHT) {
            frame_a_data_f32.extend_from_slice(&[1.0, 0.0, 0.0, 1.0]); // R, G, B, A as f32 (normalized)
        }
        let frame_a_data_u8: Vec<u8> = frame_a_data_f32.iter().flat_map(|&f| f.to_ne_bytes()).collect();
        let frame_a_tex = create_texture_with_data(
            &device, &queue, WIDTH, HEIGHT, &frame_a_data_u8, 
            Some("Frame A Test Texture"), TextureFormat::Rgba32Float, 
            TextureUsages::TEXTURE_BINDING
        );
        let frame_a_view = frame_a_tex.create_view(&wgpu::TextureViewDescriptor::default());

        // Create frame_b (all blue: 0,0,255,255 as Rgba32Float)
        let mut frame_b_data_f32: Vec<f32> = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
        for _ in 0..(WIDTH * HEIGHT) {
            frame_b_data_f32.extend_from_slice(&[0.0, 0.0, 1.0, 1.0]); // R, G, B, A as f32
        }
        let frame_b_data_u8: Vec<u8> = frame_b_data_f32.iter().flat_map(|&f| f.to_ne_bytes()).collect();
        let frame_b_tex = create_texture_with_data(
            &device, &queue, WIDTH, HEIGHT, &frame_b_data_u8, 
            Some("Frame B Test Texture"), TextureFormat::Rgba32Float, 
            TextureUsages::TEXTURE_BINDING
        );
        let frame_b_view = frame_b_tex.create_view(&wgpu::TextureViewDescriptor::default());

        // Create flow_tex (zero flow: (0.0, 0.0) for all pixels)
        let zero_flow_data: Vec<(f32, f32)> = vec![(0.0, 0.0); (WIDTH * HEIGHT) as usize];
        let flow_tex = create_flow_texture_with_data(
            &device, &queue, WIDTH, HEIGHT, &zero_flow_data, 
            Some("Zero Flow Test Texture")
        );
        let flow_tex_view = flow_tex.create_view(&wgpu::TextureViewDescriptor::default());

        // Create output texture (Rgba8Unorm)
        let out_tex = create_output_texture(
            &device, WIDTH, HEIGHT, TextureFormat::Rgba8Unorm, 
            Some("Output Test Texture")
        );
        let out_tex_view = out_tex.create_view(&wgpu::TextureViewDescriptor::default());

        // Create samplers
        let image_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Image Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let flow_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Flow Sampler (Nearest)"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest, // Flow data is often precise, no filtering
            min_filter: FilterMode::Nearest,
            ..Default::default()
        });

        // Call interpolate
        interpolator.interpolate(
            &queue, 
            &frame_a_view, &frame_b_view, &flow_tex_view, &out_tex_view, 
            &image_sampler, &flow_sampler, 
            WIDTH, HEIGHT, TIME_T
        ).expect("Interpolation failed");

        // Read back the output texture
        let output_bytes = pollster::block_on(
            read_texture_to_cpu(&device, &queue, &out_tex, WIDTH, HEIGHT, 4) // Rgba8Unorm is 4 bytes/pixel
        ).expect("Failed to read texture to CPU");

        // Assert pixel-wise average
        // Expected: R=0.5*1.0=0.5 -> 127 or 128; G=0; B=0.5*1.0=0.5 -> 127 or 128; A=1.0 -> 255
        // Allow some tolerance for f32 to u8 conversion and potential minor GPU differences.
        let expected_r = (1.0 * (1.0 - TIME_T) + 0.0 * TIME_T * 255.0).round() as u8;
        let expected_g = (0.0 * (1.0 - TIME_T) + 0.0 * TIME_T * 255.0).round() as u8;
        let expected_b = (0.0 * (1.0 - TIME_T) + 1.0 * TIME_T * 255.0).round() as u8;
        let expected_a = (1.0 * (1.0 - TIME_T) + 1.0 * TIME_T * 255.0).round() as u8; // Should be 255
        
        // Corrected expected values calculation for Rgba8Unorm output from normalized float inputs
        // c0_r = 1.0, c1_r = 0.0. mix(1.0, 0.0, 0.5) = 0.5.  0.5 * 255 = 127.5 -> 127 or 128
        // c0_b = 0.0, c1_b = 1.0. mix(0.0, 1.0, 0.5) = 0.5.  0.5 * 255 = 127.5 -> 127 or 128
        // For color channels, result = ((1-t)*c0 + t*c1) * 255. For alpha, result = ((1-t)*a0 + t*a1) * 255

        let expected_pixel = [
            ((1.0 - TIME_T) * 1.0 + TIME_T * 0.0).clamp(0.0, 1.0) * 255.0,
            ((1.0 - TIME_T) * 0.0 + TIME_T * 0.0).clamp(0.0, 1.0) * 255.0,
            ((1.0 - TIME_T) * 0.0 + TIME_T * 1.0).clamp(0.0, 1.0) * 255.0,
            ((1.0 - TIME_T) * 1.0 + TIME_T * 1.0).clamp(0.0, 1.0) * 255.0,
        ];

        let mut mismatches = 0;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let idx = ((y * WIDTH + x) * 4) as usize;
                let r = output_bytes[idx];
                let g = output_bytes[idx + 1];
                let b = output_bytes[idx + 2];
                let a = output_bytes[idx + 3];

                // Check with a small tolerance due to float precision and rounding
                let r_diff = (r as f32 - expected_pixel[0]).abs();
                let g_diff = (g as f32 - expected_pixel[1]).abs();
                let b_diff = (b as f32 - expected_pixel[2]).abs();
                let a_diff = (a as f32 - expected_pixel[3]).abs();

                if r_diff > 1.5 || g_diff > 1.5 || b_diff > 1.5 || a_diff > 1.5 { // Allow diff of 1 due to rounding
                    if mismatches < 5 { // Print only first few mismatches
                        println!(
                            "Mismatch at ({}, {}): Got [{}, {}, {}, {}], Expected ~[{}, {}, {}, {}] (raw expected: {:?})",
                            x, y, r, g, b, a, 
                            expected_pixel[0].round() as u8, expected_pixel[1].round() as u8, 
                            expected_pixel[2].round() as u8, expected_pixel[3].round() as u8,
                            expected_pixel
                        );
                    }
                    mismatches += 1;
                }
            }
        }
        assert_eq!(mismatches, 0, "Pixel mismatch count: {}", mismatches);
    }
} 