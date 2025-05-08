use std::sync::Arc;
use wgpu::{Device, Queue, Buffer, ComputePipeline, BindGroup, ShaderModule, BufferDescriptor, BufferUsages, util::DeviceExt};
use anyhow::{Result, anyhow};
use crate::gpu::GpuResources;

/// Frame interpolation quality levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterpolationQuality {
    /// High quality, slower
    High,
    /// Medium quality, balanced
    Medium,
    /// Low quality, faster
    Low,
}

/// Motion estimation method
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotionEstimationMethod {
    /// Optical flow based
    OpticalFlow,
    /// Block matching based
    BlockMatching,
    /// Simplified/approximate estimation
    Simplified,
}

/// Frame interpolator trait
pub trait FrameInterpolator {
    /// Initialize the interpolator
    fn initialize(&mut self, width: u32, height: u32) -> Result<()>;
    
    /// Generate an intermediate frame between two input frames
    fn interpolate(&self, frame1: &[u8], frame2: &[u8], t: f32) -> Result<Vec<u8>>;
    
    /// Get the name of this interpolator
    fn name(&self) -> &'static str;
    
    /// Set the quality level
    fn set_quality(&mut self, quality: InterpolationQuality) -> Result<()>;
    
    /// Get the current quality level
    fn quality(&self) -> InterpolationQuality;
}

/// Optical flow-based frame interpolator
pub struct OpticalFlowInterpolator {
    /// Frame width
    width: u32,
    /// Frame height
    height: u32,
    /// Interpolation quality
    quality: InterpolationQuality,
    /// Initialized flag
    initialized: bool,
    /// GPU resources
    gpu_resources: Option<Arc<GpuResources>>,
    /// Flow estimation pipeline
    flow_pipeline: Option<ComputePipeline>,
    /// Flow estimation shader
    flow_shader: Option<ShaderModule>,
    /// Flow bind group
    flow_bind_group: Option<BindGroup>,
    /// Frame1 buffer
    frame1_buffer: Option<Buffer>,
    /// Frame2 buffer
    frame2_buffer: Option<Buffer>,
    /// Flow buffer
    flow_buffer: Option<Buffer>,
    /// Intermediate frame buffer
    intermediate_buffer: Option<Buffer>,
    /// Output frame buffer
    output_buffer: Option<Buffer>,
    /// Staging buffer
    staging_buffer: Option<Buffer>,
    /// Motion sensitivity (0.0-1.0)
    motion_sensitivity: f32,
    /// Sharpness (0.0-1.0)
    sharpness: f32,
}

impl OpticalFlowInterpolator {
    /// Create a new optical flow interpolator
    pub fn new(quality: InterpolationQuality) -> Self {
        Self {
            width: 0,
            height: 0,
            quality,
            initialized: false,
            gpu_resources: None,
            flow_pipeline: None,
            flow_shader: None,
            flow_bind_group: None,
            frame1_buffer: None,
            frame2_buffer: None,
            flow_buffer: None,
            intermediate_buffer: None,
            output_buffer: None,
            staging_buffer: None,
            motion_sensitivity: 0.7,
            sharpness: 0.5,
        }
    }
    
    /// Set the GPU resources
    pub fn set_gpu_resources(&mut self, gpu_resources: Arc<GpuResources>) {
        self.gpu_resources = Some(gpu_resources);
        self.initialized = false;
    }
    
    /// Set motion sensitivity
    pub fn set_motion_sensitivity(&mut self, sensitivity: f32) {
        self.motion_sensitivity = sensitivity.clamp(0.0, 1.0);
    }
    
    /// Set sharpness
    pub fn set_sharpness(&mut self, sharpness: f32) {
        self.sharpness = sharpness.clamp(0.0, 1.0);
    }
    
    /// Get current device
    fn device(&self) -> Option<&Device> {
        if let Some(resources) = &self.gpu_resources {
            Some(&resources.device)
        } else {
            None
        }
    }
    
    /// Get current queue
    fn queue(&self) -> Option<&Queue> {
        if let Some(resources) = &self.gpu_resources {
            Some(&resources.queue)
        } else {
            None
        }
    }
    
