// External crates
use anyhow::Result;

// Public modules
pub mod capture;
pub mod ui;
pub mod upscale;
pub mod renderer;
pub mod render;
pub mod logger;

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

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application initialization
pub fn init() -> Result<()> {
    // Initialize logger
    let logs_dir = dirs::data_dir()
        .map(|dir| dir.join("NU_Scaler").join("logs").to_string_lossy().to_string())
        .unwrap_or_else(|| "logs".to_string());
    
    // Initialize logger with logs directory
    if let Err(e) = logger::init_logger(Some(&logs_dir), true) {
        eprintln!("Warning: Failed to initialize logger: {}", e);
    }
    
    // Log application startup
    log::info!("NU_Scaler v{} starting up", VERSION);
    
    // Initialize other app components here
    
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
    use log::{debug, info, warn, error};
    
    info!("Starting borderless upscaling with {:?} technology at {:?} quality", technology, quality);
    debug!("Target FPS: {}, Algorithm: {:?}", fps, algorithm);
    
    // Create a capturer for the source
    let mut capturer = match capture::create_capturer() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to create screen capturer: {}", e);
            return Err(anyhow::anyhow!("Failed to create screen capturer: {}", e));
        }
    };
    
    // Create a stop signal for the upscaling thread
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_clone = stop_signal.clone();
    
    // Create a frame buffer to share frames between threads
    let frame_buffer = Arc::new(capture::common::FrameBuffer::new(5));
    let frame_buffer_clone = frame_buffer.clone();
    
    info!("Starting capture thread");
    
    // Start a capture thread
    let capture_thread = std::thread::spawn(move || {
        let target_frame_time = std::time::Duration::from_secs_f64(1.0 / fps as f64);
        let mut frames_captured = 0;
        let mut last_fps_log = std::time::Instant::now();
        let mut errors_count = 0;
        
        debug!("Capture thread started, target frame time: {:?}", target_frame_time);
        
        while !stop_signal_clone.load(Ordering::SeqCst) {
            let start_time = std::time::Instant::now();
            
            // Capture frame
            match capturer.capture_frame(&source) {
                Ok(frame) => {
                    // Log dimensions occasionally
                    if frames_captured % 100 == 0 {
                        debug!("Captured frame {}x{}", frame.width(), frame.height());
                    }
                    
                    // Log performance every second
                    if last_fps_log.elapsed().as_secs() >= 1 {
                        let actual_fps = frames_captured as f32 / last_fps_log.elapsed().as_secs_f32();
                        debug!("Capture performance: {:.2} FPS", actual_fps);
                        frames_captured = 0;
                        last_fps_log = std::time::Instant::now();
                    }
                    
                    // Track frame count
                    frames_captured += 1;
                    
                    // Use our logger utility
                    logger::log_capture_event(
                        &format!("{:?}", source),
                        frame.width(),
                        frame.height()
                    );
                    
                    // Push frame to buffer
                    if let Err(e) = frame_buffer_clone.add_frame(frame) {
                        warn!("Failed to add frame to buffer: {}", e);
                    }
                },
                Err(e) => {
                    errors_count += 1;
                    error!("Error capturing frame: {}", e);
                    
                    // Break after too many consecutive errors
                    if errors_count > 10 {
                        error!("Too many capture errors, stopping capture thread");
                        break;
                    }
                    
                    // Add small delay before trying again
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
            
            // Sleep to maintain target FPS
            let elapsed = start_time.elapsed();
            if elapsed < target_frame_time {
                std::thread::sleep(target_frame_time - elapsed);
            } else if frames_captured % 10 == 0 {
                // Log if we're falling behind on FPS
                warn!("Frame capture taking longer than target: {:?} > {:?}", 
                      elapsed, target_frame_time);
            }
        }
        
        info!("Capture thread stopped");
    });
    
    // Start the fullscreen UI on the main thread to avoid multiple event loops
    info!("Starting fullscreen renderer");
    let result = match renderer::fullscreen::run_fullscreen_upscaler(
        frame_buffer,
        stop_signal.clone(),
        technology,
        quality,
        algorithm,
    ) {
        Ok(_) => {
            info!("Fullscreen renderer completed successfully");
            Ok(())
        },
        Err(e) => {
            // Signal the capture thread to stop even if there was an error
            stop_signal.store(true, Ordering::SeqCst);
            
            error!("Fullscreen renderer failed: {}", e);
            // If it's the lock file or EventLoop error, provide a more user-friendly message
            if e.contains("another instance") || e.contains("already running") {
                Err(anyhow::anyhow!("Another NU_Scaler window is already open. Please close it before starting a new upscaling session."))
            } else {
                Err(anyhow::anyhow!("{}", e))
            }
        }
    };
    
    // Wait for capture thread to finish
    match capture_thread.join() {
        Ok(_) => info!("Capture thread joined successfully"),
        Err(e) => error!("Failed to join capture thread: {:?}", e),
    }
    
    result
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
    _source: capture::CaptureTarget,
    technology: upscale::UpscalingTechnology,
    quality: upscale::UpscalingQuality,
    _fps: u32,
    _algorithm: Option<UpscalingAlgorithm>,
) -> Result<()> {
    println!("Starting fullscreen upscaling with {:?} technology at {:?} quality", 
             technology, quality);
    
    // This is a placeholder implementation until all modules are fully working
    println!("Fullscreen rendering is currently under development");
    
    // Return success for now
    Ok(())
}

// Convert a string algorithm name to the UpscalingAlgorithm enum
pub fn string_to_algorithm(alg_str: &str) -> Option<UpscalingAlgorithm> {
    match alg_str.to_lowercase().as_str() {
        "nearest" | "nearestneighbor" => Some(UpscalingAlgorithm::NearestNeighbor),
        "bilinear" => Some(UpscalingAlgorithm::Bilinear),
        "bicubic" => Some(UpscalingAlgorithm::Bicubic),
        "lanczos2" => Some(UpscalingAlgorithm::Lanczos2),
        "lanczos3" => Some(UpscalingAlgorithm::Lanczos3),
        "mitchell" => Some(UpscalingAlgorithm::Mitchell),
        "area" => Some(UpscalingAlgorithm::Area),
        _ => None,
    }
} 

#[cfg(test)]
mod import_test {
    use super::*;
}
