mod capture;
mod ui;

use anyhow::Result;
use clap::{App, Arg, SubCommand};
#[cfg(feature = "capture_opencv")]
use std::path::Path;
#[cfg(feature = "capture_opencv")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "capture_opencv")]
use std::time::{Duration, Instant};
#[cfg(feature = "capture_opencv")]
use capture::{CaptureTarget, common, window_finder};
use Nu_scaler::upscale::{UpscalingTechnology, UpscalingQuality};
use Nu_scaler::UpscalingAlgorithm;

fn main() -> Result<()> {
    // Parse command line arguments
    let matches = App::new(Nu_scaler::app_name())
        .version(Nu_scaler::app_version())
        .author("Your Name <your.email@example.com>")
        .about("Image and video upscaling using AI/ML techniques")
        .subcommand(
            SubCommand::with_name("upscale")
                .about("Upscale an image file")
                .arg(
                    Arg::with_name("input")
                        .help("Input image file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("output")
                        .help("Output image file")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::with_name("tech")
                        .help("Upscaling technology (fsr, dlss, or fallback)")
                        .long("tech")
                        .short('t')
                        .takes_value(true)
                        .default_value("fallback"),
                )
                .arg(
                    Arg::with_name("quality")
                        .help("Quality preset (ultra, quality, balanced, performance)")
                        .long("quality")
                        .short('q')
                        .takes_value(true)
                        .default_value("balanced"),
                )
                .arg(
                    Arg::with_name("scale")
                        .help("Scale factor (e.g., 1.5, 2.0)")
                        .long("scale")
                        .short('s')
                        .takes_value(true)
                        .default_value("2.0"),
                )
                .arg(
                    Arg::with_name("algorithm")
                        .help("Upscaling algorithm for traditional upscalers (nearest, bilinear, bicubic, lanczos2, lanczos3, mitchell, area)")
                        .long("algorithm")
                        .short('a')
                        .takes_value(true),
                ),
        )
        .get_matches();

    // If upscale subcommand was used, handle it
    if let Some(matches) = matches.subcommand_matches("upscale") {
        // Get input and output paths
        let input_path = matches.value_of("input").unwrap();
        let output_path = matches.value_of("output").unwrap();
        
        // Get upscaling technology
        let tech_str = matches.value_of("tech").unwrap();
        let technology = match tech_str.to_lowercase().as_str() {
            "fsr" => UpscalingTechnology::FSR,
            "dlss" => UpscalingTechnology::DLSS,
            "fallback" => UpscalingTechnology::Fallback,
            _ => {
                eprintln!("Unknown upscaling technology: {}", tech_str);
                eprintln!("Using fallback technology");
                UpscalingTechnology::Fallback
            }
        };
        
        // Get quality preset
        let quality_str = matches.value_of("quality").unwrap();
        let quality = match quality_str.to_lowercase().as_str() {
            "ultra" => UpscalingQuality::Ultra,
            "quality" => UpscalingQuality::Quality,
            "balanced" => UpscalingQuality::Balanced,
            "performance" => UpscalingQuality::Performance,
            _ => {
                eprintln!("Unknown quality preset: {}", quality_str);
                eprintln!("Using balanced preset");
                UpscalingQuality::Balanced
            }
        };
        
        // Get scale factor
        let scale_factor = matches.value_of("scale").unwrap()
            .parse::<f32>().unwrap_or_else(|_| {
                eprintln!("Invalid scale factor, using default of 2.0");
                2.0
            });
        
        // Get algorithm if specified
        let algorithm = matches.value_of("algorithm").map(|alg_str| {
            match alg_str.to_lowercase().as_str() {
                "nearest" => UpscalingAlgorithm::NearestNeighbor,
                "bilinear" => UpscalingAlgorithm::Bilinear,
                "bicubic" => UpscalingAlgorithm::Bicubic,
                "lanczos2" => UpscalingAlgorithm::Lanczos2,
                "lanczos3" => UpscalingAlgorithm::Lanczos3,
                "mitchell" => UpscalingAlgorithm::Mitchell,
                "area" => UpscalingAlgorithm::Area,
                _ => {
                    eprintln!("Unknown algorithm: {}, using algorithm based on quality", alg_str);
                    match quality {
                        UpscalingQuality::Ultra => UpscalingAlgorithm::Lanczos3,
                        UpscalingQuality::Quality => UpscalingAlgorithm::Lanczos2,
                        UpscalingQuality::Balanced => UpscalingAlgorithm::Bicubic,
                        UpscalingQuality::Performance => UpscalingAlgorithm::Bilinear,
                    }
                }
            }
        });
        
        // Perform upscaling
        println!("Upscaling {} to {} using {:?} technology with {:?} quality at {}x scale",
            input_path, output_path, technology, quality, scale_factor);
            
        if let Some(alg) = algorithm {
            println!("Using algorithm: {:?}", alg);
            Nu_scaler::upscale_image_with_algorithm(input_path, output_path, technology, quality, scale_factor, alg)?;
        } else {
            Nu_scaler::upscale_image(input_path, output_path, technology, quality, scale_factor)?;
        }
        println!("Upscaling completed successfully!");
        
        return Ok(());
    }
    
    // Initialize the application
    Nu_scaler::init()?;
    
    // Run the UI
    Nu_scaler::ui::run_ui()
}

/// Run the command-line interface demo
#[cfg(feature = "capture_opencv")]
fn run_cli_demo() -> Result<()> {
    println!("OS-Specific Screen Capture Demo");
    
    // List all available windows
    println!("\nAvailable windows:");
    let windows = common::list_available_windows()?;
    for (i, window) in windows.iter().enumerate() {
        println!("{}: {} ({:?})", i + 1, window.title, window.id);
    }
    
    // Get screen dimensions
    let (width, height) = common::get_screen_dimensions()?;
    println!("\nScreen dimensions: {}x{}", width, height);
    
    // Capture fullscreen
    println!("\nCapturing fullscreen...");
    let fullscreen_path = Path::new("fullscreen.png");
    common::capture_screenshot(&CaptureTarget::FullScreen, fullscreen_path)?;
    println!("Saved to {}", fullscreen_path.display());
    
    // Capture a specific window if available
    if !windows.is_empty() {
        let window = &windows[0];
        println!("\nCapturing window: {}", window.title);
        let window_path = Path::new("window.png");
        common::capture_screenshot(&CaptureTarget::WindowById(window.id.clone()), window_path)?;
        println!("Saved to {}", window_path.display());
    }
    
    // Capture a specific region
    println!("\nCapturing region (top-left quarter of screen)...");
    let region_path = Path::new("region.png");
    common::capture_screenshot(
        &CaptureTarget::Region { 
            x: 0, 
            y: 0, 
            width: width / 2, 
            height: height / 2 
        },
        region_path
    )?;
    println!("Saved to {}", region_path.display());
    
    // Demo window finder
    if windows.len() > 1 {
        println!("\nWindow finder demo:");
        let search_term = "browser";  // Example search term
        println!("Searching for windows matching '{}'", search_term);
        
        let matches = window_finder::find_matching_windows(&windows, search_term);
        if !matches.is_empty() {
            println!("Found {} matching windows:", matches.len());
            for (window, score) in matches {
                println!("- {} (match score: {:.2})", window.title, score);
            }
        } else {
            println!("No matching windows found");
        }
    }

    // Ask if user wants to run the live capture demo
    println!("\nDo you want to run the live capture demo? (y/n)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() == "y" {
        run_live_capture_demo()?;
    }
    
    Ok(())
}

/// Demonstrates the live capture functionality
#[cfg(feature = "capture_opencv")]
fn run_live_capture_demo() -> Result<()> {
    println!("Starting live capture demo (will capture 100 frames)...");
    
    // Create a frame buffer with capacity for 10 frames
    let buffer = Arc::new(common::FrameBuffer::new(10));
    let stop_signal = Arc::new(Mutex::new(false));
    
    // Target to capture (fullscreen)
    let target = CaptureTarget::FullScreen;
    
    // Start live capture thread at 30 FPS
    let capture_buffer = Arc::clone(&buffer);
    let capture_stop = Arc::clone(&stop_signal);
    let capture_handle = common::start_live_capture_thread(
        target,
        30, // 30 FPS
        capture_buffer,
        capture_stop,
    )?;
    
    // Start a frame processor thread
    let process_buffer = Arc::clone(&buffer);
    let process_stop = Arc::clone(&stop_signal);
    let process_handle = common::process_frame_buffer(
        process_buffer,
        process_stop,
        30, // Process at same rate as capture
        |frame| {
            // In a real application, this is where you would perform upscaling and interpolation
            // For now, just print frame dimensions and time
            println!(
                "Processing frame: {}x{} at {:?}", 
                frame.width(), 
                frame.height(), 
                Instant::now()
            );
            Ok(())
        }
    )?;
    
    // Wait for 100 frames (approximately 3-4 seconds at 30 FPS)
    let frames_to_capture = 100;
    let start_time = Instant::now();
    
    println!("Capturing frames for approximately {} seconds...", frames_to_capture / 30);
    
    // Monitor frame count
    loop {
        let frame_count = buffer.len()?;
        if frame_count >= frames_to_capture {
            break;
        }
        
        // Also break if it's taking too long (timeout after 10 seconds)
        if start_time.elapsed() > Duration::from_secs(10) {
            println!("Timeout reached. Captured {} frames.", frame_count);
            break;
        }
        
        std::thread::sleep(Duration::from_millis(100));
    }
    
    // Signal threads to stop
    {
        let mut stop = stop_signal.lock().unwrap();
        *stop = true;
    }
    
    // Wait for threads to finish
    let _ = capture_handle.join();
    let _ = process_handle.join();
    
    // Print statistics
    let elapsed = start_time.elapsed();
    let frame_count = buffer.len()?;
    let fps = frame_count as f64 / elapsed.as_secs_f64();
    
    println!("Live capture demo finished!");
    println!("Frames captured: {}", frame_count);
    println!("Elapsed time: {:.2} seconds", elapsed.as_secs_f64());
    println!("Average FPS: {:.2}", fps);
    
    Ok(())
}
