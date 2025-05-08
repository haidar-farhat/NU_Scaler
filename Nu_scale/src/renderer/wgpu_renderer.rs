use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use anyhow::Result;
use wgpu::{ShaderModule, BindGroup, BindGroupLayout, Buffer, Texture, TextureView, Sampler, Adapter, Backends, Device, DeviceDescriptor, Features, Instance, InstanceFlags, Limits, PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureFormat, TextureUsages};
use image::RgbaImage;
use winit::window::Window;

const STATS_WINDOW_SIZE: usize = 120;

/// Performance statistics for the renderer
pub struct RenderStats {
    frame_times: [f32; STATS_WINDOW_SIZE],
    current_index: usize,
    total_frames: u64,
    last_frame_time: Instant,
}

impl RenderStats {
    pub fn new() -> Self {
        Self {
            frame_times: [0.0; STATS_WINDOW_SIZE],
            current_index: 0,
            total_frames: 0,
            last_frame_time: Instant::now(),
        }
    }

    pub fn add_frame(&mut self) {
        let elapsed = self.last_frame_time.elapsed().as_secs_f32() * 1000.0;
        self.frame_times[self.current_index] = elapsed;
        self.current_index = (self.current_index + 1) % STATS_WINDOW_SIZE;
        self.total_frames += 1;
        self.last_frame_time = Instant::now();
    }

    pub fn average_fps(&self) -> f32 {
        let sum: f32 = self.frame_times.iter().sum();
        if sum <= 0.0 { 0.0 } else { 1000.0 / (sum / STATS_WINDOW_SIZE as f32) }
    }

    pub fn average_frame_time(&self) -> f32 {
        let sum: f32 = self.frame_times.iter().sum();
        if sum <= 0.0 { 0.0 } else { sum / STATS_WINDOW_SIZE as f32 }
    }

    pub fn total_frames(&self) -> u64 {
        self.total_frames
    }
}

/// WGPU render resources for direct rendering
pub struct WgpuRenderResources {
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    vertex_buffer: wgpu::Buffer,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    texture_size: (u32, u32),
}

/// Fullscreen shader module
const FULLSCREEN_SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vert_idx: u32) -> @builtin(position) vec4<f32> {
    let pos = array(
        vec2(-1.0, -1.0),
        vec2(3.0, -1.0),
        vec2(-1.0, 3.0),
    );
    return vec4(pos[vert_idx], 0.0, 1.0);
}

@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0) @binding(1)
var samp: sampler;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, pos.xy / vec2(1920.0, 1080.0));
}
"#;

/// Triple buffer implementation for frame data
pub struct TripleBuffer {
    buffers: [Arc<Mutex<Option<RgbaImage>>>; 3],
    current_index: AtomicUsize,
}

impl TripleBuffer {
    pub fn new() -> Self {
        Self {
            buffers: [
                Arc::new(Mutex::new(None)),
                Arc::new(Mutex::new(None)),
                Arc::new(Mutex::new(None)),
            ],
            current_index: AtomicUsize::new(0),
        }
    }

    pub fn write(&self, frame: RgbaImage) {
        let next_index = (self.current_index.load(Ordering::Acquire) + 1) % 3;
        if let Ok(mut buffer) = self.buffers[next_index].lock() {
            *buffer = Some(frame);
            self.current_index.store(next_index, Ordering::Release);
        }
    }

    pub fn read(&self) -> Option<RgbaImage> {
        let index = self.current_index.load(Ordering::Acquire);
        if let Ok(buffer) = self.buffers[index].lock() {
            buffer.clone()
        } else {
            None
        }
    }
}

/// WGPU renderer implementation
pub struct WgpuRenderer {
    instance: Instance,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_resources: Option<WgpuRenderResources>,
    stats: RenderStats,
    vsync: bool,
}

impl WgpuRenderer {
    pub async fn new(window: &Window) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            flags: InstanceFlags::default(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface = unsafe { instance.create_surface(window) }?;
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to find an appropriate adapter")?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok(Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
            size,
            render_resources: None,
            stats: RenderStats::new(),
            vsync: true,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn create_render_resources(&mut self, width: u32, height: u32) -> Result<()> {
        // Create shader module
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fullscreen Shader"),
            source: wgpu::ShaderSource::Wgsl(FULLSCREEN_SHADER.into()),
        });

        // Create texture
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Upscaled Frame Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Frame Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create bind group layout
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Frame Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Frame Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Create render pipeline
        let render_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fullscreen Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fullscreen Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create vertex buffer (empty since we use vertex shader to generate vertices)
        let vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: 0,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        self.render_resources = Some(WgpuRenderResources {
            render_pipeline,
            bind_group,
            bind_group_layout,
            vertex_buffer,
            texture,
            texture_view,
            sampler,
            texture_size: (width, height),
        });

        Ok(())
    }

    pub fn update_texture(&mut self, frame: &RgbaImage) -> Result<()> {
        if let Some(resources) = &mut self.render_resources {
            let (width, height) = frame.dimensions();
            
            // Check if we need to resize the texture
            if resources.texture_size != (width, height) {
                self.create_render_resources(width, height)?;
            }

            // Update texture data
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &resources.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                frame.as_raw(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }

        Ok(())
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if let Some(resources) = &self.render_resources {
                render_pass.set_pipeline(&resources.render_pipeline);
                render_pass.set_bind_group(0, &resources.bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Update performance stats
        self.stats.add_frame();

        Ok(())
    }

    pub fn stats(&self) -> &RenderStats {
        &self.stats
    }

    pub fn set_vsync(&mut self, vsync: bool) {
        self.vsync = vsync;
        self.config.present_mode = if vsync {
            wgpu::PresentMode::Fifo
        } else {
            wgpu::PresentMode::Immediate
        };
        self.surface.configure(&self.device, &self.config);
    }
} 