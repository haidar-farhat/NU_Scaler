use anyhow::Result;
use image::RgbaImage;
use std::path::Path;

pub mod fsr;
pub mod dlss;
pub mod common;

use common::UpscalingAlgorithm;

// Quality levels for upscaling
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpscalingQuality {
    Ultra,
    Quality,
    Balanced,
    Performance,
}

// Supported upscaling technologies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpscalingTechnology {
    // No upscaling
    None,
    // AMD FidelityFX Super Resolution
    FSR,
    // NVIDIA Deep Learning Super Sampling
    DLSS,
    // Fallback to simple bilinear/bicubic
    Fallback,
}

// Trait for upscaling implementations
pub trait Upscaler {
    // Initialize the upscaler
    fn initialize(&mut self, input_width: u32, input_height: u32, output_width: u32, output_height: u32) -> Result<()>;
    
    // Upscale a single image
    fn upscale(&self, input: &RgbaImage) -> Result<RgbaImage>;
    
    // Cleanup resources
    fn cleanup(&mut self) -> Result<()>;
    
    // Check if this upscaler is supported on the current hardware
    fn is_supported() -> bool where Self: Sized;
    
    // Get the name of this upscaler
    fn name(&self) -> &'static str;
    
    // Get the quality level
    fn quality(&self) -> UpscalingQuality;
    
    // Set the quality level
    fn set_quality(&mut self, quality: UpscalingQuality) -> Result<()>;
}

// Factory function to create an upscaler based on the technology
pub fn create_upscaler(
    technology: UpscalingTechnology,
    quality: UpscalingQuality,
    algorithm: Option<UpscalingAlgorithm>,
) -> Result<Box<dyn Upscaler>> {
    match technology {
        UpscalingTechnology::FSR => {
            if fsr::FsrUpscaler::is_supported() {
                let upscaler = fsr::FsrUpscaler::new(quality)?;
                Ok(Box::new(upscaler))
            } else {
                // Fall back to basic upscaling if FSR is not supported
                let upscaler = if let Some(alg) = algorithm {
                    common::BasicUpscaler::with_algorithm(quality, alg)
                } else {
                    common::BasicUpscaler::new(quality)
                };
                println!("FSR not supported, falling back to basic upscaling");
                Ok(Box::new(upscaler))
            }
        },
        UpscalingTechnology::DLSS => {
            if dlss::DlssUpscaler::is_supported() {
                let upscaler = dlss::DlssUpscaler::new(quality)?;
                Ok(Box::new(upscaler))
            } else {
                // Fall back to FSR if DLSS is not supported
                if fsr::FsrUpscaler::is_supported() {
                    let upscaler = fsr::FsrUpscaler::new(quality)?;
                    println!("DLSS not supported, falling back to FSR");
                    Ok(Box::new(upscaler))
                } else {
                    // Fall back to basic upscaling if neither is supported
                    let upscaler = if let Some(alg) = algorithm {
                        common::BasicUpscaler::with_algorithm(quality, alg)
                    } else {
                        common::BasicUpscaler::new(quality)
                    };
                    println!("Neither DLSS nor FSR supported, falling back to basic upscaling");
                    Ok(Box::new(upscaler))
                }
            }
        },
        UpscalingTechnology::None => {
            // No upscaling, just return a pass-through upscaler
            let upscaler = common::PassThroughUpscaler::new();
            Ok(Box::new(upscaler))
        },
        UpscalingTechnology::Fallback => {
            // Basic upscaling with specified algorithm if provided
            let upscaler = if let Some(alg) = algorithm {
                common::BasicUpscaler::with_algorithm(quality, alg)
            } else {
                common::BasicUpscaler::new(quality)
            };
            Ok(Box::new(upscaler))
        },
    }
}

// Utility function to upscale an image file
pub fn upscale_image_file(
    input_path: &Path,
    output_path: &Path,
    technology: UpscalingTechnology,
    quality: UpscalingQuality,
    scale_factor: f32,
    algorithm: Option<UpscalingAlgorithm>,
) -> Result<()> {
    // Load the input image
    let input_image = image::open(input_path)?.to_rgba8();
    
    // Calculate output dimensions
    let input_width = input_image.width();
    let input_height = input_image.height();
    let output_width = (input_width as f32 * scale_factor) as u32;
    let output_height = (input_height as f32 * scale_factor) as u32;
    
    // Create and initialize upscaler
    let mut upscaler = create_upscaler(technology, quality, algorithm)?;
    upscaler.initialize(input_width, input_height, output_width, output_height)?;
    
    // Upscale the image
    let output_image = upscaler.upscale(&input_image)?;
    
    // Save the output image
    output_image.save(output_path)?;
    
    // Cleanup
    upscaler.cleanup()?;
    
    Ok(())
}

// Simplified version for the lib.rs interface
pub fn upscale_image(
    input_path: &str,
    output_path: &str,
    technology: UpscalingTechnology,
    quality: UpscalingQuality,
    scale_factor: f32,
    algorithm: Option<UpscalingAlgorithm>,
) -> Result<()> {
    upscale_image_file(
        &std::path::Path::new(input_path),
        &std::path::Path::new(output_path),
        technology,
        quality,
        scale_factor,
        algorithm,
    )
} 