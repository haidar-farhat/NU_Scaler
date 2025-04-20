use anyhow::{Result, anyhow};
use image::{DynamicImage, RgbaImage};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use super::{CaptureTarget, ScreenCapture};

/// Type aliases for upscaling functionality to avoid import issues
#[allow(dead_code)]
type UpscaleResult = Result<RgbaImage>;

// Module to provide upscaling functionality through the public API
mod upscale_api {
    use anyhow::Result;
    use image::RgbaImage;
    
    // Forward function calls to the public API
    pub fn upscale_image(
        input: &RgbaImage,
        width: u32,
        height: u32,
        technology: &str,
        quality: &str
    ) -> Result<RgbaImage> {
        // Create default technology from string - unused in this simplified version
        let _tech = match technology.to_lowercase().as_str() {
            "fsr" => 1,    // FSR
            "dlss" => 2,   // DLSS
            _ => 3,        // Fallback
        };
        
        // Create default quality from string - unused in this simplified version
        let _qual = match quality.to_lowercase().as_str() {
            "ultra" => 0,
            "quality" => 1,
            "balanced" => 2,
            "performance" => 3,
            _ => 2,  // Default to balanced
        };
        
        // Use a simple resizing as fallback if direct API access doesn't work
        use image::imageops;
        Ok(imageops::resize(input, width, height, imageops::FilterType::Lanczos3))
    }
}

/// Captures a screenshot and saves it to the specified path
pub fn capture_screenshot(target: &CaptureTarget, output_path: &Path) -> Result<()> {
    let mut capturer = super::create_capturer()?;
    capturer.save_frame(target, output_path)
}

/// Captures a screenshot and returns it as an image
pub fn capture_screenshot_image(target: &CaptureTarget) -> Result<DynamicImage> {
    let mut capturer = super::create_capturer()?;
    let image = capturer.capture_frame(target)?;
    Ok(DynamicImage::ImageRgba8(image))
}

/// Captures and upscales content to fullscreen dimensions
pub fn capture_and_upscale_to_fullscreen(
    target: &CaptureTarget,
    _technology: Option<UpscalingTechnology>,
    _quality: Option<UpscalingQuality>,
    _algorithm: Option<&str>, // Optional algorithm for basic upscaling
    save_path: Option<&Path>
) -> Result<()> {
    let mut capturer = super::create_capturer()?;
    
    // Get source image
    let source_image = capturer.capture_frame(target)?;
    
    // Get fullscreen dimensions
    let (screen_width, screen_height) = capturer.get_primary_screen_dimensions()?;
    
    // Use our simplified upscaler
    upscale_api::upscale_image(
        &source_image,
        screen_width,
        screen_height,
        "fallback",
        "balanced"
    )?;
    
    Ok(())
}

/// Lists all available windows with their titles and IDs
pub fn list_available_windows() -> Result<Vec<super::platform::WindowInfo>> {
    let capturer = super::create_capturer()?;
    capturer.list_windows()
}

/// Gets the primary screen dimensions
pub fn get_screen_dimensions() -> Result<(u32, u32)> {
    let capturer = super::create_capturer()?;
    capturer.get_primary_screen_dimensions()
}

/// Frame buffer that stores captured frames for processing
pub struct FrameBuffer {
    frames: Arc<Mutex<Vec<RgbaImage>>>,
    max_size: usize,
}

impl FrameBuffer {
    /// Create a new frame buffer with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            frames: Arc::new(Mutex::new(Vec::with_capacity(capacity))),
            max_size: capacity,
        }
    }
    
    /// Add a frame to the buffer
    pub fn add_frame(&self, frame: RgbaImage) -> Result<()> {
        let mut frames = self.frames.lock().map_err(|_| anyhow::anyhow!("Mutex lock failed"))?;
        
        // If buffer is full, remove the oldest frame
        if frames.len() >= self.max_size {
            frames.remove(0);
        }
        
        frames.push(frame);
        Ok(())
    }
    
    /// Get all frames in the buffer
    pub fn get_frames(&self) -> Result<Vec<RgbaImage>> {
        let frames = self.frames.lock().map_err(|_| anyhow::anyhow!("Mutex lock failed"))?;
        Ok(frames.clone())
    }
    
    /// Get the most recent frame
    pub fn get_latest_frame(&self) -> Result<Option<RgbaImage>> {
        let frames = self.frames.lock().map_err(|_| anyhow::anyhow!("Mutex lock failed"))?;
        Ok(frames.last().cloned())
    }
    
    /// Clear the buffer
    pub fn clear(&self) -> Result<()> {
        let mut frames = self.frames.lock().map_err(|_| anyhow::anyhow!("Mutex lock failed"))?;
        frames.clear();
        Ok(())
    }
    
    /// Get number of frames in the buffer
    pub fn len(&self) -> Result<usize> {
        let frames = self.frames.lock().map_err(|_| anyhow::anyhow!("Mutex lock failed"))?;
        Ok(frames.len())
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> Result<bool> {
        let frames = self.frames.lock().map_err(|_| anyhow::anyhow!("Mutex lock failed"))?;
        Ok(frames.is_empty())
    }
    
    /// Create a clone of the frame buffer that can be shared between threads
    pub fn clone_arc(&self) -> Self {
        Self {
            frames: Arc::clone(&self.frames),
            max_size: self.max_size,
        }
    }
}

