use anyhow::{anyhow, Result};
use log;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Instant,
};

use image::{
    DynamicImage,
    imageops,
    imageops::FilterType,
    RgbaImage,
};

use crate::{
    capture::{CaptureTarget, ScreenCapture},
    upscale::{UpscalingQuality, UpscalingTechnology},
};

/// Type alias for upscaling functionality
#[allow(dead_code)]
type UpscaleResult = Result<RgbaImage>;

/// Module to provide upscaling functionality through the public API
mod upscale_api {
    use anyhow::Result;
    use image::{imageops, imageops::FilterType, RgbaImage};

    pub fn upscale_image(
        input: &RgbaImage,
        width: u32,
        height: u32,
        technology: &str,
        quality: &str,
    ) -> Result<RgbaImage> {
        // Map technology/quality strings if needed (unused in simplified version)
        let _ = match technology.to_lowercase().as_str() {
            "fsr" => 1,
            "dlss" => 2,
            _ => 3,
        };
        let _ = match quality.to_lowercase().as_str() {
            "ultra" => 0,
            "quality" => 1,
            "balanced" => 2,
            "performance" => 3,
            _ => 2,
        };

        // Fallback resizing
        Ok(imageops::resize(input, width, height, FilterType::Lanczos3))
    }
}

/// Captures a screenshot and saves it to the specified path
pub fn capture_screenshot(target: &CaptureTarget, output_path: &Path) -> Result<()> {
    let mut capturer = super::create_capturer()?;
    capturer.save_frame(target, output_path)
}

/// Captures a screenshot and returns it as a DynamicImage
pub fn capture_screenshot_image(target: &CaptureTarget) -> Result<DynamicImage> {
    let mut capturer = super::create_capturer()?;
    let frame = capturer.capture_frame(target)?;
    Ok(DynamicImage::ImageRgba8(frame))
}

/// Captures and upscales content to fullscreen dimensions
pub fn capture_and_upscale_to_fullscreen(
    target: &CaptureTarget,
    _tech: Option<UpscalingTechnology>,
    _qual: Option<UpscalingQuality>,
    _algorithm: Option<&str>,
    _save_path: Option<&Path>,
) -> Result<()> {
    let mut capturer = super::create_capturer()?;
    let source = capturer.capture_frame(target)?;
    let (w, h) = capturer.get_primary_screen_dimensions()?;

    upscale_api::upscale_image(&source, w, h, "fallback", "balanced")?;
    Ok(())
}

/// Lists all available windows with their titles and IDs
pub fn list_available_windows() -> Result<Vec<super::platform::WindowInfo>> {
    let capturer = super::create_capturer()?;
    capturer.list_windows()
}

/// Gets the primary screen dimensions
pub fn get_screen_dimensions() -> Result<(u32, u32)> {
    let capturer = super::create_capturer()?;
    capturer.get_primary_screen_dimensions()
}

/// A frame buffer that stores captured frames for processing
pub struct FrameBuffer {
    frames: Arc<Mutex<Vec<RgbaImage>>>,
    max_size: usize,
}

impl FrameBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            frames: Arc::new(Mutex::new(Vec::with_capacity(capacity))),
            max_size: capacity,
        }
    }

    pub fn add_frame(&self, frame: RgbaImage) -> Result<()> {
        let mut guard = self.frames.lock().map_err(|_| anyhow!("Mutex lock failed"))?;
        if guard.len() >= self.max_size {
            guard.remove(0);
        }
        guard.push(frame);
        Ok(())
    }

    pub fn get_frames(&self) -> Result<Vec<RgbaImage>> {
        let guard = self.frames.lock().map_err(|_| anyhow!("Mutex lock failed"))?;
        Ok(guard.clone())
    }

    pub fn get_latest_frame(&self) -> Result<Option<RgbaImage>> {
        let guard = self.frames.lock().map_err(|_| anyhow!("Mutex lock failed"))?;
        Ok(guard.last().cloned())
    }

    pub fn clear(&self) -> Result<()> {
        let mut guard = self.frames.lock().map_err(|_| anyhow!("Mutex lock failed"))?;
        guard.clear();
        Ok(())
    }

    pub fn len(&self) -> Result<usize> {
        let guard = self.frames.lock().map_err(|_| anyhow!("Mutex lock failed"))?;
        Ok(guard.len())
    }

    pub fn is_empty(&self) -> Result<bool> {
        let guard = self.frames.lock().map_err(|_| anyhow!("Mutex lock failed"))?;
        Ok(guard.is_empty())
    }

    pub fn clone_arc(&self) -> Self {
        Self {
            frames: Arc::clone(&self.frames),
            max_size: self.max_size,
        }
    }
}

