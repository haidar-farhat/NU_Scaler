use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::fs::{File, OpenOptions};
use std::io::{Error as IoError, ErrorKind};
use anyhow::Result;
use eframe::{self, egui};
use egui::{Vec2, ColorImage, TextureOptions, TextureId};
use image::RgbaImage;
use std::path::Path;
use std::time::{Instant, Duration};
use log::{warn, error, trace, info};
use std::panic::AssertUnwindSafe;

use crate::capture::common::FrameBuffer;
use crate::upscale::{Upscaler, UpscalingTechnology, UpscalingQuality};
use crate::upscale::common::UpscalingAlgorithm;
use crate::capture::CaptureTarget;
use crate::capture::platform::WindowInfo;
use crate::capture::ScreenCapture;

// Constants for texture size limits
const MAX_TEXTURE_SIZE: u32 = 16384; // Maximum dimension for a texture (width or height)
const MAX_TEXTURE_MEMORY_MB: u64 = 2048; // Maximum memory allowed for a texture in MB

// Define a constant for the lock file path
const LOCK_FILE_PATH: &str = "nu_scaler_fullscreen.lock";

/// Create a lock file to ensure only one instance can run fullscreen mode
fn create_lock_file() -> std::io::Result<Option<File>> {
    // Try to get the app data directory
    let lock_path = if let Some(data_dir) = dirs::data_dir() {
        let app_dir = data_dir.join("NU_Scaler");
        // Create the directory if it doesn't exist
        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir)?;
        }
        app_dir.join(LOCK_FILE_PATH)
    } else {
        std::path::PathBuf::from(LOCK_FILE_PATH)
    };
    
    // If lock file exists, check if it's stale
    if lock_path.exists() {
        let is_stale = match std::fs::read_to_string(&lock_path) {
            Ok(content) => {
                // Read PID from lock file
                if let Ok(pid) = content.trim().parse::<u32>() {
                    // On Windows, check if the process exists
                    #[cfg(windows)]
                    {
                        use std::process::Command;
                        // Try to query the process - if it doesn't exist, this will fail
                        let output = Command::new("tasklist")
                            .args(&["/FI", &format!("PID eq {}", pid), "/NH"])
                            .output();
                        
                        match output {
                            Ok(output) => {
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                // If the process is not in the list, the lock is stale
                                let is_stale = !output_str.contains(&pid.to_string());
                                if is_stale {
                                    log::info!("Detected stale lock file from non-existent process {}", pid);
                                }
                                is_stale
                            },
                            Err(_) => {
                                // If we can't check, assume it's not stale
                                false
                            }
                        }
                    }
                    
                    // On Unix systems, check differently
                    #[cfg(unix)]
                    {
                        use std::process::Command;
                        // Check if the process exists
                        let output = Command::new("ps")
                            .args(&["-p", &pid.to_string()])
                            .output();
                            
                        match output {
                            Ok(output) => {
                                // The process doesn't exist if ps returns no lines beyond the header
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                let lines = output_str.lines().count();
                                let is_stale = lines <= 1;
                                if is_stale {
                                    log::info!("Detected stale lock file from non-existent process {}", pid);
                                }
                                is_stale
                            },
                            Err(_) => false
                        }
                    }
                    
                    // Default for other platforms
                    #[cfg(not(any(windows, unix)))]
                    {
                        // Can't check on other platforms, assume it's not stale
                        false
                    }
                } else {
                    // Invalid PID in lock file, consider it stale
                    log::warn!("Invalid PID in lock file, treating as stale");
                    true
                }
            },
            Err(_) => {
                // Can't read lock file, assume it's stale
                log::warn!("Couldn't read lock file, treating as stale");
                true
            }
        };
        
        // Remove stale lock file
        if is_stale {
            log::info!("Removing stale lock file at {:?}", lock_path);
            let _ = std::fs::remove_file(&lock_path);
        } else {
            log::warn!("Lock file is active (not stale) at {:?}", lock_path);
            return Ok(None);
        }
    }
    
    // Try to create the lock file with exclusive access
    match OpenOptions::new().write(true).create_new(true).open(&lock_path) {
        Ok(file) => {
            log::info!("Created lock file at {:?}", lock_path);
            // Write the current process ID to the lock file
            if let Err(e) = std::io::Write::write_all(&mut std::io::BufWriter::new(&file), 
                                                     format!("{}", std::process::id()).as_bytes()) {
                log::warn!("Failed to write PID to lock file: {}", e);
            }
            Ok(Some(file))
        },
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            log::warn!("Lock file already exists at {:?}, another instance may be running", lock_path);
            Ok(None)
        },
        Err(e) => {
            log::error!("Failed to create lock file: {}", e);
            Err(e)
        }
    }
}

/// Remove the lock file when the application exits
fn remove_lock_file() {
    let lock_path = if let Some(data_dir) = dirs::data_dir() {
        data_dir.join("NU_Scaler").join(LOCK_FILE_PATH)
    } else {
        std::path::PathBuf::from(LOCK_FILE_PATH)
    };
    
    if let Err(e) = std::fs::remove_file(&lock_path) {
        log::warn!("Failed to remove lock file: {}", e);
    } else {
        log::info!("Removed lock file at {:?}", lock_path);
    }
}

