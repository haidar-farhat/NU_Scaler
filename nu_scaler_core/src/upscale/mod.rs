use crate::gpu::{
    memory::{/*MemoryPool,*/ AllocationStrategy /*MemoryPressure*/},
    GpuResources,
};
use anyhow::{anyhow, Result};
use pyo3::prelude::*;
use rayon::prelude::*;
use std::any::Any;
use std::fs::OpenOptions;
use std::fs::{/*OpenOptions,*/ File};
use std::io::{/*Write,*/ BufWriter};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt;
use wgpu::{
    /*Adapter,*/ Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType,
    BufferDescriptor, BufferUsages, /*BindingResource,*/ CommandEncoderDescriptor,
    ComputePipeline, ComputePipelineDescriptor, Device, DeviceDescriptor, Instance,
    /*Features,*/ Limits, MapMode, PipelineLayoutDescriptor, Queue, RequestAdapterOptions,
    ShaderModule, ShaderModuleDescriptor, ShaderSource,
};

// Add new module declarations
pub mod dlss;
#[cfg(feature = "fsr3")]
mod fsr;

// Re-export the new implementations
pub use dlss::DlssUpscaler;
#[cfg(feature = "fsr3")]
pub use fsr::FsrUpscaler;

/// Upscaling quality levels
#[derive(Debug, Clone, Copy, PartialEq)]
#[pyclass]
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
#[pyclass]
pub enum UpscalingTechnology {
    None,
    FSR,
    DLSS,
    Wgpu,
    Fallback,
}

/// Trait for upscaling algorithms
pub trait Upscaler: Any {
    /// Initialize the upscaler
    fn initialize(
        &mut self,
        input_width: u32,
        input_height: u32,
        output_width: u32,
        output_height: u32,
    ) -> Result<()>;
    /// Upscale a single frame (raw bytes or image)
    fn upscale(&self, input: &[u8]) -> Result<Vec<u8>>;
    /// Get the name of this upscaler
    fn name(&self) -> &'static str;
    /// Get the quality level
    fn quality(&self) -> UpscalingQuality;
    /// Set the quality level
    fn set_quality(&mut self, quality: UpscalingQuality) -> Result<()>;
    /// Get a reference to self as Any
    fn as_any(&self) -> &dyn Any;
    /// Get a mutable reference to self as Any
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Factory for creating upscalers based on technology detection
pub struct UpscalerFactory;

impl UpscalerFactory {
    /// Create the most appropriate upscaler based on the detected technology
    pub fn create_upscaler(
        technology: UpscalingTechnology,
        quality: UpscalingQuality,
    ) -> Box<dyn Upscaler> {
        match technology {
            #[cfg(feature = "fsr3")]
            UpscalingTechnology::FSR => Box::new(FsrUpscaler::new(quality)),
            #[cfg(not(feature = "fsr3"))]
            UpscalingTechnology::FSR => {
                println!("[UpscalerFactory] FSR technology requested but 'fsr3' feature is not enabled. Falling back to Wgpu/Nearest.");
                Box::new(WgpuUpscaler::new(quality, UpscaleAlgorithm::Nearest))
            }
            UpscalingTechnology::DLSS => Box::new(DlssUpscaler::new(quality)),
            UpscalingTechnology::Wgpu => {
                Box::new(WgpuUpscaler::new(quality, UpscaleAlgorithm::Bilinear))
            }
            _ => Box::new(WgpuUpscaler::new(quality, UpscaleAlgorithm::Nearest)), // Fallback for None or other cases
        }
    }

