use anyhow::Result;
use wgpu::{Device, Queue, ShaderModule, ComputePipeline, Buffer, BindGroup, BindGroupLayout, BufferUsages, ShaderModuleDescriptor, ShaderSource, ComputePipelineDescriptor, PipelineLayoutDescriptor, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, BindGroupDescriptor, BindGroupEntry, CommandEncoderDescriptor, BufferDescriptor, MapMode};
use wgpu::util::DeviceExt;
use std::sync::Arc;

use super::{Upscaler, UpscalingQuality, UpscalingTechnology};

// FSR 1.0 EASU (Edge Adaptive Spatial Upsampling) shader
// Simplified implementation of AMD FSR algorithm
const FSR_EASU_SHADER: &str = r#"
// FSR 1.0 Edge Adaptive Spatial Upsampling shader
// Based on AMD's FidelityFX Super Resolution 1.0
// Reference: https://github.com/GPUOpen-Effects/FidelityFX-FSR

// Input dimensions and constants
struct Dimensions {
    in_width: u32,
    in_height: u32,
    out_width: u32,
    out_height: u32,
    sharpness: f32,   // Quality-dependent sharpness factor
    reserved1: f32,
    reserved2: f32,
    reserved3: f32,
}

@group(0) @binding(0) var<storage, read> input_img: array<u32>;
@group(0) @binding(1) var<storage, read_write> output_img: array<u32>;
@group(0) @binding(2) var<uniform> dims: Dimensions;

// Helper functions for color handling
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

// Fetch texel from input with clamping
fn FsrEasuF(p: vec2<i32>) -> vec3<f32> {
    // Clamp to valid coordinates
    let px = clamp(p.x, 0, i32(dims.in_width) - 1);
    let py = clamp(p.y, 0, i32(dims.in_height) - 1);
    let idx = py * i32(dims.in_width) + px;
    let rgba = unpack_rgba8(input_img[idx]);
    return rgba.rgb;
}

// Cubic filter 
fn FsrCubic(d: f32) -> f32 {
    let d2 = d * d;
    let d3 = d * d2;
    if (d <= 1.0) {
        return (2.0 - 1.5 * d - 0.5 * d3 + d2);
    } else if (d <= 2.0) {
        return (0.0 - 0.5 * d + 2.5 * d2 - d3);
    } else {
        return 0.0;
    }
}

// Edge detection and direction calculation
fn FsrDirA(p: vec2<i32>) -> vec2<f32> {
    // Sample nearby pixels
    let up = FsrEasuF(p + vec2<i32>(0, -1));
    let dn = FsrEasuF(p + vec2<i32>(0, 1));
    let lf = FsrEasuF(p + vec2<i32>(-1, 0));
    let rt = FsrEasuF(p + vec2<i32>(1, 0));
    
    // Compute gradients
    let vgx = (abs(up.r - dn.r) + abs(up.g - dn.g) + abs(up.b - dn.b)) / 3.0;
    let vgy = (abs(lf.r - rt.r) + abs(lf.g - rt.g) + abs(lf.b - rt.b)) / 3.0;
    
    // Determine direction
    let dir = vec2<f32>(vgx, vgy);
    return normalize(dir + vec2<f32>(0.0001, 0.0001));
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= dims.out_width || gid.y >= dims.out_height) {
        return;
    }
    
    // Normalized pixel coordinates in output
    let outCoord = vec2<f32>(f32(gid.x) + 0.5, f32(gid.y) + 0.5);
    
    // Map to input coordinates
    let inCoord = outCoord * vec2<f32>(f32(dims.in_width) / f32(dims.out_width), 
                                       f32(dims.in_height) / f32(dims.out_height));
    
    // Integer and fractional parts
    let basePos = vec2<i32>(i32(inCoord.x) - 1, i32(inCoord.y) - 1);
    let fract = fract(inCoord);
    
    // Edge detection
    let dir = FsrDirA(vec2<i32>(i32(inCoord.x), i32(inCoord.y)));
    
    // Fetch a 4x4 neighborhood 
    var colors: array<array<vec3<f32>, 4>, 4>;
    for (var y = 0; y < 4; y++) {
        for (var x = 0; x < 4; x++) {
            colors[y][x] = FsrEasuF(basePos + vec2<i32>(x, y));
        }
    }
    
    // Directional weights
    let wx = abs(dir.x) / (abs(dir.x) + abs(dir.y));
    let wy = 1.0 - wx;
    
    // Apply cubic filter along detected edge direction
    var sumColor = vec3<f32>(0.0);
    var sumWeight = 0.0;
    
    for (var y = 0; y < 4; y++) {
        for (var x = 0; x < 4; x++) {
            // Sample position relative to current pixel
            let samplePos = vec2<f32>(f32(x) - fract.x, f32(y) - fract.y);
            
            // Project distance along direction
            let dist = abs(samplePos.x * wx + samplePos.y * wy);
            
            // Apply cubic filter
            let weight = FsrCubic(dist);
            sumColor += colors[y][x] * weight;
            sumWeight += weight;
        }
    }
    
    // Normalize and apply sharpness
    var color = sumColor / max(sumWeight, 0.0001);
    
    // Apply sharpening based on quality setting
    if (dims.sharpness > 0.001) {
        let center = FsrEasuF(vec2<i32>(i32(inCoord.x), i32(inCoord.y)));
        color = mix(color, center, dims.sharpness);
    }
    
    // Write output
    let dst_idx = gid.y * dims.out_width + gid.x;
    output_img[dst_idx] = pack_rgba8(vec4<f32>(color, 1.0));
}
"#;