/// Performance metrics for the fullscreen upscaler
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    /// Time taken to capture the frame
    capture_time: Duration,
    /// Time taken to upscale the frame
    upscale_time: Duration,
    /// Time taken to render the frame
    render_time: Duration,
    /// Total time for processing a frame
    total_frame_time: Duration,
    /// Number of frames processed
    frame_count: u64,
    /// Number of black frames detected in a row
    black_frame_count: u32,
    /// Number of consecutive errors
    error_count: u32,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            capture_time: Duration::from_millis(0),
            upscale_time: Duration::from_millis(0),
            render_time: Duration::from_millis(0),
            total_frame_time: Duration::from_millis(0),
            frame_count: 0,
            black_frame_count: 0,
            error_count: 0,
        }
    }
}

/// Fullscreen upscaler UI
pub struct FullscreenUpscalerUi {
    /// Frame buffer for capturing frames
    frame_buffer: Arc<FrameBuffer>,
    /// Stop signal for capture thread
    stop_signal: Arc<AtomicBool>,
    /// Upscaler implementation
    upscaler: Box<dyn Upscaler + Send + Sync>,
    /// Upscaling algorithm
    algorithm: Option<UpscalingAlgorithm>,
    /// Texture for displaying frames
    texture: Option<egui::TextureHandle>,
    /// Time of last frame
    last_frame_time: std::time::Instant,
    /// FPS counter
    fps: f32,
    /// Number of frames processed
    frames_processed: u64,
    /// Current upscaler name
    upscaler_name: String,
    /// Current upscaling quality
    upscaler_quality: UpscalingQuality,
    /// Show performance overlay
    show_overlay: bool,
    /// Performance metrics history
    fps_history: Vec<f32>,
    /// Upscaling time history (ms)
    upscale_time_history: Vec<f32>,
    /// Last upscale time (ms)
    last_upscale_time: f32,
    /// Input size
    input_size: (u32, u32),
    /// Output size 
    output_size: (u32, u32),
    /// Source window position (x, y, width, height)
    source_window_info: Option<(i32, i32, u32, u32)>,
    /// Capture target used for this upscaling session
    capture_target: Option<CaptureTarget>,
    /// Performance metrics
    performance_metrics: PerformanceMetrics,
    /// Last update time
    last_update_time: Option<Instant>,
    /// Memory pressure counter
    memory_pressure_counter: Option<u32>,
    /// Flag to reinitialize on next update
    requires_reinitialization: bool,
    /// Flag to use a different capture method
    fallback_capture: bool,
}

impl FullscreenUpscalerUi {
    /// Create a new fullscreen upscaler UI
    fn new(
        cc: &eframe::CreationContext<'_>,
        frame_buffer: Arc<FrameBuffer>,
        stop_signal: Arc<AtomicBool>,
        upscaler: Box<dyn Upscaler + Send + Sync>,
        algorithm: Option<UpscalingAlgorithm>,
    ) -> Self {
        // Enable vsync and fullscreen
        if let Some(ctx) = &cc.wgpu_render_state {
            // Configure wgpu renderer if available
            let _ = ctx.adapter.features();
            // Additional wgpu configuration can be done here
        }
        
        // Set up UI with dark mode
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        
        // Get upscaler information
        let upscaler_name = upscaler.name().to_string();
        let upscaler_quality = upscaler.quality();
        
        Self {
            frame_buffer,
            stop_signal,
            upscaler,
            algorithm,
            texture: None,
            last_frame_time: std::time::Instant::now(),
            fps: 0.0,
            frames_processed: 0,
            upscaler_name,
            upscaler_quality,
            show_overlay: true,
            fps_history: Vec::with_capacity(120),
            upscale_time_history: Vec::with_capacity(120),
            last_upscale_time: 0.0,
            input_size: (0, 0),
            output_size: (0, 0),
            source_window_info: None,
            capture_target: None,
            performance_metrics: PerformanceMetrics::new(),
            last_update_time: None,
            memory_pressure_counter: None,
            requires_reinitialization: false,
            fallback_capture: false,
        }
    }
    
    /// Set the capture target used for this upscaling session
    pub fn set_capture_target(&mut self, target: CaptureTarget) {
        self.capture_target = Some(target.clone());
        
        // Try to get window position from target
        if let CaptureTarget::WindowByTitle(title) = &target {
            // Get window information by title
            if let Ok(capturer) = crate::capture::create_capturer() {
                if let Ok(windows) = capturer.list_windows() {
                    // Find window with matching title
                    if let Some(window) = windows.iter().find(|w| w.title.contains(title)) {
                        // Store window position and size
                        self.source_window_info = Some((
                            window.geometry.x,
                            window.geometry.y,
                            window.geometry.width,
                            window.geometry.height,
                        ));
                        log::info!("Found source window position: {:?}", self.source_window_info);
                    }
                }
            }
        }
    }
    
