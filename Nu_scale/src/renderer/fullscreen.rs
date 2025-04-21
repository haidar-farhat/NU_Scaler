use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::fs::{File, OpenOptions};
use std::io::{Error as IoError, ErrorKind};
use anyhow::Result;
use eframe::{self, egui};
use egui::{Vec2, ColorImage, TextureOptions, TextureId};
use image::RgbaImage;

use crate::capture::common::FrameBuffer;
use crate::upscale::{Upscaler, UpscalingTechnology, UpscalingQuality};
use crate::upscale::common::UpscalingAlgorithm;

// Define a constant for the lock file path
const LOCK_FILE_PATH: &str = "nu_scaler_fullscreen.lock";

/// Create a lock file to ensure only one instance can run fullscreen mode
fn create_lock_file() -> std::io::Result<Option<File>> {
    // Try to get the app data directory
    let lock_path = if let Some(data_dir) = dirs::data_dir() {
        let app_dir = data_dir.join("NU_Scaler");
        // Create the directory if it doesn't exist
        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir)?;
        }
        app_dir.join(LOCK_FILE_PATH)
    } else {
        std::path::PathBuf::from(LOCK_FILE_PATH)
    };
    
    // If lock file exists, check if it's stale
    if lock_path.exists() {
        let is_stale = match std::fs::read_to_string(&lock_path) {
            Ok(content) => {
                // Read PID from lock file
                if let Ok(pid) = content.trim().parse::<u32>() {
                    // On Windows, check if the process exists
                    #[cfg(windows)]
                    {
                        use std::process::Command;
                        // Try to query the process - if it doesn't exist, this will fail
                        let output = Command::new("tasklist")
                            .args(&["/FI", &format!("PID eq {}", pid), "/NH"])
                            .output();
                        
                        match output {
                            Ok(output) => {
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                // If the process is not in the list, the lock is stale
                                let is_stale = !output_str.contains(&pid.to_string());
                                if is_stale {
                                    log::info!("Detected stale lock file from non-existent process {}", pid);
                                }
                                is_stale
                            },
                            Err(_) => {
                                // If we can't check, assume it's not stale
                                false
                            }
                        }
                    }
                    
                    // On Unix systems, check differently
                    #[cfg(unix)]
                    {
                        use std::process::Command;
                        // Check if the process exists
                        let output = Command::new("ps")
                            .args(&["-p", &pid.to_string()])
                            .output();
                            
                        match output {
                            Ok(output) => {
                                // The process doesn't exist if ps returns no lines beyond the header
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                let lines = output_str.lines().count();
                                let is_stale = lines <= 1;
                                if is_stale {
                                    log::info!("Detected stale lock file from non-existent process {}", pid);
                                }
                                is_stale
                            },
                            Err(_) => false
                        }
                    }
                    
                    // Default for other platforms
                    #[cfg(not(any(windows, unix)))]
                    {
                        // Can't check on other platforms, assume it's not stale
                        false
                    }
                } else {
                    // Invalid PID in lock file, consider it stale
                    log::warn!("Invalid PID in lock file, treating as stale");
                    true
                }
            },
            Err(_) => {
                // Can't read lock file, assume it's stale
                log::warn!("Couldn't read lock file, treating as stale");
                true
            }
        };
        
        // Remove stale lock file
        if is_stale {
            log::info!("Removing stale lock file at {:?}", lock_path);
            let _ = std::fs::remove_file(&lock_path);
        } else {
            log::warn!("Lock file is active (not stale) at {:?}", lock_path);
            return Ok(None);
        }
    }
    
    // Try to create the lock file with exclusive access
    match OpenOptions::new().write(true).create_new(true).open(&lock_path) {
        Ok(file) => {
            log::info!("Created lock file at {:?}", lock_path);
            // Write the current process ID to the lock file
            if let Err(e) = std::io::Write::write_all(&mut std::io::BufWriter::new(&file), 
                                                     format!("{}", std::process::id()).as_bytes()) {
                log::warn!("Failed to write PID to lock file: {}", e);
            }
            Ok(Some(file))
        },
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            log::warn!("Lock file already exists at {:?}, another instance may be running", lock_path);
            Ok(None)
        },
        Err(e) => {
            log::error!("Failed to create lock file: {}", e);
            Err(e)
        }
    }
}

