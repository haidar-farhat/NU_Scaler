use anyhow::Result;

/// Target for rendering output
#[derive(Debug, Clone)]
pub enum RenderTarget {
    Window,
    Overlay,
    Offscreen,
}

/// Trait for rendering upscaled frames
pub trait Renderer {
    /// Initialize the renderer
    fn initialize(&mut self, target: RenderTarget, width: u32, height: u32) -> Result<()>;
    /// Present a frame (raw bytes or image)
    fn present(&mut self, frame: &[u8]) -> Result<()>;
    /// Resize the render target
    fn resize(&mut self, width: u32, height: u32) -> Result<()>;
}

/// Mock implementation for testing
pub struct MockRenderer;

impl Renderer for MockRenderer {
    fn initialize(&mut self, _target: RenderTarget, _width: u32, _height: u32) -> Result<()> {
        unimplemented!()
    }
    fn present(&mut self, _frame: &[u8]) -> Result<()> {
        unimplemented!()
    }
    fn resize(&mut self, _width: u32, _height: u32) -> Result<()> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_initialize_panics() {
        let mut r = MockRenderer;
        let _ = r.initialize(RenderTarget::Window, 800, 600).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_present_panics() {
        let mut r = MockRenderer;
        let _ = r.present(&[0u8; 4]).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_resize_panics() {
        let mut r = MockRenderer;
        let _ = r.resize(1024, 768).unwrap();
    }
} 