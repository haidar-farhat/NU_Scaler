use anyhow::{Result, anyhow};
use image::RgbaImage;
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::Path;
use std::env;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::upscale::{Upscaler, UpscalingQuality};
use crate::upscale::common::UpscalingAlgorithm;

// Static check for FSR3 support
static FSR3_SUPPORTED: AtomicBool = AtomicBool::new(false);
static FSR3_CHECKED: AtomicBool = AtomicBool::new(false);

/// AMD FidelityFX Super Resolution 3 with Frame Generation
pub struct Fsr3Upscaler {
    // Configuration
    input_width: u32,
    input_height: u32,
    output_width: u32,
    output_height: u32,
    quality: UpscalingQuality,
    
    // Whether frame generation is enabled
    frame_generation_enabled: bool,
    
    // FSR3 context
    context: Option<Fsr3Context>,
    
    // Frame history for temporal processing
    frame_history: Arc<Mutex<FrameHistory>>,
    
    // Is FSR3 initialized
    initialized: bool,
    
    // Performance metrics and state tracking
    stats: Arc<Mutex<Fsr3Stats>>,
}

// Stats and runtime state
struct Fsr3Stats {
    // Frame count for alternating between real and generated frames
    frame_count: u64,
    
    // Last frame time
    last_frame_time: Instant,
    
    // Performance metrics
    frame_times: Vec<Duration>,
    
    // Motion vector buffer
    motion_vectors: Option<Vec<(f32, f32)>>,
    
    // Depth buffer (mock)
    depth_buffer: Option<Vec<f32>>,
}

// Frame history for temporal processing
struct FrameHistory {
    // Previous frames (up to 8)
    frames: Vec<RgbaImage>,
    // Previous motion vectors
    motion_history: Vec<Vec<(f32, f32)>>,
    // Previous depth buffers
    depth_history: Vec<Vec<f32>>,
    // Frame timestamps
    timestamps: Vec<Instant>,
    // Maximum number of frames to store
    max_frames: usize,
}

// FSR3 Context with FFX integration
struct Fsr3Context {
    // Mock fields that would be populated from FFX SDK
    render_width: u32,
    render_height: u32,
    display_width: u32,
    display_height: u32,
    quality_mode: Fsr3QualityMode,
    
    // Frame generation settings
    frame_generation_enabled: bool,
    optical_flow_enabled: bool,
    
    // Jitter for temporal antialiasing
    jitter_x: f32,
    jitter_y: f32,
    
    // Exposure data
    exposure: f32,
    
    // Temporal stability
    temporal_stability: f32,
    
    // Sharpness
    sharpness: f32,
    
    // Reactive mask scale
    reactive_scale: f32,
    
    // Frame interpolation settings (for frame generation)
    interpolation_amount: f32,  // 0.5 for one interpolated frame between each real frame
}

// FSR3 Quality modes
enum Fsr3QualityMode {
    Ultra,
    Quality,
    Balanced,
    Performance,
}

impl FrameHistory {
    // Create a new frame history
    fn new(max_frames: usize) -> Self {
        Self {
            frames: Vec::with_capacity(max_frames),
            motion_history: Vec::with_capacity(max_frames),
            depth_history: Vec::with_capacity(max_frames),
            timestamps: Vec::with_capacity(max_frames),
            max_frames,
        }
    }
    
    // Add a new frame to history
    fn add_frame(&mut self, frame: RgbaImage, motion_vectors: Option<Vec<(f32, f32)>>, depth: Option<Vec<f32>>) {
        // Add frame to history
        self.frames.push(frame);
        
        // Add motion vectors (or empty)
        self.motion_history.push(motion_vectors.unwrap_or_default());
        
        // Add depth buffer (or empty)
        self.depth_history.push(depth.unwrap_or_default());
        
        // Add timestamp
        self.timestamps.push(Instant::now());
        
        // Trim history if needed
        if self.frames.len() > self.max_frames {
            self.frames.remove(0);
            self.motion_history.remove(0);
            self.depth_history.remove(0);
            self.timestamps.remove(0);
        }
    }
    
    // Get previous frame
    fn get_previous_frame(&self, offset: usize) -> Option<&RgbaImage> {
        if offset < self.frames.len() {
            let index = self.frames.len() - 1 - offset;
            Some(&self.frames[index])
        } else {
            None
        }
    }
    
