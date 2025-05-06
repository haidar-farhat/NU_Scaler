use anyhow::Result;
use wgpu::{Instance, Device, Queue, /*Adapter,*/ Backends, DeviceDescriptor, /*Features,*/ Limits, RequestAdapterOptions, ShaderModule, ComputePipeline, Buffer, BindGroup, BindGroupLayout, BufferUsages, ShaderModuleDescriptor, ShaderSource, ComputePipelineDescriptor, PipelineLayoutDescriptor, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, BindGroupDescriptor, BindGroupEntry, /*BindingResource,*/ CommandEncoderDescriptor, BufferDescriptor, MapMode};
use wgpu::util::DeviceExt;
use rayon::prelude::*;
use std::time::Instant;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Add new module declarations
mod fsr;
mod dlss;

// Re-export the new implementations
pub use fsr::FsrUpscaler;
pub use dlss::DlssUpscaler;

/// Upscaling quality levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpscalingQuality {
    Ultra,
    Quality,
    Balanced,
    Performance,
}

/// Supported upscaling algorithms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpscaleAlgorithm {
    Nearest,
    Bilinear,
}

/// Supported upscaling technologies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpscalingTechnology {
    None,
    FSR,
    DLSS,
    Wgpu,
    Fallback,
}

/// Trait for upscaling algorithms
pub trait Upscaler {
    /// Initialize the upscaler
    fn initialize(&mut self, input_width: u32, input_height: u32, output_width: u32, output_height: u32) -> Result<()>;
    /// Upscale a single frame (raw bytes or image)
    fn upscale(&self, input: &[u8]) -> Result<Vec<u8>>;
    /// Get the name of this upscaler
    fn name(&self) -> &'static str;
    /// Get the quality level
    fn quality(&self) -> UpscalingQuality;
    /// Set the quality level
    fn set_quality(&mut self, quality: UpscalingQuality) -> Result<()>;
}

/// Factory for creating upscalers based on technology detection
pub struct UpscalerFactory;

impl UpscalerFactory {
    /// Create the most appropriate upscaler based on the detected technology
    pub fn create_upscaler(technology: UpscalingTechnology, quality: UpscalingQuality) -> Box<dyn Upscaler> {
        match technology {
            UpscalingTechnology::FSR => Box::new(FsrUpscaler::new(quality)),
            UpscalingTechnology::DLSS => Box::new(DlssUpscaler::new(quality)),
            UpscalingTechnology::Wgpu => Box::new(WgpuUpscaler::new(quality, UpscaleAlgorithm::Bilinear)),
            _ => Box::new(WgpuUpscaler::new(quality, UpscaleAlgorithm::Nearest)),
        }
    }
    
    /// Share device and queue with all upscalers
    pub fn set_shared_resources(upscaler: &mut Box<dyn Upscaler>, device: Arc<Device>, queue: Arc<Queue>) -> Result<()> {
        // Cast to specific types to share resources
        if let Some(fsr) = upscaler.as_mut().downcast_mut::<FsrUpscaler>() {
            fsr.set_device_queue(device, queue);
        } else if let Some(dlss) = upscaler.as_mut().downcast_mut::<DlssUpscaler>() {
            dlss.set_device_queue(device, queue);
        }
        
        Ok(())
    }
}

/// Mock implementation for testing
pub struct MockUpscaler;

impl Upscaler for MockUpscaler {
    fn initialize(&mut self, _input_width: u32, _input_height: u32, _output_width: u32, _output_height: u32) -> Result<()> {
        unimplemented!()
    }
    fn upscale(&self, _input: &[u8]) -> Result<Vec<u8>> {
        unimplemented!()
    }
    fn name(&self) -> &'static str {
        "MockUpscaler"
    }
    fn quality(&self) -> UpscalingQuality {
        UpscalingQuality::Quality
    }
    fn set_quality(&mut self, _quality: UpscalingQuality) -> Result<()> {
        unimplemented!()
    }
}

