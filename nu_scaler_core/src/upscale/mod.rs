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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_initialize_panics() {
        let mut up = MockUpscaler;
        let _ = up.initialize(1, 1, 2, 2).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_upscale_panics() {
        let up = MockUpscaler;
        let _ = up.upscale(&[0u8; 4]).unwrap();
    }

    #[test]
    fn test_name_and_quality() {
        let up = MockUpscaler;
        assert_eq!(up.name(), "MockUpscaler");
        assert_eq!(up.quality(), UpscalingQuality::Quality);
    }

    #[test]
    #[should_panic]
    fn test_set_quality_panics() {
        let mut up = MockUpscaler;
        let _ = up.set_quality(UpscalingQuality::Ultra).unwrap();
    }
} 