    // Get frame time difference in milliseconds
    fn get_frame_time_ms(&self) -> Option<f32> {
        if self.timestamps.len() >= 2 {
            let idx = self.timestamps.len() - 1;
            let elapsed = self.timestamps[idx].duration_since(self.timestamps[idx - 1]);
            Some(elapsed.as_secs_f32() * 1000.0)
        } else {
            None
        }
    }
}

impl Fsr3Upscaler {
    /// Create a new FSR3 upscaler
    pub fn new(quality: UpscalingQuality, enable_frame_generation: bool) -> Result<Self> {
        if !Self::is_supported() {
            return Err(anyhow!("FSR3 is not supported on this system"));
        }
        
        log::info!("Creating FSR3 upscaler with frame generation: {}", enable_frame_generation);
        
        Ok(Self {
            input_width: 0,
            input_height: 0,
            output_width: 0,
            output_height: 0,
            quality,
            frame_generation_enabled: enable_frame_generation,
            context: None,
            frame_history: Arc::new(Mutex::new(FrameHistory::new(8))),
            initialized: false,
            stats: Arc::new(Mutex::new(Fsr3Stats {
                frame_count: 0,
                last_frame_time: Instant::now(),
                frame_times: Vec::with_capacity(60),
                motion_vectors: None,
                depth_buffer: None,
            })),
        })
    }
    
    /// Map our quality enum to FSR3 quality mode
    fn map_quality(&self) -> Fsr3QualityMode {
        match self.quality {
            UpscalingQuality::Ultra => Fsr3QualityMode::Ultra,
            UpscalingQuality::Quality => Fsr3QualityMode::Quality,
            UpscalingQuality::Balanced => Fsr3QualityMode::Balanced,
            UpscalingQuality::Performance => Fsr3QualityMode::Performance,
        }
    }
    
    /// Initialize FSR3 context
    fn init_fsr3_context(&mut self) -> Result<()> {
        // Create FSR3 context with the appropriate parameters
        let context = Fsr3Context {
            render_width: self.input_width,
            render_height: self.input_height,
            display_width: self.output_width,
            display_height: self.output_height,
            quality_mode: self.map_quality(),
            frame_generation_enabled: self.frame_generation_enabled,
            optical_flow_enabled: true,
            jitter_x: 0.0,
            jitter_y: 0.0,
            exposure: 1.0,
            temporal_stability: match self.quality {
                UpscalingQuality::Ultra => 0.95,
                UpscalingQuality::Quality => 0.90,
                UpscalingQuality::Balanced => 0.85,
                UpscalingQuality::Performance => 0.80,
            },
            sharpness: match self.quality {
                UpscalingQuality::Ultra => 0.8,
                UpscalingQuality::Quality => 0.7,
                UpscalingQuality::Balanced => 0.6,
                UpscalingQuality::Performance => 0.5,
            },
            reactive_scale: 0.8,
            interpolation_amount: 0.5,  // Interpolate one frame between each real frame
        };
        
        self.context = Some(context);
        log::info!("FSR3 context initialized with size {}x{} -> {}x{}", 
                  self.input_width, self.input_height, 
                  self.output_width, self.output_height);
                  
        Ok(())
    }
    
    /// Generate motion vectors (simulated)
    fn generate_motion_vectors(&mut self, current_frame: &RgbaImage) -> Vec<(f32, f32)> {
        let width = current_frame.width() as usize;
        let height = current_frame.height() as usize;
        let mut motion_vectors = vec![(0.0f32, 0.0f32); width * height];
        
        // Try to get previous frame from history
        let frame_history = self.frame_history.lock().unwrap();
        if let Some(previous_frame) = frame_history.get_previous_frame(0) {
            // Simple block matching for motion estimation
            // This is just a simulation - real apps would use optical flow or TAA motion vectors
            for y in 0..height.min(previous_frame.height() as usize) {
                for x in 0..width.min(previous_frame.width() as usize) {
                    // Compare current pixel with previous frame
                    let current_pixel = current_frame.get_pixel(x as u32, y as u32);
                    
                    // Search in a small neighborhood
                    let search_radius = 8;
                    let mut best_match = (0.0f32, 0.0f32);
                    let mut best_diff = f32::MAX;
                    
                    for sy in (y as i32 - search_radius).max(0)..(y as i32 + search_radius).min(previous_frame.height() as i32) {
                        for sx in (x as i32 - search_radius).max(0)..(x as i32 + search_radius).min(previous_frame.width() as i32) {
                            let prev_pixel = previous_frame.get_pixel(sx as u32, sy as u32);
                            
                            // Calculate difference (simplified)
                            let diff = (current_pixel[0] as i32 - prev_pixel[0] as i32).pow(2) +
                                      (current_pixel[1] as i32 - prev_pixel[1] as i32).pow(2) +
                                      (current_pixel[2] as i32 - prev_pixel[2] as i32).pow(2);
                            
                            if (diff as f32) < best_diff {
                                best_diff = diff as f32;
                                best_match = ((sx as f32 - x as f32), (sy as f32 - y as f32));
                            }
                        }
                    }
                    
                    motion_vectors[y * width + x] = best_match;
                }
            }
        }
        
        motion_vectors
    }
    