    /// Update source window position (for tracking moving windows)
    fn update_source_window_position(&mut self, frame: &mut eframe::Frame) {
        // Only track windows by title for now
        if let Some(target) = &self.capture_target {
            // We only need to check window positions for specific windows
            if let CaptureTarget::WindowByTitle(title) = target {
                // Get window information by title
                if let Ok(capturer) = crate::capture::create_capturer() {
                    if let Ok(windows) = capturer.list_windows() {
                        // Find window with matching title
                        if let Some(window) = windows.iter().find(|w| w.title.contains(title)) {
                            // Check if position changed
                            let new_pos = (
                                window.geometry.x,
                                window.geometry.y,
                                window.geometry.width,
                                window.geometry.height,
                            );
                            
                            if self.source_window_info != Some(new_pos) {
                                log::debug!("Window position changed from {:?} to {:?}", 
                                        self.source_window_info, new_pos);
                                        
                                // Update stored position
                                self.source_window_info = Some(new_pos);
                                
                                // Update overlay window position
                                frame.set_window_pos(egui::pos2(new_pos.0 as f32, new_pos.1 as f32));
                                frame.set_window_size(egui::vec2(new_pos.2 as f32, new_pos.3 as f32));
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Update the texture with a new frame
    fn update_texture(&mut self) -> Result<bool> {
        // Check if we have a texture already
        if self.texture.is_none() {
            // Create a placeholder texture if needed
            let ctx = egui::Context::default();
            self.texture = Some(ctx.load_texture(
                "placeholder",
                egui::ColorImage::example(),
                Default::default()
            ));
            log::warn!("Texture not initialized, created placeholder");
        }

        // Add performance guard for very frequent updates
        if let Some(last_update) = self.last_update_time {
            let now = Instant::now();
            let elapsed = now.duration_since(last_update);
            
            // If updates are happening too frequently (< 10ms apart), throttle
            if elapsed < Duration::from_millis(10) {
                log::debug!("Throttling texture update ({}ms since last update)", elapsed.as_millis());
                return Ok(false);
            }
            
            self.last_update_time = Some(now);
        } else {
            self.last_update_time = Some(Instant::now());
        }

        // Capture performance metrics for this frame
        let capture_start = Instant::now();
        
        // Safely get a frame from the buffer
        let frame = match self.frame_buffer.get_latest_frame() {
            Ok(Some(frame)) => frame,
            Ok(None) => {
                log::debug!("No frame available in buffer");
                return Ok(false);
            },
            Err(e) => {
                log::error!("Failed to get frame: {}", e);
                return Ok(false);
            }
        };

        // Measure and log capture time
        let capture_duration = capture_start.elapsed();
        self.performance_metrics.capture_time = capture_duration;
        log::debug!("Frame capture completed in {:?}", capture_duration);

        // Check if we have a valid frame
        if frame.width() == 0 || frame.height() == 0 {
            log::warn!("Captured frame has invalid dimensions: {}x{}", frame.width(), frame.height());
            return Ok(false);
        }

        log::debug!("Processing frame with dimensions: {}x{}", frame.width(), frame.height());

        // Do not process if the captured frame is all black (possible capture failure)
        let is_all_black = self.safe_check_if_all_black(&frame);
        
        if is_all_black {
            log::warn!("Captured frame appears to be all black, possible capture failure");
            self.performance_metrics.black_frame_count += 1;
            
            // After several black frames, try recovery
            if self.performance_metrics.black_frame_count > 3 {
                log::info!("Multiple black frames detected, forcing capture method change");
                self.fallback_capture = !self.fallback_capture;
                self.performance_metrics.black_frame_count = 0;
            }
        } else {
            // Reset black frame counter
            self.performance_metrics.black_frame_count = 0;
        }

        // Memory resource guard - skip this frame if system is under memory pressure
        if self.is_memory_pressure() {
            log::warn!("System under memory pressure, skipping frame processing");
            return Ok(false);
        }

        // Attempt to upscale the frame with normal error handling (no catch_unwind)
        let upscale_start = Instant::now();
        let upscaled_result = self.upscale_frame(frame.clone());
        
        // Measure and log upscaling time
        let upscale_duration = upscale_start.elapsed();
        self.performance_metrics.upscale_time = upscale_duration;
        log::debug!("Frame upscaled in {:?}", upscale_duration);

        let upscaled = match upscaled_result {
            Some(upscaled) => upscaled,
            None => {
                log::error!("Failed to upscale frame");
                
                // As a fallback, use the original frame or a simpler scaling method
                log::warn!("Using fallback scaling due to upscaling failure");
                let output_width = (frame.width() as f32 * 1.5) as u32;
                let output_height = (frame.height() as f32 * 1.5) as u32;
                
                // Simple nearest-neighbor scaling as emergency fallback
                image::imageops::resize(
                    &frame, 
                    output_width, 
                    output_height, 
                    image::imageops::FilterType::Nearest
                )
            }
        };

        // Convert the upscaled image to raw bytes for the GPU
        // Use defensive coding to avoid crashes
        let render_start = Instant::now();
        
        // Make sure the upscaled image is valid before processing
        if upscaled.width() == 0 || upscaled.height() == 0 {
            log::error!("Invalid upscaled image dimensions: {}x{}", upscaled.width(), upscaled.height());
            return Ok(false);
        }
        
        // Safety check on the raw data length
        let raw_upscaled = upscaled.as_raw();
        let expected_size = upscaled.width() as usize * upscaled.height() as usize * 4;
        if raw_upscaled.len() != expected_size {
            log::error!("Upscaled image data size mismatch: got {} bytes, expected {}", 
                       raw_upscaled.len(), expected_size);
            return Ok(false);
        }

        // Update texture data with upscaled frame
        if let Some(texture) = &mut self.texture {
            // Create the image data safely
            let image_size = [upscaled.width() as usize, upscaled.height() as usize];
            
            // Create the color image directly (no catch_unwind)
            let result = (|| -> anyhow::Result<()> {
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    image_size,
                    raw_upscaled
                );
                
                // Update the texture with the upscaled data
                texture.set(
                    color_image,
                    egui::TextureOptions::default()
                );
                Ok(())
            })();
            
            if let Err(e) = result {
                log::error!("Error during texture update: {}", e);
                return Ok(false);
            }
        } else {
            log::error!("Texture is not available");
            return Ok(false);
        }

        // Measure and log rendering time
        let render_duration = render_start.elapsed();
        self.performance_metrics.render_time = render_duration;
        
        // Total processing time for this frame
        let total_duration = capture_start.elapsed();
        self.performance_metrics.total_frame_time = total_duration;
        self.performance_metrics.frame_count += 1;
        
        // Log performance every 100 frames
        if self.performance_metrics.frame_count % 100 == 0 {
            log::info!("Performance metrics (last 100 frames):");
            log::info!("  Capture: {:?}", self.performance_metrics.capture_time);
            log::info!("  Upscale: {:?}", self.performance_metrics.upscale_time);
            log::info!("  Render: {:?}", self.performance_metrics.render_time);
            log::info!("  Total: {:?}", self.performance_metrics.total_frame_time);
            log::info!("  FPS: {:.2}", 1.0 / self.performance_metrics.total_frame_time.as_secs_f64());
        }

        Ok(true)
    }
    
    /// A safe version of check_if_all_black that doesn't use unsafe methods
    fn safe_check_if_all_black(&mut self, frame: &RgbaImage) -> bool {
        // If the frame is empty, consider it black
        if frame.width() == 0 || frame.height() == 0 {
            return true;
        }
        
        // Sample a subset of pixels to determine if frame is black
        let sample_step_x = frame.width().max(1) / 10;
        let sample_step_y = frame.height().max(1) / 10;
        
        // Use a defensive approach to avoid panics
        for y in (0..frame.height()).step_by(sample_step_y as usize) {
            for x in (0..frame.width()).step_by(sample_step_x as usize) {
                // Make sure coordinates are in bounds
                if x < frame.width() && y < frame.height() {
                    let pixel = frame.get_pixel(x, y);
                    if pixel[0] > 5 || pixel[1] > 5 || pixel[2] > 5 {
                        return false;
                    }
                }
            }
        }
        
        true
    }
    
    /// Check if the system is under memory pressure
    fn is_memory_pressure(&mut self) -> bool {
        // A simple heuristic - could be expanded with actual system monitoring
        match self.memory_pressure_counter {
            Some(counter) if counter > 10 => {
                // Reset counter occasionally to allow recovery
                self.memory_pressure_counter = Some(0);
                true
            },
            Some(counter) => {
                self.memory_pressure_counter = Some(counter + 1);
                false
            },
            None => {
                self.memory_pressure_counter = Some(0);
                false
            }
        }
    }

    /// Check if a frame is mostly black (indicating potential capture issues)
    fn is_black_frame(&mut self, frame: &RgbaImage) -> bool {
        // Sample a grid of pixels throughout the image rather than checking every pixel
        // This is more efficient for large frames
        let sample_step_x = frame.width().max(1) / 10;
        let sample_step_y = frame.height().max(1) / 10;
        
        // Count black pixels in our sample
        let mut black_count = 0;
        let mut total_sampled = 0;
        
        for y in (0..frame.height()).step_by(sample_step_y as usize) {
            for x in (0..frame.width()).step_by(sample_step_x as usize) {
                let pixel = frame.get_pixel(x, y);
                // Consider a pixel "black" if it's very dark (all channels < 10)
                if pixel[0] < 10 && pixel[1] < 10 && pixel[2] < 10 {
                    black_count += 1;
                }
                total_sampled += 1;
            }
        }
        
        // Also check the center region more thoroughly
        let center_x = frame.width() / 2;
        let center_y = frame.height() / 2;
        let center_width = frame.width() / 4;
        let center_height = frame.height() / 4;
        
        for y in center_y.saturating_sub(center_height/2)..center_y.saturating_add(center_height/2) {
            for x in center_x.saturating_sub(center_width/2)..center_x.saturating_add(center_width/2) {
                if x < frame.width() && y < frame.height() {
                    let pixel = frame.get_pixel(x, y);
                    if pixel[0] < 10 && pixel[1] < 10 && pixel[2] < 10 {
                        black_count += 1;
                    }
                    total_sampled += 1;
                }
            }
        }
        
        // If more than 90% of sampled pixels are black, consider it a black frame
        let black_percentage = black_count as f32 / total_sampled.max(1) as f32;
        let is_black = black_percentage > 0.9;
        
        if is_black {
            log::warn!("Detected a mostly black frame: {:.1}% black pixels", black_percentage * 100.0);
        }
        
        is_black
    }

    /// Upscale a frame using the configured upscaler
    fn upscale_frame(&mut self, frame: RgbaImage) -> Option<RgbaImage> {
        // Use std::panic::catch_unwind to handle potential panics during upscaling
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            let (width, height) = frame.dimensions();
            if width == 0 || height == 0 {
                warn!("Invalid frame dimensions: {}x{}", width, height);
                return None;
            }

            // Check if the frame is all black, which could indicate capture issues
            if self.is_black_frame(&frame) {
                trace!("Skipping upscaling of all-black frame");
                return Some(frame.clone());
            }

            // Check for system memory pressure
            if self.is_memory_pressure() {
                warn!("System memory pressure detected, skipping upscale");
                return Some(frame.clone());
            }

            // Calculate scale factor based on upscaler settings
            // Assume a default scale of 1.5 if no configuration is available
            let scale_factor = 1.5;
            let target_width = (width as f32 * scale_factor).round() as u32;
            let target_height = (height as f32 * scale_factor).round() as u32;

            // Validate dimensions to prevent excessive memory usage
            if target_width > MAX_TEXTURE_SIZE || target_height > MAX_TEXTURE_SIZE {
                warn!(
                    "Target dimensions exceed maximum allowed: {}x{} (max: {})",
                    target_width, target_height, MAX_TEXTURE_SIZE
                );
                return Some(frame.clone());
            }

            // Check if dimensions would require excessive memory
            let estimated_memory = (target_width as u64 * target_height as u64 * 4) / (1024 * 1024);
            if estimated_memory > MAX_TEXTURE_MEMORY_MB {
                warn!(
                    "Estimated memory for upscaled frame exceeds limit: {} MB (max: {} MB)",
                    estimated_memory, MAX_TEXTURE_MEMORY_MB
                );
                return Some(frame.clone());
            }

            // Return original frame if no upscaling is needed
            if scale_factor.abs() < 1.01 || (target_width == width && target_height == height) {
                trace!("No upscaling needed, returning original frame");
                return Some(frame.clone());
            }

            let start = Instant::now();

            // Perform the upscaling directly with our existing upscaler
            match self.upscaler.upscale(&frame) {
                Ok(upscaled) => {
                    let duration = start.elapsed();
                    trace!(
                        "Upscaled {}x{} to {}x{} using {} in {:?}",
                        width,
                        height,
                        target_width,
                        target_height,
                        self.upscaler.name(),
                        duration
                    );
                    
                    // Update performance metrics
                    self.performance_metrics.upscale_time = duration;
                    
                    // Final safety check on dimensions
                    let (actual_width, actual_height) = upscaled.dimensions();
                    if actual_width != target_width || actual_height != target_height {
                        warn!(
                            "Upscaler produced incorrect dimensions: expected {}x{}, got {}x{}",
                            target_width, target_height, actual_width, actual_height
                        );
                    }
                    
                    Some(upscaled)
                }
                Err(e) => {
                    error!("Failed to upscale frame: {}", e);
                    // Return original frame on error
                    Some(frame.clone())
                }
            }
        }));

        // Handle the result of catch_unwind
        match result {
            Ok(upscaled_result) => upscaled_result,
            Err(panic_error) => {
                // Log the panic and return the original frame
                if let Some(error_str) = panic_error.downcast_ref::<String>() {
                    error!("Panic during upscaling: {}", error_str);
                } else if let Some(error_str) = panic_error.downcast_ref::<&str>() {
                    error!("Panic during upscaling: {}", error_str);
                } else {
                    error!("Unknown panic during upscaling");
                }
                
                // Return the original frame as a fallback
                Some(frame)
            }
        }
    }
    
    /// Draw the performance overlay
    fn draw_performance_overlay(&self, ui: &mut egui::Ui) {
        // Early return if overlay is disabled
        if !self.show_overlay {
            return;
        }
        
        // Background panel for metrics
        egui::Frame::default()
            .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 180))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)))
            .inner_margin(egui::Margin::same(10.0))
            .rounding(egui::Rounding::same(5.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Title
                    ui.add(egui::Label::new(
                        egui::RichText::new("NU_Scaler Performance")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(220, 220, 220))
                    ));
                    
                    ui.add_space(5.0);
                    
                    // FPS and frame count
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("FPS: {:.1}", self.fps))
                                .color(egui::Color32::from_rgb(120, 220, 120))
                                .size(14.0)
                        ));
                        
                        ui.add_space(15.0);
                        
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("Frames: {}", self.frames_processed))
                                .color(egui::Color32::WHITE)
                                .size(14.0)
                        ));
                    });
                    
                    // Upscaler info
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("Upscaler: {}", self.upscaler_name))
                                .color(egui::Color32::from_rgb(220, 180, 120))
                                .size(14.0)
                        ));
                        
                        ui.add_space(15.0);
                        
                        let quality_color = match self.upscaler_quality {
                            UpscalingQuality::Ultra => egui::Color32::from_rgb(120, 220, 120),
                            UpscalingQuality::Quality => egui::Color32::from_rgb(180, 220, 120),
                            UpscalingQuality::Balanced => egui::Color32::from_rgb(220, 220, 120),
                            UpscalingQuality::Performance => egui::Color32::from_rgb(220, 180, 120),
                        };
                        
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("Quality: {:?}", self.upscaler_quality))
                                .color(quality_color)
                                .size(14.0)
                        ));
                    });
                    
                    // Algorithm info if present
                    if let Some(alg) = self.algorithm {
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("Algorithm: {:?}", alg))
                                .color(egui::Color32::from_rgb(180, 180, 220))
                                .size(14.0)
                        ));
                    }
                    
                    // Resolution info
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("Input: {}x{}", self.input_size.0, self.input_size.1))
                                .color(egui::Color32::WHITE)
                                .size(14.0)
                        ));
                        
                        ui.add_space(10.0);
                        
                        ui.add(egui::Label::new(
                            egui::RichText::new("→")
                                .color(egui::Color32::from_rgb(180, 180, 180))
                                .size(14.0)
                        ));
                        
                        ui.add_space(10.0);
                        
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("Output: {}x{}", self.output_size.0, self.output_size.1))
                                .color(egui::Color32::WHITE)
                                .size(14.0)
                        ));
                        
                        // Calculate and show scale factor
                        if self.input_size.0 > 0 && self.input_size.1 > 0 {
                            let scale_x = self.output_size.0 as f32 / self.input_size.0 as f32;
                            let scale_y = self.output_size.1 as f32 / self.input_size.1 as f32;
                            
                            ui.add_space(10.0);
                            
                            ui.add(egui::Label::new(
                                egui::RichText::new(format!("({:.1}x)", (scale_x + scale_y) / 2.0))
                                    .color(egui::Color32::from_rgb(180, 220, 180))
                                    .size(14.0)
                            ));
                        }
                    });
                    
                    // Performance details
                    ui.add_space(5.0);
                    
                    ui.add(egui::Label::new(
                        egui::RichText::new(format!("Upscale time: {:.2}ms", self.last_upscale_time))
                            .color(egui::Color32::from_rgb(220, 220, 120))
                            .size(14.0)
                    ));
                    
                    // FPS Graph if we have enough history
                    if self.fps_history.len() > 2 {
                        ui.add_space(10.0);
                        
                        let max_fps = self.fps_history.iter().cloned().fold(0.0_f32, f32::max).max(1.0);
                        
                        ui.add(egui::Label::new(
                            egui::RichText::new("FPS History")
                                .color(egui::Color32::WHITE)
                                .size(12.0)
                        ));
                        
                        let height = 40.0;
                        let graph = egui::plot::Plot::new("fps_history")
                            .height(height)
                            .show_background(false)
                            .allow_zoom(false)
                            .allow_drag(false)
                            .include_y(0.0)
                            .include_y(max_fps)
                            .show_axes([false; 2]);
                            
                        graph.show(ui, |plot_ui| {
                            let fps_points: Vec<[f64; 2]> = self.fps_history.iter()
                                .enumerate()
                                .map(|(i, &fps)| [i as f64, fps as f64])
                                .collect();
                            
                            let line = egui::plot::Line::new(egui::plot::PlotPoints::from(fps_points))
                                .color(egui::Color32::from_rgb(120, 220, 120))
                                .width(1.5);
                                
                            plot_ui.line(line);
                        });
                    }
                    
                    // Help text
                    ui.add_space(10.0);
                    ui.add(egui::Label::new(
                        egui::RichText::new("Press ESC to exit fullscreen mode")
                            .color(egui::Color32::from_rgb(180, 180, 180))
                            .size(12.0)
                    ));
                });
            });
    }

    /// Configure UI settings after initialization
    pub fn configure_ui(&mut self, ctx: &egui::Context) {
        // Set up UI with dark mode
        ctx.set_visuals(egui::Visuals::dark());
    }
}

