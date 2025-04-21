use anyhow::Result;
use log::{debug, error, info, trace, warn, LevelFilter};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};

// Ensure the logger is only initialized once
static INIT: Once = Once::new();

/// Initialize the logger with both console and file output
pub fn init_logger(log_dir: Option<&str>, verbose: bool) -> Result<()> {
    let result = std::thread::spawn(move || {
        INIT.call_once(|| {
            // Determine log level based on verbose flag
            let log_level = if verbose {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            };
            
            let mut builder = env_logger::Builder::new();
            builder.filter_level(log_level);
            
            // If log_dir is provided, add a file logger
            if let Some(dir) = log_dir {
                // Create log directory if it doesn't exist
                let log_dir_path = Path::new(dir);
                if !log_dir_path.exists() {
                    if let Err(e) = fs::create_dir_all(log_dir_path) {
                        eprintln!("Failed to create log directory: {}", e);
                    }
                }
                
                // Generate log filename with timestamp
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let log_filename = format!("nu_scaler_{}.log", timestamp);
                let log_path = log_dir_path.join(log_filename);
                
                // Try to open log file
                if let Ok(log_file) = File::create(&log_path) {
                    // Write to both stderr and file
                    builder.format(|buf, record| {
                        writeln!(
                            buf,
                            "[{} {} {}:{}] {}",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                            record.level(),
                            record.file().unwrap_or("unknown"),
                            record.line().unwrap_or(0),
                            record.args()
                        )
                    });
                    
                    // Set up the file logger to write to the file
                    let _ = builder.target(env_logger::Target::Pipe(Box::new(log_file)));
                } else {
                    eprintln!("Failed to create log file at: {:?}", log_path);
                    // Fallback to stderr only
                    builder.target(env_logger::Target::Stderr);
                }
            } else {
                // No log file requested, use stderr
                builder.target(env_logger::Target::Stderr);
            }
            
            // Initialize the logger
            let _ = builder.try_init();
            
            info!("NU_Scaler logger initialized at level {:?}", log_level);
            
            // Log system information
            log_system_info();
        });
    }).join();

    if let Err(e) = result {
        error!("Failed to initialize logger: {:?}", e);
        return Err(anyhow::anyhow!("Failed to initialize logger"));
    }
    
    Ok(())
}

/// Log detailed system information
fn log_system_info() {
    info!("NU_Scaler v{}", env!("CARGO_PKG_VERSION"));
    
    // Log OS information
    #[cfg(target_os = "windows")]
    info!("Operating System: Windows");
    #[cfg(target_os = "macos")]
    info!("Operating System: macOS");
    #[cfg(target_os = "linux")]
    info!("Operating System: Linux");
    
    // Log enabled features
    #[cfg(feature = "gui")]
    info!("Feature: GUI enabled");
    #[cfg(feature = "fsr")]
    info!("Feature: FSR enabled");
    #[cfg(feature = "dlss")]
    info!("Feature: DLSS enabled");
    #[cfg(feature = "capture_opencv")]
    info!("Feature: OpenCV capture enabled");
    
    // Log available upscalers
    log_available_upscalers();
}

/// Log available upscaling technologies
fn log_available_upscalers() {
    use crate::upscale::fsr::FsrUpscaler;
    use crate::upscale::dlss::DlssUpscaler;
    
    info!("Checking available upscalers...");
    
    #[cfg(feature = "fsr")]
    {
        let fsr_supported = FsrUpscaler::is_supported();
        info!("FSR support: {}", if fsr_supported { "Available" } else { "Not available" });
    }
    
    #[cfg(feature = "dlss")]
    {
        let dlss_supported = DlssUpscaler::is_supported();
        info!("DLSS support: {}", if dlss_supported { "Available" } else { "Not available" });
    }
    
    info!("Basic upscaler: Available");
}

/// Macro for convenient logging of function entry and exit
#[macro_export]
macro_rules! log_function {
    () => {
        let _log_guard = $crate::logger::FunctionLogger::new(
            module_path!(),
            file!(),
            line!(),
            stringify!(#[function_name])
        );
    };
}

/// Helper struct for function entry/exit logging
pub struct FunctionLogger {
    module: &'static str,
    file: &'static str,
    line: u32,
    function: &'static str,
}

impl FunctionLogger {
    pub fn new(module: &'static str, file: &'static str, line: u32, function: &'static str) -> Self {
        trace!("ENTER: {}::{} ({}:{})", module, function, file, line);
        Self { module, file, line, function }
    }
}

impl Drop for FunctionLogger {
    fn drop(&mut self) {
        trace!("EXIT: {}::{} ({}:{})", self.module, self.function, self.file, self.line);
    }
}

/// Log performance metrics for a function or operation
pub fn log_performance(operation: &str, duration_ms: f64) {
    debug!("PERF: {} took {:.2}ms", operation, duration_ms);
}

/// Log capture events
pub fn log_capture_event(source: &str, width: u32, height: u32) {
    debug!("CAPTURE: {} ({}x{})", source, width, height);
}

/// Log upscaling events
pub fn log_upscale_event(
    technology: &str, 
    quality: &str, 
    input_dims: (u32, u32), 
    output_dims: (u32, u32),
    duration_ms: f64
) {
    debug!(
        "UPSCALE: {} ({}) {}x{} → {}x{} took {:.2}ms", 
        technology, quality, 
        input_dims.0, input_dims.1, 
        output_dims.0, output_dims.1,
        duration_ms
    );
}

/// Log errors with context
pub fn log_error<E: std::fmt::Display>(context: &str, error: E) {
    error!("ERROR in {}: {}", context, error);
} 