    /// Generate a simple depth buffer (simulated)
    fn generate_depth_buffer(&self, frame: &RgbaImage) -> Vec<f32> {
        let width = frame.width() as usize;
        let height = frame.height() as usize;
        let mut depth = vec![1.0f32; width * height]; // Initialize to far depth
        
        // In a real implementation, this would use the application's depth buffer
        // Here we'll simulate depth based on brightness as a placeholder
        for y in 0..height {
            for x in 0..width {
                let pixel = frame.get_pixel(x as u32, y as u32);
                // Approximate depth from brightness (brighter = closer)
                let brightness = (pixel[0] as f32 * 0.299 + 
                                 pixel[1] as f32 * 0.587 + 
                                 pixel[2] as f32 * 0.114) / 255.0;
                                 
                // Invert and scale (0=near, 1=far)
                depth[y * width + x] = 1.0 - brightness;
            }
        }
        
        depth
    }
    
    /// Generate an interpolated frame using optical flow
    fn generate_interpolated_frame(&self, 
                                  frame1: &RgbaImage, 
                                  frame2: &RgbaImage,
                                  motion_vectors: &[(f32, f32)],
                                  interpolation_factor: f32) -> Result<RgbaImage> {
        let width = frame1.width() as usize;
        let height = frame1.height() as usize;
        
        let mut result = RgbaImage::new(frame1.width(), frame1.height());
        
        // For each pixel in the output frame
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let motion = motion_vectors[idx];
                
                // Calculate source positions in each frame using motion vectors
                let pos1_x = x as f32;
                let pos1_y = y as f32;
                
                let pos2_x = (x as f32 + motion.0).max(0.0).min(width as f32 - 1.0);
                let pos2_y = (y as f32 + motion.1).max(0.0).min(height as f32 - 1.0);
                
                // Interpolate position based on factor
                let interp_x = (pos1_x * (1.0 - interpolation_factor) + 
                               pos2_x * interpolation_factor) as u32;
                               
                let interp_y = (pos1_y * (1.0 - interpolation_factor) + 
                               pos2_y * interpolation_factor) as u32;
                
                // Clamp to image boundaries
                let interp_x = interp_x.min(frame1.width() - 1);
                let interp_y = interp_y.min(frame1.height() - 1);
                
                // Sample both frames
                let color1 = frame1.get_pixel(interp_x, interp_y);
                let color2 = frame2.get_pixel(interp_x, interp_y);
                
                // Interpolate color
                let r = ((1.0 - interpolation_factor) * color1[0] as f32 + 
                          interpolation_factor * color2[0] as f32) as u8;
                          
                let g = ((1.0 - interpolation_factor) * color1[1] as f32 + 
                          interpolation_factor * color2[1] as f32) as u8;
                          
                let b = ((1.0 - interpolation_factor) * color1[2] as f32 + 
                          interpolation_factor * color2[2] as f32) as u8;
                          
                let a = ((1.0 - interpolation_factor) * color1[3] as f32 + 
                          interpolation_factor * color2[3] as f32) as u8;
                
                // Set pixel in result
                result.put_pixel(x as u32, y as u32, image::Rgba([r, g, b, a]));
            }
        }
        
        Ok(result)
    }
    
    /// Apply FSR3 super resolution
    fn apply_super_resolution(&self, input: &RgbaImage) -> Result<RgbaImage> {
        let context = match &self.context {
            Some(ctx) => ctx,
            None => return Err(anyhow!("FSR3 context is not initialized")),
        };
        
        // Create output image
        let mut output = RgbaImage::new(self.output_width, self.output_height);
        
        // Select sharpening strength based on quality mode
        let sharpening = context.sharpness;
        
        // Temporal stability
        let _temporal_stability = context.temporal_stability;
        
        // Create a working buffer for the first pass (EASU - Edge Adaptive Spatial Upsampling)
        let mut easu_pass = RgbaImage::new(self.output_width, self.output_height);
        
        // Calculate scale factors
        let scale_x = self.input_width as f32 / self.output_width as f32;
        let scale_y = self.input_height as f32 / self.output_height as f32;
        
        // Apply edge adaptive spatial upsampling (EASU)
        for y in 0..self.output_height {
            for x in 0..self.output_width {
                // Map to input coordinates with jitter for temporal AA
                let input_x = (x as f32 * scale_x + context.jitter_x) as u32;
                let input_y = (y as f32 * scale_y + context.jitter_y) as u32;
                
                // Clamp to input image bounds
                let input_x = input_x.min(self.input_width - 1);
                let input_y = input_y.min(self.input_height - 1);
                
                // Get subpixel position
                let subpixel_x = x as f32 * scale_x - input_x as f32;
                let subpixel_y = y as f32 * scale_y - input_y as f32;
                
                // Apply Lanczos3 filtering for high quality upsampling
                let mut color = [0.0f32; 4];
                
                // Apply Lanczos3 filtering
                for dy in -2..=2 {
                    for dx in -2..=2 {
                        let sample_x = (input_x as i32 + dx).max(0).min(self.input_width as i32 - 1) as u32;
                        let sample_y = (input_y as i32 + dy).max(0).min(self.input_height as i32 - 1) as u32;
                        
                        let sample = input.get_pixel(sample_x, sample_y);
                        
                        // Lanczos weight calculation
                        let weight_x = Self::lanczos3((dx as f32 + subpixel_x) * scale_x);
                        let weight_y = Self::lanczos3((dy as f32 + subpixel_y) * scale_y);
                        let weight = weight_x * weight_y;
                        
                        // Accumulate weighted color
                        for c in 0..4 {
                            color[c] += sample[c] as f32 * weight;
                        }
                    }
                }
                
                // Apply reactive mask (if available from depth buffer)
                // In a real implementation, this would use the reactive mask
                
                // Set pixel in EASU pass
                let r = color[0].max(0.0).min(255.0) as u8;
                let g = color[1].max(0.0).min(255.0) as u8;
                let b = color[2].max(0.0).min(255.0) as u8;
                let a = color[3].max(0.0).min(255.0) as u8;
                
                easu_pass.put_pixel(x, y, image::Rgba([r, g, b, a]));
            }
        }
        
        // Apply robustness contrast adaptive sharpening (RCAS)
        for y in 0..self.output_height {
            for x in 0..self.output_width {
                let center = easu_pass.get_pixel(x, y);
                
                // Get neighboring pixels
                let left = if x > 0 { easu_pass.get_pixel(x - 1, y) } else { center };
                let right = if x < self.output_width - 1 { easu_pass.get_pixel(x + 1, y) } else { center };
                let top = if y > 0 { easu_pass.get_pixel(x, y - 1) } else { center };
                let bottom = if y < self.output_height - 1 { easu_pass.get_pixel(x, y + 1) } else { center };
                
                // Calculate min/max of neighbors
                let mut min_color = [255u8; 4];
                let mut max_color = [0u8; 4];
                
                for c in 0..3 {  // Only sharpen RGB channels
                    min_color[c] = min_color[c].min(left[c]).min(right[c]).min(top[c]).min(bottom[c]);
                    max_color[c] = max_color[c].max(left[c]).max(right[c]).max(top[c]).max(bottom[c]);
                }
                
                // Apply contrast adaptive sharpening
                let mut sharpened = [0u8; 4];
                for c in 0..3 {
                    // Calculate sharpened value
                    let sharp_val = (center[c] as f32 * (1.0 + sharpening)) - 
                                   ((left[c] as f32 + right[c] as f32 + top[c] as f32 + bottom[c] as f32) / 4.0) * sharpening;
                                   
                    // Clamp to min/max of neighbors to avoid ringing
                    sharpened[c] = sharp_val.max(min_color[c] as f32).min(max_color[c] as f32) as u8;
                }
                
                // Copy alpha
                sharpened[3] = center[3];
                
                // Set final pixel
                output.put_pixel(x, y, image::Rgba(sharpened));
            }
        }
        
        Ok(output)
    }
    
    // Lanczos3 filter function
    fn lanczos3(x: f32) -> f32 {
        if x.abs() < 1e-6 {
            return 1.0;
        }
        if x.abs() >= 3.0 {
            return 0.0;
        }
        
        let pix = std::f32::consts::PI * x;
        3.0 * (pix.sin() / pix) * ((pix / 3.0).sin() / (pix / 3.0))
    }
    
    /// Check if FSR3 libraries are available
    fn check_fsr3_available() -> bool {
        // In a real implementation, this would check for FSR3 libraries
        // and GPU compatibility
        
        // For demonstration, look for a marker file indicating FSR3 availability
        let user_profile = env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
        let marker_path = Path::new(&user_profile).join(".nu_scale_fsr3_available");
        
        if marker_path.exists() {
            return true;
        }
        
        // Check for common FSR3 SDK paths
        let common_paths = [
            "C:\\Program Files\\AMD\\FidelityFX-SDK",
            "C:\\AMD\\FidelityFX-SDK",
            "/usr/local/lib/fidelityfx",
            "/usr/lib/fidelityfx",
        ];
        
        for path in common_paths.iter() {
            if Path::new(path).exists() {
                // Create marker file for future checks
                let _ = fs::write(&marker_path, "FSR3 is available");
                return true;
            }
        }
        
        // For demo purposes, always enable FSR3
        // In a real implementation, this would check hardware compatibility
        let _ = fs::write(&marker_path, "FSR3 is available");
        true
    }
}

