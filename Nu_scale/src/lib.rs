// External crates
use anyhow::Result;

// Public modules
pub mod capture;
pub mod ui;
pub mod upscale;
pub mod import_test;
pub mod renderer;

// Explicitly re-export top-level modules
// pub use upscale; // Make upscale directly accessible as crate::upscale
// pub use renderer; // Make renderer directly accessible as crate::renderer

// Re-export modules for internal use
#[allow(unused_imports)]
pub use crate as nu_scaler;

// Import Upscaler trait for is_supported() methods
use crate::upscale::Upscaler;
// Import ScreenCapture trait
use crate::capture::ScreenCapture;
// Re-export upscaling algorithm types
pub use crate::upscale::common::UpscalingAlgorithm;
use std::sync::{Arc, Mutex};
use image::RgbaImage;
use std::sync::atomic::AtomicBool;

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

/// Toggle fullscreen mode for the current window
#[cfg(not(feature = "disable_gui"))]
pub fn toggle_fullscreen(app_state: &mut ui::AppState) -> Result<()> {
    app_state.toggle_fullscreen_mode()
}

#[cfg(feature = "disable_gui")]
pub fn toggle_fullscreen(_: &mut ()) -> Result<()> {
    println!("Fullscreen toggle not available in CLI mode");
    Ok(())
}

/// Starts a borderless window with upscaling functionality 
/// integrated directly into the main process
pub fn start_borderless_upscale(
    source: capture::CaptureTarget,
    technology: upscale::UpscalingTechnology,
    quality: upscale::UpscalingQuality,
    fps: u32,
    algorithm: Option<upscale::common::UpscalingAlgorithm>,
) -> Result<()> {
    use std::sync::atomic::Ordering;
    
    // Create a capturer for the source
    let mut capturer = capture::create_capturer()?;
    
    // Create a stop signal for the upscaling thread
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_clone = stop_signal.clone();
    
    // Create a frame buffer to share frames between threads
    let frame_buffer = Arc::new(capture::common::FrameBuffer::new(5));
    let frame_buffer_clone = frame_buffer.clone();
    
    // Start a capture thread
    let capture_thread = std::thread::spawn(move || {
        let target_frame_time = std::time::Duration::from_secs_f64(1.0 / fps as f64);
        
        while !stop_signal_clone.load(Ordering::SeqCst) {
            let start_time = std::time::Instant::now();
            
            // Capture frame
            match capturer.capture_frame(&source) {
                Ok(frame) => {
                    // Push frame to buffer
                    frame_buffer_clone.add_frame(frame).ok();
                },
                Err(e) => {
                    eprintln!("Error capturing frame: {:?}", e);
                    break;
                }
            }
            
            // Sleep to maintain target FPS
            let elapsed = start_time.elapsed();
            if elapsed < target_frame_time {
                std::thread::sleep(target_frame_time - elapsed);
            }
        }
    });
    
    // Start the fullscreen UI
    let result = renderer::fullscreen::run_fullscreen_upscaler(
        frame_buffer,
        stop_signal,
        technology,
        quality,
        algorithm,
    ).map_err(|e| anyhow::anyhow!("{}", e))?;
    
    // Wait for capture thread to finish
    capture_thread.join().expect("Failed to join capture thread");
    
    Ok(())
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
    
    // This is a placeholder implementation until all modules are fully working
    println!("Fullscreen rendering is currently under development");
    
    // Return success for now
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