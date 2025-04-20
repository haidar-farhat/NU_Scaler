use anyhow::{Result, anyhow};
use image::{DynamicImage, RgbaImage};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use super::{CaptureTarget, ScreenCapture};
use crate::upscale;

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
    technology: upscale::UpscalingTechnology,
    quality: upscale::UpscalingQuality,
    algorithm: Option<upscale::common::UpscalingAlgorithm>,
) -> Result<RgbaImage> {
    let mut capturer = super::create_capturer()?;
    
    // Get source image
    let source_image = capturer.capture_frame(target)?;
    
    // Get fullscreen dimensions
    let (screen_width, screen_height) = capturer.get_primary_screen_dimensions()?;
    
    // Create and initialize upscaler
    let mut upscaler = upscale::create_upscaler(technology, quality, algorithm)?;
    upscaler.initialize(
        source_image.width(),
        source_image.height(),
        screen_width,
        screen_height
    )?;
    
    // Upscale the image
    let upscaled_image = upscaler.upscale(&source_image)?;
    
    // Cleanup
    upscaler.cleanup()?;
    
    Ok(upscaled_image)
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
    technology: upscale::UpscalingTechnology,
    quality: upscale::UpscalingQuality,
    algorithm: Option<upscale::common::UpscalingAlgorithm>,
    buffer: Arc<FrameBuffer>,
    stop_signal: Arc<Mutex<bool>>,
) -> Result<thread::JoinHandle<Result<()>>> {
    // Clone Arc references for the closure
    let buffer_clone = Arc::clone(&buffer);
    let stop_signal_clone = Arc::clone(&stop_signal);
    
    let handle = thread::spawn(move || {
        let mut capturer = super::create_capturer()?;
        
        // Get fullscreen dimensions
        let (screen_width, screen_height) = capturer.get_primary_screen_dimensions()?;
        
        // Create upscaler
        let mut upscaler = upscale::create_upscaler(technology, quality, algorithm)?;
        
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
            
            // Initialize upscaler if dimensions changed
            if upscaler.needs_initialization() || 
               source_image.width() != upscaler.input_width() || 
               source_image.height() != upscaler.input_height() {
                upscaler.initialize(
                    source_image.width(),
                    source_image.height(),
                    screen_width,
                    screen_height
                )?;
            }
            
            // Upscale the frame
            let upscaled_frame = upscaler.upscale(&source_image)?;
            
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
                let frames_behind = (behind.as_secs_f64() / frame_duration.as_secs_f64()).ceil() as u32;
                next_frame_time = now + frame_duration.mul_f32(0.5); // Try to catch up gradually
            }
        }
        
        // Cleanup
        upscaler.cleanup()?;
        
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
                let frames_behind = (behind.as_secs_f64() / frame_duration.as_secs_f64()).ceil() as u32;
                next_frame_time += frame_duration * frames_behind;
            }
        }
        
        Ok(())
    });
    
    Ok(handle)
} 