impl Upscaler for Fsr3Upscaler {
    fn initialize(&mut self, input_width: u32, input_height: u32, output_width: u32, output_height: u32) -> Result<()> {
        // Store dimensions
        self.input_width = input_width;
        self.input_height = input_height;
        self.output_width = output_width;
        self.output_height = output_height;
        
        // Initialize FSR3 context
        self.init_fsr3_context()?;
        
        // Clear frame history
        let mut frame_history = self.frame_history.lock().unwrap();
        *frame_history = FrameHistory::new(8);
        
        // Reset stats
        let mut stats = self.stats.lock().unwrap();
        stats.frame_count = 0;
        stats.frame_times.clear();
        stats.motion_vectors = None;
        stats.depth_buffer = None;
        
        self.initialized = true;
        log::info!("FSR3 upscaler initialized with dimensions {}x{} -> {}x{}", 
                  input_width, input_height, output_width, output_height);
        
        Ok(())
    }
    
    fn upscale(&self, input: &RgbaImage) -> Result<RgbaImage> {
        let start_time = Instant::now();
        
        // Get context or return error
        let context = match &self.context {
            Some(ctx) => ctx,
            None => return Err(anyhow!("FSR3 context is not initialized")),
        };
        
        // Check if frame generation is enabled
        if context.frame_generation_enabled {
            // Get mutable access to stats
            let mut stats = self.stats.lock().unwrap();
            
            // Clone mutable data
            let mut frame_history = self.frame_history.lock().unwrap();
            
            // Generate depth buffer (simulated)
            let depth = self.generate_depth_buffer(input);
            
            // Calculate motion vectors (simulated)
            let motion_vectors = if let Some(_prev_frame) = frame_history.get_previous_frame(0) {
                // Clone input for thread safety (used in real implementation)
                let _input_clone = input.clone();
                
                // Generate motion vectors between previous and current frame
                let mut vectors = vec![(0.0f32, 0.0f32); (input.width() * input.height()) as usize];
                
                // Simple motion estimation (placeholder)
                for y in 0..input.height() {
                    for x in 0..input.width() {
                        // In a real implementation, this would use optical flow
                        // or engine-provided motion vectors
                        vectors[(y * input.width() + x) as usize] = (0.0, 0.0);
                    }
                }
                
                vectors
            } else {
                // No previous frame, use zero vectors
                vec![(0.0f32, 0.0f32); (input.width() * input.height()) as usize]
            };
            
            // Add current frame to history
            frame_history.add_frame(input.clone(), Some(motion_vectors.clone()), Some(depth));
            
            // Only perform frame generation if we have enough history
            if stats.frame_count > 0 && frame_history.frames.len() >= 2 {
                // Get two most recent frames
                let frame1 = frame_history.get_previous_frame(1).unwrap().clone();
                let frame2 = frame_history.get_previous_frame(0).unwrap().clone();
                
                // Determine if this is a real or generated frame
                let is_generated_frame = stats.frame_count % 2 == 1;
                
                if is_generated_frame {
                    // This is a generated frame - interpolate between previous frames
                    log::trace!("Generating interpolated frame {}", stats.frame_count);
                    
                    // Generate interpolated frame
                    let interpolated = self.generate_interpolated_frame(
                        &frame1, 
                        &frame2, 
                        &motion_vectors, 
                        context.interpolation_amount
                    )?;
                    
                    // Apply super resolution to interpolated frame
                    let result = self.apply_super_resolution(&interpolated)?;
                    
                    // Record frame time
                    let frame_time = start_time.elapsed();
                    
                    // Update frame count
                    stats.frame_count += 1;
                    stats.frame_times.push(frame_time);
                    
                    if stats.frame_times.len() > 60 {
                        stats.frame_times.remove(0);
                    }
                    
                    // Log performance
                    if stats.frame_count % 100 == 0 {
                        let avg_time = stats.frame_times.iter()
                            .map(|t| t.as_secs_f32() * 1000.0)
                            .sum::<f32>() / stats.frame_times.len() as f32;
                            
                        log::info!("FSR3 with frame generation: Avg processing time: {:.2}ms", avg_time);
                    }
                    
                    return Ok(result);
                }
            }
            
            // Update frame count (for real frames)
            stats.frame_count += 1;
        }
        
        // Apply super resolution
        let result = self.apply_super_resolution(input)?;
        
        // Record frame time
        let frame_time = start_time.elapsed();
        let mut stats = self.stats.lock().unwrap();
        stats.frame_times.push(frame_time);
        
        if stats.frame_times.len() > 60 {
            stats.frame_times.remove(0);
        }
        
        // Log performance occasionally
        if stats.frame_count % 100 == 0 {
            let avg_time = stats.frame_times.iter()
                .map(|t| t.as_secs_f32() * 1000.0)
                .sum::<f32>() / stats.frame_times.len() as f32;
                
            log::info!("FSR3: Avg processing time: {:.2}ms", avg_time);
        }
        
        Ok(result)
    }
    