/// WGSL compute shader with dynamic dimensions via uniform buffer (Nearest Neighbor)
const NN_UPSCALE_SHADER: &str = r#"
struct Dimensions {
    in_width: u32,
    in_height: u32,
    out_width: u32,
    out_height: u32,
}
@group(0) @binding(0) var<storage, read> input_img: array<u32>;
@group(0) @binding(1) var<storage, read_write> output_img: array<u32>;
@group(0) @binding(2) var<uniform> dims: Dimensions;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= dims.out_width || gid.y >= dims.out_height) {
        return;
    }
    let src_x = (gid.x * dims.in_width) / dims.out_width;
    let src_y = (gid.y * dims.in_height) / dims.out_height;
    let src_idx = src_y * dims.in_width + src_x;
    let dst_idx = gid.y * dims.out_width + gid.x;
    output_img[dst_idx] = input_img[src_idx];
}
"#;

/// WGSL compute shader for bilinear upscaling (RGBA8, dynamic dimensions)
const BILINEAR_UPSCALE_SHADER: &str = r#"
struct Dimensions {
    in_width: u32,
    in_height: u32,
    out_width: u32,
    out_height: u32,
}
@group(0) @binding(0) var<storage, read> input_img: array<u32>;
@group(0) @binding(1) var<storage, read_write> output_img: array<u32>;
@group(0) @binding(2) var<uniform> dims: Dimensions;

fn unpack_rgba8(p: u32) -> vec4<f32> {
    return vec4<f32>(
        f32((p >> 0) & 0xFF),
        f32((p >> 8) & 0xFF),
        f32((p >> 16) & 0xFF),
        f32((p >> 24) & 0xFF)
    ) / 255.0;
}
fn pack_rgba8(v: vec4<f32>) -> u32 {
    let r = u32(clamp(v.x, 0.0, 1.0) * 255.0);
    let g = u32(clamp(v.y, 0.0, 1.0) * 255.0);
    let b = u32(clamp(v.z, 0.0, 1.0) * 255.0);
    let a = u32(clamp(v.w, 0.0, 1.0) * 255.0);
    return (a << 24) | (b << 16) | (g << 8) | r;
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= dims.out_width || gid.y >= dims.out_height) {
        return;
    }
    let fx = f32(gid.x) * f32(dims.in_width) / f32(dims.out_width);
    let fy = f32(gid.y) * f32(dims.in_height) / f32(dims.out_height);
    let x0 = u32(fx);
    let y0 = u32(fy);
    let x1 = min(x0 + 1, dims.in_width - 1);
    let y1 = min(y0 + 1, dims.in_height - 1);
    let dx = fx - f32(x0);
    let dy = fy - f32(y0);
    let idx00 = y0 * dims.in_width + x0;
    let idx10 = y0 * dims.in_width + x1;
    let idx01 = y1 * dims.in_width + x0;
    let idx11 = y1 * dims.in_width + x1;
    let c00 = unpack_rgba8(input_img[idx00]);
    let c10 = unpack_rgba8(input_img[idx10]);
    let c01 = unpack_rgba8(input_img[idx01]);
    let c11 = unpack_rgba8(input_img[idx11]);
    let c0 = mix(c00, c10, dx);
    let c1 = mix(c01, c11, dx);
    let c = mix(c0, c1, dy);
    let dst_idx = gid.y * dims.out_width + gid.x;
    output_img[dst_idx] = pack_rgba8(c);
}
"#;

/// GPU-accelerated upscaler using WGPU
pub struct WgpuUpscaler {
    quality: UpscalingQuality,
    algorithm: UpscaleAlgorithm,
    input_width: u32,
    input_height: u32,
    output_width: u32,
    output_height: u32,
    initialized: bool,
    // WGPU fields
    instance: Option<Instance>,
    device: Option<Device>,
    queue: Option<Queue>,
    shader: Option<ShaderModule>,
    pipeline: Option<ComputePipeline>,
    input_buffer: Option<Buffer>,
    output_buffer: Option<Buffer>,
    dimensions_buffer: Option<Buffer>,
    bind_group_layout: Option<BindGroupLayout>,
    buffer_pool: Vec<Buffer>,
    buffer_pool_index: AtomicUsize,
    buffer_pool_bind_groups: Vec<BindGroup>,
    fallback_bind_group: Option<BindGroup>,
    staging_buffer: Option<Buffer>,
    // Advanced settings
    thread_count: u32,
    buffer_pool_size: u32,
    gpu_allocator: String,
    shader_path: String,
}

