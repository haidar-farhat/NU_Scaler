mod capture;
mod ui;

use anyhow::Result;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use capture::{CaptureTarget, common, window_finder};

fn main() -> Result<()> {
    // Check for command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    // If we have any arguments, run the CLI version
    if args.len() > 1 && args[1] == "--cli" {
        run_cli_demo()?;
    } else {
        // Otherwise, run the UI version
        ui::run_ui()?;
    }
    
    Ok(())
}

/// Run the command-line interface demo
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
