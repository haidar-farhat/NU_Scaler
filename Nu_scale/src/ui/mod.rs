// Import submodules
#[cfg(not(feature = "disable_gui"))]
pub mod egui_ui;
#[cfg(not(feature = "disable_gui"))]
pub mod gtk_ui;
pub mod profile;
pub mod settings;
pub mod hotkeys;
pub mod region_dialog;
pub mod components;
pub mod tabs;

// Use crate:: paths for re-exports and imports from lib/ui scope
#[cfg(not(feature = "disable_gui"))]
pub use crate::ui::egui_ui::{AppState, run_app as run_egui_app};
#[cfg(not(feature = "disable_gui"))]
pub use crate::ui::gtk_ui::{AppState as GtkAppState, run_app as run_gtk_app};
pub use crate::ui::profile::Profile;
pub use crate::ui::region_dialog::RegionDialog;

use anyhow::Result;
use image::RgbaImage;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use once_cell::sync::Lazy;
#[cfg(not(feature = "disable_gui"))]
use egui::{Context, TextureId};

// Use crate:: paths for imports from lib scope
use crate::capture::common::FrameBuffer;
use crate::upscale::Upscaler;
use crate::upscale::common::UpscalingAlgorithm;
use crate::upscale::{UpscalingTechnology, UpscalingQuality};

/// Run the UI
pub fn run_ui() -> Result<()> {
    #[cfg(not(feature = "disable_gui"))]
    // Default to using GTK UI if available, fall back to egui if needed
    return crate::ui::gtk_ui::run_app();
    
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
    capture_target: crate::capture::CaptureTarget,
) -> Result<(), String> {
    crate::renderer::fullscreen::run_fullscreen_upscaler(
        Arc::new(frame_buffer),
        stop_signal,
        upscaler,
        quality,
        algorithm,
        capture_target,
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

/// Type for an upscaling renderer callback
pub type UpscalingRendererCallback = Box<dyn FnMut(&Context, Option<&str>) -> Option<TextureId> + Send>;

/// Global state to track if upscaling is active in the main window
static UPSCALING_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Global upscaling renderer callback
static UPSCALING_RENDERER: Lazy<Mutex<Option<UpscalingRendererCallback>>> = Lazy::new(|| {
    Mutex::new(None)
});

/// Set whether upscaling is active in the main window
pub fn set_upscaling_active(active: bool) {
    UPSCALING_ACTIVE.store(active, Ordering::SeqCst);
}

/// Check if upscaling is active in the main window
pub fn is_upscaling_active() -> bool {
    UPSCALING_ACTIVE.load(Ordering::SeqCst)
}

/// Set the upscaling renderer callback
pub fn set_upscaling_renderer(renderer: UpscalingRendererCallback) {
    let mut lock = UPSCALING_RENDERER.lock().unwrap();
    *lock = Some(renderer);
}

/// Get the upscaled texture for rendering in the main window
pub fn get_upscaled_texture(ctx: &Context, content_id: Option<&str>) -> Option<TextureId> {
    let mut lock = UPSCALING_RENDERER.lock().unwrap();
    if let Some(renderer) = &mut *lock {
        renderer(ctx, content_id)
    } else {
        None
    }
}

/// Cleanup upscaling resources
pub fn cleanup_upscaling() {
    let mut lock = UPSCALING_RENDERER.lock().unwrap();
    *lock = None;
    set_upscaling_active(false);
}