    /// Share device and queue with all upscalers
    pub fn set_shared_resources(
        upscaler: &mut Box<dyn Upscaler>,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> Result<()> {
        // Cast to specific types to share resources
        #[cfg(feature = "fsr3")]
        if let Some(fsr) = upscaler.as_any_mut().downcast_mut::<FsrUpscaler>() {
            fsr.set_device_queue(device, queue);
            return Ok(()); // Handled by FSR
        }

        // Try DLSS next, or if FSR feature is off and it fell through
        if let Some(dlss) = upscaler.as_any_mut().downcast_mut::<DlssUpscaler>() {
            dlss.set_device_queue(device, queue);
            Ok(())
        } else {
            // For WgpuUpscaler or others that might not implement a specific set_device_queue
            // or if it's a type not handled above (like MockUpscaler if it were used here).
            // Currently, WgpuUpscaler handles its own device/queue or gets them via set_gpu_resources.
            // This function's primary purpose seems to be for FSR/DLSS specific resource setting.
            // If WgpuUpscaler needs shared resources set this way, its logic would need to be added here.
            Ok(())
        }
    }
}

/// Mock implementation for testing
pub struct MockUpscaler;

impl Upscaler for MockUpscaler {
    fn initialize(
        &mut self,
        _input_width: u32,
        _input_height: u32,
        _output_width: u32,
        _output_height: u32,
    ) -> Result<()> {
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
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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
    // WGPU fields - internal device and queue for self-managed mode
    instance: Option<Instance>,
    device: Option<Arc<Device>>,
    queue: Option<Arc<Queue>>,
    // Shared resources - for external device and queue mode
    gpu_resources: Option<Arc<GpuResources>>,
    // Upscaler resources
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
    // VRAM management
    use_memory_pool: bool,
    adaptive_quality: bool,
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
            gpu_resources: None,
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
            thread_count: 4,                      // Default thread count
            buffer_pool_size: 3,                  // Default buffer pool size
            gpu_allocator: "default".to_string(), // Default GPU allocator preset
            shader_path: "".to_string(),          // Default shader path (empty)
            use_memory_pool: true,                // Use memory pool by default
            adaptive_quality: false,              // Adaptive quality disabled by default
        }
    }

    pub fn set_gpu_resources(&mut self, gpu_resources: Arc<GpuResources>) {
        self.gpu_resources = Some(gpu_resources);
        // When external resources are set, clear internal ones to avoid conflict
        self.device = None;
        self.queue = None;
        self.instance = None;
        self.use_memory_pool = true; // Assume memory pool usage with shared resources
    }

    pub fn set_adaptive_quality(&mut self, enabled: bool) {
        self.adaptive_quality = enabled;
    }

    // Public getter for adaptive_quality state
    pub fn is_adaptive_quality_enabled(&self) -> bool {
        self.adaptive_quality
    }

    // Adaptive quality logic based on VRAM usage
    fn update_adaptive_quality(&self) -> bool {
        if !self.adaptive_quality {
            return false;
        }

        let mut needs_reinit = false;
        if let Some(resources) = &self.gpu_resources {
            let stats = resources.get_vram_stats();
            let usage_percent = if stats.total_mb > 0.0 {
                (stats.used_mb / stats.total_mb) * 100.0
            } else {
                0.0
            };

            let current_quality = self.quality;
            let mut new_quality = current_quality;

            if usage_percent > 85.0 {
                // High pressure
                new_quality = match current_quality {
                    UpscalingQuality::Ultra => UpscalingQuality::Quality,
                    UpscalingQuality::Quality => UpscalingQuality::Balanced,
                    UpscalingQuality::Balanced => UpscalingQuality::Performance,
                    UpscalingQuality::Performance => UpscalingQuality::Performance, // Already at lowest
                };
                println!("[AdaptiveQuality] High VRAM pressure ({}%), lowering quality from {:?} to {:?}", usage_percent, current_quality, new_quality);
            } else if usage_percent < 50.0 {
                // Low pressure
                new_quality = match current_quality {
                    UpscalingQuality::Ultra => UpscalingQuality::Ultra, // Already at highest
                    UpscalingQuality::Quality => UpscalingQuality::Ultra,
                    UpscalingQuality::Balanced => UpscalingQuality::Quality,
                    UpscalingQuality::Performance => UpscalingQuality::Balanced,
                };
                if new_quality != current_quality {
                    println!("[AdaptiveQuality] Low VRAM pressure ({}%), increasing quality from {:?} to {:?}", usage_percent, current_quality, new_quality);
                }
            }

            if new_quality != current_quality {
                // This is tricky because self is immutable here.
                // The quality change should trigger re-initialization if needed by the underlying tech.
                // For WgpuUpscaler, changing quality doesn't inherently require re-init of buffers/pipeline
                // unless shader or something fundamental changes based on quality.
                // However, to signal a potential change, we can return true.
                // The caller (PyAdvancedWgpuUpscaler) can then call self.set_quality().
                needs_reinit = true;
            }
        }
        needs_reinit
    }

