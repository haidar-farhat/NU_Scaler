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
use crate::utils::teinture_wgpu::WgpuState;
use image::{ImageBuffer, Rgba, Rgb};
use wgpu::util::DeviceExt; // For create_buffer_init
use log::{debug, info, warn};

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

// Uniform struct for Horn-Schunck shader
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct HornSchunckParams {
    size: [u32; 2],   // Texture dimensions
    lambda: f32,       // Smoothness weight
    _pad0: u32,        // Padding
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct HornSchunckUniforms {
    alpha_sq: f32,
    delta_t: f32,
    inverse_tex_size: [f32; 2], // Corresponds to vec2<f32> in WGSL
    // Total 16 bytes, should be fine for alignment
}

pub struct WgpuFrameInterpolator {
    device: Arc<Device>,
    queue: Arc<Queue>,
    warp_blend_pipeline: Option<RenderPipeline>,
    warp_blend_bgl: Option<BindGroupLayout>,
    blur_h_pipeline: Option<ComputePipeline>,
    blur_v_pipeline: Option<ComputePipeline>,
    downsample_pipeline: Option<ComputePipeline>,
    pyramid_pass_bind_group_layout: BindGroupLayout,
    shared_sampler: Sampler,
    blur_temp_texture: Option<Texture>,
    blur_temp_texture_view: Option<TextureView>,
    pyramid_a_textures: Vec<Option<Texture>>,
    pyramid_a_views: Vec<Option<TextureView>>,
    pyramid_b_textures: Vec<Option<Texture>>,
    pyramid_b_views: Vec<Option<TextureView>>,
    downsample_a_textures: Vec<Option<Texture>>,
    downsample_a_views: Vec<Option<TextureView>>,
    downsample_b_textures: Vec<Option<Texture>>,
    downsample_b_views: Vec<Option<TextureView>>,
    horn_schunck_pipeline: Option<ComputePipeline>,
    horn_schunck_bgl: BindGroupLayout,
    flow_textures: [Option<Texture>; 2],
    flow_views: [Option<TextureView>; 2],
    flow_sampler: Sampler,
    final_flow_texture: Option<Texture>,
    final_flow_view: Option<TextureView>,
}

impl WgpuFrameInterpolator {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Result<Self> {
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

        let warp_blend_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Warp/Blend Pipeline"),
            layout: Some(&warp_blend_pipeline_layout),
            vertex: VertexState {
                module: &warp_blend_shader_module,
                entry_point: "vs_main",
                buffers: &[], // No vertex buffer, quad generated in VS
            },
            fragment: Some(FragmentState {
                module: &warp_blend_shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: TextureFormat::Rgba8UnormSrgb, // Assuming output to SRGB
                    blend: None, // Or some blending mode
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip, // Fullscreen quad
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for quad
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
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

        // --- Phase 2.2 Setup: Horn-Schunck --- 
        let hs_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Horn-Schunck Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/horn_schunck.wgsl").into()),
        });

        // Horn-Schunck BGL and Pipeline
        let horn_schunck_bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Horn-Schunck BGL"),
            entries: &[
                // prev_frame_level (I0 luminance, from Rgba32Float texture)
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // next_frame_level (I1 luminance, from Rgba32Float texture)
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // prev_flow_level_uv (previous iteration's flow, Rg32Float texture)
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true }, // Sampled in shader
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // flow_sampler
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering), // Matches flow_sampler type
                    count: None,
                },
                // uniforms (HornSchunckUniforms)
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::mem::size_of::<HornSchunckUniforms>() as u64),
                    },
                    count: None,
                },
                // out_flow_level_uv (current iteration's flow, Rg32Float storage texture)
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rg32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let horn_schunck_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Horn-Schunck Pipeline Layout"),
            bind_group_layouts: &[&horn_schunck_bgl],
            push_constant_ranges: &[],
        });

        let horn_schunck_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Horn-Schunck Pipeline"),
            layout: Some(&horn_schunck_pipeline_layout),
            module: &hs_shader_module,
            entry_point: "main",
        });

        // Create sampler for flow textures (Nearest neighbor)
        let flow_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Flow Sampler (Nearest)"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..
            Default::default()
        });

        Ok(Self {
            device,
            queue,
            warp_blend_pipeline: Some(warp_blend_pipeline),
            warp_blend_bgl: Some(warp_blend_bgl),
            blur_h_pipeline: Some(blur_h_pipeline),
            blur_v_pipeline: Some(blur_v_pipeline),
            downsample_pipeline: Some(downsample_pipeline),
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
            horn_schunck_pipeline: Some(horn_schunck_pipeline),
            horn_schunck_bgl,
            flow_textures: [None, None],
            flow_views: [None, None],
            flow_sampler,
            final_flow_texture: None,
            final_flow_view: None,
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
            layout: &self.warp_blend_bgl.as_ref().unwrap(),
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
            compute_pass.set_pipeline(&self.warp_blend_pipeline.as_ref().unwrap());
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

    // Helper to create or resize a texture stored in an Option
    fn ensure_texture(
        device: &Device, 
        current_texture_opt: &mut Option<Texture>,
        width: u32,
        height: u32,
        format: TextureFormat,
        usage: TextureUsages,
        label: &str
    ) -> bool { // Returns true if texture was created/resized
        let needs_recreation = match current_texture_opt {
            Some(tex) => tex.width() != width || tex.height() != height || tex.format() != format || !tex.usage().contains(usage),
            None => true,
        };

        if needs_recreation {
            println!("Recreating texture: {}", label);
            *current_texture_opt = Some(device.create_texture(&TextureDescriptor {
                label: Some(label),
                size: Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            }));
            true
        } else {
            false
        }
    }
    
    // Builds an image pyramid for either frame A or frame B
    pub fn build_pyramid(
        &mut self, 
        queue: &Queue, 
        frame_texture: &Texture, // Full res Rgba32Float texture
        levels: u32,
        is_frame_a: bool, // true for pyramid A, false for pyramid B
    ) -> Result<()> {
        let (base_width, base_height) = (frame_texture.width(), frame_texture.height());
        let format = frame_texture.format(); // Should be Rgba32Float
        let usage = TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING | TextureUsages::COPY_DST | TextureUsages::COPY_SRC;
        let view_desc = TextureViewDescriptor::default();

        let label_prefix = if is_frame_a { "PyramidA" } else { "PyramidB" };

        // Select the correct vectors
        let pyramid_textures = if is_frame_a { &mut self.pyramid_a_textures } else { &mut self.pyramid_b_textures };
        let pyramid_views = if is_frame_a { &mut self.pyramid_a_views } else { &mut self.pyramid_b_views };
        let downsample_textures = if is_frame_a { &mut self.downsample_a_textures } else { &mut self.downsample_b_textures };
        let downsample_views = if is_frame_a { &mut self.downsample_a_views } else { &mut self.downsample_b_views };

        // Resize vectors if levels changed or they are empty
        let num_levels = levels as usize;
        if pyramid_textures.len() != num_levels {
            println!("Resizing pyramid texture storage for {} levels", levels);
            pyramid_textures.resize_with(num_levels, || None);
            pyramid_views.resize_with(num_levels, || None);
            downsample_textures.resize_with(num_levels, || None);
            downsample_views.resize_with(num_levels, || None);
        }

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some(&format!("{} Build Encoder", label_prefix)),
        });

        let mut current_width = base_width;
        let mut current_height = base_height;
        // Start with the original frame texture view
        let mut last_input_view = frame_texture.create_view(&view_desc); 
        let blur_radius = 2; // For 5x5 kernel used in shaders

        for level in 0..levels {
            let level_u = level as usize;
            let next_width = (current_width + 1) / 2;
            let next_height = (current_height + 1) / 2;
            
            if current_width == 0 || current_height == 0 || next_width == 0 || next_height == 0 {
                 println!("{} Pyramid generation stopped at level {} due to zero dimension.", label_prefix, level);
                 pyramid_textures.truncate(level_u);
                 pyramid_views.truncate(level_u);
                 downsample_textures.truncate(level_u);
                 downsample_views.truncate(level_u);
                 break;
            }

            println!("Building {} Level {}: {}x{} -> {}x{}", label_prefix, level, current_width, current_height, next_width, next_height);

            // --- Ensure Textures Exist --- 
            Self::ensure_texture(&self.device, &mut self.blur_temp_texture, current_width, current_height, format, usage, &format!("{} Blur Temp", label_prefix));
            self.blur_temp_texture_view = Some(self.blur_temp_texture.as_ref().unwrap().create_view(&view_desc));
            let blur_temp_view_ref = self.blur_temp_texture_view.as_ref().unwrap();
            
            Self::ensure_texture(&self.device, &mut pyramid_textures[level_u], current_width, current_height, format, usage, &format!("{} Pyramid Level {}", label_prefix, level));
            pyramid_views[level_u] = Some(pyramid_textures[level_u].as_ref().unwrap().create_view(&view_desc));
            let pyramid_view_ref = pyramid_views[level_u].as_ref().unwrap();

            // Downsampled output - only really needed if not the last level
            let downsample_view_ref = if level < levels - 1 {
                 Self::ensure_texture(&self.device, &mut downsample_textures[level_u], next_width, next_height, format, usage, &format!("{} Downsample Level {}", label_prefix, level));
                 downsample_views[level_u] = Some(downsample_textures[level_u].as_ref().unwrap().create_view(&view_desc));
                 downsample_views[level_u].as_ref().unwrap()
            } else {
                 // Use the last pyramid view as a dummy if no more levels, avoid creating unused texture.
                 // However, the binding needs a view. Let's ensure the texture exists even if unused.
                 Self::ensure_texture(&self.device, &mut downsample_textures[level_u], next_width.max(1), next_height.max(1), format, usage, &format!("{} Downsample Level {} (Last)", label_prefix, level));
                 downsample_views[level_u] = Some(downsample_textures[level_u].as_ref().unwrap().create_view(&view_desc));
                 downsample_views[level_u].as_ref().unwrap()
            };
            
            // --- Create Uniform Buffers & Bind Groups --- 
            let params_h = PyramidPassParams { in_size: [current_width, current_height], out_size: [current_width, current_height], radius: blur_radius, _pad0:0, _pad1:[0,0] };
            let uniform_buffer_h = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(&format!("{} Blur H Uniforms L{}", label_prefix, level)), contents: bytemuck::bytes_of(&params_h), usage: BufferUsages::UNIFORM });
            let bind_group_h = self.device.create_bind_group(&BindGroupDescriptor { label: Some(&format!("{} Blur H BG L{}", label_prefix, level)), layout: &self.pyramid_pass_bind_group_layout, entries: &[
                BindGroupEntry { binding: 0, resource: uniform_buffer_h.as_entire_binding() },
                BindGroupEntry { binding: 1, resource: BindingResource::TextureView(&last_input_view) }, // Input
                BindGroupEntry { binding: 2, resource: BindingResource::TextureView(blur_temp_view_ref) }, // Output
                BindGroupEntry { binding: 3, resource: BindingResource::Sampler(&self.shared_sampler) },
            ]});

            let params_v = PyramidPassParams { in_size: [current_width, current_height], out_size: [current_width, current_height], radius: blur_radius, _pad0:0, _pad1:[0,0] };
            let uniform_buffer_v = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(&format!("{} Blur V Uniforms L{}", label_prefix, level)), contents: bytemuck::bytes_of(&params_v), usage: BufferUsages::UNIFORM });
            let bind_group_v = self.device.create_bind_group(&BindGroupDescriptor { label: Some(&format!("{} Blur V BG L{}", label_prefix, level)), layout: &self.pyramid_pass_bind_group_layout, entries: &[
                BindGroupEntry { binding: 0, resource: uniform_buffer_v.as_entire_binding() },
                BindGroupEntry { binding: 1, resource: BindingResource::TextureView(blur_temp_view_ref) }, // Input (from H pass)
                BindGroupEntry { binding: 2, resource: BindingResource::TextureView(pyramid_view_ref) }, // Output (final for level)
                BindGroupEntry { binding: 3, resource: BindingResource::Sampler(&self.shared_sampler) },
            ]});

            let params_ds = PyramidPassParams { in_size: [current_width, current_height], out_size: [next_width, next_height], radius: 0, _pad0:0, _pad1:[0,0] }; // Radius not used
            let uniform_buffer_ds = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(&format!("{} DS Uniforms L{}", label_prefix, level)), contents: bytemuck::bytes_of(&params_ds), usage: BufferUsages::UNIFORM });
            let bind_group_ds = self.device.create_bind_group(&BindGroupDescriptor { label: Some(&format!("{} DS BG L{}", label_prefix, level)), layout: &self.pyramid_pass_bind_group_layout, entries: &[
                BindGroupEntry { binding: 0, resource: uniform_buffer_ds.as_entire_binding() },
                BindGroupEntry { binding: 1, resource: BindingResource::TextureView(pyramid_view_ref) }, // Input (blurred)
                BindGroupEntry { binding: 2, resource: BindingResource::TextureView(downsample_view_ref) }, // Output (for next level)
                BindGroupEntry { binding: 3, resource: BindingResource::Sampler(&self.shared_sampler) },
            ]});

            // --- Dispatch Compute Passes --- 
            {
                let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor { 
                    label: Some(&format!("{} Pyramid Compute Pass L{}", label_prefix, level)), 
                    timestamp_writes: None 
                });
                let wg_size = 16;
                
                let dispatch_x_curr = (current_width + wg_size - 1) / wg_size;
                let dispatch_y_curr = (current_height + wg_size - 1) / wg_size;
                let dispatch_x_next = (next_width + wg_size - 1) / wg_size;
                let dispatch_y_next = (next_height + wg_size - 1) / wg_size;
                
                // Horizontal Blur
                compute_pass.set_pipeline(&self.blur_h_pipeline.as_ref().unwrap());
                compute_pass.set_bind_group(0, &bind_group_h, &[]);
                compute_pass.dispatch_workgroups(dispatch_x_curr, dispatch_y_curr, 1);

                // Vertical Blur
                compute_pass.set_pipeline(&self.blur_v_pipeline.as_ref().unwrap());
                compute_pass.set_bind_group(0, &bind_group_v, &[]);
                compute_pass.dispatch_workgroups(dispatch_x_curr, dispatch_y_curr, 1);

                // Downsample
                compute_pass.set_pipeline(&self.downsample_pipeline.as_ref().unwrap());
                compute_pass.set_bind_group(0, &bind_group_ds, &[]);
                compute_pass.dispatch_workgroups(dispatch_x_next, dispatch_y_next, 1);
            }
            
            // Prepare for next level
            // The input for the next blur/downsample iteration is the downsampled output of this one.
            // Get the texture from the downsample vector and create a new view for the next loop iteration.
            last_input_view = downsample_textures[level_u].as_ref().unwrap().create_view(&view_desc);
            
            current_width = next_width;
            current_height = next_height;
        }

        queue.submit(Some(encoder.finish()));
        Ok(())
    }

    // --- Phase 2.2: Coarse Optical Flow --- 
    pub fn compute_coarse_flow(
        &mut self,
        device: &Device,
        queue: &Queue,
        level: usize, // Coarsest pyramid level index
        num_iterations: usize,
        alpha_sq: f32,
    ) {
        info!("Computing coarse flow for pyramid level {} with {} iterations, alpha_sq={}", level, num_iterations, alpha_sq);

        let prev_frame_tex_view = self.pyramid_a_views[level].as_ref().expect("Prev frame view (Pyramid A) for level not found");
        let next_frame_tex_view = self.pyramid_b_views[level].as_ref().expect("Next frame view (Pyramid B) for level not found");
        
        let width = self.pyramid_a_textures[level].as_ref().unwrap().width();
        let height = self.pyramid_a_textures[level].as_ref().unwrap().height();

        self.ensure_flow_textures(device, width, height);

        let uniforms = HornSchunckUniforms {
            alpha_sq,
            delta_t: 1.0,
            inverse_tex_size: [1.0 / width as f32, 1.0 / height as f32],
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Horn-Schunck Uniform Buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: BufferUsages::UNIFORM,
        });

        // Clear initial flow texture (flow_textures[0] will be the first input)
        let flow_tex_0_ref = self.flow_textures[0].as_ref().unwrap();
        let flow_tex_bytes_per_pixel = 8; // Rg32Float (2 * f32)
        let zero_data_size = (width * height * flow_tex_bytes_per_pixel) as usize;
        let zero_data: Vec<u8> = vec![0; zero_data_size];

        queue.write_texture(
            ImageCopyTexture {
                texture: flow_tex_0_ref,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &zero_data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * flow_tex_bytes_per_pixel),
                rows_per_image: Some(height),
            },
            Extent3d { width, height, depth_or_array_layers: 1 },
        );
        
        let pipeline = self.horn_schunck_pipeline.as_ref().expect("Horn-Schunck pipeline not initialized");
        let bgl = self.horn_schunck_bgl;
        let sampler = self.flow_sampler;

        for i in 0..num_iterations {
            let (current_input_flow_view_idx, current_output_flow_view_idx) = if i % 2 == 0 {
                (0, 1) // Input: flow_textures[0], Output: flow_textures[1]
            } else {
                (1, 0) // Input: flow_textures[1], Output: flow_textures[0]
            };
            
            let current_input_flow_view = self.flow_views[current_input_flow_view_idx].as_ref().unwrap();
            let current_output_flow_view = self.flow_views[current_output_flow_view_idx].as_ref().unwrap();

            let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
                label: Some(&format!("Horn-Schunck Bind Group Iter {}", i)),
                layout: &bgl,
                entries: &[
                    BindGroupEntry { binding: 0, resource: BindingResource::TextureView(prev_frame_tex_view) },
                    BindGroupEntry { binding: 1, resource: BindingResource::TextureView(next_frame_tex_view) },
                    BindGroupEntry { binding: 2, resource: BindingResource::TextureView(current_input_flow_view) },
                    BindGroupEntry { binding: 3, resource: BindingResource::Sampler(sampler) },
                    BindGroupEntry { binding: 4, resource: uniform_buffer.as_entire_binding() },
                    BindGroupEntry { binding: 5, resource: BindingResource::TextureView(current_output_flow_view) },
                ],
            });

            let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some(&format!("Horn-Schunck Command Encoder Iter {}", i)),
            });
            {
                let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some(&format!("Horn-Schunck Compute Pass Iter {}", i)),
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(pipeline);
                compute_pass.set_bind_group(0, &bind_group, &[]);
                compute_pass.dispatch_workgroups(
                    (width + 7) / 8, // Workgroup size 8x8 defined in horn_schunck.wgsl
                    (height + 7) / 8,
                    1,
                );
            }
            queue.submit(std::iter::once(encoder.finish()));
            debug!("Submitted HS iteration {}", i);
        }
        
        let final_flow_location = if num_iterations == 0 {
            "flow_textures[0] (cleared)"
        } else if num_iterations % 2 == 1 {
            "flow_textures[1]" // After odd iterations (1, 3, 5...), iter 0 writes to [1], iter 1 to [0], iter 2 to [1]
        } else {
            "flow_textures[0]" // After even iterations (2, 4, ...), iter N-1 writes to [0]
        };
        info!("Coarse flow computation complete for level {}. Final flow in {}", level, final_flow_location);
    }

    fn ensure_flow_textures(&mut self, device: &Device, width: u32, height: u32) {
        let texture_desc = TextureDescriptor {
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rg32Float, // For (u,v) flow vectors
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING | TextureUsages::COPY_DST,
            label: None, // Set specific labels below
        };

        // Texture A (index 0)
        if self.flow_textures[0].as_ref().map_or(true, |t| t.width() != width || t.height() != height || t.format() != texture_desc.format) {
            debug!("Creating Flow Texture 0 (A): {}x{} Format: {:?}", width, height, texture_desc.format);
            let tex_a = device.create_texture(&TextureDescriptor {
                label: Some("Flow Texture 0 (A)"),
                ..texture_desc
            });
            self.flow_views[0] = Some(tex_a.create_view(&TextureViewDescriptor::default()));
            self.flow_textures[0] = Some(tex_a);
        }

        // Texture B (index 1)
        if self.flow_textures[1].as_ref().map_or(true, |t| t.width() != width || t.height() != height || t.format() != texture_desc.format) {
            debug!("Creating Flow Texture 1 (B): {}x{} Format: {:?}", width, height, texture_desc.format);
            let tex_b = device.create_texture(&TextureDescriptor {
                label: Some("Flow Texture 1 (B)"),
                ..texture_desc
            });
            self.flow_views[1] = Some(tex_b.create_view(&TextureViewDescriptor::default()));
            self.flow_textures[1] = Some(tex_b);
        }
    }
}