/// Remove the lock file when the application exits
fn remove_lock_file() {
    let lock_path = if let Some(data_dir) = dirs::data_dir() {
        data_dir.join("NU_Scaler").join(LOCK_FILE_PATH)
    } else {
        std::path::PathBuf::from(LOCK_FILE_PATH)
    };
    
    if let Err(e) = std::fs::remove_file(&lock_path) {
        log::warn!("Failed to remove lock file: {}", e);
    } else {
        log::info!("Removed lock file at {:?}", lock_path);
    }
}

/// Fullscreen upscaler UI
pub struct FullscreenUpscalerUi {
    /// Frame buffer for capturing frames
    frame_buffer: Arc<FrameBuffer>,
    /// Stop signal for capture thread
    stop_signal: Arc<AtomicBool>,
    /// Upscaler implementation
    upscaler: Box<dyn Upscaler>,
    /// Upscaling algorithm
    algorithm: Option<UpscalingAlgorithm>,
    /// Texture for displaying frames
    texture: Option<egui::TextureHandle>,
    /// Time of last frame
    last_frame_time: std::time::Instant,
    /// FPS counter
    fps: f32,
    /// Number of frames processed
    frames_processed: u64,
    /// Current upscaler name
    upscaler_name: String,
    /// Current upscaling quality
    upscaler_quality: UpscalingQuality,
    /// Show performance overlay
    show_overlay: bool,
    /// Performance metrics history
    fps_history: Vec<f32>,
    /// Upscaling time history (ms)
    upscale_time_history: Vec<f32>,
    /// Last upscale time (ms)
    last_upscale_time: f32,
    /// Input size
    input_size: (u32, u32),
    /// Output size 
    output_size: (u32, u32),
}

impl FullscreenUpscalerUi {
    /// Create a new fullscreen upscaler UI
    fn new(
        cc: &eframe::CreationContext<'_>,
        frame_buffer: Arc<FrameBuffer>,
        stop_signal: Arc<AtomicBool>,
        upscaler: Box<dyn Upscaler>,
        algorithm: Option<UpscalingAlgorithm>,
    ) -> Self {
        // Enable vsync and fullscreen
        if let Some(ctx) = &cc.wgpu_render_state {
            // Configure wgpu renderer if available
            let _ = ctx.adapter.features();
            // Additional wgpu configuration can be done here
        }
        
        // Set up UI with dark mode
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        
        // Get upscaler information
        let upscaler_name = upscaler.name().to_string();
        let upscaler_quality = upscaler.quality();
        
        Self {
            frame_buffer,
            stop_signal,
            upscaler,
            algorithm,
            texture: None,
            last_frame_time: std::time::Instant::now(),
            fps: 0.0,
            frames_processed: 0,
            upscaler_name,
            upscaler_quality,
            show_overlay: true,
            fps_history: Vec::with_capacity(120),
            upscale_time_history: Vec::with_capacity(120),
            last_upscale_time: 0.0,
            input_size: (0, 0),
            output_size: (0, 0),
        }
    }
    
    /// Update the texture with the latest frame
    fn update_texture(&mut self, ctx: &egui::Context) {
        // Get the latest frame from the buffer
        if let Ok(Some(frame)) = self.frame_buffer.get_latest_frame() {
            // Store input size
            self.input_size = (frame.width(), frame.height());
            
            // Measure upscaling time
            let upscale_start = std::time::Instant::now();
            
            // Upscale the frame
            if let Ok(upscaled) = self.upscale_frame(&frame) {
                // Measure upscaling time
                let upscale_time = upscale_start.elapsed().as_secs_f32() * 1000.0;
                self.last_upscale_time = upscale_time;
                
                // Keep history of upscale times (max 120 frames)
                self.upscale_time_history.push(upscale_time);
                if self.upscale_time_history.len() > 120 {
                    self.upscale_time_history.remove(0);
                }
                
                // Store output size
                self.output_size = (upscaled.width(), upscaled.height());
                
                // Convert to egui::ColorImage
                let size = [upscaled.width() as usize, upscaled.height() as usize];
                let mut color_data = Vec::with_capacity(size[0] * size[1] * 4);
                
                for y in 0..upscaled.height() {
                    for x in 0..upscaled.width() {
                        let pixel = upscaled.get_pixel(x, y);
                        color_data.push(pixel[0]);
                        color_data.push(pixel[1]);
                        color_data.push(pixel[2]);
                        color_data.push(pixel[3]);
                    }
                }
                
                // Create or update the texture
                let color_image = ColorImage::from_rgba_unmultiplied(size, &color_data);
                
                self.texture = Some(ctx.load_texture(
                    "frame_texture",
                    color_image,
                    TextureOptions::LINEAR
                ));
                
                // Update stats
                self.frames_processed += 1;
                let elapsed = self.last_frame_time.elapsed();
                self.fps = 1.0 / elapsed.as_secs_f32();
                self.last_frame_time = std::time::Instant::now();
                
                // Keep history of fps (max 120 frames)
                self.fps_history.push(self.fps);
                if self.fps_history.len() > 120 {
                    self.fps_history.remove(0);
                }
                
                // Log performance metrics occasionally
                if self.frames_processed % 100 == 0 {
                    let avg_fps = self.fps_history.iter().sum::<f32>() / self.fps_history.len() as f32;
                    let avg_upscale_time = self.upscale_time_history.iter().sum::<f32>() / self.upscale_time_history.len() as f32;
                    
                    log::info!("Performance: Avg FPS: {:.1}, Avg upscale time: {:.2}ms, Input: {}x{}, Output: {}x{}", 
                              avg_fps, avg_upscale_time, 
                              self.input_size.0, self.input_size.1,
                              self.output_size.0, self.output_size.1);
                }
            }
        }
    }
    
