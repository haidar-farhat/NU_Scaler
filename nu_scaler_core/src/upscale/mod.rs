use anyhow::Result;
use wgpu::{Instance, Device, Queue, Adapter, Backends, DeviceDescriptor, Features, Limits, RequestAdapterOptions, ShaderModule, ComputePipeline, Buffer, BindGroup, BindGroupLayout, BufferUsages, ShaderModuleDescriptor, ShaderSource, ComputePipelineDescriptor, PipelineLayoutDescriptor, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, BindGroupDescriptor, BindGroupEntry, BindingResource, CommandEncoderDescriptor, BufferDescriptor, MapMode};

/// Upscaling quality levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpscalingQuality {
    Ultra,
    Quality,
    Balanced,
    Performance,
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

/// WGSL compute shader with dynamic dimensions via uniform buffer
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

/// GPU-accelerated upscaler using WGPU
pub struct WgpuUpscaler {
    quality: UpscalingQuality,
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
    bind_group: Option<BindGroup>,
    bind_group_layout: Option<BindGroupLayout>,
}

impl WgpuUpscaler {
    pub fn new(quality: UpscalingQuality) -> Self {
        Self {
            quality,
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
            bind_group: None,
            bind_group_layout: None,
        }
    }
}

impl Upscaler for WgpuUpscaler {
    fn initialize(&mut self, input_width: u32, input_height: u32, output_width: u32, output_height: u32) -> Result<()> {
        self.input_width = input_width;
        self.input_height = input_height;
        self.output_width = output_width;
        self.output_height = output_height;

        // WGPU setup
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })).ok_or_else(|| anyhow::anyhow!("No suitable GPU adapter found"))?;
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("WgpuUpscaler Device"),
                required_features: Features::empty(),
                required_limits: Limits::default(),
            },
            None,
        ))?;

        // Create shader module
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("NN Upscale Shader"),
            source: ShaderSource::Wgsl(NN_UPSCALE_SHADER.into()),
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

        // Create bind group
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Upscale Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: input_buffer.as_entire_binding(),
            }, BindGroupEntry {
                binding: 1,
                resource: output_buffer.as_entire_binding(),
            }, BindGroupEntry {
                binding: 2,
                resource: dimensions_buffer.as_entire_binding(),
            }],
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

        self.instance = Some(instance);
        self.device = Some(device);
        self.queue = Some(queue);
        self.shader = Some(shader);
        self.pipeline = Some(pipeline);
        self.input_buffer = Some(input_buffer);
        self.output_buffer = Some(output_buffer);
        self.dimensions_buffer = Some(dimensions_buffer);
        self.bind_group = Some(bind_group);
        self.bind_group_layout = Some(bind_group_layout);
        self.initialized = true;
        Ok(())
    }
    fn upscale(&self, input: &[u8]) -> Result<Vec<u8>> {
        if !self.initialized {
            anyhow::bail!("WgpuUpscaler not initialized");
        }
        let device = self.device.as_ref().ok_or_else(|| anyhow::anyhow!("No device"))?;
        let queue = self.queue.as_ref().ok_or_else(|| anyhow::anyhow!("No queue"))?;
        let pipeline = self.pipeline.as_ref().ok_or_else(|| anyhow::anyhow!("No pipeline"))?;
        let bind_group = self.bind_group.as_ref().ok_or_else(|| anyhow::anyhow!("No bind group"))?;
        let input_buffer = self.input_buffer.as_ref().ok_or_else(|| anyhow::anyhow!("No input buffer"))?;
        let output_buffer = self.output_buffer.as_ref().ok_or_else(|| anyhow::anyhow!("No output buffer"))?;
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

        // Encode and dispatch compute shader
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Upscale Encoder"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Upscale Compute Pass"),
            });
            cpass.set_pipeline(pipeline);
            cpass.set_bind_group(0, bind_group, &[]);
            let x_groups = (self.output_width + 7) / 8;
            let y_groups = (self.output_height + 7) / 8;
            cpass.dispatch_workgroups(x_groups, y_groups, 1);
        }
        queue.submit(Some(encoder.finish()));

        // Download result from GPU
        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(MapMode::Read, move |v| sender.send(v).unwrap());
        device.poll(wgpu::Maintain::Wait);
        receiver.recv().unwrap().map_err(|_| anyhow::anyhow!("Failed to map output buffer"))?;
        let data = buffer_slice.get_mapped_range().to_vec();
        output_buffer.unmap();
        Ok(data)
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
        let mut up = WgpuUpscaler::new(UpscalingQuality::Quality);
        assert!(!up.initialized);
        up.initialize(640, 480, 1280, 960).unwrap();
        assert!(up.initialized);
        assert_eq!(up.input_width, 640);
        assert_eq!(up.output_width, 1280);
    }
} 