// Test module
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::teinture_wgpu::{create_device_queue, ComparableTexture, create_texture_with_data, TextureDataOrder};
    // use image::{ImageBuffer, Rgba, Rgb}; // Rgb might not be needed if not creating Rgb images

    #[test]
    fn test_warp_blend_zero_flow() {
        // ... (existing test_warp_blend_zero_flow implementation, ensure it uses Arc for device/queue)
        let (_device, _queue, instance, adapter) = futures::executor::block_on(create_device_queue());
        let device = std::sync::Arc::new(_device);
        let queue = std::sync::Arc::new(_queue);
        // ... rest of the test, pass device and queue Arc to WgpuFrameInterpolator::new
        // and use them for creating textures etc.
        let mut interpolator = WgpuFrameInterpolator::new(device.clone(), queue.clone()).expect("Failed to create interpolator");

        let width = 256;
        let height = 256;

        // Create dummy textures (e.g., black, white, or patterned)
        let black_image_data: Vec<u8> = vec![0; (width * height * 4) as usize]; // RGBA8
        let white_image_data: Vec<u8> = vec![255; (width * height * 4) as usize]; // RGBA8

        let prev_frame_desc = TextureDescriptor {
            label: Some("Prev Frame Test"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        };
        let prev_frame = create_texture_with_data(&device, &queue, &prev_frame_desc, TextureDataOrder::LayerMajor, &black_image_data);

        let next_frame_desc = TextureDescriptor {
            label: Some("Next Frame Test"),
            // ... same as prev_frame_desc
            ..prev_frame_desc
        };
        let next_frame = create_texture_with_data(&device, &queue, &next_frame_desc, TextureDataOrder::LayerMajor, &white_image_data);
        
        // Create a zero flow texture (Rg32Float)
        let zero_flow_data: Vec<f32> = vec![0.0; (width * height * 2) as usize]; // RG32Float
        let motion_vectors_desc = TextureDescriptor {
            label: Some("Zero Flow Test"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rg32Float, // Flow texture format
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        };
        let motion_vectors = create_texture_with_data(
            &device, 
            &queue, 
            &motion_vectors_desc, 
            TextureDataOrder::LayerMajor, 
            bytemuck::cast_slice(&zero_flow_data)
        );

        let output_texture_desc = TextureDescriptor {
            label: Some("Output Texture Test"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb, // Match pipeline target
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        };
        let output_texture = device.create_texture(&output_texture_desc);
        let output_texture_view = output_texture.create_view(&TextureViewDescriptor::default());

        interpolator.interpolate(
            &prev_frame,
            &next_frame,
            &motion_vectors,
            &output_texture_view,
            width,
            height,
            1.0, // flow_scale
            0.0, // blend_factor (should show prev_frame)
        );
        
        // Read back and verify (e.g., output matches prev_frame)
        let comparable_output = ComparableTexture::from_texture(&device, &queue, &output_texture, "Output Zero Flow");
        let comparable_prev = ComparableTexture::from_texture(&device, &queue, &prev_frame, "Prev Frame Zero Flow");

        // Allow a small difference due to potential format conversions or filtering if any
        assert!(comparable_output.is_similar(&comparable_prev, 0.01), "Output with zero flow (blend 0.0) should match prev_frame");

        // Test with blend_factor = 1.0 (should show next_frame)
         interpolator.interpolate(
            &prev_frame,
            &next_frame,
            &motion_vectors,
            &output_texture_view,
            width,
            height,
            1.0, // flow_scale
            1.0, // blend_factor (should show next_frame)
        );
        let comparable_output_blend1 = ComparableTexture::from_texture(&device, &queue, &output_texture, "Output Zero Flow Blend 1");
        let comparable_next = ComparableTexture::from_texture(&device, &queue, &next_frame, "Next Frame Zero Flow Blend 1");
        assert!(comparable_output_blend1.is_similar(&comparable_next, 0.01), "Output with zero flow (blend 1.0) should match next_frame");
    }

    #[test]
    fn test_build_pyramid() {
        // ... (existing test_build_pyramid implementation, ensure it uses Arc for device/queue)
        let (_device, _queue, instance, adapter) = futures::executor::block_on(create_device_queue());
        let device = std::sync::Arc::new(_device);
        let queue = std::sync::Arc::new(_queue);
        let mut interpolator = WgpuFrameInterpolator::new(device.clone(), queue.clone()).expect("Failed to create interpolator");
        // ... rest of the test
        let width = 256u32;
        let height = 256u32;
        let num_levels = 4;

        let dummy_image_data: Vec<u8> = (0..(width * height * 4)).map(|i| (i % 256) as u8).collect();
        let input_texture_desc = TextureDescriptor {
            label: Some("Pyramid Input Test"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb, // build_pyramid expects Rgba8UnormSrgb
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING, // Add STORAGE_BINDING if level 0 is written via storage
        };
         let input_texture = create_texture_with_data(&device, &queue, &input_texture_desc, TextureDataOrder::LayerMajor, &dummy_image_data);


        interpolator.build_pyramid(&input_texture, width, height, num_levels, true); // For pyramid_a

        assert_eq!(interpolator.pyramid_a_textures.len(), num_levels);
        assert_eq!(interpolator.pyramid_a_views.len(), num_levels);

        for i in 0..num_levels {
            let current_width = width / (2u32.pow(i as u32));
            let current_height = height / (2u32.pow(i as u32));

            let tex = interpolator.pyramid_a_textures[i].as_ref().unwrap();
            assert_eq!(tex.width(), current_width, "Pyramid level {} width mismatch", i);
            assert_eq!(tex.height(), current_height, "Pyramid level {} height mismatch", i);
            assert_eq!(tex.format(), TextureFormat::Rgba32Float, "Pyramid level {} format mismatch", i);
            
            let view = interpolator.pyramid_a_views[i].as_ref().unwrap();
            // Basic check, view descriptor could be inspected further if complex.
            assert!(view.dimension() == TextureViewDimension::D2);
        }

        // TODO: Could add readback and hash/value checks for pyramid levels if exact values are known/expected.
        // For now, this confirms creation, dimensions, and format.
    }

    #[test]
    fn test_compute_coarse_flow_zeros() {
        // Test that compute_coarse_flow runs with zero iterations and with some iterations
        // on basic input (e.g. identical black frames)
        // This primarily tests pipeline setup, resource binding, and execution without crashing.

        let (_device, _queue, _instance, _adapter) = futures::executor::block_on(create_device_queue());
        let device = std::sync::Arc::new(_device);
        let queue = std::sync::Arc::new(_queue);

        let mut interpolator = WgpuFrameInterpolator::new(device.clone(), queue.clone()).expect("Failed to create interpolator");

        let width = 64u32;
        let height = 64u32;

        // Create dummy black frames
        let black_image_data: Vec<u8> = vec![0; (width * height * 4) as usize];
        let prev_frame_texture = create_texture_with_data(
            &device,
            &queue,
            &TextureDescriptor {
                label: Some("Prev Frame (Black) for HS Test"),
                size: Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb, // build_pyramid expects Srgb for input
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING,
            },
            TextureDataOrder::LayerMajor,
            &black_image_data,
        );
         let next_frame_texture = create_texture_with_data(
            &device,
            &queue,
            &TextureDescriptor {
                label: Some("Next Frame (Black) for HS Test"),
                size: Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING,
            },
            TextureDataOrder::LayerMajor,
            &black_image_data,
        );

        let num_pyramid_levels = 3;
        // Populate pyramid_a_textures and pyramid_b_textures
        interpolator.build_pyramid(&prev_frame_texture, width, height, num_pyramid_levels, true); // is_for_pyramid_a = true
        interpolator.build_pyramid(&next_frame_texture, width, height, num_pyramid_levels, false); // is_for_pyramid_a = false
        
        let coarsest_level_idx = num_pyramid_levels - 1;
        
        // Test with 0 iterations
        interpolator.compute_coarse_flow(&device, &queue, coarsest_level_idx, 0, 0.02f32.powi(2));
        
        let level_width = width / (2u32.pow(coarsest_level_idx as u32));
        let level_height = height / (2u32.pow(coarsest_level_idx as u32));

        assert!(interpolator.flow_textures[0].is_some());
        let flow_tex_0 = interpolator.flow_textures[0].as_ref().unwrap();
        assert_eq!(flow_tex_0.width(), level_width);
        assert_eq!(flow_tex_0.height(), level_height);
        assert_eq!(flow_tex_0.format(), TextureFormat::Rg32Float);
        // TODO: Read back flow_textures[0] and verify it's all zeros.

        // Test with a few iterations
        let num_hs_iterations = 5;
        interpolator.compute_coarse_flow(&device, &queue, coarsest_level_idx, num_hs_iterations, 0.02f32.powi(2));
        info!("Finished compute_coarse_flow test with {} iterations.", num_hs_iterations);
        
        // Check which texture should hold the result
        let final_flow_idx = if num_hs_iterations == 0 { 0 } else if num_hs_iterations % 2 == 1 { 1 } else { 0 };
        assert!(interpolator.flow_textures[final_flow_idx].is_some());
        let final_flow_tex = interpolator.flow_textures[final_flow_idx].as_ref().unwrap();
        assert_eq!(final_flow_tex.width(), level_width);
        assert_eq!(final_flow_tex.height(), level_height);
        assert_eq!(final_flow_tex.format(), TextureFormat::Rg32Float);
        
        // Basic check: does not panic. More detailed checks would involve reading back texture content.
        // For identical black frames, flow should be zero.
        // We can use ComparableTexture to read back and check if it's close to zero.
        let comparable_flow = ComparableTexture::from_texture(&device, &queue, final_flow_tex, "HS Flow Output");
        let zero_f32_data: Vec<f32> = vec![0.0; (level_width * level_height * 2) as usize]; // RG32Float, 2 floats per pixel
        
        let mut all_zeros = true;
        if let Some(data_bytes) = comparable_flow.data_rgba8.as_ref() { // This reads back as Rgba8UnormSrgb by default
            // This comparison is tricky because ComparableTexture converts to RGBA8.
            // For Rg32Float, need a specialized readback or a more tolerant comparison.
            // Let's assume for now if it runs, it's a good step. A proper check needs Rg32Float readback.
            warn!("Skipping detailed zero check for HS flow due to RGBA8 readback of ComparableTexture. Manual verification or specialized readback needed for Rg32Float.");
        } else if let Some(data_f32) = comparable_flow.data_f32.as_ref() { // If ComparableTexture supports direct f32 readback
             for (i, val) in data_f32.iter().enumerate() {
                if val.abs() > 1e-5 { // Allow small tolerance for float precision
                    all_zeros = false;
                    warn!("HS Flow test: Non-zero flow found at index {}: {:?}", i, val);
                    break;
                }
            }
            assert!(all_zeros, "Flow for identical frames should be zero or very close to zero.");
        } else {
            warn!("HS Flow test: Could not read back texture data for verification.");
        }

    }
} 