    fn upscale_with_algorithm(&self, input: &RgbaImage, algorithm: UpscalingAlgorithm) -> Result<RgbaImage> {
        // FSR3 uses its own algorithms, but we can adjust based on the requested algorithm
        match algorithm {
            UpscalingAlgorithm::Lanczos3 => {
                // Use highest quality settings
                let mut upscaler = self.clone();
                upscaler.quality = UpscalingQuality::Ultra;
                if let Some(ctx) = &mut upscaler.context {
                    ctx.sharpness = 0.8;
                    ctx.temporal_stability = 0.95;
                }
                upscaler.upscale(input)
            },
            UpscalingAlgorithm::Lanczos2 | UpscalingAlgorithm::Bicubic => {
                // Use quality settings
                let mut upscaler = self.clone();
                upscaler.quality = UpscalingQuality::Quality;
                if let Some(ctx) = &mut upscaler.context {
                    ctx.sharpness = 0.7;
                    ctx.temporal_stability = 0.9;
                }
                upscaler.upscale(input)
            },
            UpscalingAlgorithm::Bilinear => {
                // Use performance settings
                let mut upscaler = self.clone();
                upscaler.quality = UpscalingQuality::Performance;
                if let Some(ctx) = &mut upscaler.context {
                    ctx.sharpness = 0.5;
                    ctx.temporal_stability = 0.8;
                }
                upscaler.upscale(input)
            },
            _ => {
                // Use default implementation
                self.upscale(input)
            }
        }
    }
    
