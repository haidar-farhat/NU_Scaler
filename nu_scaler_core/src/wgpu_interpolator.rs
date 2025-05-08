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
    CommandEncoderDescriptor, ComputePassDescriptor,
};

// Uniform structure for the warp/blend shader
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InterpolationUniforms {
    output_dims: [u32; 2], // width, height
    time_t: f32,
    _padding: u32, // WGSL uniform structs require members to be aligned to 4 bytes. vec2<u32> is 8, f32 is 4.
                   // For robust layout, ensure total size is multiple of 16 for some platforms if it's a larger struct.
                   // Here, [u32;2] + f32 + u32 = 8 + 4 + 4 = 16 bytes. This should be fine.
}

pub struct WgpuFrameInterpolator {
    device: Arc<Device>,
    // queue: Arc<Queue>, // The queue is used per-call in interpolate, not stored if not needed otherwise

    warp_blend_pipeline: ComputePipeline,
    warp_blend_bind_group_layout: BindGroupLayout,
    // Optical flow pipeline and related resources will be added later
}

impl WgpuFrameInterpolator {
    pub fn new(device: Arc<Device>) -> Result<Self> {
        let warp_blend_shader_source = r#"
            struct InterpolationUniforms {
                output_dims: vec2<u32>,
                time_t: f32,
                // _padding: u32, // Not needed if vec2<u32> aligns correctly
            };

            @group(0) @binding(0) var frame_a_tex: texture_2d<f32>;
            @group(0) @binding(1) var frame_b_tex: texture_2d<f32>;
            @group(0) @binding(2) var flow_texture: texture_storage_2d<rg32float, read>;
            @group(0) @binding(3) var output_texture: texture_storage_2d<rgba8unorm, write>;
            @group(0) @binding(4) var frame_sampler: sampler;
            @group(0) @binding(5) var<uniform> uniforms: InterpolationUniforms;

            @compute @workgroup_size(16, 16, 1)
            fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                if (gid.x >= uniforms.output_dims.x || gid.y >= uniforms.output_dims.y) {
                    return;
                }

                let output_coord_i32 = vec2<i32>(i32(gid.x), i32(gid.y));
                let current_pixel_center = vec2<f32>(f32(gid.x) + 0.5, f32(gid.y) + 0.5);
                let flow_motion_vector = textureLoad(flow_texture, output_coord_i32).xy;
                let output_dims_f32 = vec2<f32>(f32(uniforms.output_dims.x), f32(uniforms.output_dims.y));

                let uv_sample_coord_frame_a = (current_pixel_center - uniforms.time_t * flow_motion_vector) / output_dims_f32;
                let uv_sample_coord_frame_b = (current_pixel_center + (1.0 - uniforms.time_t) * flow_motion_vector) / output_dims_f32;

                let color_frame_a = textureSample(frame_a_tex, frame_sampler, uv_sample_coord_frame_a);
                let color_frame_b = textureSample(frame_b_tex, frame_sampler, uv_sample_coord_frame_b);

                let interpolated_color = mix(color_frame_a, color_frame_b, uniforms.time_t);
                textureStore(output_texture, output_coord_i32, interpolated_color);
            }
        "#;

        let warp_blend_shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Warp/Blend Shader Module"),
            source: ShaderSource::Wgsl(warp_blend_shader_source.into()),
        });

        let warp_blend_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Warp/Blend Bind Group Layout"),
            entries: &[
                // frame_a_tex (texture_2d<f32>)
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // frame_b_tex (texture_2d<f32>)
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // flow_texture (texture_storage_2d<rg32float, read>)
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: TextureFormat::Rg32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // output_texture (texture_storage_2d<rgba8unorm, write>)
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm, // Common output format
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // frame_sampler
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // uniforms buffer
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None, //wgpu::BufferSize::new(std::mem::size_of::<InterpolationUniforms>() as u64),
                    },
                    count: None,
                },
            ],
        });

        let warp_blend_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Warp/Blend Pipeline Layout"),
            bind_group_layouts: &[&warp_blend_bind_group_layout],
            push_constant_ranges: &[],
        });

        let warp_blend_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Warp/Blend Pipeline"),
            layout: Some(&warp_blend_pipeline_layout),
            module: &warp_blend_shader_module,
            entry_point: "main",
        });

        Ok(Self {
            device,
            // queue,
            warp_blend_pipeline,
            warp_blend_bind_group_layout,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn interpolate(
        &self,
        queue: &Queue, // Pass queue as an argument for the call
        frame_a_view: &TextureView,
        frame_b_view: &TextureView,
        flow_texture_view: &TextureView, // Assumed to be populated by optical flow pass
        output_texture_view: &TextureView,
        sampler: &Sampler, // Sampler for frame_a and frame_b
        width: u32,
        height: u32,
        time_t: f32, // Interpolation factor 0.0 to 1.0
    ) -> Result<()> {
        let uniforms_data = InterpolationUniforms {
            output_dims: [width, height],
            time_t,
            _padding: 0, // Ensure struct is Pod
        };
        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Interpolation Uniform Buffer"),
            contents: bytemuck::bytes_of(&uniforms_data),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Warp/Blend Bind Group"),
            layout: &self.warp_blend_bind_group_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: BindingResource::TextureView(frame_a_view) },
                BindGroupEntry { binding: 1, resource: BindingResource::TextureView(frame_b_view) },
                BindGroupEntry { binding: 2, resource: BindingResource::TextureView(flow_texture_view) },
                BindGroupEntry { binding: 3, resource: BindingResource::TextureView(output_texture_view) },
                BindGroupEntry { binding: 4, resource: BindingResource::Sampler(sampler) },
                BindGroupEntry { binding: 5, resource: uniform_buffer.as_entire_binding() },
            ],
        });

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Interpolate Command Encoder"),
        });
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Warp/Blend Compute Pass"),
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

    // Future methods for optical flow:
    // pub fn compute_optical_flow(&self, /* ... params ... */) -> Result<Texture> { /* ... */ }
}

// Helper function to create a dummy flow texture (e.g., zero flow) for testing
pub fn create_dummy_flow_texture(device: &Device, width: u32, height: u32) -> (Texture, TextureView) {
    let texture_size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
    let flow_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Dummy Flow Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TextureFormat::Rg32Float, // For (dx, dy)
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_DST, // Allow writing dummy data
        view_formats: &[],
    });
    // Optionally, write zero data to it or some pattern
    // For zero data, it might be initialized to zero by default, or use queue.write_texture

    let flow_texture_view = flow_texture.create_view(&wgpu::TextureViewDescriptor::default());
    (flow_texture, flow_texture_view)
} 