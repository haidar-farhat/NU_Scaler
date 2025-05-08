use anyhow::Result;
use image::RgbaImage;
use std::path::Path;
use crate::UpscalingAlgorithm;

pub mod fsr;
pub mod fsr3;
pub mod dlss;
pub mod common;
pub mod vulkan;
pub mod xess;

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
    // AMD FidelityFX Super Resolution 3 with Frame Generation
    FSR3,
    // NVIDIA Deep Learning Super Sampling
    DLSS,
    // Intel Xe Super Sampling
    XeSS,
    // NVIDIA Image Scaling
    NIS,
    // CUDA-based upscaling
    CUDA,
    // Vulkan-based upscaling
    Vulkan,
    // GPU (Future Vulkan implementation)
    GPU,
    // Fallback to simple bilinear/bicubic
    Fallback,
}

/// Defines the interface for different upscaling technologies
pub trait Upscaler {
    // Initialize the upscaler
    fn initialize(&mut self, input_width: u32, input_height: u32, output_width: u32, output_height: u32) -> Result<()>;
    
    // Upscale a single image
    fn upscale(&self, input: &RgbaImage) -> Result<RgbaImage>;
    
    // Upscale a single image with a specific algorithm (optional, defaults to calling upscale)
    fn upscale_with_algorithm(&self, input: &RgbaImage, algorithm: UpscalingAlgorithm) -> Result<RgbaImage> {
        // Default implementation just calls the standard upscale
        // Specific implementations might override this if they can change algorithms on the fly
        // or if they need to create a temporary upscaler with the new algorithm.
        log::warn!("Upscaling with algorithm {:?} requested, but {} does not support dynamic algorithm change. Using default.", 
                   algorithm, self.name());
        self.upscale(input)
    }
    
    // Check if the technology is supported on the current system
    fn is_supported() -> bool where Self: Sized;
    
    // Get the name of this upscaler
    fn name(&self) -> &'static str;
    
    // Get the quality level
    fn quality(&self) -> UpscalingQuality;
    
    // Set the quality level
    fn set_quality(&mut self, quality: UpscalingQuality) -> Result<()>;
    
    // Cleanup resources
    fn cleanup(&mut self) -> Result<()>;

    // Check if the upscaler needs initialization
    fn needs_initialization(&self) -> bool;

    // Get current input width
    fn input_width(&self) -> u32;

    // Get current input height
    fn input_height(&self) -> u32;
}