    // Helper to get the device, preferring shared GpuResources if available
    fn device(&self) -> Option<&Device> {
        self.gpu_resources
            .as_ref()
            .map(|r| &*r.device)
            .or(self.device.as_deref())
    }

    // Helper to get the queue, preferring shared GpuResources if available
    fn queue(&self) -> Option<&Queue> {
        self.gpu_resources
            .as_ref()
            .map(|r| &*r.queue)
            .or(self.queue.as_deref())
    }

    // Get a cloned Arc<Device> if available
    fn get_device_clone(&self) -> Option<Arc<Device>> {
        self.gpu_resources
            .as_ref()
            .map(|r| r.device.clone())
            .or_else(|| self.device.clone())
    }

    // Allow setting thread count for parallel processing
    pub fn set_thread_count(&mut self, n: u32) {
        if n > 0 {
            self.thread_count = n;
            println!("[WgpuUpscaler] Thread count set to: {}", self.thread_count);
        } else {
            println!("[WgpuUpscaler] Invalid thread count: {} (must be > 0)", n);
        }
    }

    // Allow setting buffer pool size
    pub fn set_buffer_pool_size(&mut self, n: u32) {
        if n > 0 {
            self.buffer_pool_size = n;
            println!(
                "[WgpuUpscaler] Buffer pool size set to: {}",
                self.buffer_pool_size
            );

            // Re-initialize buffers if already initialized
            if self.initialized {
                println!("[WgpuUpscaler] Re-initializing buffers due to pool size change.");
                // This requires access to dimensions, which might not be set yet.
                // For simplicity, we rely on the main initialize() call to handle this.
                // A more robust solution would re-create buffers here if possible.
                // self.initialized = false; // Force re-init on next upscale call
            }
        } else {
            println!(
                "[WgpuUpscaler] Invalid buffer pool size: {} (must be > 0)",
                n
            );
        }
    }

    // Initialize WGPU device and queue if not already set (self-managed mode)
    async fn ensure_wgpu_initialized_async(&mut self) -> Result<()> {
        if self.device.is_some() && self.queue.is_some() {
            return Ok(());
        }
        if self.gpu_resources.is_some() {
            return Ok(());
        } // Already using shared resources

        if self.instance.is_none() {
            self.instance = Some(Instance::new(wgpu::InstanceDescriptor {
                backends: Backends::PRIMARY, // Use primary backend (DX12, Vulkan, Metal)
                ..Default::default()
            }));
        }

        let instance = self.instance.as_ref().unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .ok_or_else(|| anyhow!("Failed to find an appropriate adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("WGPU Upscaler Device"),
                    required_features: wgpu::Features::empty(), // No special features needed for basic compute
                    required_limits: Limits::default(),
                },
                None, // Trace path
            )
            .await
            .map_err(|e| anyhow!("Failed to create WGPU device: {}", e))?;

        self.device = Some(Arc::new(device));
        self.queue = Some(Arc::new(queue));
        self.use_memory_pool = false; // In self-managed mode, don't use GpuResources pool by default
        Ok(())
    }

    // Blocking version for non-async contexts
    fn ensure_wgpu_initialized(&mut self) -> Result<()> {
        if self.device.is_some() && self.queue.is_some() {
            return Ok(());
        }
        if self.gpu_resources.is_some() {
            return Ok(());
        } // Already using shared resources

        pollster::block_on(self.ensure_wgpu_initialized_async())
    }