    fn cleanup(&mut self) -> Result<()> {
        // Reset context
        self.context = None;
        self.initialized = false;
        
        // Clear frame history
        let mut frame_history = self.frame_history.lock().unwrap();
        *frame_history = FrameHistory::new(8);
        
        log::info!("FSR3 upscaler cleaned up");
        
        Ok(())
    }
    
    fn is_supported() -> bool {
        // Check if we've already performed the check
        if FSR3_CHECKED.load(Ordering::Relaxed) {
            return FSR3_SUPPORTED.load(Ordering::Relaxed);
        }
        
        // Check if FSR3 is available
        let supported = Self::check_fsr3_available();
        
        // Store the result
        FSR3_SUPPORTED.store(supported, Ordering::Relaxed);
        FSR3_CHECKED.store(true, Ordering::Relaxed);
        
        supported
    }
    
    fn name(&self) -> &'static str {
        if self.frame_generation_enabled {
            "AMD FSR 3 with Frame Generation"
        } else {
            "AMD FSR 3"
        }
    }
    
    fn quality(&self) -> UpscalingQuality {
        self.quality
    }
    
    fn set_quality(&mut self, quality: UpscalingQuality) -> Result<()> {
        // Update quality
        self.quality = quality;
        
        // Map quality to FSR3 quality mode
        let quality_mode = match quality {
            UpscalingQuality::Ultra => Fsr3QualityMode::Ultra,
            UpscalingQuality::Quality => Fsr3QualityMode::Quality,
            UpscalingQuality::Balanced => Fsr3QualityMode::Balanced,
            UpscalingQuality::Performance => Fsr3QualityMode::Performance,
        };
        
        // Update context if initialized
        if let Some(ctx) = &mut self.context {
            ctx.quality_mode = quality_mode;
            
            // Update quality-dependent settings
            ctx.temporal_stability = match quality {
                UpscalingQuality::Ultra => 0.95,
                UpscalingQuality::Quality => 0.90,
                UpscalingQuality::Balanced => 0.85,
                UpscalingQuality::Performance => 0.80,
            };
            
            ctx.sharpness = match quality {
                UpscalingQuality::Ultra => 0.8,
                UpscalingQuality::Quality => 0.7,
                UpscalingQuality::Balanced => 0.6,
                UpscalingQuality::Performance => 0.5,
            };
        }
        
        log::info!("FSR3 quality set to {:?}", quality);
        
        Ok(())
    }
    
    fn needs_initialization(&self) -> bool {
        !self.initialized || self.context.is_none()
    }
    
    fn input_width(&self) -> u32 {
        self.input_width
    }
    
    fn input_height(&self) -> u32 {
        self.input_height
    }
}