impl eframe::App for FullscreenUpscalerUi {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Update the texture with the latest frame
        // Safe error handling to avoid crashes
        match self.update_texture() {
            Ok(_) => {},
            Err(e) => {
                log::error!("Error updating texture: {}", e);
                // Continue anyway, to avoid crashing the app
            }
        }
        
        // Check for ESC key to exit fullscreen mode
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            // Signal the capture thread to stop
            self.stop_signal.store(true, Ordering::SeqCst);
            
            // Close the application
            frame.close();
            return;
        }
        
        // Check for F1 key to toggle performance overlay
        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            self.show_overlay = !self.show_overlay;
        }
        
        // Force the window to be opaque black instead of transparent
        ctx.set_visuals(egui::Visuals::dark());
        
        // Use a dark background instead of transparent
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(10, 10, 10)))
            .show(ctx, |ui| {
                if let Some(texture) = &self.texture {
                    // Get available size
                    let available_size = ui.available_size();
                    let texture_size = texture.size_vec2();
                    
                    // Calculate the scaling to fit in the available space
                    // while maintaining aspect ratio
                    let aspect_ratio = texture_size.x / texture_size.y;
                    let width = available_size.x;
                    let height = width / aspect_ratio;
                    
                    // Center the image if it's smaller than the available space
                    let rect = if height <= available_size.y {
                        let y_offset = (available_size.y - height) / 2.0;
                        egui::Rect::from_min_size(
                            egui::pos2(0.0, y_offset),
                            Vec2::new(width, height)
                        )
                    } else {
                        let height = available_size.y;
                        let width = height * aspect_ratio;
                        let x_offset = (available_size.x - width) / 2.0;
                        egui::Rect::from_min_size(
                            egui::pos2(x_offset, 0.0),
                            Vec2::new(width, height)
                        )
                    };
                    
                    // Simple rendering without catch_unwind
                    // Use a defensive try pattern to handle errors
                    match (|| {
                        ui.put(rect, egui::Image::new(texture.id(), egui::Vec2::new(rect.width(), rect.height())));
                        Ok::<(), String>(())
                    })() {
                        Ok(_) => {},
                        Err(e) => log::error!("Error rendering texture: {}", e)
                    };
                    
                    // Draw performance overlay in the top-right corner only if enabled
                    if self.show_overlay {
                        let overlay_width = 250.0;
                        let overlay_rect = egui::Rect::from_min_size(
                            egui::pos2(ui.available_rect_before_wrap().right() - overlay_width - 10.0, 10.0),
                            Vec2::new(overlay_width, 0.0) // Height will be determined by content
                        );
                        
                        ui.allocate_ui_at_rect(overlay_rect, |ui| {
                            self.draw_performance_overlay(ui);
                        });
                    }
                } else {
                    // Show loading message if no texture is available
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("Waiting for frames...");
                            ui.add_space(10.0);
                            ui.label("If you don't see any content, please ensure the source window is visible and not minimized.");
                            ui.add_space(5.0);
                            ui.label("Press ESC to exit and try again.");
                        });
                    });
                }
            });
        
        // Calculate target repaint interval based on upscaling performance
        let target_fps = if self.fps_history.len() > 5 {
            // Use recent average FPS to determine optimal repaint interval
            let recent_fps: f32 = self.fps_history.iter().rev().take(5).sum::<f32>() / 5.0;
            // Cap the FPS to a reasonable range
            recent_fps.clamp(15.0, 60.0) // Lower max to 60 to reduce strain
        } else {
            30.0 // More conservative default FPS
        };
        
        let target_frame_time = std::time::Duration::from_secs_f32(1.0 / target_fps);
        
        // Request a repaint after the calculated interval
        ctx.request_repaint_after(target_frame_time);
        
        // Safe window position update without catch_unwind
        self.update_source_window_position(frame);
    }
}