// FSR 1.0 RCAS (Robust Contrast Adaptive Sharpening) shader for post-processing
// This applies sharpening based on local contrast
const FSR_RCAS_SHADER: &str = r#"
// FSR 1.0 Robust Contrast Adaptive Sharpening shader
// Based on AMD's FidelityFX Super Resolution 1.0

struct Dimensions {
    width: u32,
    height: u32,
    sharpness: f32,
    reserved1: f32,
    reserved2: f32,
    reserved3: f32,
    reserved4: f32,
    reserved5: f32,
}

@group(0) @binding(0) var<storage, read> input_img: array<u32>;
@group(0) @binding(1) var<storage, read_write> output_img: array<u32>;
@group(0) @binding(2) var<uniform> dims: Dimensions;

// Helper functions for color handling
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

fn FsrRcasSample(p: vec2<i32>) -> vec3<f32> {
    // Clamp to valid coordinates
    let px = clamp(p.x, 0, i32(dims.width) - 1);
    let py = clamp(p.y, 0, i32(dims.height) - 1);
    let idx = py * i32(dims.width) + px;
    let rgba = unpack_rgba8(input_img[idx]);
    return rgba.rgb;
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= dims.width || gid.y >= dims.height) {
        return;
    }
    
    let pos = vec2<i32>(i32(gid.x), i32(gid.y));
    
    // Sample a 3x3 neighborhood
    let center = FsrRcasSample(pos);
    let top = FsrRcasSample(pos + vec2<i32>(0, -1));
    let bottom = FsrRcasSample(pos + vec2<i32>(0, 1));
    let left = FsrRcasSample(pos + vec2<i32>(-1, 0));
    let right = FsrRcasSample(pos + vec2<i32>(1, 0));
    
    // Calculate luma for each sample (approximation of perceived brightness)
    let lumCenter = dot(center, vec3<f32>(0.299, 0.587, 0.114));
    let lumTop = dot(top, vec3<f32>(0.299, 0.587, 0.114));
    let lumBottom = dot(bottom, vec3<f32>(0.299, 0.587, 0.114));
    let lumLeft = dot(left, vec3<f32>(0.299, 0.587, 0.114));
    let lumRight = dot(right, vec3<f32>(0.299, 0.587, 0.114));
    
    // Calculate min and max luma in neighborhood
    let minLum = min(lumCenter, min(min(lumTop, lumBottom), min(lumLeft, lumRight)));
    let maxLum = max(lumCenter, max(max(lumTop, lumBottom), max(lumLeft, lumRight)));
    
    // Calculate local contrast
    let localContrast = maxLum - minLum;
    
    // Sharpen strength varies based on local contrast
    let sharpenStrength = dims.sharpness * (1.0 - smoothstep(0.0, 0.2, localContrast));
    
    // Calculate and apply sharpening
    let sharpen = center;
    let laplacian = 4.0 * center - top - bottom - left - right;
    
    // Apply sharpening
    let result = center + laplacian * sharpenStrength;
    
    // Output
    let dst_idx = gid.y * dims.width + gid.x;
    output_img[dst_idx] = pack_rgba8(vec4<f32>(result, 1.0));
}
"#;

