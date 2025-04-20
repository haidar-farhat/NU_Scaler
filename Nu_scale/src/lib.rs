// External crates
use anyhow::Result;

// Public modules
pub mod capture;
pub mod ui;
pub mod upscale;

// Import Upscaler trait for is_supported() methods
use upscale::Upscaler;
// Import ScreenCapture trait
use capture::ScreenCapture;
// Re-export upscaling algorithm types
pub use upscale::common::UpscalingAlgorithm;
use std::sync::{Arc, Mutex};
use image::RgbaImage;

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

/// Start fullscreen upscaling mode which captures, upscales and renders frames in real-time
pub fn start_fullscreen_upscale_renderer(
    source: capture::CaptureTarget,
    technology: upscale::UpscalingTechnology,
    quality: upscale::UpscalingQuality,
    fps: u32,
    algorithm: Option<UpscalingAlgorithm>,
) -> Result<()> {
    println!("Starting fullscreen upscaling with {:?} technology at {:?} quality", 
             technology, quality);
    
    // Create frame buffer to store captured frames
    let buffer = Arc::new(capture::common::FrameBuffer::new(5));
    let stop_signal = Arc::new(Mutex::new(false));
    
    // Start capture thread
    let capture_buffer = Arc::clone(&buffer);
    let capture_stop = Arc::clone(&stop_signal);
    let capture_handle = capture::common::start_live_capture_thread(
        source.clone(),
        fps,
        capture_buffer,
        capture_stop,
    )?;
    
    // Create upscaler and wrap in Arc<Mutex<>> for thread safety
    let mut upscaler = upscale::create_upscaler(technology, quality, algorithm)?;
    
    // Get screen dimensions for fullscreen rendering
    let capturer = capture::create_capturer()?;
    let (screen_width, screen_height) = capturer.get_primary_screen_dimensions()?;
    
    // Initialize upscaler with target dimensions
    // We don't know input dimensions yet, but we'll assume fullscreen input for now
    // The actual input dimensions will come from the frames
    upscaler.initialize(screen_width, screen_height, screen_width, screen_height)?;
    
    // Move upscaler to an Arc<Mutex<>> for sharing between threads
    let upscaler = Arc::new(Mutex::new(upscaler));
    
    // Start a wgpu-based renderer for fullscreen display
    let render_buffer = Arc::clone(&buffer);
    let render_stop = Arc::clone(&stop_signal);
    let render_upscaler = Arc::clone(&upscaler);
    
    // Run the renderer in the UI context using eframe
    let _ui_result = ui::run_fullscreen_renderer(render_buffer, render_stop, move |frame: &RgbaImage| {
        // Lock the upscaler for this frame processing
        let mut upscaler = render_upscaler.lock().map_err(|_| anyhow::anyhow!("Failed to lock upscaler"))?;
        
        // Check if we need to reinitialize with the actual input dimensions
        if upscaler.input_width() != frame.width() || upscaler.input_height() != frame.height() {
            upscaler.initialize(frame.width(), frame.height(), screen_width, screen_height)?;
        }
        
        // Perform upscaling using the created upscaler
        let upscaled = upscaler.upscale(frame)?;
        Ok(upscaled)
    })?;
    
    // Clean up
    {
        let mut stop = stop_signal.lock().unwrap();
        *stop = true;
    }
    
    // Wait for capture thread to finish
    if let Err(e) = capture_handle.join() {
        println!("Error joining capture thread: {:?}", e);
    }
    
    // Clean up upscaler resources
    if let Ok(mut upscaler) = upscaler.lock() {
        upscaler.cleanup()?;
    }
    
    Ok(())
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