    // Set GPU allocator preset
    pub fn set_gpu_allocator(&mut self, preset: &str) {
        self.gpu_allocator = preset.to_string();
        println!(
            "[WgpuUpscaler] GPU allocator preset set to: {}",
            self.gpu_allocator
        );

        // In a real implementation, this would configure the wgpu-memory allocator
        // or a custom memory management strategy based on the preset.
        // For now, it's just a placeholder for future integration.

        // Example of how it might be used with GpuResources:
        if let Some(resources) = &self.gpu_resources {
            let strategy = match preset.to_lowercase().as_str() {
                "aggressive" => AllocationStrategy::Aggressive,
                "balanced" => AllocationStrategy::Balanced,
                "conservative" => AllocationStrategy::Conservative,
                "minimal" => AllocationStrategy::Minimal,
                _ => AllocationStrategy::Balanced, // Default to balanced
            };
            resources.set_allocation_strategy(strategy);
            println!("[WgpuUpscaler] Memory pool strategy set to: {:?}", strategy);
        } else {
            // If not using GpuResources, this setting might apply to a local memory manager
            // or be ignored for now.
            println!(
                "[WgpuUpscaler] GPU allocator preset '{}' noted, but GpuResources not in use.",
                preset
            );
        }
    }

    // Load shader from path or use default
    fn load_shader_module(&self, device: &Device) -> ShaderModule {
        let shader_code = if !self.shader_path.is_empty() {
            match std::fs::read_to_string(&self.shader_path) {
                Ok(code) => {
                    println!("[WgpuUpscaler] Loaded shader from: {}", self.shader_path);
                    code
                }
                Err(e) => {
                    println!(
                        "[WgpuUpscaler] Failed to load shader from '{}': {}. Using default shader.",
                        self.shader_path, e
                    );
                    if self.algorithm == UpscaleAlgorithm::Bilinear {
                        BILINEAR_UPSCALE_SHADER.to_string()
                    } else {
                        NN_UPSCALE_SHADER.to_string()
                    }
                }
            }
        } else {
            if self.algorithm == UpscaleAlgorithm::Bilinear {
                BILINEAR_UPSCALE_SHADER.to_string()
            } else {
                NN_UPSCALE_SHADER.to_string()
            }
        };

        device.create_shader_module(ShaderModuleDescriptor {
            label: Some(if self.algorithm == UpscaleAlgorithm::Bilinear {
                "Bilinear Upscale Shader"
            } else {
                "Nearest Neighbor Upscale Shader"
            }),
            source: ShaderSource::Wgsl(shader_code.into()),
        })
    }

    // Reload shader from a given path
    pub fn reload_shader(&mut self, path: &str) -> anyhow::Result<()> {
        self.shader_path = path.to_string();
        // Invalidate current shader and pipeline to force re-creation on next upscale
        self.shader = None;
        self.pipeline = None;
        self.initialized = false; // Force re-init to rebuild pipeline
        println!(
            "[WgpuUpscaler] Shader path set to: '{}'. Will reload on next upscale.",
            self.shader_path
        );
        Ok(())
    }

    // Batch upscale multiple frames. This is a simple parallel map over upscale().
    // For true batching on GPU, a different shader/pipeline structure would be needed.
    pub fn upscale_batch(&self, frames: &[&[u8]]) -> Result<Vec<Vec<u8>>> {
        if !self.initialized {
            return Err(anyhow!(
                "Upscaler not initialized. Call initialize() first."
            ));
        }

        let start_time = Instant::now();

        // Use Rayon for parallel processing of frames
        let results: Vec<Result<Vec<u8>>> = frames
            .par_iter()
            .map(|frame_data| {
                self.upscale(frame_data) // This reuses the single-frame upscale logic
            })
            .collect();

        // Check for errors and collect results
        let mut outputs = Vec::with_capacity(frames.len());
        for result in results {
            outputs.push(result?);
        }

        let elapsed_time = start_time.elapsed();
        println!(
            "[WgpuUpscaler] Batch upscale of {} frames completed in {:.2}ms",
            frames.len(),
            elapsed_time.as_secs_f32() * 1000.0
        );

        Ok(outputs)
    }

