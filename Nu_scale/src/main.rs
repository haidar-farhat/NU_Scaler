mod capture;
mod ui;

use anyhow::{Result, anyhow};
use clap::{Arg, App, SubCommand};
use nu_scaler::capture::CaptureTarget;
use nu_scaler::upscale::{UpscalingTechnology, UpscalingQuality};
use nu_scaler::UpscalingAlgorithm;
use nu_scaler::{init, start_borderless_upscale};

fn main() -> Result<()> {
    // Initialize global configuration for the application
    init()?;

    // Simple CLI app to bypass the GUI issues
    let matches = App::new("NU_Scaler")
        .version("0.1.0")
        .author("NU_Scaler Team")
        .about("Real-time upscaling app for screen capture")
        .subcommand(
            SubCommand::with_name("fullscreen")
                .about("Capture and upscale the screen in fullscreen mode")
                .arg(
                    Arg::with_name("source")
                        .long("source")
                        .help("Source to capture: fullscreen, window:<title>, or region:<x>,<y>,<width>,<height>")
                        .takes_value(true)
                        .default_value("fullscreen")
                )
                .arg(
                    Arg::with_name("tech")
                        .long("tech")
                        .help("Upscaling technology: fsr, dlss, or fallback")
                        .takes_value(true)
                        .default_value("fallback")
                )
                .arg(
                    Arg::with_name("quality")
                        .long("quality")
                        .help("Upscaling quality: ultra, quality, balanced, or performance")
                        .takes_value(true)
                        .default_value("quality")
                )
                .arg(
                    Arg::with_name("fps")
                        .long("fps")
                        .help("Target frame rate")
                        .takes_value(true)
                        .default_value("60")
                )
                .arg(
                    Arg::with_name("algorithm")
                        .long("algorithm")
                        .help("Upscaling algorithm (for fallback tech): lanczos3, bilinear, bicubic, etc.")
                        .takes_value(true)
                )
        )
        .get_matches();

    // If fullscreen command is used, capture the screen and upscale
    if let Some(matches) = matches.subcommand_matches("fullscreen") {
        // Process source
        let source_str = matches.value_of("source").unwrap_or("fullscreen");
        let source = parse_source(source_str)?;
        
        // Process technology
        let tech_str = matches.value_of("tech").unwrap_or("fallback");
        let tech = match tech_str {
            "fsr" => UpscalingTechnology::FSR,
            "dlss" => UpscalingTechnology::DLSS,
            "fallback" | _ => UpscalingTechnology::Fallback,
        };
        
        // Process quality
        let quality_str = matches.value_of("quality").unwrap_or("quality");
        let quality = match quality_str {
            "ultra" => UpscalingQuality::Ultra,
            "quality" => UpscalingQuality::Quality,
            "balanced" => UpscalingQuality::Balanced,
            "performance" | _ => UpscalingQuality::Performance,
        };
        
        // Process FPS
        let fps = matches.value_of("fps")
            .unwrap_or("60")
            .parse::<u32>()
            .unwrap_or(60);
        
        // Process algorithm
        let algorithm = matches.value_of("algorithm")
            .map(|alg| local_string_to_algorithm(alg));
        
        // Start fullscreen upscaling
        println!("Starting fullscreen upscaling with {:?} technology at {:?} quality", tech, quality);
        println!("Press ESC to exit");
        start_borderless_upscale(source, tech, quality, fps, algorithm)?;
        return Ok(());
    }
    
    // Launch the GUI by default (but we'll bypass it for now)
    println!("NU_Scaler CLI");
    println!("Run with 'fullscreen' subcommand to start the upscaler");
    println!("Example: nu_scaler fullscreen --source fullscreen --tech fallback --quality balanced --fps 60 --algorithm lanczos3");
    Ok(())
}

/// Parse the source string into a CaptureTarget
fn parse_source(source_str: &str) -> Result<CaptureTarget> {
    if source_str == "fullscreen" {
        return Ok(CaptureTarget::FullScreen);
    } else if source_str.starts_with("window:") {
        let title = source_str.strip_prefix("window:")
            .unwrap_or("")
            .to_string();
        return Ok(CaptureTarget::WindowByTitle(title));
    } else if source_str.starts_with("region:") {
        let coords = source_str.strip_prefix("region:")
            .unwrap_or("")
            .split(',')
            .filter_map(|s| s.parse::<i32>().ok())
            .collect::<Vec<_>>();
        
        if coords.len() >= 4 {
            return Ok(CaptureTarget::Region {
                x: coords[0],
                y: coords[1],
                width: coords[2] as u32,
                height: coords[3] as u32,
            });
        }
        return Err(anyhow!("Invalid region format. Use region:x,y,width,height"));
    }
    
    Err(anyhow!("Invalid source format. Use fullscreen, window:<title>, or region:x,y,width,height"))
}

/// Utility function to convert a string algorithm name to the UpscalingAlgorithm enum
fn local_string_to_algorithm(alg_str: &str) -> UpscalingAlgorithm {
    nu_scaler::string_to_algorithm(alg_str).unwrap_or(UpscalingAlgorithm::Lanczos3)
}
