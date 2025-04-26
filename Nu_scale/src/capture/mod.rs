// Screen capture abstraction layer

pub mod platform;
pub mod window_finder;
pub mod common;
pub mod frame_buffer;
pub mod frame_buffer_ext;

use anyhow::Result;
use image::RgbaImage;
use std::path::Path;
use std::time::{Duration, Instant};
use thiserror::Error;

// Re-export platform-specific implementations
#[cfg(windows)]
pub use platform::windows as platform_impl;
#[cfg(unix)]
pub use platform::linux as platform_impl;

/// Capture target specification
#[derive(Debug, Clone)]
pub enum CaptureTarget {
    /// Capture the entire screen (primary monitor)
    FullScreen,
    /// Capture a specific window by title (fuzzy matching)
    WindowByTitle(String),
    /// Capture a specific window by ID
    WindowById(platform::WindowId),
    /// Capture a specific region of the screen
    Region { x: i32, y: i32, width: u32, height: u32 },
}

/// Error types specific to screen capturing
#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Window with specified criteria not found")]
    WindowNotFound,
    #[error("Failed to capture screen: {0}")]
    CaptureFailed(String),
    #[error("Operation not supported on this platform")]
    UnsupportedOperation,
    #[error("Invalid capture parameters")]
    InvalidParameters,
    #[error("Capture stream interrupted")]
    StreamInterrupted,
}

/// Callback for live frame capture
pub type FrameCallback = dyn FnMut(&RgbaImage) -> Result<bool>;

/// The main trait that all screen capturers must implement
pub trait ScreenCapture {
    /// Create a new screen capturer
    fn new() -> Result<Self> where Self: Sized;
    
    /// Capture a single frame based on the specified target
    fn capture_frame(&mut self, target: &CaptureTarget) -> Result<RgbaImage>;
    
    /// Save the captured frame to a file
    fn save_frame(&mut self, target: &CaptureTarget, path: &Path) -> Result<()> {
        let image = self.capture_frame(target)?;
        image.save(path).map_err(|e| anyhow::anyhow!("Failed to save image: {}", e))
    }
    
    /// Start a live capture session at the specified frame rate
    /// The callback will be called for each frame, and capture will continue
    /// until the callback returns false or there's an error
    fn start_live_capture(
        &mut self, 
        target: &CaptureTarget, 
        fps: u32, 
        callback: &mut FrameCallback
    ) -> Result<()> {
        let frame_duration = Duration::from_secs_f64(1.0 / fps as f64);
        let mut next_frame_time = Instant::now();
        
        loop {
            // Capture a frame
            let frame = self.capture_frame(target)?;
            
            // Call the callback with the frame
            // If callback returns false, stop capturing
            if !callback(&frame)? {
                break;
            }
            
            // Calculate time until next frame
            next_frame_time += frame_duration;
            let now = Instant::now();
            
            if next_frame_time > now {
                // Sleep until next frame
                std::thread::sleep(next_frame_time.duration_since(now));
            } else {
                // We're behind schedule, adjust next_frame_time
                let behind = now.duration_since(next_frame_time);
                let frames_behind = (behind.as_secs_f64() / frame_duration.as_secs_f64()).ceil() as u32;
                next_frame_time += frame_duration * frames_behind;
            }
        }
        
        Ok(())
    }
    
    /// List available windows
    fn list_windows(&self) -> Result<Vec<platform::WindowInfo>>;
    
    /// Get primary screen dimensions
    fn get_primary_screen_dimensions(&self) -> Result<(u32, u32)>;
}

/// Creates a platform-specific screen capturer
pub fn create_capturer() -> Result<impl ScreenCapture> {
    platform_impl::PlatformScreenCapture::new()
} 