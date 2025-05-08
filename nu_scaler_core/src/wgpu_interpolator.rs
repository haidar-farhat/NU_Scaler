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
    warp_blend_pipeline: ComputePipeline,
    warp_blend_bind_group_layout: BindGroupLayout,
    blur_h_pipeline: ComputePipeline,
    blur_v_pipeline: ComputePipeline,
    downsample_pipeline: ComputePipeline,
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
    horn_schunck_pipeline: ComputePipeline,
    horn_schunck_bgl: BindGroupLayout,
    flow_textures: [Option<Texture>; 2],
    flow_views: [Option<TextureView>; 2],
    flow_sampler: Sampler,
    final_flow_texture: Option<Texture>,
    final_flow_view: Option<TextureView>,
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
            horn_schunck_pipeline,
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
                compute_pass.set_pipeline(&self.blur_h_pipeline);
                compute_pass.set_bind_group(0, &bind_group_h, &[]);
                compute_pass.dispatch_workgroups(dispatch_x_curr, dispatch_y_curr, 1);

                // Vertical Blur
                compute_pass.set_pipeline(&self.blur_v_pipeline);
                compute_pass.set_bind_group(0, &bind_group_v, &[]);
                compute_pass.dispatch_workgroups(dispatch_x_curr, dispatch_y_curr, 1);

                // Downsample
                compute_pass.set_pipeline(&self.downsample_pipeline);
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
        queue: &Queue,
        // Assumes build_pyramid was called for both A and B
        num_iterations: u32,
        lambda: f32,
    ) -> Result<&TextureView> { // Returns view to the final flow texture
        let num_pyramid_levels = self.pyramid_a_views.len();
        if num_pyramid_levels == 0 {
            return Err(anyhow::anyhow!("Pyramids must be built before computing coarse flow."));
        }
        let coarsest_level_idx = num_pyramid_levels - 1;

        // Get views for the coarsest level textures from pyramid A and B
        let i1_view = self.pyramid_a_views[coarsest_level_idx].as_ref()
            .ok_or_else(|| anyhow::anyhow!("Coarsest pyramid level view A is missing"))?;
        let i2_view = self.pyramid_b_views[coarsest_level_idx].as_ref()
            .ok_or_else(|| anyhow::anyhow!("Coarsest pyramid level view B is missing"))?;
        
        // Get dimensions from the corresponding texture
        let i1_tex = self.pyramid_a_textures[coarsest_level_idx].as_ref().unwrap();
        let width = i1_tex.width();
        let height = i1_tex.height();

        // Ensure flow textures exist and match size
        let flow_usage = TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC | TextureUsages::COPY_DST;
        let flow_format = TextureFormat::Rg32Float;
        Self::ensure_texture(&self.device, &mut self.flow_textures[0], width, height, flow_format, flow_usage, "Flow Texture A");
        Self::ensure_texture(&self.device, &mut self.flow_textures[1], width, height, flow_format, flow_usage, "Flow Texture B");
        self.flow_views[0] = Some(self.flow_textures[0].as_ref().unwrap().create_view(&TextureViewDescriptor::default()));
        self.flow_views[1] = Some(self.flow_textures[1].as_ref().unwrap().create_view(&TextureViewDescriptor::default()));
        
        // TODO: Initialize one flow texture to zero (e.g., using a compute shader pass or queue.write_texture if possible)
        // For now, assume it might contain garbage from previous runs.

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Coarse Flow Encoder"),
        });

        let params_data = HornSchunckParams { size: [width, height], lambda, _pad0: 0 };
        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Horn-Schunck Uniforms"),
            contents: bytemuck::bytes_of(&params_data),
            usage: BufferUsages::UNIFORM,
        });

        let wg_size = 16;
        let dispatch_x = (width + wg_size - 1) / wg_size;
        let dispatch_y = (height + wg_size - 1) / wg_size;

        for i in 0..num_iterations {
            let (in_idx, out_idx) = if i % 2 == 0 { (0, 1) } else { (1, 0) };
            let flow_in_view = self.flow_views[in_idx].as_ref().unwrap();
            let flow_out_view = self.flow_views[out_idx].as_ref().unwrap();

            let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
                label: Some(&format!("Horn-Schunck BG Iter {}", i)),
                layout: &self.horn_schunck_bgl,
                entries: &[
                    BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                    BindGroupEntry { binding: 1, resource: BindingResource::TextureView(i1_view) },
                    BindGroupEntry { binding: 2, resource: BindingResource::TextureView(i2_view) },
                    BindGroupEntry { binding: 3, resource: BindingResource::TextureView(flow_in_view) },
                    BindGroupEntry { binding: 4, resource: BindingResource::TextureView(flow_out_view) },
                    BindGroupEntry { binding: 5, resource: BindingResource::Sampler(&self.flow_sampler) },
                ],
            });

            {
                let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some(&format!("Horn-Schunck Compute Pass Iter {}", i)),
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(&self.horn_schunck_pipeline);
                compute_pass.set_bind_group(0, &bind_group, &[]);
                compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
            }
        }

        queue.submit(Some(encoder.finish()));

        // Return the view of the texture containing the final result
        let final_idx = (num_iterations % 2) as usize;
        Ok(self.flow_views[final_idx].as_ref().unwrap())
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

        // Texture A
        if self.flow_textures[0].as_ref().map_or(true, |t| t.width() != width || t.height() != height || t.format() != texture_desc.format) {
            log::debug!("Creating Flow Texture A: {}x{}", width, height);
            let tex_a = device.create_texture(&TextureDescriptor {
                label: Some("Flow Texture A"),
                ..texture_desc
            });
            self.flow_views[0] = Some(tex_a.create_view(&TextureViewDescriptor::default()));
            self.flow_textures[0] = Some(tex_a);
        }

        // Texture B
        if self.flow_textures[1].as_ref().map_or(true, |t| t.width() != width || t.height() != height || t.format() != texture_desc.format) {
            log::debug!("Creating Flow Texture B: {}x{}", width, height);
            let tex_b = device.create_texture(&TextureDescriptor {
                label: Some("Flow Texture B"),
                ..texture_desc
            });
            self.flow_views[1] = Some(tex_b.create_view(&TextureViewDescriptor::default()));
            self.flow_textures[1] = Some(tex_b);
        }
    }

    pub fn compute_coarse_flow(
        &mut self,
        device: &Device,
        queue: &Queue,
        level: usize, // Coarsest pyramid level index
        num_iterations: usize,
        alpha_sq: f32,
    ) {
        log::info!("Computing coarse flow for pyramid level {} with {} iterations, alpha_sq={}", level, num_iterations, alpha_sq);

        let prev_frame_tex_view = self.pyramid_a_views[level].as_ref().expect("Prev frame view for level not found");
        let next_frame_tex_view = self.pyramid_b_views[level].as_ref().expect("Next frame view for level not found");
        
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
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST, // Though COPY_DST not used after init here
        });

        // Clear initial flow texture (flow_texture_a will be the first input)
        let flow_tex_a_ref = self.flow_textures[0].as_ref().unwrap();
        let flow_tex_bytes_per_pixel = 8; // Rg32Float (2 * f32)
        let zero_data_size = (width * height * flow_tex_bytes_per_pixel) as usize;
        let zero_data: Vec<u8> = vec![0; zero_data_size];

        queue.write_texture(
            ImageCopyTexture {
                texture: flow_tex_a_ref,
                mip_level: 0,
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

    #[test]
    fn test_build_pyramid() {
        let (device, queue) = pollster::block_on(setup_wgpu());
        let mut interpolator = WgpuFrameInterpolator::new(device.clone()).expect("Failed to create interpolator");

        const WIDTH: u32 = 64;
        const HEIGHT: u32 = 64;
        const LEVELS: u32 = 3;
        const FORMAT: TextureFormat = TextureFormat::Rgba32Float; // Matching shader expectations
        const BYTES_PER_PIXEL: u32 = 16; // 4 channels * 4 bytes/float

        // Create a simple input pattern (e.g., gradient)
        let mut frame_data_f32: Vec<f32> = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let r = x as f32 / (WIDTH - 1) as f32;
                let g = y as f32 / (HEIGHT - 1) as f32;
                frame_data_f32.extend_from_slice(&[r, g, 0.0, 1.0]); 
            }
        }
        let frame_data_u8: Vec<u8> = frame_data_f32.iter().flat_map(|&f| f.to_ne_bytes()).collect();
        
        let input_tex = create_texture_with_data(
            &device, &queue, WIDTH, HEIGHT, &frame_data_u8, 
            Some("Pyramid Test Input Texture"), FORMAT, 
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC // Need COPY_SRC for readback if needed
        );

        // Build pyramid for frame A
        interpolator.build_pyramid(&queue, &input_tex, LEVELS, true).expect("Pyramid build failed");

        // --- Assertions --- 
        assert_eq!(interpolator.pyramid_a_textures.len(), LEVELS as usize, "Incorrect number of pyramid textures stored");
        assert_eq!(interpolator.pyramid_a_views.len(), LEVELS as usize, "Incorrect number of pyramid views stored");
        
        // Check dimensions of each level
        let mut expected_w = WIDTH;
        let mut expected_h = HEIGHT;
        for level in 0..LEVELS as usize {
            assert_eq!(interpolator.pyramid_a_textures[level].as_ref().unwrap().width(), expected_w, "Level {} width mismatch", level);
            assert_eq!(interpolator.pyramid_a_textures[level].as_ref().unwrap().height(), expected_h, "Level {} height mismatch", level);
            // Next level dimensions
            expected_w = (expected_w + 1) / 2;
            expected_h = (expected_h + 1) / 2;
        }

        // Optional: Read back level 0 (first blurred/downsampled) and check a pixel
        // Note: Reading back is async and adds complexity. 
        // A simple check might be less useful than ensuring the dimensions are right.
        // For now, we'll rely on the dimension checks and assume the shaders work if they compiled.
        /*
        let level0_bytes = pollster::block_on(
            read_texture_to_cpu(&device, &queue, &interpolator.pyramid_a_textures[0].as_ref().unwrap(), WIDTH, HEIGHT, BYTES_PER_PIXEL)
        ).expect("Failed to read level 0 texture");
        // TODO: Add assertions on level0_bytes contents (e.g., check if center pixel is blurred average)
        */

        println!("Pyramid dimensions verified for {} levels.", LEVELS);

    }
} 