    /// Create the optical flow compute shader
    fn create_flow_shader(&self, device: &Device) -> ShaderModule {
        // This is a simplified implementation of an optical flow shader
        // In a real implementation, you'd want to use a more sophisticated algorithm
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Optical Flow Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("../shaders/optical_flow.wgsl"))),
        })
    }
    
    /// Compute optical flow between two frames
    fn compute_optical_flow(&self, frame1: &[u8], frame2: &[u8]) -> Result<Vec<f32>> {
        let device = self.device().ok_or_else(|| anyhow!("No device available"))?;
        let queue = self.queue().ok_or_else(|| anyhow!("No queue available"))?;
        
        // In absence of the actual compute shader, we'll simulate optical flow calculation
        // This would normally be done on the GPU using a compute shader
        
        let mut flow = vec![0.0f32; (self.width * self.height * 2) as usize]; // x,y flow components
        
        // Parameters that would affect the quality of the flow computation
        let window_size = match self.quality {
            InterpolationQuality::High => 15,
            InterpolationQuality::Medium => 11,
            InterpolationQuality::Low => 7,
        };
        
        let stride = match self.quality {
            InterpolationQuality::High => 1,
            InterpolationQuality::Medium => 2,
            InterpolationQuality::Low => 4,
        };
        
        // Simple CPU implementation of Lucas-Kanade optical flow
        // For each pixel, compute gradient and temporal derivative
        let channels = 4; // RGBA
        let width = self.width as usize;
        let height = self.height as usize;
        
        for y in stride..(height as usize - stride) {
            for x in stride..(width as usize - stride) {
                if (x % stride != 0) || (y % stride != 0) {
                    continue;
                }
                
                let idx = (y * width + x) * channels;
                
                // Calculate spatial derivatives (simplified to just using one color channel - green)
                let i_x = (frame1[idx + 1 + channels] as i32 - frame1[idx + 1 - channels] as i32) / 2;
                let i_y = (frame1[idx + 1 + width * channels] as i32 - frame1[idx + 1 - width * channels] as i32) / 2;
                
                // Calculate temporal derivative
                let i_t = frame2[idx + 1] as i32 - frame1[idx + 1] as i32;
                
                // Calculate flow if gradients are strong enough
                let threshold = 5.0;
                if i_x.abs() as f32 + i_y.abs() as f32 > threshold {
                    // Simplified optical flow equation
                    let divisor = (i_x * i_x + i_y * i_y) as f32;
                    if divisor > 0.1 {
                        let flow_x = -(i_t * i_x) as f32 / divisor;
                        let flow_y = -(i_t * i_y) as f32 / divisor;
                        
                        // Clamp flow values to reasonable range
                        let max_motion = 20.0 * self.motion_sensitivity;
                        let flow_x = flow_x.clamp(-max_motion, max_motion);
                        let flow_y = flow_y.clamp(-max_motion, max_motion);
                        
                        let flow_idx = (y * width + x) * 2;
                        flow[flow_idx] = flow_x;
                        flow[flow_idx + 1] = flow_y;
                    }
                }
            }
        }
        
        // Smooth the flow field
        let mut smoothed_flow = flow.clone();
        for y in stride..(height - stride) {
            for x in stride..(width - stride) {
                if (x % stride != 0) || (y % stride != 0) {
                    continue;
                }
                
                let flow_idx = (y * width + x) * 2;
                
                // Simple box filter
                let mut sum_x = 0.0;
                let mut sum_y = 0.0;
                let mut count = 0.0;
                
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let nx = x as isize + dx;
                        let ny = y as isize + dy;
                        
                        if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                            let neighbor_idx = (ny as usize * width + nx as usize) * 2;
                            sum_x += flow[neighbor_idx];
                            sum_y += flow[neighbor_idx + 1];
                            count += 1.0;
                        }
                    }
                }
                
                if count > 0.0 {
                    smoothed_flow[flow_idx] = sum_x / count;
                    smoothed_flow[flow_idx + 1] = sum_y / count;
                }
            }
        }
        
        // Fill in flow for pixels we skipped
        if stride > 1 {
            let mut final_flow = vec![0.0f32; (width * height * 2) as usize];
            
            for y in 0..height {
                for x in 0..width {
                    let flow_idx = (y * width + x) * 2;
                    
                    if (x % stride == 0) && (y % stride == 0) {
                        final_flow[flow_idx] = smoothed_flow[flow_idx];
                        final_flow[flow_idx + 1] = smoothed_flow[flow_idx + 1];
                    } else {
                        // Bilinear interpolation of nearest flow vectors
                        let grid_x = (x / stride) * stride;
                        let grid_y = (y / stride) * stride;
                        
                        let next_grid_x = ((x / stride) + 1) * stride;
                        let next_grid_y = ((y / stride) + 1) * stride;
                        
                        if next_grid_x >= width || next_grid_y >= height {
                            final_flow[flow_idx] = smoothed_flow[(grid_y * width + grid_x) * 2];
                            final_flow[flow_idx + 1] = smoothed_flow[(grid_y * width + grid_x) * 2 + 1];
                            continue;
                        }
                        
                        let x_ratio = (x - grid_x) as f32 / stride as f32;
                        let y_ratio = (y - grid_y) as f32 / stride as f32;
                        
                        let idx_tl = (grid_y * width + grid_x) * 2;
                        let idx_tr = (grid_y * width + next_grid_x) * 2;
                        let idx_bl = (next_grid_y * width + grid_x) * 2;
                        let idx_br = (next_grid_y * width + next_grid_x) * 2;
                        
                        // Interpolate flow_x
                        let top = smoothed_flow[idx_tl] * (1.0 - x_ratio) + smoothed_flow[idx_tr] * x_ratio;
                        let bottom = smoothed_flow[idx_bl] * (1.0 - x_ratio) + smoothed_flow[idx_br] * x_ratio;
                        final_flow[flow_idx] = top * (1.0 - y_ratio) + bottom * y_ratio;
                        
                        // Interpolate flow_y
                        let top = smoothed_flow[idx_tl + 1] * (1.0 - x_ratio) + smoothed_flow[idx_tr + 1] * x_ratio;
                        let bottom = smoothed_flow[idx_bl + 1] * (1.0 - x_ratio) + smoothed_flow[idx_br + 1] * x_ratio;
                        final_flow[flow_idx + 1] = top * (1.0 - y_ratio) + bottom * y_ratio;
                    }
                }
            }
            
            return Ok(final_flow);
        }
        
        Ok(smoothed_flow)
    }
}

