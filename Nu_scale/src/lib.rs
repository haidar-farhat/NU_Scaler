// External crates
use anyhow::Result;

// Public modules
pub mod capture;
pub mod ui;
pub mod upscale;

// Import Upscaler trait for is_supported() methods
use upscale::Upscaler;

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
    upscale::upscale_image_file(
        &std::path::Path::new(input_path),
        &std::path::Path::new(output_path),
        technology,
        quality,
        scale_factor,
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