impl WgpuUpscaler {
    pub fn new(quality: UpscalingQuality, algorithm: UpscaleAlgorithm) -> Self {
        Self {
            quality,
            algorithm,
            input_width: 0,
            input_height: 0,
            output_width: 0,
            output_height: 0,
            initialized: false,
            instance: None,
            device: None,
            queue: None,
            shader: None,
            pipeline: None,
            input_buffer: None,
            output_buffer: None,
            dimensions_buffer: None,
            bind_group_layout: None,
            buffer_pool: Vec::new(),
            buffer_pool_index: AtomicUsize::new(0),
            buffer_pool_bind_groups: Vec::new(),
            fallback_bind_group: None,
            staging_buffer: None,
            thread_count: 4,
            buffer_pool_size: 4,
            gpu_allocator: "Default".to_string(),
            shader_path: "".to_string(),
        }
    }
    pub fn set_thread_count(&mut self, n: u32) {
        self.thread_count = n;
        println!("[WgpuUpscaler] Set thread count: {}", n);
        // Configure Rayon thread pool if needed
        if n > 1 {
            let _ = rayon::ThreadPoolBuilder::new().num_threads(n as usize).build_global();
        }
    }
    pub fn set_buffer_pool_size(&mut self, n: u32) {
        self.buffer_pool_size = n;
        println!("[WgpuUpscaler] Set buffer pool size: {}", n);

        self.buffer_pool.clear();
        self.buffer_pool_bind_groups.clear();

        if let (Some(device), Some(layout), Some(input_buf), Some(dims_buf)) = (
            self.device.as_ref(),
            self.bind_group_layout.as_ref(),
            self.input_buffer.as_ref(),
            self.dimensions_buffer.as_ref(),
        ) {
            let buffer_size = (self.output_width * self.output_height * 4) as u64;
            if buffer_size == 0 { return; } // Avoid creating zero-sized buffers

            for i in 0..n {
                let output_buf = device.create_buffer(&BufferDescriptor {
                    label: Some(&format!("Output Buffer (Pool {})", i)),
                    size: buffer_size,
                    usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });
                let bind_group = device.create_bind_group(&BindGroupDescriptor {
                    label: Some(&format!("Upscale Bind Group (Pool {})", i)),
                    layout: layout,
                    entries: &[BindGroupEntry {
                        binding: 0,
                        resource: input_buf.as_entire_binding(),
                    }, BindGroupEntry {
                        binding: 1,
                        resource: output_buf.as_entire_binding(),
                    }, BindGroupEntry {
                        binding: 2,
                        resource: dims_buf.as_entire_binding(),
                    }],
                });
                self.buffer_pool.push(output_buf);
                self.buffer_pool_bind_groups.push(bind_group);
            }
        }
        self.buffer_pool_index.store(0, Ordering::SeqCst);
    }
    pub fn set_gpu_allocator(&mut self, preset: &str) {
        self.gpu_allocator = preset.to_string();
        match preset {
            "Aggressive" => {
                // Pre-allocate a large pool, never shrink
                let n = self.buffer_pool_size.max(8);
                self.buffer_pool.clear();
                if let Some(device) = self.device.as_ref() {
                    for _ in 0..n {
                        let buf = device.create_buffer(&BufferDescriptor {
                            label: Some("Output Buffer (Aggressive)"),
                            size: (self.output_width * self.output_height * 4 * 2) as u64,
                            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                            mapped_at_creation: false,
                        });
                        self.buffer_pool.push(buf);
                    }
                }
                println!("[WgpuUpscaler] Aggressive allocator: pre-allocated {} large buffers", n);
            }
            "Conservative" => {
                // Free all buffers, allocate per use
                self.buffer_pool.clear();
                println!("[WgpuUpscaler] Conservative allocator: will allocate/free per use");
            }
            _ => {
                // Default: as before
                self.set_buffer_pool_size(self.buffer_pool_size);
                println!("[WgpuUpscaler] Default allocator: buffer pool size {}", self.buffer_pool_size);
            }
        }
    }
    pub fn reload_shader(&mut self, path: &str) -> anyhow::Result<()> {
        use std::fs;
        let code = fs::read_to_string(path)?;
        self.shader_path = path.to_string();
        if let (Some(device), Some(bind_group_layout)) = (self.device.as_ref(), self.bind_group_layout.as_ref()) {
            let shader = device.create_shader_module(ShaderModuleDescriptor {
                label: Some("Upscale Shader (Reloaded)"),
                source: ShaderSource::Wgsl(code.into()),
            });
            let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Upscale Pipeline Layout (Reloaded)"),
                bind_group_layouts: &[bind_group_layout],
                push_constant_ranges: &[],
            });
            let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
                label: Some("Upscale Pipeline (Reloaded)"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "main",
            });
            self.shader = Some(shader);
            self.pipeline = Some(pipeline);
            println!("[WgpuUpscaler] Shader reloaded from {}", path);
        } else {
            println!("[WgpuUpscaler] Cannot reload shader: device or bind_group_layout missing");
        }
        Ok(())
    }
    pub fn upscale_batch(&self, frames: &[&[u8]]) -> Result<Vec<Vec<u8>>> {
        let start = Instant::now();
        let results: Vec<_> = if self.thread_count > 1 {
            frames.par_iter().enumerate().map(|(i, frame)| {
                let t0 = Instant::now();
                let out = self.upscale(frame);
                let t1 = Instant::now();
                println!("[Batch {}] Frame time: {:.2} ms", i, (t1 - t0).as_secs_f64() * 1000.0);
                out
            }).collect()
        } else {
            frames.iter().enumerate().map(|(i, frame)| {
                let t0 = Instant::now();
                let out = self.upscale(frame);
                let t1 = Instant::now();
                println!("[Batch {}] Frame time: {:.2} ms", i, (t1 - t0).as_secs_f64() * 1000.0);
                out
            }).collect()
        };
        let total = Instant::now() - start;
        println!("[Batch] Total time: {:.2} ms for {} frames", total.as_secs_f64() * 1000.0, frames.len());
        results.into_iter().collect()
    }
}

