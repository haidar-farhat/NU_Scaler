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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_capture_frame_panics() {
        let mut cap = MockCapture;
        let _ = cap.capture_frame(&CaptureTarget::FullScreen).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_list_windows_panics() {
        let cap = MockCapture;
        let _ = cap.list_windows().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_get_primary_screen_dimensions_panics() {
        let cap = MockCapture;
        let _ = cap.get_primary_screen_dimensions().unwrap();
    }
} 