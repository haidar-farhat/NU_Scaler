use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use anyhow::Result;
use image::RgbaImage;
use winit::window::Window;
use crate::renderer::wgpu_renderer::{WgpuRenderer, TripleBuffer, RenderStats};
use eframe::{self, egui};
use egui::{Vec2, ColorImage, TextureOptions, Context};

use crate::capture::common::FrameBuffer;
use crate::upscale::Upscaler;
use crate::upscale::common::UpscalingAlgorithm;
use crate::capture::platform::CaptureBackend;

/// Fullscreen upscaler UI implementation
pub struct FullscreenUpscalerUi {
    window: Arc<Window>,
    wgpu_renderer: Option<WgpuRenderer>,
    triple_buffer: TripleBuffer,
    stop_signal: Arc<AtomicBool>,
    upscaler: Arc<Mutex<Option<Box<dyn Upscaler>>>>,
    algorithm: Arc<Mutex<Option<UpscalingAlgorithm>>>,
    show_stats: bool,
    vsync: bool,
}

impl FullscreenUpscalerUi {
    pub async fn new(surface: &wgpu::Surface<'_>, window: &Window, upscaler: Box<dyn Upscaler>, algorithm: Option<UpscalingAlgorithm>, vsync: bool) -> Result<Self> {
        let wgpu_renderer = WgpuRenderer::new(surface, window, vsync).await?;
        
        Ok(Self {
            window: Arc::new(window.clone()),
            wgpu_renderer: Some(wgpu_renderer),
            triple_buffer: TripleBuffer::new(),
            stop_signal: Arc::new(AtomicBool::new(false)),
            upscaler: Arc::new(Mutex::new(Some(upscaler))),
            algorithm: Arc::new(Mutex::new(algorithm)),
            show_stats: true,
            vsync,
        })
    }

    pub fn write_frame(&self, frame: RgbaImage) {
        self.triple_buffer.write(frame);
    }

    pub fn update(&mut self) -> Result<()> {
        if let Some(frame) = self.triple_buffer.read() {
            if let Some(renderer) = &mut self.wgpu_renderer {
                // Upscale the frame if we have an upscaler
                let upscaled = if let Ok(Some(upscaler)) = self.upscaler.lock() {
                    if let Ok(Some(algorithm)) = self.algorithm.lock() {
                        upscaler.upscale(&frame, Some(algorithm.clone())).unwrap_or(frame)
                    } else {
                        upscaler.upscale(&frame, None).unwrap_or(frame)
                    }
                } else {
                    frame
                };

                renderer.update_texture(&upscaled)?;
                renderer.render()?;
            }
        }

        Ok(())
    }

    pub fn set_upscaler(&self, upscaler: Box<dyn Upscaler>) {
        if let Ok(mut current) = self.upscaler.lock() {
            *current = Some(upscaler);
        }
    }

    pub fn set_algorithm(&self, algorithm: Option<UpscalingAlgorithm>) {
        if let Ok(mut current) = self.algorithm.lock() {
            *current = algorithm;
        }
    }

    pub fn stop(&self) {
        self.stop_signal.store(true, Ordering::Relaxed);
    }

    pub fn is_stopped(&self) -> bool {
        self.stop_signal.load(Ordering::Relaxed)
    }

    pub fn toggle_stats(&mut self) {
        self.show_stats = !self.show_stats;
    }

    pub fn toggle_vsync(&mut self) {
        self.vsync = !self.vsync;
        if let Some(renderer) = &mut self.wgpu_renderer {
            renderer.set_vsync(self.vsync);
        }
    }

    fn render_stats_overlay(&self, ctx: &Context) {
        if !self.show_stats { return; }

        if let Some(renderer) = &self.wgpu_renderer {
            let stats = renderer.stats();
            
            egui::Window::new("Performance")
                .title_bar(false)
                .resizable(false)
                .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
                .show(ctx, |ui| {
                    ui.label(format!("FPS: {:.1}", stats.average_fps()));
                    ui.label(format!("Frame Time: {:.2}ms", stats.average_frame_time()));
                    
                    if let Some(resources) = &renderer.render_resources {
                        ui.label(format!("Resolution: {}x{}", 
                            resources.texture_size.0,
                            resources.texture_size.1
                        ));
                    }
                    
                    egui::CollapsingHeader::new("Advanced")
                        .show(ui, |ui| {
                            ui.label(format!("Total Frames: {}", stats.total_frames()));
                            ui.label(format!("VSync: {}", self.vsync));
                        });
                });
        }
    }
}