/// Create an upscaler for the given technology and quality
fn create_upscaler(
    technology: UpscalingTechnology,
    quality: UpscalingQuality,
    algorithm: Option<UpscalingAlgorithm>,
) -> Result<Box<dyn Upscaler + Send + Sync>> {
    // Special case for FSR3 since it requires extra setup
    if technology == UpscalingTechnology::FSR3 {
        if crate::upscale::fsr3::Fsr3Upscaler::is_supported() {
            log::info!("Using FSR3 with frame generation for upscaling");
            return crate::upscale::fsr3::Fsr3Upscaler::new(quality, true)
                .map(|upscaler| Box::new(upscaler) as Box<dyn Upscaler + Send + Sync>);
        } else {
            log::warn!("FSR3 not supported, falling back to alternative upscaler");
            // Fall through to standard upscaler creation
        }
    }
    
    crate::upscale::create_upscaler(technology, quality, algorithm)
}

/// Run the fullscreen upscaler UI
pub fn run_fullscreen_upscaler(
    frame_buffer: Arc<FrameBuffer>,
    stop_signal: Arc<AtomicBool>,
    technology: UpscalingTechnology,
    quality: UpscalingQuality,
    algorithm: Option<UpscalingAlgorithm>,
    capture_target: CaptureTarget,
) -> Result<(), String> {
    // Try to create a lock file to ensure only one instance runs
    let lock_file_result = create_lock_file();
    
    // Check if we got a lock
    if let Ok(None) = lock_file_result {
        return Err("Another instance of NU_Scaler is already running in fullscreen mode. Please close it before starting a new session.".to_string());
    }
    
    // Handle error cases but continue
    if let Err(e) = &lock_file_result {
        log::error!("Failed to check for running instances: {}", e);
        // Continue anyway, but log the error
    }
    
    // Create an upscaler with the given technology and quality
    let upscaler = match create_upscaler(technology, quality, algorithm) {
        Ok(u) => u,
        Err(e) => {
            // Release the lock if we fail to create the upscaler
            remove_lock_file();
            return Err(format!("Failed to create upscaler: {}", e));
        }
    };
    
    // Log the upscaler we're actually using
    log::info!("Using upscaler: {} with quality: {:?}", upscaler.name(), upscaler.quality());
    
    // Get the window info from the capture target
    let mut window_info = None;
    
    if let CaptureTarget::WindowByTitle(title) = &capture_target {
        if let Ok(capturer) = crate::capture::create_capturer() {
            if let Ok(windows) = capturer.list_windows() {
                // Find window with matching title
                if let Some(window) = windows.iter().find(|w| w.title.contains(title)) {
                    // Store window position and size
                    window_info = Some((
                        window.geometry.x,
                        window.geometry.y,
                        window.geometry.width,
                        window.geometry.height,
                    ));
                    log::info!("Found source window: {} at position {:?}", title, window_info);
                }
            }
        }
    }
    
    // If we couldn't get window info from the capture target, try getting it from a frame
    if window_info.is_none() {
        window_info = match frame_buffer.get_latest_frame() {
            Ok(Some(frame)) => {
                // Since we don't have position info, just use the dimensions
                log::info!("Using frame dimensions: {}x{}", frame.width(), frame.height());
                Some((0, 0, frame.width(), frame.height()))
            },
            _ => {
                // If we can't get a frame yet, use default dimensions
                log::warn!("Could not get frame dimensions, using default 1280x720");
                Some((0, 0, 1280, 720))
            }
        };
    }
    
    // Instead of creating a new window, we'll create a renderer that can be integrated into the main window
    log::info!("Starting in-place upscaling with {} at {:?} quality", upscaler.name(), quality);
    
    // Create our UI object
    let mut ui = FullscreenUpscalerUi::new_boxed(
        frame_buffer,
        stop_signal.clone(),
        upscaler,
        algorithm,
    );
    
    // Set the capture target
    ui.set_capture_target(capture_target.clone());
    
    // Instead of running as a separate window, we'll return the UI component for integration
    // into the main application window. 
    // This is a placeholder - the actual implementation depends on how the main window is structured.
    
    // Create an integration point to the main application UI
    // For now, we'll just create a simplified renderer that can be passed back to the main UI
    
    // Signal that upscaling is active in the main window
    crate::ui::set_upscaling_active(true);
    
    // Set up a callback to apply upscaling to the main window's content
    // This part would need to integrate with the main window's UI system
    crate::ui::set_upscaling_renderer(Box::new(move |ctx, content| {
        // Apply upscaling to the content and render it in the main window context
        ui.update_texture();
        
        // If we have a texture, render it
        if let Some(texture) = &ui.texture {
            return Some(texture.id());
        }
        
        None
    }));
    
    Ok(())
}

