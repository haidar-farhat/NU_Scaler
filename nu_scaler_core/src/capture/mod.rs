use anyhow::Result;
use image::{RgbaImage, Rgba};

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

/// Basic cross-platform fallback implementation
pub struct BasicCapture;

impl ScreenCapture for BasicCapture {
    fn capture_frame(&mut self, _target: &CaptureTarget) -> Result<Vec<u8>> {
        // Create a 640x480 solid color image
        let mut img = RgbaImage::new(640, 480);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([0, 128, 255, 255]);
        }
        Ok(img.into_raw())
    }
    fn list_windows(&self) -> Result<Vec<String>> {
        Ok(vec!["Dummy Window 1".to_string(), "Dummy Window 2".to_string()])
    }
    fn get_primary_screen_dimensions(&self) -> Result<(u32, u32)> {
        Ok((640, 480))
    }
}

pub mod realtime;

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

    #[test]
    fn test_basic_capture_frame() {
        let mut cap = BasicCapture;
        let buf = cap.capture_frame(&CaptureTarget::FullScreen).unwrap();
        assert_eq!(buf.len(), 640 * 480 * 4);
    }

    #[test]
    fn test_basic_list_windows() {
        let cap = BasicCapture;
        let windows = cap.list_windows().unwrap();
        assert!(windows.len() >= 1);
    }

    #[test]
    fn test_basic_get_primary_screen_dimensions() {
        let cap = BasicCapture;
        let (w, h) = cap.get_primary_screen_dimensions().unwrap();
        assert_eq!((w, h), (640, 480));
    }
} 