/// GPU-accelerated FSR (FidelityFX Super Resolution) upscaler
pub struct FsrUpscaler {
    quality: UpscalingQuality,
    input_width: u32,
    input_height: u32,
    output_width: u32,
    output_height: u32,
    initialized: bool,
    // WGPU resources
    device: Option<Arc<Device>>,
    queue: Option<Arc<Queue>>,
    easu_shader: Option<ShaderModule>,
    rcas_shader: Option<ShaderModule>,
    easu_pipeline: Option<ComputePipeline>,
    rcas_pipeline: Option<ComputePipeline>,
    input_buffer: Option<Buffer>,
    intermediate_buffer: Option<Buffer>, 
    output_buffer: Option<Buffer>,
    easu_dimensions_buffer: Option<Buffer>,
    rcas_dimensions_buffer: Option<Buffer>,
    easu_bind_group_layout: Option<BindGroupLayout>,
    rcas_bind_group_layout: Option<BindGroupLayout>,
    easu_bind_group: Option<BindGroup>,
    rcas_bind_group: Option<BindGroup>,
    staging_buffer: Option<Buffer>,
}

impl FsrUpscaler {
    /// Create a new FSR upscaler with the given quality level
    pub fn new(quality: UpscalingQuality) -> Self {
        Self {
            quality,
            input_width: 0,
            input_height: 0,
            output_width: 0,
            output_height: 0,
            initialized: false,
            device: None,
            queue: None,
            easu_shader: None,
            rcas_shader: None,
            easu_pipeline: None,
            rcas_pipeline: None,
            input_buffer: None,
            intermediate_buffer: None,
            output_buffer: None,
            easu_dimensions_buffer: None,
            rcas_dimensions_buffer: None,
            easu_bind_group_layout: None,
            rcas_bind_group_layout: None,
            easu_bind_group: None,
            rcas_bind_group: None,
            staging_buffer: None,
        }
    }
    
    /// Get the sharpness value based on quality setting
    fn get_sharpness(&self) -> f32 {
        match self.quality {
            UpscalingQuality::Ultra => 0.9,
            UpscalingQuality::Quality => 0.7,
            UpscalingQuality::Balanced => 0.5,
            UpscalingQuality::Performance => 0.3,
        }
    }
    
    /// Set device and queue for the upscaler (allows reusing from main app)
    pub fn set_device_queue(&mut self, device: Arc<Device>, queue: Arc<Queue>) {
        self.device = Some(device);
        self.queue = Some(queue);
    }
}

impl Upscaler for FsrUpscaler {
    fn initialize(&mut self, input_width: u32, input_height: u32, output_width: u32, output_height: u32) -> Result<()> {
        if self.initialized &&
           self.input_width == input_width &&
           self.input_height == input_height &&
           self.output_width == output_width &&
           self.output_height == output_height {
            return Ok(());
        }

        println!("[FsrUpscaler] Initializing {}x{} -> {}x{} (quality: {:?})", 
            input_width, input_height, output_width, output_height, self.quality);
        
        self.input_width = input_width;
        self.input_height = input_height;
        self.output_width = output_width;
        self.output_height = output_height;
        
        let device = if let Some(dev) = &self.device {
            dev.clone()
        } else {
            // Create instance and adapter if not provided
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
            
            let adapter = pollster::block_on(instance.request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    ..Default::default()
                }
            )).ok_or_else(|| anyhow::anyhow!("Failed to find a suitable GPU adapter"))?;
            
            // Get device and queue
            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("FSR Upscaler Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            ))?;
            
            let device_arc = Arc::new(device);
            self.queue = Some(Arc::new(queue));
            self.device = Some(device_arc.clone());
            device_arc
        };
        
        let queue = self.queue.as_ref().unwrap().clone();
        