/// Starts fullscreen upscaled capture in a background thread
pub fn start_fullscreen_upscaled_capture(
    target: CaptureTarget,
    fps: u32,
    technology: &str,
    quality: &str,
    _algorithm: Option<&str>,
    buffer: Arc<FrameBuffer>,
    stop_signal: Arc<Mutex<bool>>,
) -> Result<thread::JoinHandle<Result<()>>> {
    let buf = Arc::clone(&buffer);
    let stop = Arc::clone(&stop_signal);
    let tech = technology.to_string();
    let qual = quality.to_string();

    let handle = thread::spawn(move || {
        let mut capturer = super::create_capturer()?;
        let (w, h) = capturer.get_primary_screen_dimensions()?;
        let frame_delay = std::time::Duration::from_secs_f64(1.0 / fps as f64);
        let mut next_time = Instant::now();

        loop {
            if *stop.lock().map_err(|_| anyhow!("Mutex lock failed"))? {
                break;
            }

            let frame = capturer.capture_frame(&target)?;
            let up = upscale_api::upscale_image(&frame, w, h, &tech, &qual)?;
            buf.add_frame(up)?;

            next_time += frame_delay;
            let now = Instant::now();
            if next_time > now {
                thread::sleep(next_time - now);
            } else {
                next_time = now + (frame_delay / 2);
            }
        }
        Ok(())
    });

    Ok(handle)
}

/// Starts a live capture thread that pushes frames to a buffer
pub fn start_live_capture_thread(
    target: CaptureTarget,
    fps: u32,
    buffer: Arc<FrameBuffer>,
    stop_signal: Arc<AtomicBool>,
) -> Result<thread::JoinHandle<Result<()>>> {
    let buf = buffer.clone_arc();
    let stop = stop_signal.clone();

    let handle = thread::spawn(move || {
        log::info!(
            "Capture thread started. Target: {:?}, FPS: {}, Capacity: {}",
            target,
            fps,
            buf.max_size
        );
        let mut capturer = super::create_capturer()?;
        let frame_delay = std::time::Duration::from_secs_f64(1.0 / fps as f64);
        let mut next_time = Instant::now();

        while !stop.load(Ordering::SeqCst) {
            let _start = Instant::now();
            match capturer.capture_frame(&target) {
                Ok(frame) => {
                    buf.add_frame(frame)?;
                }
                Err(e) => {
                    log::error!("Capture error: {}", e);
                    thread::sleep(std::time::Duration::from_millis(50));
                }
            }

            next_time += frame_delay;
            let now = Instant::now();
            if next_time > now {
                thread::sleep(next_time - now);
            } else {
                next_time = now + frame_delay;
            }
        }
        log::info!("Capture thread stopped.");
        Ok(())
    });

    Ok(handle)
}