// Factory function to create an upscaler based on the technology
pub fn create_upscaler(
    technology: UpscalingTechnology,
    quality: UpscalingQuality,
    algorithm: Option<UpscalingAlgorithm>,
) -> Result<Box<dyn Upscaler + Send + Sync>> {
    match technology {
        UpscalingTechnology::FSR => {
            if !fsr::FsrUpscaler::is_supported() {
                log::info!("FSR not supported, falling back to basic upscaling");
                return create_basic_upscaler(quality, algorithm);
            }
            let upscaler = fsr::FsrUpscaler::new(quality)?;
            Ok(Box::new(upscaler))
        },
        UpscalingTechnology::FSR3 => {
            if !fsr3::Fsr3Upscaler::is_supported() {
                log::info!("FSR3 not supported, falling back to FSR2");
                // Try regular FSR first
                if fsr::FsrUpscaler::is_supported() {
                    log::info!("Falling back to FSR2");
                    let upscaler = fsr::FsrUpscaler::new(quality)?;
                    return Ok(Box::new(upscaler));
                }
                return create_basic_upscaler(quality, algorithm);
            }
            
            // Create FSR3 upscaler
            let upscaler = fsr3::Fsr3Upscaler::new(quality, false)?;
            Ok(Box::new(upscaler))
        },
        UpscalingTechnology::DLSS => {
            if !dlss::DlssUpscaler::is_supported() {
                log::info!("DLSS not supported, falling back to FSR");
                
                // Try FSR as fallback
                if fsr::FsrUpscaler::is_supported() {
                    log::info!("Falling back to FSR");
                    let upscaler = fsr::FsrUpscaler::new(quality)?;
                    return Ok(Box::new(upscaler));
                }
                
                log::info!("FSR not supported either, falling back to basic upscaling");
                return create_basic_upscaler(quality, algorithm);
            }
            
            // Create DLSS upscaler
            let upscaler = dlss::DlssUpscaler::new(quality)?;
            Ok(Box::new(upscaler))
        },
        UpscalingTechnology::XeSS => {
            // Check if XeSS is supported
            if xess::XeSSUpscaler::is_supported() {
                log::info!("Using Intel XeSS for upscaling");
                match xess::XeSSUpscaler::new(quality) {
                    Ok(upscaler) => {
                        return Ok(Box::new(upscaler));
                    },
                    Err(e) => {
                        log::error!("Failed to create XeSS upscaler: {}", e);
                        // Fall through to fallbacks
                    }
                }
            } else {
                log::info!("Intel XeSS not supported, checking other technologies");
            }

            // Try FSR next
            if fsr::FsrUpscaler::is_supported() {
                log::info!("Falling back to FSR");
                let upscaler = fsr::FsrUpscaler::new(quality)?;
                return Ok(Box::new(upscaler));
            }
            
            // Try DLSS next
            if dlss::DlssUpscaler::is_supported() {
                log::info!("Falling back to DLSS");
                let upscaler = dlss::DlssUpscaler::new(quality)?;
                return Ok(Box::new(upscaler));
            }
            
            // Finally fall back to basic
            log::info!("No GPU acceleration available, falling back to basic upscaling");
            return create_basic_upscaler(quality, algorithm);
        },
        UpscalingTechnology::NIS => {
            // Currently NIS is not implemented, so fall back to other technologies
            log::info!("NIS not implemented, falling back to other technologies");
            
            // Try FSR next
            if fsr::FsrUpscaler::is_supported() {
                log::info!("Falling back to FSR");
                let upscaler = fsr::FsrUpscaler::new(quality)?;
                return Ok(Box::new(upscaler));
            }
            
            // Try DLSS next
            if dlss::DlssUpscaler::is_supported() {
                log::info!("Falling back to DLSS");
                let upscaler = dlss::DlssUpscaler::new(quality)?;
                return Ok(Box::new(upscaler));
            }
            
            // Fall back to basic upscaling
            log::info!("No GPU acceleration available, falling back to basic upscaling");
            return create_basic_upscaler(quality, algorithm);
        },
        UpscalingTechnology::CUDA => {
            log::info!("CUDA upscaling is not available in this build");
            
            // Try other GPU technologies first
            if fsr::FsrUpscaler::is_supported() {
                log::info!("Falling back to FSR");
                let upscaler = fsr::FsrUpscaler::new(quality)?;
                return Ok(Box::new(upscaler));
            }
            
            // Try DLSS next
            if dlss::DlssUpscaler::is_supported() {
                log::info!("Falling back to DLSS");
                let upscaler = dlss::DlssUpscaler::new(quality)?;
                return Ok(Box::new(upscaler));
            }
            
            log::info!("No GPU acceleration available, falling back to basic upscaling");
            return create_basic_upscaler(quality, algorithm);
        },
        UpscalingTechnology::Vulkan => {
            // Check if Vulkan is supported
            if !crate::render::VulkanRenderer::is_supported() {
                log::info!("Vulkan not supported, trying other technologies");
                
                // Try FSR next
                if fsr::FsrUpscaler::is_supported() {
                    log::info!("Falling back to FSR");
                    let upscaler = fsr::FsrUpscaler::new(quality)?;
                    return Ok(Box::new(upscaler));
                }
                
                // Try DLSS next
                if dlss::DlssUpscaler::is_supported() {
                    log::info!("Falling back to DLSS");
                    let upscaler = dlss::DlssUpscaler::new(quality)?;
                    return Ok(Box::new(upscaler));
                }
                
                log::info!("No GPU acceleration available, falling back to basic upscaling");
                return create_basic_upscaler(quality, algorithm);
            }
            
            // Create Vulkan upscaler using adapter pattern
            log::info!("Creating Vulkan-based upscaler");
            let alg = algorithm.unwrap_or_else(|| quality_to_algorithm(quality));
            
            // Currently not implemented
            log::warn!("Vulkan upscaler not fully implemented, using fallback");
            return create_basic_upscaler(quality, Some(alg));
        },
        UpscalingTechnology::GPU => {
            // Placeholder for Vulkan implementation
            // For now, always fall back to checking other technologies
            log::info!("GPU (Vulkan) upscaler not yet implemented, checking other technologies");

            // Try FSR next
            if fsr::FsrUpscaler::is_supported() {
                log::info!("Falling back to FSR");
                let upscaler = fsr::FsrUpscaler::new(quality)?;
                return Ok(Box::new(upscaler));
            }
            
            // Try DLSS next
            if dlss::DlssUpscaler::is_supported() {
                log::info!("Falling back to DLSS");
                let upscaler = dlss::DlssUpscaler::new(quality)?;
                return Ok(Box::new(upscaler));
            }
            
            // Finally fall back to basic
            log::info!("No GPU acceleration available, falling back to basic upscaling");
            return create_basic_upscaler(quality, algorithm);
        },
        UpscalingTechnology::None => {
            // No upscaling, just return a pass-through upscaler
            let upscaler = common::PassThroughUpscaler::new();
            Ok(Box::new(upscaler))
        },
        UpscalingTechnology::Fallback => {
            // Basic upscaling with specified algorithm if provided
            create_basic_upscaler(quality, algorithm)
        },
    }
}

// Helper function to create a basic upscaler to reduce code duplication
fn create_basic_upscaler(
    quality: UpscalingQuality,
    algorithm: Option<UpscalingAlgorithm>
) -> Result<Box<dyn Upscaler + Send + Sync>> {
    let upscaler = if let Some(alg) = algorithm {
        common::BasicUpscaler::with_algorithm(quality, alg)
    } else {
        common::BasicUpscaler::new(quality)
    };
    Ok(Box::new(upscaler))
}

// Helper function to convert quality to algorithm
fn quality_to_algorithm(quality: UpscalingQuality) -> UpscalingAlgorithm {
    match quality {
        UpscalingQuality::Ultra => UpscalingAlgorithm::Lanczos3,
        UpscalingQuality::Quality => UpscalingAlgorithm::Bicubic,
        UpscalingQuality::Balanced => UpscalingAlgorithm::Bicubic,
        UpscalingQuality::Performance => UpscalingAlgorithm::Bilinear,
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