        // Create EASU shader (Edge Adaptive Spatial Upsampling)
        let easu_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("FSR EASU Shader"),
            source: ShaderSource::Wgsl(FSR_EASU_SHADER.into()),
        });
        
        // Create RCAS shader (Robust Contrast Adaptive Sharpening)
        let rcas_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("FSR RCAS Shader"),
            source: ShaderSource::Wgsl(FSR_RCAS_SHADER.into()),
        });
        
        // Create buffers
        let input_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("FSR Input Buffer"),
            size: (input_width * input_height * 4) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Intermediate buffer for EASU output / RCAS input
        let intermediate_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("FSR Intermediate Buffer"),
            size: (output_width * output_height * 4) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Final output buffer
        let output_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("FSR Output Buffer"),
            size: (output_width * output_height * 4) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Staging buffer for reading results
        let staging_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("FSR Staging Buffer"),
            size: (output_width * output_height * 4) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create EASU dimensions buffer with sharpness parameter
        let sharpness = self.get_sharpness();
        let easu_dims = [
            input_width, input_height, output_width, output_height,
            sharpness.to_bits(), 0_u32, 0_u32, 0_u32
        ];
        let easu_dimensions_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("FSR EASU Dimensions Buffer"),
            contents: bytemuck::cast_slice(&easu_dims),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        
        // Create RCAS dimensions buffer with sharpness parameter
        let rcas_dims = [
            output_width, output_height,
            sharpness.to_bits(), 0_u32, 0_u32, 0_u32, 0_u32, 0_u32
        ];
        let rcas_dimensions_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("FSR RCAS Dimensions Buffer"),
            contents: bytemuck::cast_slice(&rcas_dims),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        
        // Create EASU bind group layout
        let easu_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("FSR EASU Bind Group Layout"),
            entries: &[
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
        
        // Create RCAS bind group layout (similar but with different buffers)
        let rcas_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("FSR RCAS Bind Group Layout"),
            entries: &[
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
        
        // Create EASU pipeline
        let easu_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("FSR EASU Pipeline Layout"),
            bind_group_layouts: &[&easu_bind_group_layout],
            push_constant_ranges: &[],
        });
        let easu_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("FSR EASU Pipeline"),
            layout: Some(&easu_pipeline_layout),
            module: &easu_shader,
            entry_point: "main",
        });
        
        // Create RCAS pipeline
        let rcas_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("FSR RCAS Pipeline Layout"),
            bind_group_layouts: &[&rcas_bind_group_layout],
            push_constant_ranges: &[],
        });
        let rcas_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("FSR RCAS Pipeline"),
            layout: Some(&rcas_pipeline_layout),
            module: &rcas_shader,
            entry_point: "main",
        });
        
        // Create EASU bind group
        let easu_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("FSR EASU Bind Group"),
            layout: &easu_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: intermediate_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: easu_dimensions_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Create RCAS bind group
        let rcas_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("FSR RCAS Bind Group"),
            layout: &rcas_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: intermediate_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: output_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: rcas_dimensions_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Store objects
        self.easu_shader = Some(easu_shader);
        self.rcas_shader = Some(rcas_shader);
        self.easu_pipeline = Some(easu_pipeline);
        self.rcas_pipeline = Some(rcas_pipeline);
        self.input_buffer = Some(input_buffer);
        self.intermediate_buffer = Some(intermediate_buffer);
        self.output_buffer = Some(output_buffer);
        self.easu_dimensions_buffer = Some(easu_dimensions_buffer);
        self.rcas_dimensions_buffer = Some(rcas_dimensions_buffer);
        self.easu_bind_group_layout = Some(easu_bind_group_layout);
        self.rcas_bind_group_layout = Some(rcas_bind_group_layout);
        self.easu_bind_group = Some(easu_bind_group);
        self.rcas_bind_group = Some(rcas_bind_group);
        self.staging_buffer = Some(staging_buffer);
        
        self.initialized = true;
        Ok(())
    }

    fn upscale(&self, input: &[u8]) -> Result<Vec<u8>> {
        if !self.initialized {
            anyhow::bail!("FsrUpscaler not initialized");
        }
        
        let device = self.device.as_ref().ok_or_else(|| anyhow::anyhow!("Device not initialized"))?;
        let queue = self.queue.as_ref().ok_or_else(|| anyhow::anyhow!("Queue not initialized"))?;
        let easu_pipeline = self.easu_pipeline.as_ref().ok_or_else(|| anyhow::anyhow!("EASU pipeline not initialized"))?;
        let rcas_pipeline = self.rcas_pipeline.as_ref().ok_or_else(|| anyhow::anyhow!("RCAS pipeline not initialized"))?;
        let easu_bind_group = self.easu_bind_group.as_ref().ok_or_else(|| anyhow::anyhow!("EASU bind group not initialized"))?;
        let rcas_bind_group = self.rcas_bind_group.as_ref().ok_or_else(|| anyhow::anyhow!("RCAS bind group not initialized"))?;
        let input_buffer = self.input_buffer.as_ref().ok_or_else(|| anyhow::anyhow!("Input buffer not initialized"))?;
        let staging_buffer = self.staging_buffer.as_ref().ok_or_else(|| anyhow::anyhow!("Staging buffer not initialized"))?;
        
        // Check input size
        let expected_size = (self.input_width * self.input_height * 4) as usize;
        if input.len() != expected_size {
            anyhow::bail!("Input buffer size mismatch: expected {} got {}", expected_size, input.len());
        }
        
        // Write input data to the input buffer
        queue.write_buffer(input_buffer, 0, input);
        
        // Create and submit command encoder with both passes
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("FSR Upscale Encoder"),
        });
        
        // EASU pass (upscaling)
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("FSR EASU Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(easu_pipeline);
            compute_pass.set_bind_group(0, easu_bind_group, &[]);
            
            // Calculate dispatch size (8x8 workgroups)
            let x_groups = (self.output_width + 7) / 8;
            let y_groups = (self.output_height + 7) / 8;
            compute_pass.dispatch_workgroups(x_groups, y_groups, 1);
        }
        
        // RCAS pass (sharpening)
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("FSR RCAS Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(rcas_pipeline);
            compute_pass.set_bind_group(0, rcas_bind_group, &[]);
            
            // Calculate dispatch size (8x8 workgroups)
            let x_groups = (self.output_width + 7) / 8;
            let y_groups = (self.output_height + 7) / 8;
            compute_pass.dispatch_workgroups(x_groups, y_groups, 1);
        }
        
        // Copy result to staging buffer for reading
        let output_buffer = self.output_buffer.as_ref().ok_or_else(|| anyhow::anyhow!("Output buffer not initialized"))?;
        encoder.copy_buffer_to_buffer(
            output_buffer, 0,
            staging_buffer, 0,
            (self.output_width * self.output_height * 4) as u64
        );
        
        // Submit work
        queue.submit(Some(encoder.finish()));
        
        // Map staging buffer and read result
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        
        buffer_slice.map_async(MapMode::Read, move |v| {
            sender.send(v).unwrap();
        });
        
        device.poll(wgpu::Maintain::Wait);
        
        match receiver.recv() {
            Ok(Ok(())) => {
                let data = buffer_slice.get_mapped_range().to_vec();
                drop(buffer_slice.get_mapped_range());
                staging_buffer.unmap();
                Ok(data)
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("Buffer mapping error: {:?}", e)),
            Err(e) => Err(anyhow::anyhow!("Channel receive error: {:?}", e)),
        }
    }

    fn name(&self) -> &'static str {
        "FSR Upscaler"
    }

    fn quality(&self) -> UpscalingQuality {
        self.quality
    }

    fn set_quality(&mut self, quality: UpscalingQuality) -> Result<()> {
        if self.quality == quality {
            return Ok(());
        }
        
        self.quality = quality;
        
        // Update sharpness parameters in dimension buffers if initialized
        if self.initialized {
            if let (Some(queue), Some(easu_buffer), Some(rcas_buffer)) = (
                &self.queue,
                &self.easu_dimensions_buffer,
                &self.rcas_dimensions_buffer
            ) {
                let sharpness = self.get_sharpness();
                
                // Update EASU dimensions
                let easu_sharpness_offset = 4 * std::mem::size_of::<u32>() as u64;
                queue.write_buffer(easu_buffer, easu_sharpness_offset, bytemuck::cast_slice(&[sharpness.to_bits()]));
                
                // Update RCAS dimensions
                let rcas_sharpness_offset = 2 * std::mem::size_of::<u32>() as u64;
                queue.write_buffer(rcas_buffer, rcas_sharpness_offset, bytemuck::cast_slice(&[sharpness.to_bits()]));
            }
        }
        
        Ok(())
    }
} 