impl eframe::App for FullscreenUpscalerUi {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Handle exit
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.stop();
            frame.close();
        }

        // Toggle stats overlay
        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            self.toggle_stats();
        }

        // Toggle vsync
        if ctx.input(|i| i.key_pressed(egui::Key::F2)) {
            self.toggle_vsync();
        }

        // Process frame
        if let Err(e) = self.update() {
            log::error!("Error updating frame: {}", e);
        }

        // Render stats overlay
        self.render_stats_overlay(ctx);

        // Request repaint based on vsync setting
        ctx.request_repaint_after(Duration::from_secs_f32(
            if self.vsync { 1.0 / 60.0 } else { 0.0 }
        ));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop();
    }
}

/// Fullscreen upscaler UI
pub struct FullscreenUpscalerUiOld {
    /// Frame buffer for capturing frames
    frame_buffer: Arc<FrameBuffer>,
    /// Stop signal for capture thread
    stop_signal: Arc<Mutex<bool>>,
    /// Upscaler implementation
    upscaler: Box<dyn Upscaler>,
    /// Upscaling algorithm
    algorithm: Option<UpscalingAlgorithm>,
    /// Texture for displaying frames
    texture: Option<egui::TextureHandle>,
    /// Time of last frame
    last_frame_time: Instant,
    /// FPS counter
    fps: f32,
    /// Number of frames processed
    frames_processed: u64,
    /// Backend for capturing
    capture_backend: Box<dyn CaptureBackend>,
}

impl FullscreenUpscalerUiOld {
    /// Create a new fullscreen upscaler UI
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        frame_buffer: Arc<FrameBuffer>,
        stop_signal: Arc<Mutex<bool>>,
        upscaler: Box<dyn Upscaler>,
        algorithm: Option<UpscalingAlgorithm>,
    ) -> Self {
        // Initialize the appropriate capture backend based on platform
        #[cfg(target_os = "windows")]
        let capture_backend = {
            let backend = crate::capture::platform::windows::WgpuWindowsCapture::new().unwrap();
            std::mem::forget(backend);
            Box::new(crate::capture::platform::windows::WgpuWindowsCapture::new().unwrap())
        };
        
        #[cfg(target_os = "linux")]
        let capture_backend = {
            use crate::capture::platform::linux::{detect_backend, LinuxBackendType};
            match detect_backend() {
                LinuxBackendType::Wayland => Box::new(crate::capture::platform::linux::WaylandCapture::new().unwrap()),
                LinuxBackendType::X11 => Box::new(crate::capture::platform::linux::X11Capture::new().unwrap()),
                LinuxBackendType::Unknown => {
                    // Fallback to X11 if detection fails
                    Box::new(crate::capture::platform::linux::X11Capture::new().unwrap())
                }
            }
        };
        
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        let capture_backend = Box::new(crate::capture::platform::generic::GenericCapture::new().unwrap());

        // Enable vsync and fullscreen
        if let Some(ctx) = &cc.wgpu_render_state {
            // Configure wgpu renderer if available
            let _ = ctx.adapter.features();
            // Additional wgpu configuration can be done here
        }
        
        // Set up UI with dark mode
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        
        Self {
            frame_buffer,
            stop_signal,
            upscaler,
            algorithm,
            texture: None,
            last_frame_time: Instant::now(),
            fps: 0.0,
            frames_processed: 0,
            capture_backend,
        }
    }
    
    /// Update the texture with the latest frame
    fn update_texture(&mut self, ctx: &egui::Context) {
        // Get the latest frame from the buffer
        if let Ok(frame) = self.frame_buffer.get_latest_frame() {
            // Process frame with the capture backend
            if let Some(processed_frame) = self.capture_backend.process_frame(&frame) {
                // Calculate input dimensions
                let input_width = processed_frame.width();
                let input_height = processed_frame.height();
                
                // Upscale the frame
                if let Ok(upscaled) = self.upscale_frame(&processed_frame) {
                    // Convert to egui::ColorImage
                    let size = [upscaled.width() as usize, upscaled.height() as usize];
                    let mut pixels = Vec::with_capacity(size[0] * size[1]);
                    
                    for y in 0..upscaled.height() {
                        for x in 0..upscaled.width() {
                            let pixel = upscaled.get_pixel(x, y);
                            pixels.push(egui::Color32::from_rgba_unmultiplied(
                                pixel[0], pixel[1], pixel[2], pixel[3]
                            ));
                        }
                    }
                    
                    // Create or update the texture
                    let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);
                    
                    self.texture = Some(ctx.load_texture(
                        "frame_texture",
                        color_image,
                        TextureOptions::LINEAR
                    ));
                    
                    // Update stats
                    self.frames_processed += 1;
                    let elapsed = self.last_frame_time.elapsed();
                    self.fps = 1.0 / elapsed.as_secs_f32();
                    self.last_frame_time = Instant::now();
                }
            }
        }
    }
    
    /// Upscale a frame using the configured upscaler
    fn upscale_frame(&mut self, frame: &RgbaImage) -> Result<RgbaImage> {
        // Use the configured upscaler to process the frame
        self.upscaler.upscale(frame, self.algorithm)
    }
}