/// Integration method for the main window to use this upscaler
impl FullscreenUpscalerUi {
    // Method to render upscaled content in any UI context
    pub fn render_upscaled_content(&self, ui: &mut egui::Ui) -> bool {
        if let Some(texture) = &self.texture {
            // Get available size
            let available_size = ui.available_size();
            let texture_size = texture.size_vec2();
            
            // Calculate the scaling to fit in the available space
            // while maintaining aspect ratio
            let aspect_ratio = texture_size.x / texture_size.y;
            let width = available_size.x;
            let height = width / aspect_ratio;
            
            // Center the image if it's smaller than the available space
            let rect = if height <= available_size.y {
                let y_offset = (available_size.y - height) / 2.0;
                egui::Rect::from_min_size(
                    egui::pos2(0.0, y_offset),
                    Vec2::new(width, height)
                )
            } else {
                let height = available_size.y;
                let width = height * aspect_ratio;
                let x_offset = (available_size.x - width) / 2.0;
                egui::Rect::from_min_size(
                    egui::pos2(x_offset, 0.0),
                    Vec2::new(width, height)
                )
            };
            
            // Draw the texture to cover the entire space
            ui.put(rect, egui::Image::new(texture.id(), texture_size));
            
            // Draw performance overlay in the top-right corner only if enabled
            if self.show_overlay {
                let overlay_width = 250.0;
                let overlay_rect = egui::Rect::from_min_size(
                    egui::pos2(ui.available_rect_before_wrap().right() - overlay_width - 10.0, 10.0),
                    Vec2::new(overlay_width, 0.0) // Height will be determined by content
                );
                
                ui.allocate_ui_at_rect(overlay_rect, |ui| {
                    self.draw_performance_overlay(ui);
                });
            }
            
            true
        } else {
            // Show loading message if no texture is available
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Waiting for frames...");
                    ui.add_space(10.0);
                    ui.label("If you don't see any content, please ensure the source window is visible and not minimized.");
                    ui.add_space(5.0);
                    ui.label("Press ESC to exit and try again.");
                });
            });
            false
        }
    }
    
    // Method to check and handle ESC key for exit
    pub fn check_exit(&self, ctx: &egui::Context) -> bool {
        // Check for ESC key to exit fullscreen mode
        ctx.input(|i| i.key_pressed(egui::Key::Escape))
    }
}