    /// Upscale a frame using the configured upscaler
    fn upscale_frame(&mut self, frame: &RgbaImage) -> Result<RgbaImage> {
        // Check if upscaler needs initialization
        if self.upscaler.needs_initialization() || 
           frame.width() != self.upscaler.input_width() || 
           frame.height() != self.upscaler.input_height() {
            log::debug!("Initializing upscaler with dimensions {}x{} -> {}x{}", 
                      frame.width(), frame.height(), 
                      (frame.width() as f32 * 1.5) as u32, 
                      (frame.height() as f32 * 1.5) as u32);
            
            // Initialize with 1.5x scale factor by default
            let _ = self.upscaler.initialize(
                frame.width(), 
                frame.height(), 
                (frame.width() as f32 * 1.5) as u32, 
                (frame.height() as f32 * 1.5) as u32
            );
        }
        
        // Use the configured upscaler to process the frame with the algorithm
        match self.algorithm {
            Some(alg) => self.upscaler.upscale_with_algorithm(frame, alg),
            None => self.upscaler.upscale(frame)
        }
    }
    
    /// Draw the performance overlay
    fn draw_performance_overlay(&self, ui: &mut egui::Ui) {
        // Early return if overlay is disabled
        if !self.show_overlay {
            return;
        }
        
        // Background panel for metrics
        egui::Frame::default()
            .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 180))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)))
            .inner_margin(egui::Margin::same(10.0))
            .rounding(egui::Rounding::same(5.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Title
                    ui.add(egui::Label::new(
                        egui::RichText::new("NU_Scaler Performance")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(220, 220, 220))
                    ));
                    
                    ui.add_space(5.0);
                    
                    // FPS and frame count
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("FPS: {:.1}", self.fps))
                                .color(egui::Color32::from_rgb(120, 220, 120))
                                .size(14.0)
                        ));
                        
                        ui.add_space(15.0);
                        
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("Frames: {}", self.frames_processed))
                                .color(egui::Color32::WHITE)
                                .size(14.0)
                        ));
                    });
                    
                    // Upscaler info
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("Upscaler: {}", self.upscaler_name))
                                .color(egui::Color32::from_rgb(220, 180, 120))
                                .size(14.0)
                        ));
                        
                        ui.add_space(15.0);
                        
                        let quality_color = match self.upscaler_quality {
                            UpscalingQuality::Ultra => egui::Color32::from_rgb(120, 220, 120),
                            UpscalingQuality::Quality => egui::Color32::from_rgb(180, 220, 120),
                            UpscalingQuality::Balanced => egui::Color32::from_rgb(220, 220, 120),
                            UpscalingQuality::Performance => egui::Color32::from_rgb(220, 180, 120),
                        };
                        
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("Quality: {:?}", self.upscaler_quality))
                                .color(quality_color)
                                .size(14.0)
                        ));
                    });
                    
                    // Algorithm info if present
                    if let Some(alg) = self.algorithm {
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("Algorithm: {:?}", alg))
                                .color(egui::Color32::from_rgb(180, 180, 220))
                                .size(14.0)
                        ));
                    }
                    
                    // Resolution info
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("Input: {}x{}", self.input_size.0, self.input_size.1))
                                .color(egui::Color32::WHITE)
                                .size(14.0)
                        ));
                        
                        ui.add_space(10.0);
                        
                        ui.add(egui::Label::new(
                            egui::RichText::new("→")
                                .color(egui::Color32::from_rgb(180, 180, 180))
                                .size(14.0)
                        ));
                        
                        ui.add_space(10.0);
                        
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("Output: {}x{}", self.output_size.0, self.output_size.1))
                                .color(egui::Color32::WHITE)
                                .size(14.0)
                        ));
                        
                        // Calculate and show scale factor
                        if self.input_size.0 > 0 && self.input_size.1 > 0 {
                            let scale_x = self.output_size.0 as f32 / self.input_size.0 as f32;
                            let scale_y = self.output_size.1 as f32 / self.input_size.1 as f32;
                            
                            ui.add_space(10.0);
                            
                            ui.add(egui::Label::new(
                                egui::RichText::new(format!("({:.1}x)", (scale_x + scale_y) / 2.0))
                                    .color(egui::Color32::from_rgb(180, 220, 180))
                                    .size(14.0)
                            ));
                        }
                    });
                    
                    // Performance details
                    ui.add_space(5.0);
                    
                    ui.add(egui::Label::new(
                        egui::RichText::new(format!("Upscale time: {:.2}ms", self.last_upscale_time))
                            .color(egui::Color32::from_rgb(220, 220, 120))
                            .size(14.0)
                    ));
                    
                    // FPS Graph if we have enough history
                    if self.fps_history.len() > 2 {
                        ui.add_space(10.0);
                        
                        let max_fps = self.fps_history.iter().cloned().fold(0.0_f32, f32::max).max(1.0);
                        
                        ui.add(egui::Label::new(
                            egui::RichText::new("FPS History")
                                .color(egui::Color32::WHITE)
                                .size(12.0)
                        ));
                        
                        let height = 40.0;
                        let graph = egui::plot::Plot::new("fps_history")
                            .height(height)
                            .show_background(false)
                            .allow_zoom(false)
                            .allow_drag(false)
                            .include_y(0.0)
                            .include_y(max_fps)
                            .show_axes([false; 2]);
                            
                        graph.show(ui, |plot_ui| {
                            let fps_points: Vec<[f64; 2]> = self.fps_history.iter()
                                .enumerate()
                                .map(|(i, &fps)| [i as f64, fps as f64])
                                .collect();
                            
                            let line = egui::plot::Line::new(egui::plot::PlotPoints::from(fps_points))
                                .color(egui::Color32::from_rgb(120, 220, 120))
                                .width(1.5);
                                
                            plot_ui.line(line);
                        });
                    }
                    
                    // Help text
                    ui.add_space(10.0);
                    ui.add(egui::Label::new(
                        egui::RichText::new("Press ESC to exit fullscreen mode")
                            .color(egui::Color32::from_rgb(180, 180, 180))
                            .size(12.0)
                    ));
                });
            });
    }

    /// Configure UI settings after initialization
    pub fn configure_ui(&mut self, ctx: &egui::Context) {
        // Set up UI with dark mode
        ctx.set_visuals(egui::Visuals::dark());
    }
}