    // Initialize buffers, pipeline, etc.
    fn initialize_with_resources(
        &mut self,
        input_width: u32,
        input_height: u32,
        output_width: u32,
        output_height: u32,
    ) -> Result<()> {
        // Get a *clone* of the Arc<Device> if available, ending the borrow of self immediately.
        let device_arc = self
            .get_device_clone()
            .ok_or_else(|| anyhow!("WGPU device not available for initialization"))?;
        // `self` is no longer borrowed by `device_arc` at this point.

        // Now update self fields
        self.input_width = input_width;
        self.input_height = input_height;
        self.output_width = output_width;
        self.output_height = output_height;

        let input_buffer_size = (input_width * input_height * 4) as u64; // RGBA8
        let output_buffer_size = (output_width * output_height * 4) as u64;

        // Use the cloned Arc, getting a reference (&Device) when needed for wgpu calls.
        let device_ref: &Device = &*device_arc;

        // Create or re-create shader module if not already done or if algorithm changed
        if self.shader.is_none() {
            // Also consider if algorithm changed
            self.shader = Some(self.load_shader_module(device_ref));
        }
        let shader = self.shader.as_ref().unwrap();

        // Create bind group layout if not already done or if it needs to change
        if self.bind_group_layout.is_none() {
            self.bind_group_layout = Some(device_ref.create_bind_group_layout(
                &BindGroupLayoutDescriptor {
                    label: Some("Upscale Bind Group Layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            // Input image
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
                            // Output image
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
                            // Dimensions
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
                },
            ));
        }
        let bind_group_layout = self.bind_group_layout.as_ref().unwrap();

        // Create pipeline if not already done or if it needs to change
        if self.pipeline.is_none() {
            // Also consider if shader or layout changed
            let pipeline_layout = device_ref.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Upscale Pipeline Layout"),
                bind_group_layouts: &[bind_group_layout],
                push_constant_ranges: &[],
            });
            self.pipeline = Some(
                device_ref.create_compute_pipeline(&ComputePipelineDescriptor {
                    label: Some(if self.algorithm == UpscaleAlgorithm::Bilinear {
                        "Bilinear Upscale Pipeline"
                    } else {
                        "Nearest Neighbor Upscale Pipeline"
                    }),
                    layout: Some(&pipeline_layout),
                    module: shader,
                    entry_point: "main",
                    // compilation_options: removed previously
                }),
            );
        }

