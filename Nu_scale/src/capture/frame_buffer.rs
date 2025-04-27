use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use image::RgbaImage;
use anyhow::Result;

/// Frame type that can be shared between threads
pub type Frame = RgbaImage;

/// Frame buffer that stores captured frames for processing
pub struct FrameBuffer {
    /// Frames stored in the buffer
    frames: Arc<Mutex<VecDeque<Arc<Frame>>>>,
    /// Maximum number of frames to store
    max_size: usize,
}

impl FrameBuffer {
    /// Create a new frame buffer with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            frames: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            max_size: capacity,
        }
    }
    
    /// Helper method for frame access
    fn get_frame<F, T>(&self, accessor: F) -> Result<T, String>
    where
        F: FnOnce(&VecDeque<Arc<Frame>>) -> T,
    {
        let frames = self.frames.lock()
            .map_err(|e| format!("Failed to lock frame buffer: {}", e))?;
        Ok(accessor(&frames))
    }
    
    /// Add a frame to the buffer
    pub fn add_frame(&self, frame: Frame) -> Result<(), String> {
        let mut frames = self.frames.lock()
            .map_err(|e| format!("Failed to lock frame buffer: {}", e))?;
        
        // If buffer is full, remove the oldest frame
        if frames.len() >= self.max_size {
            frames.pop_front();
        }
        
        // Add the new frame
        frames.push_back(Arc::new(frame));
        Ok(())
    }
    
    /// Get the most recent frame
    pub fn get_latest_frame(&self) -> Result<Option<Arc<Frame>>, String> {
        self.get_frame(|frames| frames.back().cloned())
    }
    
    /// Get the latest frame from the buffer with a timeout
    /// 
    /// Returns:
    /// - Ok(Some(frame)) if a frame was found
    /// - Ok(None) if no frame was found within the timeout
    /// - Err(String) if an error occurred
    pub fn get_latest_frame_timeout(&self, timeout: Duration) -> Result<Option<Arc<Frame>>, String> {
        let start = Instant::now();
        
        // First attempt to get frame immediately
        match self.get_latest_frame() {
            Ok(Some(frame)) => return Ok(Some(frame)),
            Ok(None) => {}  // No frame available yet, will wait
            Err(e) => return Err(e),
        }
        
        // Wait for a frame to arrive, with timeout
        while start.elapsed() < timeout {
            // Small sleep to avoid busy waiting
            std::thread::sleep(Duration::from_millis(5));
            
            match self.get_latest_frame() {
                Ok(Some(frame)) => return Ok(Some(frame)),
                Ok(None) => continue,
                Err(e) => return Err(e),
            }
        }
        
        // If we get here, we timed out
        Ok(None)
    }
    
    /// Get all frames in the buffer
    pub fn get_frames(&self) -> Result<Vec<Arc<Frame>>, String> {
        self.get_frame(|frames| frames.iter().cloned().collect())
    }
    
    /// Clear the buffer
    pub fn clear(&self) -> Result<(), String> {
        let mut frames = self.frames.lock()
            .map_err(|e| format!("Failed to lock frame buffer: {}", e))?;
        frames.clear();
        Ok(())
    }
    
    /// Get number of frames in the buffer
    pub fn len(&self) -> Result<usize, String> {
        self.get_frame(|frames| frames.len())
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> Result<bool, String> {
        self.get_frame(|frames| frames.is_empty())
    }
    
    /// Create a clone of the frame buffer that can be shared between threads
    pub fn clone_arc(&self) -> Self {
        Self {
            frames: Arc::clone(&self.frames),
            max_size: self.max_size,
        }
    }
} 