impl eframe::App for FullscreenUpscalerUiOld {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Update the texture with the latest frame
        self.update_texture(ctx);
        
        // Check for ESC key to exit fullscreen mode
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            // Signal the capture thread to stop
            if let Ok(mut stop) = self.stop_signal.lock() {
                *stop = true;
            }
            
            // Close the application
            frame.close();
            return;
        }
        
        // Show the upscaled frame on the entire window
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::BLACK))
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
                    ui.put(rect, egui::Image::new(texture.id(), rect.size()));
                    
                    // Show FPS in corner
                    ui.painter().text(
                        egui::pos2(10.0, 20.0),
                        egui::Align2::LEFT_TOP,
                        format!("FPS: {:.1}", self.fps),
                        egui::FontId::proportional(14.0),
                        egui::Color32::WHITE
                    );
                    
                    // Show frame count
                    ui.painter().text(
                        egui::pos2(10.0, 40.0),
                        egui::Align2::LEFT_TOP,
                        format!("Frames: {}", self.frames_processed),
                        egui::FontId::proportional(14.0),
                        egui::Color32::WHITE
                    );
                    
                    // Show backend info
                    ui.painter().text(
                        egui::pos2(10.0, 60.0),
                        egui::Align2::LEFT_TOP,
                        format!("Backend: {}", self.capture_backend.backend_name()),
                        egui::FontId::proportional(14.0),
                        egui::Color32::WHITE
                    );
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

/// Run the fullscreen upscaler UI
pub fn run_fullscreen_upscaler(
    frame_buffer: Arc<FrameBuffer>,
    stop_signal: Arc<Mutex<bool>>,
    upscaler: Box<dyn Upscaler>,
    algorithm: Option<UpscalingAlgorithm>,
) -> Result<()> {
    // Create eframe options for fullscreen
    let options = eframe::NativeOptions {
        maximized: true,
        decorated: false,
        transparent: false,
        vsync: true,
        initial_window_size: Some(Vec2::new(1920.0, 1080.0)),
        renderer: eframe::Renderer::Wgpu,  // Explicitly use WGPU renderer
        ..Default::default()
    };
    
    // Run the application
    eframe::run_native(
        "NU Scale - Fullscreen Mode",
        options,
        Box::new(|cc| Box::new(FullscreenUpscalerUiOld::new(
            cc,
            frame_buffer,
            stop_signal,
            upscaler,
            algorithm,
        )))
    ).map_err(|e| anyhow::anyhow!("Failed to run fullscreen upscaler: {}", e))?;
    
    Ok(())
} 