// Import submodules
#[cfg(not(feature = "disable_gui"))]
pub mod egui_ui;
pub mod profile;
pub mod settings;
pub mod hotkeys;
pub mod region_dialog;
pub mod components;
pub mod tabs;

// Use crate:: paths for re-exports and imports from lib/ui scope
#[cfg(not(feature = "disable_gui"))]
pub use crate::ui::egui_ui::{AppState, run_app};
pub use crate::ui::profile::Profile;
pub use crate::ui::region_dialog::RegionDialog;

use anyhow::Result;
use image::RgbaImage;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

// Use crate:: paths for imports from lib scope
use crate::capture::common::FrameBuffer;
use crate::upscale::Upscaler;
use crate::upscale::common::UpscalingAlgorithm;
use crate::upscale::{UpscalingTechnology, UpscalingQuality};

/// Run the UI
pub fn run_ui() -> Result<()> {
    #[cfg(not(feature = "disable_gui"))]
    return crate::ui::egui_ui::run_app();
    
    #[cfg(feature = "disable_gui")]
    {
        println!("GUI is disabled. Use the CLI mode instead.");
        Ok(())
    }
}

/// Function type for processing frames during fullscreen rendering
pub type FrameProcessor = dyn FnMut(&RgbaImage) -> Result<RgbaImage> + Send + 'static;

/// Run a fullscreen renderer using wgpu and egui for hardware-accelerated upscaling
pub fn run_fullscreen_renderer(
    _buffer: Arc<crate::capture::common::FrameBuffer>,
    _upscaler: Arc<dyn Upscaler + Send + Sync>,
    _stop_signal: Arc<AtomicBool>,
    _capture_rate: Duration,
    _upscale_algorithm: Option<UpscalingAlgorithm>,
) -> Result<()> {
    println!("Fullscreen renderer started!");
    _stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

/// Runs a fullscreen upscaler window using egui
pub fn run_fullscreen_upscaler(
    frame_buffer: FrameBuffer,
    stop_signal: Arc<AtomicBool>,
    upscaler: UpscalingTechnology,
    quality: UpscalingQuality,
    algorithm: Option<UpscalingAlgorithm>,
) -> Result<(), String> {
    crate::renderer::fullscreen::run_fullscreen_upscaler(
        Arc::new(frame_buffer),
        stop_signal,
        upscaler,
        quality,
        algorithm,
    )
}

/// Runs the fullscreen renderer with the given buffer and upscaler
pub fn run_fullscreen_renderer_with_buffer(
    _buffer: Arc<crate::capture::common::FrameBuffer>,
    _upscaler: Arc<dyn Upscaler + Send + Sync>,
    _stop_signal: Arc<AtomicBool>,
    _capture_rate: Duration,
    _upscale_algorithm: Option<UpscalingAlgorithm>,
) -> Result<()> {
    println!("Fullscreen renderer with buffer started!");
    _stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

/// Convert a string to an UpscalingAlgorithm enum
pub fn string_to_algorithm(algorithm: &str) -> UpscalingAlgorithm {
    match algorithm.to_lowercase().as_str() {
        "nearest" => UpscalingAlgorithm::NearestNeighbor,
        "bilinear" => UpscalingAlgorithm::Bilinear,
        "bicubic" => UpscalingAlgorithm::Bicubic,
        "lanczos" | "lanczos3" => UpscalingAlgorithm::Lanczos3,
        "lanczos2" => UpscalingAlgorithm::Lanczos2,
        "mitchell" => UpscalingAlgorithm::Mitchell,
        "area" => UpscalingAlgorithm::Area,
        "balanced" => UpscalingAlgorithm::Balanced,
        _ => UpscalingAlgorithm::Lanczos3, // Default
    }
}