impl FrameInterpolator for OpticalFlowInterpolator {
    fn initialize(&mut self, width: u32, height: u32) -> Result<()> {
        if self.initialized && self.width == width && self.height == height {
            return Ok(());
        }
        
        self.width = width;
        self.height = height;
        
        // Get device and queue from GPU resources
        let device = self.device().ok_or_else(|| anyhow!("No device available"))?;
        let queue = self.queue().ok_or_else(|| anyhow!("No queue available"))?;
        
        // Create buffers for the interpolation pipeline
        let frame_size = (width * height * 4) as u64; // RGBA
        let flow_size = (width * height * 2 * 4) as u64; // 2 components (x,y) as f32
        
        // Create buffers for frame data
        self.frame1_buffer = Some(device.create_buffer(&BufferDescriptor {
            label: Some("Frame 1 Buffer"),
            size: frame_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        
        self.frame2_buffer = Some(device.create_buffer(&BufferDescriptor {
            label: Some("Frame 2 Buffer"),
            size: frame_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        
        // Create buffer for flow data
        self.flow_buffer = Some(device.create_buffer(&BufferDescriptor {
            label: Some("Flow Buffer"),
            size: flow_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));
        
        // Create output buffer
        self.output_buffer = Some(device.create_buffer(&BufferDescriptor {
            label: Some("Output Buffer"),
            size: frame_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));
        
        // Create staging buffer for reading back results
        self.staging_buffer = Some(device.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: frame_size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        
        println!("[OpticalFlowInterpolator] Initialized with dimensions: {}x{}", width, height);
        self.initialized = true;
        
        Ok(())
    }
    
    fn interpolate(&self, frame1: &[u8], frame2: &[u8], t: f32) -> Result<Vec<u8>> {
        if !self.initialized {
            return Err(anyhow!("Interpolator not initialized"));
        }
        
        let frame_size = (self.width * self.height * 4) as usize;
        
        if frame1.len() != frame_size || frame2.len() != frame_size {
            return Err(anyhow!("Frame size mismatch"));
        }
        
        // Step 1: Compute optical flow between the two frames
        let flow = self.compute_optical_flow(frame1, frame2)?;
        
        // Step 2: Warp frames according to the flow and blend them
        let mut output = vec![0u8; frame_size];
        let width = self.width as usize;
        let height = self.height as usize;
        let channels = 4; // RGBA
        
        for y in 0..height {
            for x in 0..width {
                let pixel_idx = (y * width + x) * channels;
                let flow_idx = (y * width + x) * 2;
                
                // Get the flow vector at this position
                let flow_x = flow[flow_idx];
                let flow_y = flow[flow_idx + 1];
                
                // Calculate contribution from frame1 (backward flow)
                let src_x1 = x as f32 + flow_x * t;
                let src_y1 = y as f32 + flow_y * t;
                
                // Calculate contribution from frame2 (forward flow)
                let src_x2 = x as f32 - flow_x * (1.0 - t);
                let src_y2 = y as f32 - flow_y * (1.0 - t);
                
                // Sample from both frames and blend
                let pixel1 = self.sample_frame(frame1, src_x1, src_y1);
                let pixel2 = self.sample_frame(frame2, src_x2, src_y2);
                
                // For each channel, blend the two samples
                for c in 0..channels {
                    // Linear interpolation between the two frames
                    output[pixel_idx + c] = ((1.0 - t) * pixel1[c] as f32 + t * pixel2[c] as f32) as u8;
                }
                
                // Apply sharpening if enabled
                if self.sharpness > 0.0 {
                    // Simple unsharp mask
                    let center = output[pixel_idx + 1] as f32; // Use green channel for luminance
                    let mut sum = 0.0;
                    let mut count = 0.0;
                    
                    // Sample neighborhood
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let nx = x as isize + dx;
                            let ny = y as isize + dy;
                            
                            if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                                let neighbor_idx = (ny as usize * width + nx as usize) * channels;
                                sum += output[neighbor_idx + 1] as f32;
                                count += 1.0;
                            }
                        }
                    }
                    
                    let average = sum / count;
                    let sharpen_amount = self.sharpness * 0.5;
                    
                    // Apply sharpening to all channels
                    for c in 0..3 { // Don't sharpen alpha
                        let value = output[pixel_idx + c] as f32;
                        let sharpened = value + sharpen_amount * (value - average);
                        output[pixel_idx + c] = sharpened.clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }
        
        Ok(output)
    }
    
    fn name(&self) -> &'static str {
        "OpticalFlow"
    }
    
    fn set_quality(&mut self, quality: InterpolationQuality) -> Result<()> {
        self.quality = quality;
        self.initialized = false; // Force reinitialization with new quality settings
        Ok(())
    }
    
    fn quality(&self) -> InterpolationQuality {
        self.quality
    }
}

impl OpticalFlowInterpolator {
    // Helper method to sample a frame at a non-integer position using bilinear interpolation
    fn sample_frame(&self, frame: &[u8], x: f32, y: f32) -> [u8; 4] {
        let width = self.width as usize;
        let height = self.height as usize;
        let channels = 4; // RGBA
        
        // Clamp coordinates to valid range
        let x = x.clamp(0.0, (width - 1) as f32);
        let y = y.clamp(0.0, (height - 1) as f32);
        
        // Integer pixel coordinates
        let x0 = x.floor() as usize;
        let y0 = y.floor() as usize;
        let x1 = (x0 + 1).min(width - 1);
        let y1 = (y0 + 1).min(height - 1);
        
        // Fractional parts for interpolation
        let x_frac = x - x0 as f32;
        let y_frac = y - y0 as f32;
        
        // Get pixel values at the four surrounding integer positions
        let idx00 = (y0 * width + x0) * channels;
        let idx01 = (y0 * width + x1) * channels;
        let idx10 = (y1 * width + x0) * channels;
        let idx11 = (y1 * width + x1) * channels;
        
        let mut result = [0u8; 4];
        
        // Perform bilinear interpolation for each channel
        for c in 0..channels {
            let p00 = frame[idx00 + c] as f32;
            let p01 = frame[idx01 + c] as f32;
            let p10 = frame[idx10 + c] as f32;
            let p11 = frame[idx11 + c] as f32;
            
            // Bilinear interpolation formula
            let top = p00 * (1.0 - x_frac) + p01 * x_frac;
            let bottom = p10 * (1.0 - x_frac) + p11 * x_frac;
            let value = top * (1.0 - y_frac) + bottom * y_frac;
            
            result[c] = value as u8;
        }
        
        result
    }
}

/// Block matching-based frame interpolator (simpler but faster than optical flow)
pub struct BlockMatchingInterpolator {
    /// Frame width
    width: u32,
    /// Frame height
    height: u32,
    /// Interpolation quality
    quality: InterpolationQuality,
    /// Initialized flag
    initialized: bool,
    /// Block size for matching
    block_size: u32,
    /// Search radius
    search_radius: u32,
}

impl BlockMatchingInterpolator {
    /// Create a new block matching interpolator
    pub fn new(quality: InterpolationQuality) -> Self {
        let block_size = match quality {
            InterpolationQuality::High => 8,
            InterpolationQuality::Medium => 16,
            InterpolationQuality::Low => 32,
        };
        
        let search_radius = match quality {
            InterpolationQuality::High => 24,
            InterpolationQuality::Medium => 16,
            InterpolationQuality::Low => 8,
        };
        
        Self {
            width: 0,
            height: 0,
            quality,
            initialized: false,
            block_size,
            search_radius,
        }
    }
    
    /// Calculate the sum of absolute differences (SAD) between two blocks
    fn calculate_sad(&self, frame1: &[u8], frame2: &[u8], x1: usize, y1: usize, x2: usize, y2: usize) -> u32 {
        let width = self.width as usize;
        let channels = 4; // RGBA
        let block_size = self.block_size as usize;
        
        let mut sad = 0u32;
        
        for y in 0..block_size {
            for x in 0..block_size {
                if y1 + y >= self.height as usize || x1 + x >= width || 
                   y2 + y >= self.height as usize || x2 + x >= width {
                    continue;
                }
                
                let idx1 = ((y1 + y) * width + (x1 + x)) * channels;
                let idx2 = ((y2 + y) * width + (x2 + x)) * channels;
                
                // Calculate difference for RGB channels (ignore alpha)
                for c in 0..3 {
                    let diff = (frame1[idx1 + c] as i32 - frame2[idx2 + c] as i32).abs() as u32;
                    sad += diff;
                }
            }
        }
        
        sad
    }
    
    /// Find the best matching block using sum of absolute differences
    fn find_best_match(&self, frame1: &[u8], frame2: &[u8], block_x: usize, block_y: usize) -> (i32, i32) {
        let width = self.width as usize;
        let height = self.height as usize;
        let search_radius = self.search_radius as i32;
        
        let mut best_x = 0;
        let mut best_y = 0;
        let mut min_sad = u32::MAX;
        
        for dy in -search_radius..=search_radius {
            for dx in -search_radius..=search_radius {
                let search_x = block_x as i32 + dx;
                let search_y = block_y as i32 + dy;
                
                if search_x < 0 || search_y < 0 || 
                   search_x + self.block_size as i32 > width as i32 || 
                   search_y + self.block_size as i32 > height as i32 {
                    continue;
                }
                
                let sad = self.calculate_sad(
                    frame1, 
                    frame2, 
                    block_x, 
                    block_y, 
                    search_x as usize, 
                    search_y as usize
                );
                
                if sad < min_sad {
                    min_sad = sad;
                    best_x = dx;
                    best_y = dy;
                }
            }
        }
        
        (best_x, best_y)
    }
    
    /// Sample a frame at non-integer coordinates using bilinear interpolation
    fn sample_frame(&self, frame: &[u8], x: f32, y: f32) -> [u8; 4] {
        let width = self.width as usize;
        let height = self.height as usize;
        let channels = 4; // RGBA
        
        // Clamp coordinates to valid range
        let x = x.clamp(0.0, (width - 1) as f32);
        let y = y.clamp(0.0, (height - 1) as f32);
        
        // Integer pixel coordinates
        let x0 = x.floor() as usize;
        let y0 = y.floor() as usize;
        let x1 = (x0 + 1).min(width - 1);
        let y1 = (y0 + 1).min(height - 1);
        
        // Fractional parts for interpolation
        let x_frac = x - x0 as f32;
        let y_frac = y - y0 as f32;
        
        // Get pixel values at the four surrounding integer positions
        let idx00 = (y0 * width + x0) * channels;
        let idx01 = (y0 * width + x1) * channels;
        let idx10 = (y1 * width + x0) * channels;
        let idx11 = (y1 * width + x1) * channels;
        
        let mut result = [0u8; 4];
        
        // Perform bilinear interpolation for each channel
        for c in 0..channels {
            let p00 = frame[idx00 + c] as f32;
            let p01 = frame[idx01 + c] as f32;
            let p10 = frame[idx10 + c] as f32;
            let p11 = frame[idx11 + c] as f32;
            
            // Bilinear interpolation formula
            let top = p00 * (1.0 - x_frac) + p01 * x_frac;
            let bottom = p10 * (1.0 - x_frac) + p11 * x_frac;
            let value = top * (1.0 - y_frac) + bottom * y_frac;
            
            result[c] = value as u8;
        }
        
        result
    }
}

impl FrameInterpolator for BlockMatchingInterpolator {
    fn initialize(&mut self, width: u32, height: u32) -> Result<()> {
        if self.initialized && self.width == width && self.height == height {
            return Ok(());
        }
        
        self.width = width;
        self.height = height;
        
        println!("[BlockMatchingInterpolator] Initialized with dimensions: {}x{}", width, height);
        self.initialized = true;
        
        Ok(())
    }
    
    fn interpolate(&self, frame1: &[u8], frame2: &[u8], t: f32) -> Result<Vec<u8>> {
        if !self.initialized {
            return Err(anyhow!("Interpolator not initialized"));
        }
        
        let frame_size = (self.width * self.height * 4) as usize;
        
        if frame1.len() != frame_size || frame2.len() != frame_size {
            return Err(anyhow!("Frame size mismatch"));
        }
        
        let width = self.width as usize;
        let height = self.height as usize;
        let channels = 4; // RGBA
        let block_size = self.block_size as usize;
        
        // Calculate motion vectors for each block
        let blocks_x = (width + block_size - 1) / block_size;
        let blocks_y = (height + block_size - 1) / block_size;
        
        // Store motion vectors for each block (x, y)
        let mut motion_vectors = vec![(0, 0); blocks_x * blocks_y];
        
        // Step 1: Calculate motion vectors
        for by in 0..blocks_y {
            for bx in 0..blocks_x {
                let block_index = by * blocks_x + bx;
                let block_x = bx * block_size;
                let block_y = by * block_size;
                
                // Find best matching block in the second frame
                let (mv_x, mv_y) = self.find_best_match(frame1, frame2, block_x, block_y);
                motion_vectors[block_index] = (mv_x, mv_y);
            }
        }
        
        // Step 2: Generate interpolated frame using motion vectors
        let mut output = vec![0u8; frame_size];
        
        for y in 0..height {
            for x in 0..width {
                let pixel_idx = (y * width + x) * channels;
                
                // Find which block this pixel belongs to
                let block_x = x / block_size;
                let block_y = y / block_size;
                let block_index = block_y * blocks_x + block_x;
                
                // Get motion vector for this block
                let (mv_x, mv_y) = motion_vectors[block_index];
                
                // Calculate pixel position in both frames with motion compensation
                let src_x1 = x as f32 + mv_x as f32 * t;
                let src_y1 = y as f32 + mv_y as f32 * t;
                
                let src_x2 = x as f32 - mv_x as f32 * (1.0 - t);
                let src_y2 = y as f32 - mv_y as f32 * (1.0 - t);
                
                // Sample from both frames
                let pixel1 = self.sample_frame(frame1, src_x1, src_y1);
                let pixel2 = self.sample_frame(frame2, src_x2, src_y2);
                
                // Blend samples based on time parameter
                for c in 0..channels {
                    output[pixel_idx + c] = ((1.0 - t) * pixel1[c] as f32 + t * pixel2[c] as f32) as u8;
                }
            }
        }
        
        // Step 3: Apply occlusion handling - detect areas with inconsistent motion
        if let Some(processed_output) = self.handle_occlusions(frame1, frame2, &output, &motion_vectors, blocks_x, blocks_y, t) {
            return Ok(processed_output);
        }
        
        Ok(output)
    }
    
    fn name(&self) -> &'static str {
        "BlockMatching"
    }
    
    fn set_quality(&mut self, quality: InterpolationQuality) -> Result<()> {
        self.quality = quality;
        
        // Update block size and search radius based on new quality
        self.block_size = match quality {
            InterpolationQuality::High => 8,
            InterpolationQuality::Medium => 16,
            InterpolationQuality::Low => 32,
        };
        
        self.search_radius = match quality {
            InterpolationQuality::High => 24,
            InterpolationQuality::Medium => 16,
            InterpolationQuality::Low => 8,
        };
        
        self.initialized = false; // Force reinitialization with new settings
        
        Ok(())
    }
    
    fn quality(&self) -> InterpolationQuality {
        self.quality
    }
}

impl BlockMatchingInterpolator {
    /// Handle occlusions by detecting areas with inconsistent motion
    fn handle_occlusions(&self, frame1: &[u8], frame2: &[u8], output: &[u8], motion_vectors: &[(i32, i32)], 
                        blocks_x: usize, blocks_y: usize, t: f32) -> Option<Vec<u8>> {
        // This is a simplified implementation of occlusion handling
        // A more sophisticated approach would involve bidirectional motion estimation
        // and consistency checks between forward and backward motion vectors
        
        let width = self.width as usize;
        let height = self.height as usize;
        let channels = 4; // RGBA
        let block_size = self.block_size as usize;
        
        // Check if motion is smooth - if so, no need for special handling
        let mut is_motion_smooth = true;
        
        for by in 1..blocks_y {
            for bx in 1..blocks_x {
                let current_idx = by * blocks_x + bx;
                let left_idx = by * blocks_x + (bx - 1);
                let top_idx = (by - 1) * blocks_x + bx;
                
                let (cur_x, cur_y) = motion_vectors[current_idx];
                let (left_x, left_y) = motion_vectors[left_idx];
                let (top_x, top_y) = motion_vectors[top_idx];
                
                // Calculate motion difference
                let diff_left = ((cur_x - left_x).abs() + (cur_y - left_y).abs()) as f32;
                let diff_top = ((cur_x - top_x).abs() + (cur_y - top_y).abs()) as f32;
                
                // If motion difference is large, we have potential occlusions
                if diff_left > 10.0 || diff_top > 10.0 {
                    is_motion_smooth = false;
                    break;
                }
            }
            if !is_motion_smooth {
                break;
            }
        }
        
        // If motion is smooth enough, no special handling needed
        if is_motion_smooth {
            return None;
        }
        
        // Handle occlusions by blending based on motion confidence
        let mut result = output.to_vec();
        
        for by in 0..blocks_y {
            for bx in 0..blocks_x {
                let block_index = by * blocks_x + bx;
                let (mv_x, mv_y) = motion_vectors[block_index];
                
                // Calculate a confidence value for this block
                // Blocks with very different motion from neighbors get lower confidence
                let mut confidence = 1.0;
                
                if bx > 0 && by > 0 && bx < blocks_x - 1 && by < blocks_y - 1 {
                    let mut diff_sum = 0.0;
                    let mut neighbor_count = 0.0;
                    
                    // Check neighboring blocks
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            
                            let nx = bx as isize + dx;
                            let ny = by as isize + dy;
                            
                            if nx >= 0 && nx < blocks_x as isize && ny >= 0 && ny < blocks_y as isize {
                                let neighbor_idx = (ny as usize * blocks_x + nx as usize);
                                let (n_x, n_y) = motion_vectors[neighbor_idx];
                                
                                let diff = ((mv_x - n_x).abs() + (mv_y - n_y).abs()) as f32;
                                diff_sum += diff;
                                neighbor_count += 1.0;
                            }
                        }
                    }
                    
                    if neighbor_count > 0.0 {
                        let avg_diff = diff_sum / neighbor_count;
                        // Reduce confidence for blocks with high motion difference
                        confidence = 1.0 / (1.0 + avg_diff * 0.1);
                    }
                }
                
                // Apply confidence-based blending for this block
                for y in 0..block_size {
                    for x in 0..block_size {
                        let px = bx * block_size + x;
                        let py = by * block_size + y;
                        
                        if px >= width || py >= height {
                            continue;
                        }
                        
                        let pixel_idx = (py * width + px) * channels;
                        
                        // For low confidence areas, blend more from original frames
                        // rather than motion-compensated samples
                        if confidence < 0.7 {
                            // Direct blend between original frames for low confidence areas
                            for c in 0..channels {
                                result[pixel_idx + c] = ((1.0 - t) * frame1[pixel_idx + c] as f32 + 
                                                        t * frame2[pixel_idx + c] as f32) as u8;
                            }
                        }
                    }
                }
            }
        }
        
        Some(result)
    }
}

/// Factory for creating interpolators
pub struct InterpolatorFactory;

impl InterpolatorFactory {
    /// Create an interpolator
    pub fn create_interpolator(method: MotionEstimationMethod, quality: InterpolationQuality) -> Box<dyn FrameInterpolator> {
        match method {
            MotionEstimationMethod::OpticalFlow => Box::new(OpticalFlowInterpolator::new(quality)),
            MotionEstimationMethod::BlockMatching => Box::new(BlockMatchingInterpolator::new(quality)),
            MotionEstimationMethod::Simplified => Box::new(BlockMatchingInterpolator::new(quality)),
        }
    }
    
    /// Create the best interpolator for the current system
    pub fn create_best_interpolator(quality: InterpolationQuality) -> Box<dyn FrameInterpolator> {
        // For now, use optical flow as best method
        Box::new(OpticalFlowInterpolator::new(quality))
    }
}

// These functions will be exposed to Python
pub mod python {
    use super::*;
    use pyo3::prelude::*;
    use pyo3::types::PyBytes;
    
    /// Python wrapper for interpolation quality
    #[pyclass]
    #[derive(Clone, Copy)]
    pub enum PyInterpolationQuality {
        High,
        Medium,
        Low,
    }
    
    /// Python wrapper for motion estimation method
    #[pyclass]
    #[derive(Clone, Copy)]
    pub enum PyMotionEstimationMethod {
        OpticalFlow,
        BlockMatching,
        Simplified,
    }
    
    /// Python wrapper for frame interpolator
    #[pyclass]
    pub struct PyFrameInterpolator {
        inner: Box<dyn FrameInterpolator + Send>,
        frame_buffer: Option<Vec<u8>>,
    }
    
    #[pymethods]
    impl PyFrameInterpolator {
        #[new]
        #[pyo3(signature = (method = "optical_flow", quality = "medium"))]
        pub fn new(method: &str, quality: &str) -> PyResult<Self> {
            let method = match method.to_lowercase().as_str() {
                "optical_flow" => MotionEstimationMethod::OpticalFlow,
                "block_matching" => MotionEstimationMethod::BlockMatching,
                "simplified" => MotionEstimationMethod::Simplified,
                _ => MotionEstimationMethod::OpticalFlow,
            };
            
            let quality = match quality.to_lowercase().as_str() {
                "high" => InterpolationQuality::High,
                "medium" => InterpolationQuality::Medium,
                "low" => InterpolationQuality::Low,
                _ => InterpolationQuality::Medium,
            };
            
            Ok(Self {
                inner: InterpolatorFactory::create_interpolator(method, quality),
                frame_buffer: None,
            })
        }
        
        #[staticmethod]
        pub fn create_best_interpolator(quality: &str) -> PyResult<Self> {
            let quality = match quality.to_lowercase().as_str() {
                "high" => InterpolationQuality::High,
                "medium" => InterpolationQuality::Medium,
                "low" => InterpolationQuality::Low,
                _ => InterpolationQuality::Medium,
            };
            
            Ok(Self {
                inner: InterpolatorFactory::create_best_interpolator(quality),
                frame_buffer: None,
            })
        }
        
        pub fn initialize(&mut self, width: u32, height: u32) -> PyResult<()> {
            match self.inner.initialize(width, height) {
                Ok(()) => Ok(()),
                Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
            }
        }
        
        pub fn interpolate<'py>(&mut self, py: Python<'py>, frame1: &[u8], frame2: &[u8], t: f32) -> PyResult<&'py PyBytes> {
            match self.inner.interpolate(frame1, frame2, t) {
                Ok(result) => {
                    self.frame_buffer = Some(result.clone());
                    Ok(PyBytes::new(py, &result))
                },
                Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
            }
        }
        
        #[getter]
        pub fn name(&self) -> &'static str {
            self.inner.name()
        }
        
        #[getter]
        pub fn quality(&self) -> String {
            match self.inner.quality() {
                InterpolationQuality::High => "high".to_string(),
                InterpolationQuality::Medium => "medium".to_string(),
                InterpolationQuality::Low => "low".to_string(),
            }
        }
        
        #[setter]
        pub fn set_quality(&mut self, quality: &str) -> PyResult<()> {
            let q = match quality.to_lowercase().as_str() {
                "high" => InterpolationQuality::High,
                "medium" => InterpolationQuality::Medium,
                "low" => InterpolationQuality::Low,
                _ => return Err(pyo3::exceptions::PyValueError::new_err("Invalid quality setting")),
            };
            
            match self.inner.set_quality(q) {
                Ok(()) => Ok(()),
                Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
            }
        }
    }
} 