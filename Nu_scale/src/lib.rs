// External crates
use anyhow::Result;

// Public modules
pub mod capture;
pub mod ui;
pub mod upscale;

// Import Upscaler trait for is_supported() methods
use upscale::Upscaler;
// Re-export upscaling algorithm types
pub use upscale::common::UpscalingAlgorithm;

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application initialization
pub fn init() -> Result<()> {
    // Initialize app components
    Ok(())
}

/// Get the application name
pub fn app_name() -> &'static str {
    "NU Scale"
}

/// Get the application version
pub fn app_version() -> &'static str {
    VERSION
}

/// Upscale a single image file
pub fn upscale_image(
    input_path: &str,
    output_path: &str,
    technology: upscale::UpscalingTechnology,
    quality: upscale::UpscalingQuality,
    scale_factor: f32,
) -> Result<()> {
    upscale::upscale_image(
        input_path,
        output_path,
        technology,
        quality,
        scale_factor,
        None, // Use default algorithm based on quality
    )
}

/// Upscale a single image file with specified algorithm
pub fn upscale_image_with_algorithm(
    input_path: &str,
    output_path: &str,
    technology: upscale::UpscalingTechnology,
    quality: upscale::UpscalingQuality,
    scale_factor: f32,
    algorithm: UpscalingAlgorithm,
) -> Result<()> {
    upscale::upscale_image(
        input_path,
        output_path,
        technology,
        quality,
        scale_factor,
        Some(algorithm),
    )
}

/// Check if FSR is supported
pub fn is_fsr_supported() -> bool {
    upscale::fsr::FsrUpscaler::is_supported()
}

/// Check if DLSS is supported
pub fn is_dlss_supported() -> bool {
    upscale::dlss::DlssUpscaler::is_supported()
}

/// Convert a string algorithm name to the UpscalingAlgorithm enum
pub fn string_to_algorithm(alg_str: &str) -> Option<UpscalingAlgorithm> {
    match alg_str.to_lowercase().as_str() {
        "nearest" | "nearestneighbor" => Some(UpscalingAlgorithm::NearestNeighbor),
        "bilinear" => Some(UpscalingAlgorithm::Bilinear),
        "bicubic" => Some(UpscalingAlgorithm::Bicubic),
        "lanczos2" => Some(UpscalingAlgorithm::Lanczos2),
        "lanczos3" => Some(UpscalingAlgorithm::Lanczos3),
        "mitchell" => Some(UpscalingAlgorithm::Mitchell),
        "area" => Some(UpscalingAlgorithm::Area),
        _ => None
    }
} 