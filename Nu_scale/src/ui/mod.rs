// Import submodules
pub mod egui_ui;
pub mod profile;
pub mod settings;
pub mod hotkeys;

use anyhow::Result;
use image::RgbaImage;
use std::sync::{Arc, Mutex};

/// Run the UI
pub fn run_ui() -> Result<()> {
    egui_ui::run_app()
}

/// Function type for processing frames during fullscreen rendering
pub type FrameProcessor = dyn FnMut(&RgbaImage) -> Result<RgbaImage> + Send + 'static;

/// Run a fullscreen renderer using wgpu and egui for hardware-accelerated upscaling
pub fn run_fullscreen_renderer(
    buffer: Arc<crate::capture::common::FrameBuffer>,
    stop_signal: Arc<Mutex<bool>>,
    processor: impl FnMut(&RgbaImage) -> Result<RgbaImage> + Send + 'static,
) -> Result<()> {
    // Create a specialized version of the UI for fullscreen rendering
    egui_ui::run_fullscreen_renderer(buffer, stop_signal, processor)
} 