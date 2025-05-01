use anyhow::Result;

/// Upscaling quality levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpscalingQuality {
    Ultra,
    Quality,
    Balanced,
    Performance,
}

/// Supported upscaling technologies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpscalingTechnology {
    None,
    FSR,
    DLSS,
    Wgpu,
    Fallback,
}

/// Trait for upscaling algorithms
pub trait Upscaler {
    /// Initialize the upscaler
    fn initialize(&mut self, input_width: u32, input_height: u32, output_width: u32, output_height: u32) -> Result<()>;
    /// Upscale a single frame (raw bytes or image)
    fn upscale(&self, input: &[u8]) -> Result<Vec<u8>>;
    /// Get the name of this upscaler
    fn name(&self) -> &'static str;
    /// Get the quality level
    fn quality(&self) -> UpscalingQuality;
    /// Set the quality level
    fn set_quality(&mut self, quality: UpscalingQuality) -> Result<()>;
}

/// Mock implementation for testing
pub struct MockUpscaler;

impl Upscaler for MockUpscaler {
    fn initialize(&mut self, _input_width: u32, _input_height: u32, _output_width: u32, _output_height: u32) -> Result<()> {
        unimplemented!()
    }
    fn upscale(&self, _input: &[u8]) -> Result<Vec<u8>> {
        unimplemented!()
    }
    fn name(&self) -> &'static str {
        "MockUpscaler"
    }
    fn quality(&self) -> UpscalingQuality {
        UpscalingQuality::Quality
    }
    fn set_quality(&mut self, _quality: UpscalingQuality) -> Result<()> {
        unimplemented!()
    }
} 