impl eframe::App for FullscreenUpscalerUi {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Update the texture with the latest frame
        self.update_texture(ctx);
        
        // Check for ESC key to exit fullscreen mode
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            // Signal the capture thread to stop
            self.stop_signal.store(true, Ordering::SeqCst);
            
            // Close the application
            frame.close();
            return;
        }
        
        // Check for F1 key to toggle performance overlay
        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            self.show_overlay = !self.show_overlay;
        }
        
        // Show the upscaled frame on the entire window
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                if let Some(texture) = &self.texture {
                    // Get available size
                    let available_size = ui.available_size();
                    let texture_size = texture.size_vec2();
                    
                    // Calculate the scaling to fit in the available space
                    // while maintaining aspect ratio
                    let aspect_ratio = texture_size.x / texture_size.y;
                    let width = available_size.x;
                    let height = width / aspect_ratio;
                    
                    // Center the image if it's smaller than the available space
                    let rect = if height <= available_size.y {
                        let y_offset = (available_size.y - height) / 2.0;
                        egui::Rect::from_min_size(
                            egui::pos2(0.0, y_offset),
                            Vec2::new(width, height)
                        )
                    } else {
                        let height = available_size.y;
                        let width = height * aspect_ratio;
                        let x_offset = (available_size.x - width) / 2.0;
                        egui::Rect::from_min_size(
                            egui::pos2(x_offset, 0.0),
                            Vec2::new(width, height)
                        )
                    };
                    
                    // Draw the texture
                    ui.put(rect, egui::Image::new(texture.id(), texture_size));
                    
                    // Draw performance overlay in the top-right corner
                    let overlay_width = 250.0;
                    let overlay_rect = egui::Rect::from_min_size(
                        egui::pos2(ui.available_rect_before_wrap().right() - overlay_width - 10.0, 10.0),
                        Vec2::new(overlay_width, 0.0) // Height will be determined by content
                    );
                    
                    ui.allocate_ui_at_rect(overlay_rect, |ui| {
                        self.draw_performance_overlay(ui);
                    });
                } else {
                    // Show loading message if no texture is available
                    ui.centered_and_justified(|ui| {
                        ui.heading("Waiting for frames...");
                    });
                }
            });
            
        // Request continuous repaint to update the frame as soon as possible
        ctx.request_repaint();
    }
}

