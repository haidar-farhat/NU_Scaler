use image::{DynamicImage, RgbaImage};
use image::imageops;
use crate::upscale::common::UpscalingAlgorithm;
use chrono;

// let elapsed = frame_start_time.elapsed();
let _elapsed = frame_start_time.elapsed(); // Commented out or prefixed if unused 

/// Resizes an image using the specified algorithm.
pub fn resize_image(
    input: &DynamicImage,
    width: u32,
    height: u32,
    algorithm: UpscalingAlgorithm,
    frame_start_time: std::time::Instant, // Added for potential timing metrics
) -> Result<RgbaImage, String> {
    match algorithm {
        UpscalingAlgorithm::Bilinear => {
            let _elapsed = frame_start_time.elapsed(); // Use variable or remove if unneeded
            Ok(imageops::resize(input, width, height, imageops::FilterType::Triangle))
        }
        UpscalingAlgorithm::Bicubic => {
            let _elapsed = frame_start_time.elapsed();
            Ok(imageops::resize(input, width, height, imageops::FilterType::CatmullRom))
        }
        UpscalingAlgorithm::Lanczos3 => {
            let _elapsed = frame_start_time.elapsed();
            Ok(imageops::resize(input, width, height, imageops::FilterType::Lanczos3))
        }
        UpscalingAlgorithm::Lanczos2 => {
            let _elapsed = frame_start_time.elapsed();
            // Use Lanczos3 as fallback
            Ok(imageops::resize(input, width, height, imageops::FilterType::Lanczos3))
        }
        UpscalingAlgorithm::Nearest => {
            let _elapsed = frame_start_time.elapsed();
            Ok(imageops::resize(input, width, height, imageops::FilterType::Nearest))
        }
    }
}

/// Saves an image buffer to a file with timestamp.
pub fn save_image_buffer(
    buffer: &RgbaImage,
    base_path: &str,
    prefix: &str,
) -> Result<(), String> {
    let _frame_start_time = std::time::Instant::now(); // Use variable or remove if unneeded
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S_%f");
    let filename = format!("{}_{}_{}.png", prefix, base_path, timestamp);
    // ... existing code ...
} 