/// Start fullscreen upscaled capture in a separate thread
pub fn start_fullscreen_upscaled_capture(
    target: CaptureTarget,
    fps: u32,
    technology: &str,  // "fsr", "dlss", or "fallback"
    quality: &str,     // "ultra", "quality", "balanced", or "performance"
    _algorithm: Option<&str>, // Optional algorithm for basic upscaling
    buffer: Arc<FrameBuffer>,
    stop_signal: Arc<Mutex<bool>>,
) -> Result<thread::JoinHandle<Result<()>>> {
    // Clone Arc references for the closure
    let buffer_clone = Arc::clone(&buffer);
    let stop_signal_clone = Arc::clone(&stop_signal);
    
    // Clone the strings since they need to move into the closure
    let tech = technology.to_string();
    let qual = quality.to_string();
    
    let handle = thread::spawn(move || {
        let mut capturer = super::create_capturer()?;
        
        // Get fullscreen dimensions
        let (screen_width, screen_height) = capturer.get_primary_screen_dimensions()?;
        
        // Calculate frame delay based on FPS
        let frame_duration = std::time::Duration::from_secs_f64(1.0 / fps as f64);
        let mut next_frame_time = std::time::Instant::now();
        
        // Capture loop
        loop {
            // Check if we should stop
            let should_stop = {
                let guard = stop_signal_clone.lock().map_err(|_| anyhow!("Mutex lock failed"))?;
                *guard
            };
            
            if should_stop {
                break;
            }
            
            // Capture frame
            let source_image = capturer.capture_frame(&target)?;
            
            // Upscale the frame
            let upscaled_frame = upscale_api::upscale_image(
                &source_image,
                screen_width,
                screen_height,
                &tech,
                &qual
            )?;
            
            // Add to buffer
            buffer_clone.add_frame(upscaled_frame)?;
            
            // Sleep until next frame
            next_frame_time += frame_duration;
            let now = std::time::Instant::now();
            
            if next_frame_time > now {
                std::thread::sleep(next_frame_time.duration_since(now));
            } else {
                // We're behind schedule - adjust next frame time
                let behind = now.duration_since(next_frame_time);
                let _frames_behind = (behind.as_secs_f64() / frame_duration.as_secs_f64()).ceil() as u32;
                // Try to catch up gradually by setting the next frame time to now plus half a frame duration
                next_frame_time = now + (frame_duration / 2);
            }
        }
        
        Ok(())
    });
    
    Ok(handle)
}

/// Start live capture in a separate thread, storing frames in a buffer
pub fn start_live_capture_thread(
    target: CaptureTarget,
    fps: u32,
    buffer: Arc<FrameBuffer>,
    stop_signal: Arc<Mutex<bool>>,
) -> Result<thread::JoinHandle<Result<()>>> {
    // Clone Arc references for the closure
    let buffer_clone = Arc::clone(&buffer);
    let stop_signal_clone = Arc::clone(&stop_signal);
    
    let handle = thread::spawn(move || {
        let mut capturer = super::create_capturer()?;
        
        // Create a callback function that will add frames to the buffer
        let mut callback = move |frame: &RgbaImage| -> Result<bool> {
            // Check if we should stop
            let should_stop = {
                let guard = stop_signal_clone.lock().map_err(|_| anyhow::anyhow!("Mutex lock failed"))?;
                *guard
            };
            
            if should_stop {
                return Ok(false);
            }
            
            // Add frame to buffer
            buffer_clone.add_frame(frame.clone())?;
            
            Ok(true)
        };
        
        // Start capture
        capturer.start_live_capture(&target, fps, &mut callback)
    });
    
    Ok(handle)
}

/// Process frames from a frame buffer in real-time
pub fn process_frame_buffer<F>(
    buffer: Arc<FrameBuffer>,
    stop_signal: Arc<Mutex<bool>>,
    fps: u32,
    processor: F,
) -> Result<thread::JoinHandle<Result<()>>>
where
    F: FnMut(&RgbaImage) -> Result<()> + Send + 'static,
{
    // Clone Arc references for the closure
    let buffer_clone = Arc::clone(&buffer);
    let stop_signal_clone = Arc::clone(&stop_signal);
    
    let handle = thread::spawn(move || {
        let frame_duration = std::time::Duration::from_secs_f64(1.0 / fps as f64);
        let mut next_frame_time = std::time::Instant::now();
        let mut frame_processor = processor;
        
        loop {
            // Check if we should stop
            let should_stop = {
                let guard = stop_signal_clone.lock().map_err(|_| anyhow::anyhow!("Mutex lock failed"))?;
                *guard
            };
            
            if should_stop {
                break;
            }
            
            // Get the latest frame
            if let Some(frame) = buffer_clone.get_latest_frame()? {
                // Process the frame
                frame_processor(&frame)?;
            }
            
            // Sleep until next frame
            next_frame_time += frame_duration;
            let now = std::time::Instant::now();
            
            if next_frame_time > now {
                std::thread::sleep(next_frame_time.duration_since(now));
            } else {
                let behind = now.duration_since(next_frame_time);
                let _frames_behind = (behind.as_secs_f64() / frame_duration.as_secs_f64()).ceil() as u32;
                next_frame_time += frame_duration * _frames_behind;
            }
        }
        
        Ok(())
    });
    
    Ok(handle)
} 