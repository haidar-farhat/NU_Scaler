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
};

// Uniform structure for the warp/blend shader as per new spec
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InterpolationUniforms {
    size: [u32; 2],       // vec2<u32> -> offset 0, size 8
    _pad0: [u32; 2],      // vec2<u32> -> offset 8, size 8 (aligns time_t to 16)
    time_t: f32,          // f32       -> offset 16, size 4
    _pad1: [f32; 3],      // vec3<f32> -> offset 20, size 12 (total size 32 bytes)
}

impl InterpolationUniforms {
    fn new(width: u32, height: u32, time_t: f32) -> Self {
        Self {
            size: [width, height],
            _pad0: [0, 0], // Padding
            time_t,
            _pad1: [0.0, 0.0, 0.0], // Padding
        }
    }
}

pub struct WgpuFrameInterpolator {
    device: Arc<Device>,
    warp_blend_pipeline: ComputePipeline,
    warp_blend_bind_group_layout: BindGroupLayout,
}

impl WgpuFrameInterpolator {
    pub fn new(device: Arc<Device>) -> Result<Self> {
        // WGSL Shader source as per Phase 1.1
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
            @group(0) @binding(3) var flow_tex: texture_2d<vec2<f32>>; // Assuming rg32float sampled
            @group(0) @binding(4) var out_tex: texture_storage_2d<rgba8unorm, write>;
            @group(0) @binding(5) var image_sampler: sampler;
            @group(0) @binding(6) var flow_sampler: sampler; // Could be non-filtering

            @compute @workgroup_size(16, 16, 1)
            fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
                if (global_id.x >= u.size.x || global_id.y >= u.size.y) {
                    return;
                }

                let output_coord_i32 = vec2<i32>(i32(global_id.x), i32(global_id.y));
                let current_pixel_center_uv = (vec2<f32>(global_id.xy) + 0.5) / vec2<f32>(u.size);

                // Sample flow texture (flow vectors are typically stored as pixel displacements)
                let flow_pixel_delta = textureSampleLevel(flow_tex, flow_sampler, current_pixel_center_uv, 0.0).xy;

                // Normalized UV coordinates for sampling frame_a and frame_b
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
                // frame_a_tex
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                // frame_b_tex
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                // flow_tex (texture_2d<vec2<f32>> implies filterable float or unfilterable float)
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: TextureViewDimension::D2, multisampled: false }, // Assuming filterable for now, format Rg32Float
                    count: None,
                },
                // out_tex (storage texture)
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture { access: StorageTextureAccess::WriteOnly, format: TextureFormat::Rgba8Unorm, view_dimension: TextureViewDimension::D2 },
                    count: None,
                },
                // image_sampler (for frame_a, frame_b)
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // flow_sampler (for flow_tex)
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering), // Could be NonFiltering if flow data is precise per texel
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

        Ok(Self {
            device,
            warp_blend_pipeline,
            warp_blend_bind_group_layout,
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