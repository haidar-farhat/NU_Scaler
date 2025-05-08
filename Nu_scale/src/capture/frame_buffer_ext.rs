use std::sync::Arc;
use std::time::Duration;
use image::RgbaImage;
use anyhow::{Result, anyhow};

use crate::capture::common::FrameBuffer;

/// Extension trait for Arc<FrameBuffer> to add timeout functionality
pub trait ArcFrameBufferExt {
    /// Get the latest frame with a timeout
    fn get_latest_frame_timeout(&self, timeout: Duration) -> Result<Option<RgbaImage>>;
}

impl ArcFrameBufferExt for Arc<FrameBuffer> {
    fn get_latest_frame_timeout(&self, timeout: Duration) -> Result<Option<RgbaImage>> {
        let start = std::time::Instant::now();
        
        loop {
            // Try to get the latest frame
            let frame = self.get_latest_frame().map_err(|e| anyhow!(e))?;
            
            // If we have a frame or we've timed out, return it
            if frame.is_some() || start.elapsed() >= timeout {
                return Ok(frame);
            }
            
            // Sleep a bit to avoid spinning the CPU
            std::thread::sleep(Duration::from_millis(5));
        }
    }
} 