/// Create an upscaler for the given technology and quality
fn create_upscaler(
    technology: UpscalingTechnology,
    quality: UpscalingQuality,
    algorithm: Option<UpscalingAlgorithm>,
) -> Result<Box<dyn Upscaler>> {
    crate::upscale::create_upscaler(technology, quality, algorithm)
}

/// Run the fullscreen upscaler UI
pub fn run_fullscreen_upscaler(
    frame_buffer: Arc<FrameBuffer>,
    stop_signal: Arc<AtomicBool>,
    technology: UpscalingTechnology,
    quality: UpscalingQuality,
    algorithm: Option<UpscalingAlgorithm>,
) -> Result<(), String> {
    // Try to create a lock file to ensure only one instance runs
    let lock_file_result = create_lock_file();
    
    // Check if we got a lock
    if let Ok(None) = lock_file_result {
        return Err("Another instance of NU_Scaler is already running in fullscreen mode. Please close it before starting a new session.".to_string());
    }
    
    // Handle error cases but continue
    if let Err(e) = &lock_file_result {
        log::error!("Failed to check for running instances: {}", e);
        // Continue anyway, but log the error
    }
    
    // Create an upscaler with the given technology and quality
    let upscaler = match create_upscaler(technology, quality, algorithm) {
        Ok(u) => u,
        Err(e) => {
            // Release the lock if we fail to create the upscaler
            remove_lock_file();
            return Err(format!("Failed to create upscaler: {}", e));
        }
    };
    
    // Log the upscaler we're actually using
    log::info!("Using upscaler: {} with quality: {:?}", upscaler.name(), upscaler.quality());
    
    // Create eframe options for fullscreen
    let options = eframe::NativeOptions {
        maximized: true,
        decorated: false,
        transparent: false,
        vsync: true,
        initial_window_size: Some(Vec2::new(1920.0, 1080.0)),
        ..Default::default()
    };
    
    // Create the UI and wrap it in a Box so we can move it into the eframe run function
    let ui = FullscreenUpscalerUi::new_boxed(
        frame_buffer,
        stop_signal,
        upscaler,
        algorithm,
    );
    
    // Run the application
    let result = eframe::run_native(
        "NU Scale - Fullscreen Mode",
        options,
        Box::new(move |cc| {
            // Configure the UI with the creation context
            FullscreenUpscalerUi::configure(cc);
            ui
        })
    ).map_err(|e| format!("Failed to run fullscreen upscaler: {}", e));
    
    // Release the lock file when done
    remove_lock_file();
    
    result
}

impl FullscreenUpscalerUi {
    // Separate configuration of the context
    fn configure(cc: &eframe::CreationContext<'_>) {
        // Enable vsync and fullscreen
        if let Some(ctx) = &cc.wgpu_render_state {
            // Configure wgpu renderer if available
            let _ = ctx.adapter.features();
            // Additional wgpu configuration can be done here
        }
        
        // Set up UI with dark mode
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
    }
    
    // Create a new boxed instance of FullscreenUpscalerUi
    fn new_boxed(
        frame_buffer: Arc<FrameBuffer>,
        stop_signal: Arc<AtomicBool>,
        upscaler: Box<dyn Upscaler>,
        algorithm: Option<UpscalingAlgorithm>,
    ) -> Box<Self> {
        // Get upscaler information
        let upscaler_name = upscaler.name().to_string();
        let upscaler_quality = upscaler.quality();
        
        Box::new(Self {
            frame_buffer,
            stop_signal,
            upscaler,
            algorithm,
            texture: None,
            last_frame_time: std::time::Instant::now(),
            fps: 0.0,
            frames_processed: 0,
            upscaler_name,
            upscaler_quality,
            show_overlay: true,
            fps_history: Vec::with_capacity(120),
            upscale_time_history: Vec::with_capacity(120),
            last_upscale_time: 0.0,
            input_size: (0, 0),
            output_size: (0, 0),
        })
    }
} 