/// Processes frames from a buffer in real time
pub fn process_frame_buffer<F>(
    buffer: Arc<FrameBuffer>,
    stop_signal: Arc<Mutex<bool>>,
    fps: u32,
    mut processor: F,
) -> Result<thread::JoinHandle<Result<()>>>
where
    F: FnMut(&RgbaImage) -> Result<()> + Send + 'static,
{
    let buf = Arc::clone(&buffer);
    let stop = Arc::clone(&stop_signal);

    let handle = thread::spawn(move || {
        let frame_delay = std::time::Duration::from_secs_f64(1.0 / fps as f64);
        let mut next_time = Instant::now();

        loop {
            if *stop.lock().map_err(|_| anyhow!("Mutex lock failed"))? {
                break;
            }
            if let Some(frame) = buf.get_latest_frame()? {
                processor(&frame)?;
            }

            next_time += frame_delay;
            let now = Instant::now();
            if next_time > now {
                thread::sleep(next_time - now);
            } else {
                next_time = now + frame_delay;
            }
        }
        Ok(())
    });

    Ok(handle)
}

/// Captures continuously and updates shared status
pub fn run_capture_thread(
    target: CaptureTarget,
    buffer: Arc<FrameBuffer>,
    stop_signal: Arc<AtomicBool>,
    status: Arc<Mutex<String>>,
    temp_status: Arc<Mutex<Option<(String, std::time::SystemTime)>>>,
) -> Result<()> {
    log::info!("Starting run_capture_thread for {:?}", target);
    let mut capturer = super::create_capturer()?;
    let fps = 60;
    let frame_delay = std::time::Duration::from_secs_f64(1.0 / fps as f64);
    let mut next_time = Instant::now();
    let mut frames = 0;
    let mut last_log = Instant::now();
    let mut errors = 0;

    *status.lock().map_err(|_| anyhow!("Mutex lock failed"))? = format!("Capturing: {:?}", target);

    while !stop_signal.load(Ordering::SeqCst) {
        match capturer.capture_frame(&target) {
            Ok(frame) => {
                errors = 0;
                buffer.add_frame(frame)?;
                frames += 1;
                if last_log.elapsed().as_secs() >= 1 {
                    let fps_now = frames as f32 / last_log.elapsed().as_secs_f32();
                    *status.lock().unwrap() = format!("FPS: {:.1}", fps_now);
                    frames = 0;
                    last_log = Instant::now();
                }
            }
            Err(e) => {
                errors += 1;
                *temp_status.lock().unwrap() =
                    Some((format!("Error: {}", e), std::time::SystemTime::now()));
                if errors > 10 {
                    *status.lock().unwrap() = "Capture failed".into();
                    break;
                }
                thread::sleep(std::time::Duration::from_millis(100));
            }
        }

        next_time += frame_delay;
        let now = Instant::now();
        if next_time > now {
            thread::sleep(next_time - now);
        } else {
            next_time = now + frame_delay;
        }
    }

    *status.lock().unwrap() = "Stopped".into();
    log::info!("run_capture_thread exited");
    Ok(())
}

/// Resizes an image using the specified algorithm.
pub fn resize_image(
    input: &DynamicImage,
    width: u32,
    height: u32,
    algorithm: crate::upscale::common::UpscalingAlgorithm,
    frame_start_time: Instant,
) -> Result<RgbaImage, String> {
    let filter = match algorithm {
        crate::upscale::common::UpscalingAlgorithm::Nearest => FilterType::Nearest,
        crate::upscale::common::UpscalingAlgorithm::Bilinear => FilterType::Triangle,
        crate::upscale::common::UpscalingAlgorithm::Bicubic => FilterType::CatmullRom,
        crate::upscale::common::UpscalingAlgorithm::Lanczos3 => FilterType::Lanczos3,
        _ => FilterType::Lanczos3,
    };
    let _ = frame_start_time.elapsed();
    Ok(imageops::resize(input, width, height, filter))
}

/// Saves an image buffer to a file.
pub fn save_image_buffer(
    path: &Path,
    buffer: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
) -> Result<()> {
    log::info!("Saving image to {}", path.display());
    buffer.save(path).map_err(|e| anyhow!("Failed to save: {}", e))
}