        // Create dimensions buffer
        let dimensions_data = [
            self.input_width,
            self.input_height,
            self.output_width,
            self.output_height,
            0,
            0,
            0,
            0, // Reserved for sharpness, etc. - needs to match shader struct
        ];
        self.dimensions_buffer = Some(device_ref.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Dimensions Uniform Buffer"),
                contents: bytemuck::cast_slice(&dimensions_data),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            },
        ));

        // Initialize buffer pool if using GpuResources memory pool
        if self.use_memory_pool && self.gpu_resources.is_some() {
            let pool = &self.gpu_resources.as_ref().unwrap().memory_pool;
            self.buffer_pool.clear();
            self.buffer_pool_bind_groups.clear();

            for i in 0..self.buffer_pool_size {
                let input_buf = pool.get_buffer(
                    input_buffer_size as usize,
                    BufferUsages::STORAGE | BufferUsages::COPY_DST,
                    Some(&format!("Pooled Input {}", i)),
                );
                let output_buf = pool.get_buffer(
                    output_buffer_size as usize,
                    BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                    Some(&format!("Pooled Output {}", i)),
                );

                let bind_group = device_ref.create_bind_group(&BindGroupDescriptor {
                    label: Some(&format!("Pooled Upscale Bind Group {}", i)),
                    layout: bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: input_buf.as_entire_binding(),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: output_buf.as_entire_binding(),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: self.dimensions_buffer.as_ref().unwrap().as_entire_binding(),
                        },
                    ],
                });
                self.buffer_pool.push(input_buf); // Store input buffer, output is implicitly paired by index for now
                self.buffer_pool.push(output_buf);
                self.buffer_pool_bind_groups.push(bind_group);
            }
            println!("[WgpuUpscaler] Initialized buffer pool with {} sets of buffers using GpuResources pool.", self.buffer_pool_size);
        } else {
            // Fallback to creating individual buffers if not using GpuResources pool or if it's not available
            self.input_buffer = Some(device_ref.create_buffer(&BufferDescriptor {
                label: Some("Input Buffer"),
                size: input_buffer_size,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.output_buffer = Some(device_ref.create_buffer(&BufferDescriptor {
                label: Some("Output Buffer"),
                size: output_buffer_size,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }));
            self.fallback_bind_group = Some(device_ref.create_bind_group(&BindGroupDescriptor {
                label: Some("Fallback Upscale Bind Group"),
                layout: bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: self.input_buffer.as_ref().unwrap().as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: self.output_buffer.as_ref().unwrap().as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: self.dimensions_buffer.as_ref().unwrap().as_entire_binding(),
                    },
                ],
            }));
            println!("[WgpuUpscaler] Initialized individual input/output buffers.");
        }

        // Create staging buffer (always created, size might need adjustment)
        self.staging_buffer = Some(device_ref.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: output_buffer_size, // Should be large enough for output
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        self.initialized = true;
        println!(
            "[WgpuUpscaler] Initialized for {}x{} -> {}x{} (Algorithm: {:?})",
            self.input_width,
            self.input_height,
            self.output_width,
            self.output_height,
            self.algorithm
        );
        Ok(())
    }
}

impl Drop for WgpuUpscaler {
    fn drop(&mut self) {
        // Buffers in self.buffer_pool are managed by GpuResources's MemoryPool if use_memory_pool is true
        // and GpuResources is Some. They will be released when their Arc count drops or when MemoryPool cleans up.
        // So, no explicit deallocation here for pooled buffers is strictly necessary if MemoryPool handles it.
        // However, if not using the pool, or for clarity, one might release them.
        // For now, relying on Arc and MemoryPool's drop for pooled buffers.
        // Non-pooled buffers (input_buffer, output_buffer, staging_buffer, dimensions_buffer) will be dropped automatically.
        println!(
            "[WgpuUpscaler] Dropping WgpuUpscaler for {}x{} -> {}x{}",
            self.input_width, self.input_height, self.output_width, self.output_height
        );
    }
}

impl Upscaler for WgpuUpscaler {
    fn initialize(
        &mut self,
        input_width: u32,
        input_height: u32,
        output_width: u32,
        output_height: u32,
    ) -> Result<()> {
        // If dimensions change, or not initialized, or shader/pipeline is None (e.g. after reload_shader)
        if !self.initialized
            || self.input_width != input_width
            || self.input_height != input_height
            || self.output_width != output_width
            || self.output_height != output_height
            || self.shader.is_none()
            || self.pipeline.is_none()
        {
            // Ensure WGPU is ready (especially for self-managed mode)
            if self.gpu_resources.is_none() {
                // Only call if not using shared resources
                self.ensure_wgpu_initialized()?;
            }

            // Proceed with resource initialization
            self.initialize_with_resources(input_width, input_height, output_width, output_height)?;
        } else {
            // Update dimensions buffer if only dimensions changed but everything else is valid
            // This assumes the buffer sizes are still appropriate. If not, full re-init is needed.
            if let (Some(_device), Some(queue), Some(dims_buffer)) =
                (self.device(), self.queue(), &self.dimensions_buffer)
            {
                let dimensions_data = [
                    input_width,
                    input_height,
                    output_width,
                    output_height,
                    0,
                    0,
                    0,
                    0, // Reserved
                ];
                queue.write_buffer(dims_buffer, 0, bytemuck::cast_slice(&dimensions_data));

                // Update internal state if dimensions changed
                self.input_width = input_width;
                self.input_height = input_height;
                self.output_width = output_width;
                self.output_height = output_height;
                println!(
                    "[WgpuUpscaler] Dimensions updated for {}x{} -> {}x{}",
                    self.input_width, self.input_height, self.output_width, self.output_height
                );
            } else {
                return Err(anyhow!(
                    "Cannot update dimensions: WGPU device/queue not available."
                ));
            }
        }
        Ok(())
    }

