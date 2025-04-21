use anyhow::{Result, anyhow};
use clap::{Arg, App, SubCommand};
use nu_scaler::capture::CaptureTarget;
use nu_scaler::upscale::{UpscalingTechnology, UpscalingQuality};
use nu_scaler::UpscalingAlgorithm;
use nu_scaler::{init, start_borderless_upscale};
use log::{debug, info, warn, error};

fn main() -> Result<()> {
    // Simple CLI app with all needed commands
    let matches = App::new("NU_Scaler")
        .version("0.1.0")
        .author("NU_Scaler Team")
        .about("Real-time upscaling app for screen capture")
        .arg(
            Arg::with_name("verbose")
                .short('v')
                .long("verbose")
                .multiple(true)
                .help("Sets the level of verbosity (can be used multiple times)")
        )
        .arg(
            Arg::with_name("log-dir")
                .long("log-dir")
                .takes_value(true)
                .help("Directory to store log files. Default is user data directory.")
        )
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
        .subcommand(
            SubCommand::with_name("cli")
                .about("Force CLI mode even if GUI is available")
        )
        .get_matches();

    // Get logging options
    let verbose = matches.occurrences_of("verbose") > 0;
    let log_dir = matches.value_of("log-dir");
    
    // Initialize application (including logging)
    if let Err(e) = nu_scaler::logger::init_logger(log_dir, verbose) {
        eprintln!("Warning: Failed to initialize logger: {}", e);
    }
    
    // Continue with the rest of the initialization
    if let Err(e) = init() {
        error!("Failed to initialize application: {}", e);
        return Err(anyhow!("Failed to initialize application: {}", e));
    }
    
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
            .and_then(|alg| local_string_to_algorithm(alg));
        
        // Log the fullscreen upscaling parameters
        info!("Starting fullscreen upscaling");
        debug!("  Source: {:?}", source);
        debug!("  Technology: {:?}", tech);
        debug!("  Quality: {:?}", quality);
        debug!("  FPS: {}", fps);
        debug!("  Algorithm: {:?}", algorithm);
        
        // Start fullscreen upscaling
        println!("Starting fullscreen upscaling with {:?} technology at {:?} quality", tech, quality);
        println!("Press ESC to exit");
        
        // Measure and log performance
        let start_time = std::time::Instant::now();
        let result = start_borderless_upscale(source, tech, quality, fps, algorithm);
        let elapsed = start_time.elapsed();
        
        if let Err(ref e) = result {
            error!("Fullscreen upscaling failed after {:.2?}: {}", elapsed, e);
            return result;
        }
        
        info!("Fullscreen upscaling completed after {:.2?}", elapsed);
        return Ok(());
    }
    
    // Check if "cli" subcommand was explicitly used
    let force_cli = matches.subcommand_matches("cli").is_some();
    
    // Launch GUI if available and not forced to CLI mode
    #[cfg(feature = "gui")]
    if !force_cli {
        info!("Starting NU_Scaler GUI");
        return nu_scaler::ui::run_app();
    }
    
    // Fall back to CLI mode if GUI is not available or if forced
    info!("Running in CLI mode");
    println!("NU_Scaler CLI");
    println!("Run with 'fullscreen' subcommand to start the upscaler");
    println!("Example: nu_scaler fullscreen --source fullscreen --tech fallback --quality balanced --fps 60 --algorithm lanczos3");
    Ok(())
}

/// Parse the source string into a CaptureTarget
fn parse_source(source_str: &str) -> Result<CaptureTarget> {
    if source_str == "fullscreen" {
        debug!("Using fullscreen capture");
        return Ok(CaptureTarget::FullScreen);
    } else if source_str.starts_with("window:") {
        let title = source_str.strip_prefix("window:")
            .unwrap_or("")
            .to_string();
        debug!("Using window capture with title: {}", title);
        return Ok(CaptureTarget::WindowByTitle(title));
    } else if source_str.starts_with("region:") {
        let coords = source_str.strip_prefix("region:")
            .unwrap_or("")
            .split(',')
            .filter_map(|s| s.parse::<i32>().ok())
            .collect::<Vec<_>>();
        
        if coords.len() >= 4 {
            debug!("Using region capture with coords: {:?}", coords);
            return Ok(CaptureTarget::Region {
                x: coords[0],
                y: coords[1],
                width: coords[2] as u32,
                height: coords[3] as u32,
            });
        }
        warn!("Invalid region format: {}", source_str);
        return Err(anyhow!("Invalid region format. Use region:x,y,width,height"));
    }
    
    warn!("Invalid source format: {}", source_str);
    Err(anyhow!("Invalid source format. Use fullscreen, window:<title>, or region:x,y,width,height"))
}

/// Utility function to convert a string algorithm name to the UpscalingAlgorithm enum
fn local_string_to_algorithm(alg_str: &str) -> Option<nu_scaler::UpscalingAlgorithm> {
    let result = nu_scaler::string_to_algorithm(alg_str);
    if result.is_none() {
        warn!("Unknown upscaling algorithm: {}", alg_str);
    } else {
        debug!("Using upscaling algorithm: {}", alg_str);
    }
    result
}