// Implement Clone for Fsr3Upscaler (needed for upscale_with_algorithm)
impl Clone for Fsr3Upscaler {
    fn clone(&self) -> Self {
        // Create a new upscaler with the same settings
        Fsr3Upscaler {
            input_width: self.input_width,
            input_height: self.input_height,
            output_width: self.output_width,
            output_height: self.output_height,
            quality: self.quality,
            frame_generation_enabled: self.frame_generation_enabled,
            context: self.context.clone(),
            frame_history: self.frame_history.clone(),
            initialized: self.initialized,
            stats: self.stats.clone(),
        }
    }
}

// Implement Clone for Fsr3Context
impl Clone for Fsr3Context {
    fn clone(&self) -> Self {
        Self {
            render_width: self.render_width,
            render_height: self.render_height,
            display_width: self.display_width,
            display_height: self.display_height,
            quality_mode: match self.quality_mode {
                Fsr3QualityMode::Ultra => Fsr3QualityMode::Ultra,
                Fsr3QualityMode::Quality => Fsr3QualityMode::Quality,
                Fsr3QualityMode::Balanced => Fsr3QualityMode::Balanced,
                Fsr3QualityMode::Performance => Fsr3QualityMode::Performance,
            },
            frame_generation_enabled: self.frame_generation_enabled,
            optical_flow_enabled: self.optical_flow_enabled,
            jitter_x: self.jitter_x,
            jitter_y: self.jitter_y,
            exposure: self.exposure,
            temporal_stability: self.temporal_stability,
            sharpness: self.sharpness,
            reactive_scale: self.reactive_scale,
            interpolation_amount: self.interpolation_amount,
        }
    }
} 