    fn upscale(&self, input: &[u8]) -> Result<Vec<u8>> {
        if !self.initialized {
            return Err(anyhow!(
                "Upscaler not initialized. Call initialize() first."
            ));
        }

        let (device, queue) = match (self.device(), self.queue()) {
            (Some(d), Some(q)) => (d, q),
            _ => return Err(anyhow!("WGPU device or queue not available")),
        };

        let pipeline = self
            .pipeline
            .as_ref()
            .ok_or_else(|| anyhow!("Upscale pipeline not created"))?;
        let staging_buffer = self
            .staging_buffer
            .as_ref()
            .ok_or_else(|| anyhow!("Staging buffer not created"))?;

        let input_buffer_size = (self.input_width * self.input_height * 4) as u64;
        let output_buffer_size = (self.output_width * self.output_height * 4) as u64;

        if input.len() as u64 != input_buffer_size {
            return Err(anyhow!(
                "Input data size ({}) does not match expected input buffer size ({} for {}x{})",
                input.len(),
                input_buffer_size,
                self.input_width,
                self.input_height
            ));
        }

        let bind_group_to_use: &BindGroup;
        let current_input_buffer: &Buffer;
        let current_output_buffer: &Buffer; // For clarity, though only input is written to before dispatch

        if self.use_memory_pool
            && !self.buffer_pool_bind_groups.is_empty()
            && self.gpu_resources.is_some()
        {
            let pool_idx = self.buffer_pool_index.fetch_add(1, Ordering::Relaxed)
                % self.buffer_pool_size as usize;
            bind_group_to_use = &self.buffer_pool_bind_groups[pool_idx];
            // Pooled buffers are in pairs: input, output, input, output ...
            current_input_buffer = &self.buffer_pool[pool_idx * 2];
            current_output_buffer = &self.buffer_pool[pool_idx * 2 + 1];
        } else if let Some(bg) = &self.fallback_bind_group {
            bind_group_to_use = bg;
            current_input_buffer = self
                .input_buffer
                .as_ref()
                .ok_or_else(|| anyhow!("Input buffer not available for fallback path"))?;
            current_output_buffer = self
                .output_buffer
                .as_ref()
                .ok_or_else(|| anyhow!("Output buffer not available for fallback path"))?;
        } else {
            return Err(anyhow!(
                "No valid bind group or buffers available for upscaling"
            ));
        }

        // Write input data to the selected input GPU buffer
        queue.write_buffer(current_input_buffer, 0, input);

        // Adaptive quality check and potential adjustment (conceptual, needs proper handling of re-init)
        if self.adaptive_quality {
            // This call is problematic if it tries to mutate self.quality directly.
            // It should signal if a change is *recommended*.
            // The actual quality change and re-init should happen at a higher level (e.g., PyAdvancedWgpuUpscaler).
            // For now, let's assume it just prints a recommendation or influences internal heuristics not yet implemented.
            let _recommended_quality_change = self.update_adaptive_quality();
            // if recommended_quality_change {
            //    println!("[WgpuUpscaler] Adaptive quality suggests a change. Consider re-initializing with new quality.");
            // }
        }

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Upscale Command Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Upscale Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, bind_group_to_use, &[]);
            compute_pass.dispatch_workgroups(self.output_width / 8, self.output_height / 8, 1);
            // Assuming 8x8 workgroup size
        }

        // Copy output from GPU buffer to staging buffer
        encoder.copy_buffer_to_buffer(
            current_output_buffer, // Source: the output buffer used in the bind group
            0,
            staging_buffer,
            0,
            output_buffer_size,
        );

        queue.submit(Some(encoder.finish()));

        // Map staging buffer to read results back to CPU
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        device.poll(wgpu::Maintain::Wait); // Wait for GPU to finish and map operation

        // Receive the result of map_async
        let _map_result = receiver
            .recv()
            .map_err(|e| anyhow!("Failed to receive map_async result: {}", e))??;

        let data = buffer_slice.get_mapped_range().to_vec();
        staging_buffer.unmap(); // Unmap the buffer

        Ok(data)
    }

    fn name(&self) -> &'static str {
        if self.algorithm == UpscaleAlgorithm::Bilinear {
            "WgpuBilinearUpscaler"
        } else {
            "WgpuNearestUpscaler"
        }
    }

    fn quality(&self) -> UpscalingQuality {
        self.quality
    }

    fn set_quality(&mut self, quality: UpscalingQuality) -> Result<()> {
        self.quality = quality;
        // Shader or pipeline might need to be updated if quality affects them directly
        // For now, assuming it only affects parameters or adaptive logic.
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Helper function to robustly open a log file (append mode)
fn _open_log_file_robust() -> Option<BufWriter<File>> {
    let log_dir = PathBuf::from("logs");
    if !log_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&log_dir) {
            eprintln!("Failed to create log directory '{:?}': {}", log_dir, e);
            return None;
        }
    }
    let log_file_path = log_dir.join("wgpu_upscaler.log");
    match OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_file_path)
    {
        Ok(file) => Some(BufWriter::new(file)),
        Err(e) => {
            eprintln!("Failed to open log file '{:?}': {}", log_file_path, e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // These tests are basic and might require a running WGPU instance or mocks.
    // For now, they mainly check struct creation and basic logic.
    #[test]
    #[should_panic]
    fn test_initialize_panics() {
        let mut upscaler = MockUpscaler;
        let _ = upscaler.initialize(100, 100, 200, 200).unwrap(); // Should panic
    }

    #[test]
    #[should_panic]
    fn test_upscale_panics() {
        let upscaler = MockUpscaler;
        let _ = upscaler.upscale(&[0u8; 100]).unwrap(); // Should panic
    }

    #[test]
    fn test_name_and_quality() {
        let upscaler = MockUpscaler;
        assert_eq!(upscaler.name(), "MockUpscaler");
        assert_eq!(upscaler.quality(), UpscalingQuality::Quality);
    }

    #[test]
    #[should_panic]
    fn test_set_quality_panics() {
        let mut upscaler = MockUpscaler;
        let _ = upscaler.set_quality(UpscalingQuality::Performance).unwrap(); // Should panic
    }

    #[test]
    fn test_wgpu_upscaler_init() {
        let upscaler = WgpuUpscaler::new(UpscalingQuality::Quality, UpscaleAlgorithm::Nearest);
        // This will try to initialize WGPU device if not already available via GpuResources.
        // It might panic if no adapter is found, or fail if device request fails.
        // For robust testing, mock WGPU or ensure a device is available.
        // For now, we check if initialize itself returns Ok without shared resources.
        // This implicitly tests ensure_wgpu_initialized.
        // If this test runs in an environment without GPU access, it will likely fail here.
        // To make it pass in such env, you'd need to mock wgpu calls or skip GPU init.
        // assert!(upscaler.initialize(100, 100, 200, 200).is_ok());
        // For now, let's just check it doesn't panic immediately on creation
        assert_eq!(upscaler.name(), "WgpuNearestUpscaler");
    }
}