impl FullscreenUpscalerUi {
    // Separate configuration of the context
    fn configure(cc: &eframe::CreationContext<'_>) {
        // Enable vsync and fullscreen
        if let Some(ctx) = &cc.wgpu_render_state {
            // Configure wgpu renderer if available
            let _ = ctx.adapter.features();
            // Additional wgpu configuration can be done here
        }
        
        // Set up UI with dark mode
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
    }
    
    // Create a new boxed instance of FullscreenUpscalerUi
    fn new_boxed(
        frame_buffer: Arc<FrameBuffer>,
        stop_signal: Arc<AtomicBool>,
        upscaler: Box<dyn Upscaler + Send + Sync>,
        algorithm: Option<UpscalingAlgorithm>,
    ) -> Box<Self> {
        // We can't actually create the UI here because we need the CreationContext
        // from eframe, so this is just a placeholder that creates the resources
        let upscaler_name = upscaler.name().to_string();
        let upscaler_quality = upscaler.quality();
        
        Box::new(Self {
            frame_buffer,
            stop_signal,
            upscaler,
            algorithm,
            texture: None,
            last_frame_time: std::time::Instant::now(),
            fps: 0.0,
            frames_processed: 0,
            upscaler_name,
            upscaler_quality,
            show_overlay: true,
            fps_history: Vec::with_capacity(120),
            upscale_time_history: Vec::with_capacity(120),
            last_upscale_time: 0.0,
            input_size: (0, 0),
            output_size: (0, 0),
            source_window_info: None,
            capture_target: None,
            performance_metrics: PerformanceMetrics::new(),
            last_update_time: None,
            memory_pressure_counter: None,
            requires_reinitialization: false,
            fallback_capture: false,
        })
    }
} 