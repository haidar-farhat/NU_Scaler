use anyhow::Result;

/// Target for screen capture
#[derive(Debug, Clone)]
pub enum CaptureTarget {
    FullScreen,
    WindowByTitle(String),
    Region { x: i32, y: i32, width: u32, height: u32 },
}

/// Trait for screen/window/region capture
pub trait ScreenCapture {
    /// Capture a single frame from the target
    fn capture_frame(&mut self, target: &CaptureTarget) -> Result<Vec<u8>>;
    /// List available windows for capture
    fn list_windows(&self) -> Result<Vec<String>>;
    /// Get primary screen dimensions
    fn get_primary_screen_dimensions(&self) -> Result<(u32, u32)>;
}

/// Mock implementation for testing
pub struct MockCapture;

impl ScreenCapture for MockCapture {
    fn capture_frame(&mut self, _target: &CaptureTarget) -> Result<Vec<u8>> {
        unimplemented!()
    }
    fn list_windows(&self) -> Result<Vec<String>> {
        unimplemented!()
    }
    fn get_primary_screen_dimensions(&self) -> Result<(u32, u32)> {
        unimplemented!()
    }
} 