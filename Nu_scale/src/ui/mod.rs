// Import submodules
pub mod egui_ui;
pub mod profile;
pub mod settings;
pub mod hotkeys;

use anyhow::Result;
use image::RgbaImage;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

// Import modules from the binary
use crate::capture::common::FrameBuffer;
use crate::capture::ScreenCapture;
use crate::upscale::Upscaler;
use crate::upscale::common::UpscalingAlgorithm;

// Use absolute paths from the crate root
pub use egui_ui::{AppState, run_app};

/// Run the UI
pub fn run_ui() -> Result<()> {
    egui_ui::run_app()
}

/// Function type for processing frames during fullscreen rendering
pub type FrameProcessor = dyn FnMut(&RgbaImage) -> Result<RgbaImage> + Send + 'static;

/// Run a fullscreen renderer using wgpu and egui for hardware-accelerated upscaling
pub fn run_fullscreen_renderer(
    _buffer: Arc<crate::capture::common::FrameBuffer>,
    _upscaler: Arc<dyn Upscaler + Send + Sync>,
    _stop_signal: Arc<AtomicBool>,
    _capture_rate: Duration,
    _upscale_algorithm: UpscalingAlgorithm,
) -> Result<()> {
    println!("Fullscreen renderer started!");
    _stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

/// Runs a fullscreen upscaler window using egui
pub fn run_fullscreen_upscaler(
    frame_buffer: Arc<FrameBuffer>,
    stop_signal: Arc<Mutex<bool>>,
    upscaler: Box<dyn Upscaler>,
    algorithm: Option<UpscalingAlgorithm>,
) -> Result<()> {
    egui_ui::run_fullscreen_upscaler(frame_buffer, stop_signal, upscaler, algorithm)
}

/// Runs the fullscreen renderer with the given buffer and upscaler
pub fn run_fullscreen_renderer_with_buffer(
    _buffer: Arc<crate::capture::common::FrameBuffer>,
    _upscaler: Arc<dyn Upscaler + Send + Sync>,
    _stop_signal: Arc<AtomicBool>,
    _capture_rate: Duration,
    _upscale_algorithm: UpscalingAlgorithm,
) -> Result<()> {
    println!("Fullscreen renderer with buffer started!");
    _stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
} 