impl Upscaler for WgpuUpscaler {
    fn initialize(&mut self, input_width: u32, input_height: u32, output_width: u32, output_height: u32) -> Result<()> {
        if self.initialized &&
           self.input_width == input_width &&
           self.input_height == input_height &&
           self.output_width == output_width &&
           self.output_height == output_height {
            println!("[WgpuUpscaler] Already initialized with same dimensions.");
            return Ok(());
        }

        println!("[WgpuUpscaler] Initializing...");
        self.input_width = input_width;
        self.input_height = input_height;
        self.output_width = output_width;
        self.output_height = output_height;

        // Reset fields
        self.buffer_pool.clear();
        self.buffer_pool_bind_groups.clear();
        self.buffer_pool_index = AtomicUsize::new(0);

        // Create WGPU instance if not exists
        let instance = self.instance.get_or_insert_with(|| {
            println!("[WgpuUpscaler] Creating WGPU instance (Backends: {:?})", Backends::PRIMARY);
            Instance::new(wgpu::InstanceDescriptor {
                backends: Backends::PRIMARY, // Request primary backends (Vulkan/Metal/DX12)
                ..Default::default()
            })
        });

        // Request adapter
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default()))
            .ok_or_else(|| anyhow::anyhow!("Failed to find suitable adapter"))?;

        // <<< ADD LOGGING HERE >>>
        let adapter_info = adapter.get_info();
        println!("[WgpuUpscaler] Selected Adapter: {} ({:?}, Backend: {:?})", adapter_info.name, adapter_info.device_type, adapter_info.backend);

        // Request device and queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("Upscaler Device"),
                required_features: wgpu::Features::empty(),
                required_limits: Limits::default(),
            },
            None,
        ))?;

        // Select shader source
        let shader_src = match self.algorithm {
            UpscaleAlgorithm::Nearest => NN_UPSCALE_SHADER,
            UpscaleAlgorithm::Bilinear => BILINEAR_UPSCALE_SHADER,
        };
        // Create shader module
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Upscale Shader"),
            source: ShaderSource::Wgsl(shader_src.into()),
        });

        // Create buffers
        let input_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Input Buffer"),
            size: (input_width * input_height * 4) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let output_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Output Buffer"),
            size: (output_width * output_height * 4) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        // Create dimensions buffer (uniform)
        let dims = [input_width, input_height, output_width, output_height];
        let dimensions_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dimensions Buffer"),
            contents: bytemuck::cast_slice(&dims),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Upscale Bind Group Layout"),
            entries: &[ // input_img
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // output_img
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // dimensions
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create pipeline
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Upscale Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Upscale Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        // Create Staging Buffer
        let staging_buffer_size = (output_width * output_height * 4) as u64;
        let staging_buffer = if staging_buffer_size > 0 {
            Some(device.create_buffer(&BufferDescriptor {
                label: Some("Staging Buffer"),
                size: staging_buffer_size,
                usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }))
        } else {
            None
        };

        self.device = Some(device);
        self.queue = Some(queue);
        self.shader = Some(shader);
        self.pipeline = Some(pipeline);
        self.input_buffer = Some(input_buffer);
        self.output_buffer = Some(output_buffer);
        self.dimensions_buffer = Some(dimensions_buffer);
        self.bind_group_layout = Some(bind_group_layout);
        self.staging_buffer = staging_buffer;

        // Create fallback bind group using the original output buffer
        if let (Some(dev), Some(layout), Some(in_buf), Some(out_buf), Some(dims_buf)) = (
            self.device.as_ref(), self.bind_group_layout.as_ref(),
            self.input_buffer.as_ref(), self.output_buffer.as_ref(), self.dimensions_buffer.as_ref()
        ) {
            self.fallback_bind_group = Some(dev.create_bind_group(&BindGroupDescriptor {
                label: Some("Upscale Bind Group (Fallback)"),
                layout: layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: in_buf.as_entire_binding(),
                }, BindGroupEntry {
                    binding: 1,
                    resource: out_buf.as_entire_binding(),
                }, BindGroupEntry {
                    binding: 2,
                    resource: dims_buf.as_entire_binding(),
                }],
            }));
        }

        // Initialize buffer pool (this will create pooled buffers and their bind groups)
        self.set_buffer_pool_size(self.buffer_pool_size);
        self.buffer_pool_index.store(0, Ordering::SeqCst);
        self.initialized = true;
        Ok(())
    }
    fn upscale(&self, input: &[u8]) -> Result<Vec<u8>> {
        if !self.initialized {
            anyhow::bail!("WgpuUpscaler not initialized");
        }
        let staging_buffer = self.staging_buffer.as_ref().ok_or_else(|| anyhow::anyhow!("No staging buffer"))?;

        // Select output buffer and bind group from pool
        let buffer_index = self.buffer_pool_index.fetch_add(1, Ordering::SeqCst);
        let (output_buffer, bind_group) = if !self.buffer_pool.is_empty() && !self.buffer_pool_bind_groups.is_empty() {
            let idx = buffer_index % self.buffer_pool.len();
            let bg_idx = idx.min(self.buffer_pool_bind_groups.len() - 1);
            (&self.buffer_pool[idx], &self.buffer_pool_bind_groups[bg_idx])
        } else {
            (
                self.output_buffer.as_ref().ok_or_else(|| anyhow::anyhow!("No fallback output buffer"))?,
                self.fallback_bind_group.as_ref().ok_or_else(|| anyhow::anyhow!("No fallback bind group"))?
            )
        };

        let device = self.device.as_ref().ok_or_else(|| anyhow::anyhow!("No device"))?;
        let queue = self.queue.as_ref().ok_or_else(|| anyhow::anyhow!("No queue"))?;
        let pipeline = self.pipeline.as_ref().ok_or_else(|| anyhow::anyhow!("No pipeline"))?;
        let input_buffer = self.input_buffer.as_ref().ok_or_else(|| anyhow::anyhow!("No input buffer"))?;
        let dimensions_buffer = self.dimensions_buffer.as_ref().ok_or_else(|| anyhow::anyhow!("No dimensions buffer"))?;

        // Error handling: check input size
        let expected = (self.input_width * self.input_height * 4) as usize;
        if input.len() != expected {
            anyhow::bail!("Input buffer size mismatch: expected {} got {}", expected, input.len());
        }

        // Update dimensions buffer if needed
        let dims = [self.input_width, self.input_height, self.output_width, self.output_height];
        queue.write_buffer(dimensions_buffer, 0, bytemuck::cast_slice(&dims));

        // Upload input to GPU
        queue.write_buffer(input_buffer, 0, input);

        // Encode compute pass and copy to staging buffer
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Upscale Encoder"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Upscale Compute Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(pipeline);
            cpass.set_bind_group(0, bind_group, &[]);
            let x_groups = (self.output_width + 7) / 8;
            let y_groups = (self.output_height + 7) / 8;
            cpass.dispatch_workgroups(x_groups, y_groups, 1);
        }
        // Copy result from output buffer (pool) to staging buffer
        encoder.copy_buffer_to_buffer(
            output_buffer, 0,
            staging_buffer, 0,
            staging_buffer.size()
        );
        queue.submit(Some(encoder.finish()));

        // Map the staging buffer to read results
        let mapped_data: Result<Vec<u8>, anyhow::Error> = {
            let buffer_slice = staging_buffer.slice(..);
            let (sender, receiver) = std::sync::mpsc::channel();
            buffer_slice.map_async(MapMode::Read, move |v| sender.send(v).unwrap());
            device.poll(wgpu::Maintain::Wait);
            match receiver.recv() {
                Ok(Ok(())) => {
                    let data = buffer_slice.get_mapped_range().to_vec();
                    drop(buffer_slice.get_mapped_range());
                    staging_buffer.unmap(); // Unmap the staging buffer
                    Ok(data)
                }
                Ok(Err(e)) => Err(anyhow::anyhow!("Buffer map callback failed: {:?}", e)),
                Err(e) => Err(anyhow::anyhow!("Failed to receive buffer map result: {:?}", e)),
            }
        };
        mapped_data
    }
    fn name(&self) -> &'static str {
        "WgpuUpscaler"
    }
    fn quality(&self) -> UpscalingQuality {
        self.quality
    }
    fn set_quality(&mut self, quality: UpscalingQuality) -> Result<()> {
        self.quality = quality;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_initialize_panics() {
        let mut up = MockUpscaler;
        let _ = up.initialize(1, 1, 2, 2).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_upscale_panics() {
        let up = MockUpscaler;
        let _ = up.upscale(&[0u8; 4]).unwrap();
    }

    #[test]
    fn test_name_and_quality() {
        let up = MockUpscaler;
        assert_eq!(up.name(), "MockUpscaler");
        assert_eq!(up.quality(), UpscalingQuality::Quality);
    }

    #[test]
    #[should_panic]
    fn test_set_quality_panics() {
        let mut up = MockUpscaler;
        let _ = up.set_quality(UpscalingQuality::Ultra).unwrap();
    }

    #[test]
    fn test_wgpu_upscaler_init() {
        let mut up = WgpuUpscaler::new(UpscalingQuality::Quality, UpscaleAlgorithm::Nearest);
        assert!(!up.initialized);
        up.initialize(640, 480, 1280, 960).unwrap();
        assert!(up.initialized);
        assert_eq!(up.input_width, 640);
        assert_eq!(up.output_width, 1280);
    }
} 