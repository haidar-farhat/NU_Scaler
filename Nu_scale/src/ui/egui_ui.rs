#![allow(unused_variables, dead_code, unused_imports, unused_mut)]
use anyhow::{anyhow, Result};
use egui::{
    epaint::ahash::AHashMap,
    widgets::*,
    TextureHandle, Ui, Context, ViewportCommand, ViewportBuilder, TextureId, Vec2,
    RichText, Slider, Color32, Frame, Stroke, Rounding,
};
// Standard library imports
use std::{
    path::{PathBuf, Path},
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    thread,
    time::{Duration, Instant},
    marker::PhantomData,
    collections::VecDeque,
};

// Import from local crate instead of external threadpool
// Create a small internal thread pool implementation
use crossbeam_channel::{unbounded, Sender, Receiver};

struct ThreadPool {
    workers: Vec<thread::JoinHandle<()>>,
    sender: Sender<Box<dyn FnOnce() + Send + 'static>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let (sender, receiver): (Sender<Box<dyn FnOnce() + Send + 'static>>, Receiver<Box<dyn FnOnce() + Send + 'static>>) = unbounded();
        let receiver = Arc::new(receiver);

        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            let rx = Arc::clone(&receiver);
            let thread = thread::spawn(move || {
                while let Ok(job) = rx.recv() {
                    job();
                }
            });
            workers.push(thread);
        }
        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap();
    }
}

// Windows API for process management (cfg guard added in implementation)
#[cfg(windows)]
use windows::{
    Win32::System::Threading::TerminateProcess,
    Win32::Foundation::HANDLE,
};

// Use crate:: for lib modules
use crate::capture::{CaptureTarget, ScreenCapture};
use crate::capture::common::{FrameBuffer};
use crate::upscale::{
    create_upscaler, Upscaler, UpscalingQuality, UpscalingTechnology,
    common::UpscalingAlgorithm,
};

// UI-internal imports (using super::)
use super::profile::{Profile, UpscalingTechnology as ProfileUpscalingTechnology, UpscalingQuality as ProfileUpscalingQuality};
use super::settings::AppSettings;
use super::components::{StatusBar, StatusMessageType};
use super::region_dialog::RegionDialog;
use super::tabs::{TabState};

const ACCENT_COLOR: Color32 = Color32::from_rgb(0, 120, 215); // Blue accent
const SUCCESS_COLOR: Color32 = Color32::from_rgb(25, 170, 88); // Green
const WARNING_COLOR: Color32 = Color32::from_rgb(235, 165, 0); // Amber
const ERROR_COLOR: Color32 = Color32::from_rgb(209, 43, 43);   // Red

/// The main application state
pub struct AppState {
    /// Current profile
    profile: Profile,
    /// Application settings
    settings: AppSettings,
    /// Is capturing active
    is_capturing: bool,
    /// Is fullscreen mode active
    is_fullscreen: bool,
    /// Is upscaling mode active
    is_upscaling: bool,
    /// Hotkey string for toggle capture
    _toggle_capture_hotkey: String,
    /// Hotkey string for capture single frame
    single_frame_hotkey: String,
    /// Hotkey string for toggle overlay
    toggle_overlay_hotkey: String,
    /// Available profiles
    available_profiles: Vec<String>,
    /// Available windows
    available_windows: Vec<String>,
    /// Current selected window index
    selected_window_index: usize,
    /// Current capture source (radio button selection)
    capture_source_index: usize,
    /// Region selection (x, y, width, height)
    region: (i32, i32, u32, u32),
    /// Show region selection dialog
    show_region_dialog: bool,
    /// Status message
    status_message: String,
    /// Status message type
    status_message_type: StatusMessageType,
    /// Current selected tab
    selected_tab: TabState,
    /// Frame buffer for upscaling mode
    upscaling_buffer: Option<Arc<FrameBuffer>>,
    /// Stop signal for upscaling mode
    upscaling_stop_signal: Option<Arc<AtomicBool>>,
    /// Current frame texture
    frame_texture: Option<TextureHandle>,
    /// Status bar
    status_bar: StatusBar,
    /// Region dialog
    region_dialog: RegionDialog,
    /// Frame buffer
    frame_buffer: Arc<FrameBuffer>,
    /// Stop signal
    stop_signal: Arc<AtomicBool>,
    /// Capture status
    capture_status: Arc<Mutex<String>>,
    /// Temporary status message
    temp_status_message: Arc<Mutex<Option<(String, StatusMessageType, Instant)>>>,
    /// Show error dialog
    show_error_dialog: bool,
    /// Error message
    error_message: String,
    /// Phantom data
    _phantom: PhantomData<()>,
    /// Currently running upscaling process (if any)
    scaling_process: Option<std::process::Child>,
    /// Upscaler for the current upscaling mode
    upscaler: Option<Box<dyn Upscaler>>,
    /// Frames processed
    frames_processed: usize,
    /// Worker thread pool for upscaling operations
    upscale_threadpool: ThreadPool,
    /// In-progress upscaled frame
    pending_upscaled_frame: Arc<Mutex<Option<image::RgbaImage>>>,
    /// Flag to indicate an upscale operation is in progress
    upscale_in_progress: Arc<AtomicBool>,
    /// Last time an upscale was requested
    last_upscale_request: Option<Instant>,
    /// Texture dimensions that were last requested
    last_texture_dimensions: Option<(u32, u32)>,
    /// Texture cache for memory management
    texture_cache: TextureCache,
    /// Current GPU memory usage limit flag
    gpu_memory_warning: bool,
    /// Last memory check time
    last_memory_check: Option<Instant>,
    /// Frame rate budgeting for UI responsiveness
    frame_budget: FrameBudget,
    /// Upscaler operation timeout in milliseconds
    upscaler_timeout_ms: u64,
    /// Start time of current upscaling operation
    upscale_start_time: Option<Instant>,
    /// Upscaled frame texture
    upscaled_texture: Option<TextureHandle>,
    /// Upscaled frame
    upscaled_frame: Option<image::RgbaImage>,
    /// Frame timestamps
    frame_timestamps: Vec<Instant>,
    /// Current frame
    current_frame: Option<Arc<FrameBuffer>>,
    /// Auto upscale flag
    auto_upscale: bool,
    /// Upscaled frame channel for communication
    upscaled_frame_sender: std::sync::mpsc::Sender<image::RgbaImage>,
    /// Upscaled frame receiver
    upscaled_frame_receiver: std::sync::mpsc::Receiver<image::RgbaImage>,
}

// Type definition for upscaling buffer
type UpscalingBufferType = Arc<FrameBuffer>;

/// Texture cache to prevent memory leaks and improve reuse
struct TextureCache {
    /// Map of texture size -> texture handle
    textures: AHashMap<(u32, u32), TextureHandle>,
    /// Last time each texture was used
    last_used: AHashMap<(u32, u32), Instant>,
    /// Total texture memory usage in bytes
    texture_memory_usage: usize,
    /// Maximum allowed memory usage in MB
    max_memory_mb: f32,
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            textures: AHashMap::new(),
            last_used: AHashMap::new(),
            texture_memory_usage: 0,
            max_memory_mb: 512.0, // Default to 512MB, can be made configurable
        }
    }
    
    /// Get a texture of the specified size, reusing if possible, with LRU eviction
    fn get_texture(&mut self, ctx: &egui::Context, size: (u32, u32), pixels: &[u8]) -> TextureHandle {
        if let Some(texture) = self.textures.get_mut(&size) {
            // Update existing texture
            texture.set(
                egui::ColorImage::from_rgba_unmultiplied(
                    [size.0 as usize, size.1 as usize],
                    pixels,
                ),
                egui::TextureOptions::NEAREST,
            );
            self.last_used.insert(size, Instant::now());
            texture.clone()
        } else {
            // LRU eviction if over memory budget
            let mem_size = (size.0 * size.1 * 4) as usize;
            while self.get_memory_usage_mb() + (mem_size as f32 / (1024.0 * 1024.0)) > self.max_memory_mb {
                // Find the least recently used texture
                if let Some((&oldest_size, _)) = self.last_used.iter().min_by_key(|(_, &t)| t) {
                    self.textures.remove(&oldest_size);
                    self.last_used.remove(&oldest_size);
                    self.texture_memory_usage -= (oldest_size.0 * oldest_size.1 * 4) as usize;
                } else {
                    break;
                }
            }
            // Create new texture
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [size.0 as usize, size.1 as usize],
                pixels,
            );
            let texture = ctx.load_texture(
                format!("texture_{}x{}", size.0, size.1),
                image,
                egui::TextureOptions::NEAREST,
            );
            self.texture_memory_usage += mem_size;
            self.textures.insert(size, texture.clone());
            self.last_used.insert(size, Instant::now());
            texture
        }
    }
    
    /// Clean up old unused textures to prevent memory leaks
    fn cleanup_old_textures(&mut self, _ctx: &egui::Context) {
        let now = Instant::now();
        let mut to_remove = Vec::new();
        
        for (&size, &last_used) in &self.last_used {
            if now.duration_since(last_used) > Duration::from_secs(5) {
                to_remove.push(size);
            }
        }
        
        for size in to_remove {
            if let Some(_texture) = self.textures.remove(&size) {
                // Just remove from maps - egui manages texture lifetime automatically
                self.last_used.remove(&size);
                self.texture_memory_usage -= (size.0 * size.1 * 4) as usize;
            }
        }
    }
    
    /// Get total texture memory usage in MB
    fn get_memory_usage_mb(&self) -> f32 {
        self.texture_memory_usage as f32 / (1024.0 * 1024.0)
    }

    /// Get number of textures in cache
    fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Get a texture of the specified size, reusing if possible
    // Assuming texture_id on line 434 is a TextureHandle, use From<&TextureHandle>
    fn get_texture_compat(&mut self, ctx: &egui::Context, size: (u32, u32), pixels: &[u8]) -> TextureHandle {
        // This is a placeholder implementation based on TextureCache::get_texture
        // It directly creates/updates textures without the full cache logic for simplicity
        // to address potential mismatches with the provided code snippet's line 434 context.
        // TODO: Replace this with actual TextureCache::get_texture call if context matches.
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [size.0 as usize, size.1 as usize],
            pixels,
        );
        ctx.load_texture(
            format!("texture_{}x{}", size.0, size.1),
            image,
            egui::TextureOptions::NEAREST,
        )
    }


    // Placeholder for where line 434 might be, applying the tuple fix assuming texture_id is TextureId
    fn example_usage_line_434(&mut self, ui: &mut Ui, texture_id: TextureId, size: Vec2) {
         ui.add(egui::Image::new((texture_id, size)));
    }
}

/// Manages frame rate budgeting to prevent UI lag
struct FrameBudget {
    timestamps: VecDeque<Instant>,
    throttling: bool,
    max_frames_per_second: usize,
    window_size_seconds: f32,
}

impl FrameBudget {
    fn new() -> Self {
        Self {
            timestamps: VecDeque::with_capacity(120),
            throttling: false,
            max_frames_per_second: 30, // Target 30 FPS for upscaling operations
            window_size_seconds: 1.0,  // Measure frames over 1 second window
        }
    }
    
    fn add_frame(&mut self, now: Instant) {
        self.timestamps.push_back(now);
        
        // Remove timestamps older than our window
        let cutoff = now - Duration::from_secs_f32(self.window_size_seconds);
        while let Some(ts) = self.timestamps.front() {
            if *ts < cutoff {
                self.timestamps.pop_front();
            } else {
                break;
            }
        }
        
        // Update throttling state based on current rate
        let current_rate = self.timestamps.len() as f32 / self.window_size_seconds;
        self.throttling = current_rate > self.max_frames_per_second as f32;
    }
    
    fn is_throttling(&self) -> bool {
        self.throttling
    }
    
    fn reset(&mut self) {
        self.timestamps.clear();
        self.throttling = false;
    }
}

impl Default for AppState {
    fn default() -> Self {
        let settings = AppSettings::load().unwrap_or_default();
        let profile_path = format!("{}.json", settings.current_profile);
        let profile = Profile::load(&profile_path).unwrap_or_default();

        // Determine capture source index AND region *before* moving profile
        let capture_source_index = profile.capture_source;
        let region = (
            profile.region_x,
            profile.region_y,
            profile.region_width,
            profile.region_height
        );

        let available_windows = crate::capture::common::list_available_windows()
            .map(|windows| windows.iter().map(|w| w.title.clone()).collect())
            .unwrap_or_default();

        Self {
            settings,
            profile, // Move profile here
            is_capturing: false,
            is_fullscreen: false,
            is_upscaling: false,
            _toggle_capture_hotkey: "Ctrl+Alt+C".to_string(),
            single_frame_hotkey: "Ctrl+Alt+S".to_string(),
            toggle_overlay_hotkey: "Ctrl+Alt+O".to_string(),
            available_profiles: super::profile::Profile::list_profiles().unwrap_or_default(),
            available_windows,
            selected_window_index: 0,
            capture_source_index, // Use the value determined before move
            region, // Use the value determined before move
            show_region_dialog: false,
            status_message: "Ready".to_string(),
            status_message_type: StatusMessageType::Info,
            selected_tab: TabState::Capture,
            upscaling_buffer: None,
            upscaling_stop_signal: None,
            frame_texture: None,
            status_bar: StatusBar::new(String::new(), StatusMessageType::Info),
            region_dialog: RegionDialog::new(),
            frame_buffer: Arc::new(FrameBuffer::new(10)),
            stop_signal: Arc::new(AtomicBool::new(false)),
            capture_status: Arc::new(Mutex::new("Idle".to_string())),
            temp_status_message: Arc::new(Mutex::new(None)),
            show_error_dialog: false,
            error_message: String::new(),
            _phantom: PhantomData,
            scaling_process: None,
            upscaler: None,
            frames_processed: 0,
            upscale_threadpool: ThreadPool::new(2), // Use 2 worker threads for upscaling
            pending_upscaled_frame: Arc::new(Mutex::new(None)),
            upscale_in_progress: Arc::new(AtomicBool::new(false)),
            last_upscale_request: None,
            last_texture_dimensions: None,
            texture_cache: TextureCache::new(),
            gpu_memory_warning: false,
            last_memory_check: None,
            frame_budget: FrameBudget::new(),
            upscaler_timeout_ms: 5000, // 5 second timeout for upscaler operations
            upscale_start_time: None,
            upscaled_texture: None,
            upscaled_frame: None,
            frame_timestamps: Vec::new(),
            current_frame: None,
            auto_upscale: false,
            upscaled_frame_sender: std::sync::mpsc::channel().0,
            upscaled_frame_receiver: std::sync::mpsc::channel().1,
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // Check for Alt+S to upscale window under cursor
        if ctx.input(|i| i.modifiers.alt && i.key_pressed(eframe::egui::Key::S)) {
            self.upscale_window_under_cursor(ctx, frame);
            return;
        }
        
        // Check if we need to exit when in upscaling mode
        if crate::ui::is_upscaling_active() {
            if ctx.input(|i| i.key_pressed(eframe::egui::Key::Escape)) {
                // Exit upscaling mode
                crate::ui::cleanup_upscaling();
                
                // If a scaling process is running, kill it
                self.kill_scaling_process();
                
                log::info!("Exited upscaling mode via ESC key");
            }
        }

        // Central panel is the main area of the application
        egui::CentralPanel::default().show(ctx, |ui| {
            // If upscaling is active, render the upscaled content in the main window
            if crate::ui::is_upscaling_active() {
                // Get the full available size for upscaled content
                let available_rect = ui.available_rect_before_wrap();
                let mut upscale_ui = ui.child_ui(available_rect, egui::Layout::default());
                
                // Try to get upscaled texture from the renderer
                if let Some(texture_id) = crate::ui::get_upscaled_texture(ctx, None) {
                    // Render the upscaled texture to fill the window
                    let available_size = upscale_ui.available_size();
                    upscale_ui.add(
                        egui::Image::new((texture_id, available_size))
                            .fit_to_exact_size(available_size)
                    );
                } else {
                    // Show waiting message
                    upscale_ui.centered_and_justified(|ui| {
                        ui.heading("Waiting for upscaled content...");
                        ui.label("If you don't see upscaled content, ensure the source window is visible.");
                        ui.label("Press ESC to exit upscaling mode.");
                    });
                }
            } else {
                // Regular UI when not in upscaling mode
                // Top panel with app name and main actions
                egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                    self.show_top_bar(ui, frame);
                });
                
                // Status bar at bottom
                egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                    self.show_status_bar(ui);
                });
                
                // Left sidebar with tabs
                egui::SidePanel::left("side_panel")
                    .default_width(200.0)
                    .width_range(180.0..=240.0)
                    .resizable(true)
                    .show(ctx, |ui| {
                        self.show_sidebar(ui);
                    });
                
                // Central content area
                egui::CentralPanel::default().show(ctx, |ui| {
                    match self.selected_tab {
                        TabState::Capture => self.show_capture_tab(ctx, ui),
                        TabState::Settings => self.show_settings_tab(ui),
                        TabState::Advanced => self.show_advanced_tab(ui),
                    }
                });
            }
        });
        
        // Region selection dialog
        if self.show_region_dialog {
            self.show_region_dialog(ctx);
        }
        
        // Update upscaling mode if active
        if self.is_upscaling {
            self.update_upscaling_mode(ctx, frame);
        }

        // Check for upscaler timeout
        if let Some(start_time) = self.upscale_start_time {
            if start_time.elapsed().as_millis() as u64 > self.upscaler_timeout_ms {
                log::warn!("Upscale operation timed out after {}ms", self.upscaler_timeout_ms);
                self.is_upscaling = false;
                self.upscale_start_time = None;
            }
        }
        
        // Fix upscaled frame receiver logic
        if let Ok(upscaled_frame) = self.upscaled_frame_receiver.try_recv() {
            // Process upscaled frame
            self.upscaled_frame = Some(upscaled_frame.clone());
            
            // Create texture from upscaled frame
            let texture_id = self.texture_cache.get_texture(
                ctx,
                (upscaled_frame.width(), upscaled_frame.height()),
                upscaled_frame.as_raw(),
            );
            
            self.upscaled_texture = Some(texture_id);
        }
        
        // FIXED: Properly lock mutex instead of using load()
        let has_pending_upscale = match self.pending_upscaled_frame.lock() {
            Ok(guard) => guard.is_some(),
            Err(_) => false,
        };
        
        if !self.upscale_in_progress.load(Ordering::SeqCst) && !has_pending_upscale {
            // Handle upscaling logic
            if self.is_upscaling && self.last_upscale_request.is_some() {
                if let Some(frame) = &self.current_frame {
                    // FIXED: Add clone to pass Arc properly
                    self.schedule_next_upscale(frame.clone());
                }
            }
        }
        
        // FIXED: Pass ctx instead of Duration
        self.texture_cache.cleanup_old_textures(ctx);
        
        // Determine whether we need to repaint
        let mut should_repaint = false;
        
        // Only request repaint if we have a pending frame or are processing
        if self.is_upscaling || self.upscaled_frame.is_some() || has_pending_upscale {
            // Dynamic repaint rate based on memory and frame budget
            let repaint_delay_ms = if self.gpu_memory_warning {
                100 // 10 FPS when under memory pressure
            } else if self.frame_budget.is_throttling() {
                50 // 20 FPS when throttling
            } else {
                16 // Roughly 60 FPS normally
            };
            
            ctx.request_repaint_after(Duration::from_millis(repaint_delay_ms));
            should_repaint = true;
        }
        
        // Always process input even if we're not upscaling
        self.process_input(ctx);

        // Check if we should update GPU memory usage
        if let Some(last_check) = self.last_memory_check {
            // Check memory every second
            if last_check.elapsed() > Duration::from_secs(1) {
                self.check_gpu_memory_pressure(ctx);
                self.last_memory_check = Some(Instant::now());
            }
        } else {
            self.check_gpu_memory_pressure(ctx);
            self.last_memory_check = Some(Instant::now());
        }
        
        // FIXED: Clean up old textures using ctx, not Duration
        self.texture_cache.cleanup_old_textures(ctx);
        
        // Check if we need to schedule a new upscale
        // FIXED: Use proper mutex locking instead of load()
        let has_pending = match self.pending_upscaled_frame.lock() {
            Ok(guard) => guard.is_some(),
            Err(_) => false,
        };
        
        if !self.is_upscaling && 
           !has_pending &&
           self.current_frame.is_some() &&
           should_repaint {
            if let Some(frame) = &self.current_frame {
                // FIXED: Add clone to pass Arc properly
                self.schedule_next_upscale(frame.clone());
            }
        }
        
        // Check if we should reload textures based on some criteria 
        // (maybe frame rate or memory pressure)
        if self.should_reload_textures() {
            // Fix: Check the pending frame properly
            let has_pending = match self.pending_upscaled_frame.lock() {
                Ok(guard) => guard.is_some(),
                Err(_) => false,
            };
            
            if !has_pending {
                // Do texture reloading
            }
        }
    }
}

impl AppState {
    /// Configure custom fonts
    fn configure_fonts(&self, ctx: &egui::Context) {
        let fonts = eframe::egui::FontDefinitions::default();
        // Could add custom fonts here
        ctx.set_fonts(fonts);
    }

    /// Show the application top bar
    fn show_top_bar(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.heading(RichText::new("NU Scale").size(22.0).color(ACCENT_COLOR));
            ui.add_space(16.0);
            
            let save_button = ui.button(RichText::new("💾 Save Profile").size(14.0));
            if save_button.clicked() {
                if let Err(e) = self.profile.save(None) {
                    self.status_message = format!("Error saving profile: {}", e);
                    self.status_message_type = StatusMessageType::Error;
                } else {
                    self.status_message = "Profile saved".to_string();
                    self.status_message_type = StatusMessageType::Success;
                }
            }
            
            if ui.button(RichText::new("+ New Profile").size(14.0)).clicked() {
                // Show dialog to create new profile (simplified)
                let new_name = format!("Profile_{}", self.available_profiles.len() + 1);
                self.profile = Profile::new(&new_name);
                self.available_profiles.push(new_name);
                self.status_message = "New profile created".to_string();
                self.status_message_type = StatusMessageType::Success;
            }
            // test
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                
                // Scale button - combined scaling and fullscreen
                let scale_button = ui.add(egui::Button::new(
                    RichText::new("🔍 Scale").size(14.0))
                        .fill(Color32::from_rgb(180, 100, 240)));
                
                if scale_button.clicked() {
                    // Use launch_fullscreen_mode instead of start_scaling_process
                    // This applies upscaling directly in the current window
                    let ctx = ui.ctx().clone(); // Clone context if needed within this scope
                    self.launch_fullscreen_mode(&ctx, frame);
                }
                
                ui.add_space(8.0);

                // Original fullscreen mode button
                let fullscreen_button = ui.add(egui::Button::new(
                    RichText::new("🖥️ Fullscreen Mode").size(14.0))
                        .fill(Color32::from_rgb(0, 120, 215)));
                
                if fullscreen_button.clicked() {
                    let ctx = ui.ctx().clone(); // Clone context if needed
                    self.launch_fullscreen_mode(&ctx, frame);
                }
                
                ui.add_space(8.0);
                
                if self.is_capturing {
                    let stop_button = ui.add(egui::Button::new(
                        RichText::new("⏹ Stop Capture").size(14.0))
                            .fill(Color32::from_rgb(180, 60, 60)));
                    
                    if stop_button.clicked() {
                        self.is_capturing = false;
                        self.status_message = "Capture stopped".to_string();
                        self.status_message_type = StatusMessageType::Info;
                    }
                } else {
                    let start_button = ui.add(egui::Button::new(
                        RichText::new("▶ Start Capture").size(14.0))
                            .fill(Color32::from_rgb(60, 180, 60)));
                    
                    if start_button.clicked() {
                        self.is_capturing = true;
                        self.status_message = "Capture started".to_string();
                        self.status_message_type = StatusMessageType::Success;
                    }
                }
                
                if ui.button(RichText::new("📷 Capture Frame").size(14.0)).clicked() {
                    self.status_message = "Frame captured".to_string();
                    self.status_message_type = StatusMessageType::Success;
                }
            });
        });
    }
    
    /// Show the status bar
    fn show_status_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            
            // Choose color based on status type
            let status_color = match self.status_message_type {
                StatusMessageType::Info => Color32::LIGHT_GRAY,
                StatusMessageType::Success => SUCCESS_COLOR,
                StatusMessageType::Warning => WARNING_COLOR,
                StatusMessageType::Error => ERROR_COLOR,
            };
            
            ui.label(RichText::new(&self.status_message).color(status_color).monospace());
            
            // Show capture status
            if self.is_capturing {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("● CAPTURING")
                            .color(SUCCESS_COLOR)
                            .strong()
                            .size(14.0)
                    );
                });
            }
        });
    }
    
    /// Show the sidebar with tabs
    fn show_sidebar(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("Navigation");
            ui.add_space(10.0);
        });
        
        ui.separator();
        ui.add_space(10.0);
        
        let tab_button = |ui: &mut Ui, selected: bool, icon: &str, text: &str| {
            let response = ui.add(
                egui::Button::new(
                    RichText::new(format!("{} {}", icon, text))
                        .size(16.0)
                        .color(if selected { ACCENT_COLOR } else { Color32::LIGHT_GRAY })
                )
                .frame(false)
                .fill(if selected { 
                    Color32::from_rgba_premultiplied(ACCENT_COLOR.r(), ACCENT_COLOR.g(), ACCENT_COLOR.b(), 25) 
                } else { 
                    Color32::TRANSPARENT 
                })
                .min_size(Vec2::new(ui.available_width(), 36.0))
            );
            
            response
        };
        
        ui.vertical_centered_justified(|ui| {
            let capture_selected = matches!(self.selected_tab, TabState::Capture);
            let settings_selected = matches!(self.selected_tab, TabState::Settings);
            let advanced_selected = matches!(self.selected_tab, TabState::Advanced);
            
            if tab_button(ui, capture_selected, "📷", "Capture").clicked() {
                self.selected_tab = TabState::Capture;
            }
            
            if tab_button(ui, settings_selected, "⚙️", "Settings").clicked() {
                self.selected_tab = TabState::Settings;
            }
            
            if tab_button(ui, advanced_selected, "🔧", "Advanced").clicked() {
                self.selected_tab = TabState::Advanced;
            }
        });
        
        // App info at bottom of sidebar
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(4.0);
            ui.label(RichText::new("v1.0.0").monospace().weak());
            ui.label(RichText::new("NU Scale").strong());
            ui.add_space(8.0);
        });
    }
    
    /// Create a styled card frame
    fn card_frame() -> Frame {
        Frame::none()
            .fill(Color32::from_gray(30))
            .stroke(Stroke::new(1.0, Color32::from_gray(60)))
            .rounding(Rounding::same(8.0))
            .inner_margin(16.0)
            .outer_margin(Vec2::new(0.0, 8.0))
    }
    
    /// Show the region selection dialog
    fn show_region_dialog(&mut self, ctx: &Context) {
        let mut dialog = RegionDialog::new();
        
        // Set initial region values
        dialog.set_region(
            self.region.0, 
            self.region.1, 
            self.region.2 as i32, 
            self.region.3 as i32
        );
        
        if dialog.show(ctx) {
            // Dialog was confirmed with OK
            self.show_region_dialog = false;
            
            // Get region values from dialog
            let (x, y, width, height) = dialog.get_region();
            
            // Update the region
            self.region = (x, y, width as u32, height as u32);
            
            // Update the profile
            self.profile.capture_source = 2; // Region capture
            self.profile.region_x = x;
            self.profile.region_y = y;
            self.profile.region_width = width as u32;
            self.profile.region_height = height as u32;
        } else if dialog.was_cancelled() {
            // Dialog was cancelled
            self.show_region_dialog = false;
        }
    }
    
    /// Show the capture tab
    fn show_capture_tab(&mut self, _ctx: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.heading("Capture Settings");
            ui.add_space(16.0);
            
            // Profile selection
            Self::card_frame().show(ui, |ui| {
                ui.strong(RichText::new("Profile Selection").size(16.0).color(ACCENT_COLOR));
                ui.add_space(12.0);
                
                ui.horizontal(|ui| {
                    ui.label("Current Profile:");
                    ui.add_space(8.0);
                    
                    egui::ComboBox::from_id_source("profile_selector")
                        .selected_text(RichText::new(&self.profile.name).strong())
                        .width(200.0)
                        .show_ui(ui, |ui| {
                            // Need to clone names to avoid borrowing issues if profile changes
                            let profile_names = self.available_profiles.clone();
                            let mut selected_profile_name_in_combo = self.profile.name.clone(); // Use a temporary variable for the combo box state
                            let mut profile_changed = false;
                            for profile_name in &profile_names {
                                if ui.selectable_value(
                                    &mut selected_profile_name_in_combo,
                                    profile_name.clone(),
                                    profile_name
                                ).changed() {
                                    profile_changed = true; // Mark that a change occurred
                                }
                            }
                            // If the selection changed, load the profile after the loop
                            if profile_changed && selected_profile_name_in_combo != self.profile.name {
                                 // Load the selected profile
                                 if let Ok(loaded_profile) = Profile::load(&format!("{}.json", selected_profile_name_in_combo)) { // Use the temp variable
                                     self.profile = loaded_profile;
                                     self.capture_source_index = self.profile.capture_source; // Update index
                                     self.region = (self.profile.region_x, self.profile.region_y, self.profile.region_width, self.profile.region_height); // Update region
                                     self.settings.current_profile = selected_profile_name_in_combo; // Assign the final selected name
                                     let _ = self.settings.save(); // Save settings immediately
                                     self.status_message = format!("Loaded profile: {}", self.profile.name);
                                     self.status_message_type = StatusMessageType::Info;
                                 } else {
                                     self.status_message = format!("Failed to load profile: {}", selected_profile_name_in_combo);
                                     self.status_message_type = StatusMessageType::Error;
                                      // Optionally revert selected_profile_name_in_combo back to self.profile.name if load fails
                                 }
                             }
                        });
                });
                
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Save").size(14.0)).clicked() {
                        // Ensure profile name is filesystem-safe before saving if needed
                        let profile_path = format!("{}.json", self.profile.name); // Simple path for now
                        if let Err(e) = self.profile.save(Some(&profile_path)) { // Pass path to save
                             // Use status bar instead of Dialog
                            self.status_message = format!("Failed to save profile: {}", e);
                            self.status_message_type = StatusMessageType::Error;
                        } else {
                            self.status_message = "Profile saved".to_string();
                            self.status_message_type = StatusMessageType::Success;
                            // Ensure saved profile is in the list
                            if !self.available_profiles.contains(&self.profile.name) {
                                self.available_profiles.push(self.profile.name.clone());
                            }
                        }
                    }
                    
                    if ui.button(RichText::new("📋 New").size(14.0)).clicked() {
                        // TODO: Show profile name input dialog for better UX
                        let new_name = format!("Profile_{}", self.available_profiles.len() + 1);
                        self.profile = Profile::new(&new_name);
                         // Reset UI state associated with the profile
                        self.capture_source_index = self.profile.capture_source;
                        self.region = (self.profile.region_x, self.profile.region_y, self.profile.region_width, self.profile.region_height);
                        // Save the new profile immediately so it exists
                        let profile_path = format!("{}.json", self.profile.name);
                        if let Err(e) = self.profile.save(Some(&profile_path)) {
                             self.status_message = format!("Failed to save new profile: {}", e);
                             self.status_message_type = StatusMessageType::Error;
                        } else {
                             self.available_profiles.push(new_name); // Add only if save succeeded
                             self.status_message = "New profile created".to_string();
                             self.status_message_type = StatusMessageType::Success;
                        }
                    }
                    
                    if ui.button(RichText::new("❌ Delete").size(14.0)).clicked() {
                        // TODO: Show confirmation dialog
                        let profile_to_delete = self.profile.name.clone();
                        // Prevent deleting the last profile if needed, or ensure a default exists
                        if profile_to_delete != "Default" && self.available_profiles.len() > 1 { // Example: don't delete "Default" or the last one
                             let profile_path = format!("{}.json", profile_to_delete);
                             if let Ok(_) = std::fs::remove_file(&profile_path) { // Use std::fs directly
                                 self.status_message = "Profile deleted".to_string();
                                 self.status_message_type = StatusMessageType::Success;

                                 // Remove from available list
                                 self.available_profiles.retain(|p| p != &profile_to_delete);

                                 // Load the first available profile (or default)
                                 let next_profile_name = self.available_profiles.first().cloned().unwrap_or_else(|| "Default".to_string());
                                 if let Ok(loaded_profile) = Profile::load(&format!("{}.json", next_profile_name)) {
                                     self.profile = loaded_profile;
                                 } else {
                                     self.profile = Profile::default(); // Fallback to default
                                 }
                                 self.capture_source_index = self.profile.capture_source;
                                 self.region = (self.profile.region_x, self.profile.region_y, self.profile.region_width, self.profile.region_height);
                                 self.settings.current_profile = self.profile.name.clone();
                                 let _ = self.settings.save();

                             } else {
                                 self.status_message = "Error deleting profile file".to_string();
                                 self.status_message_type = StatusMessageType::Error;
                             }
                        } else {
                             self.status_message = "Cannot delete the last or default profile".to_string();
                             self.status_message_type = StatusMessageType::Warning;
                        }
                    }
                });
            });
            
            // Capture source
            Self::card_frame().show(ui, |ui| {
                ui.strong(RichText::new("Capture Source").size(16.0).color(ACCENT_COLOR));
                ui.add_space(12.0);
                
                // Fullscreen
                if ui.radio_value(
                    &mut self.capture_source_index,
                    0,
                    RichText::new("🖥️ Fullscreen").size(14.0)
                ).changed() {
                    self.profile.capture_source = 0;
                }
                
                ui.add_space(4.0);
                
                // Window
                ui.horizontal(|ui| {
                    if ui.radio_value(
                        &mut self.capture_source_index,
                        1,
                        RichText::new("🪟 Window").size(14.0)
                    ).changed() {
                         self.profile.capture_source = 1;
                         // Update window title if a window is selected
                         if let Some(win_title) = self.available_windows.get(self.selected_window_index) {
                             self.profile.window_title = win_title.clone();
                         }
                    }
                    
                    if self.capture_source_index == 1 {
                        ui.add_space(16.0);
                        
                        egui::ComboBox::from_id_source("window_selector")
                            .selected_text(
                                self.available_windows.get(self.selected_window_index)
                                    .cloned()
                                    .unwrap_or_else(|| "Select Window".to_string())
                            )
                            .width(240.0)
                            .show_ui(ui, |ui| {
                                let mut changed = false;
                                for (i, window_name) in self.available_windows.iter().enumerate() {
                                    // Use selectable_value correctly
                                    // CAPTURE the response
                                    let response = ui.selectable_label(self.selected_window_index == i, window_name);
                                    if response.clicked() { // Use the captured response
                                        if self.selected_window_index != i {
                                            self.selected_window_index = i;
                                            changed = true;
                                        }
                                    }
                                    // Add double-click handling if needed, using response
                                    // if response.double_clicked() { ... }
                                }
                                // Update profile only if selection changed
                                if changed {
                                    self.profile.capture_source = 1;
                                    self.profile.window_title = self.available_windows[self.selected_window_index].clone();
                                }
                            });
                            
                        ui.add_space(8.0);
                        
                        if ui.button(RichText::new("🔄 Refresh").size(14.0)).clicked() {
                            // Refresh window list - Use crate:: path
                            self.available_windows = crate::capture::common::list_available_windows()
                                .map(|windows| windows.iter().map(|w| w.title.clone()).collect())
                                .unwrap_or_default();
                            // Reset selection if current index is out of bounds
                            if self.selected_window_index >= self.available_windows.len() {
                                self.selected_window_index = 0;
                                if self.profile.capture_source == 1 { // Update profile title if window was selected
                                    self.profile.window_title = self.available_windows.first().cloned().unwrap_or_default();
                                }
                            }
                        }
                    }
                });
                
                ui.add_space(4.0);
                
                // Region
                ui.horizontal(|ui| {
                     if ui.radio_value(
                        &mut self.capture_source_index,
                        2,
                        RichText::new("📏 Region").size(14.0)
                    ).changed() {
                         self.profile.capture_source = 2;
                         // Update profile region fields when Region is selected
                         self.profile.region_x = self.region.0;
                         self.profile.region_y = self.region.1;
                         self.profile.region_width = self.region.2;
                         self.profile.region_height = self.region.3;
                    }
                    
                    if self.capture_source_index == 2 {
                        ui.add_space(16.0);
                        
                        if ui.button(RichText::new("Select Region").size(14.0)).clicked() {
                            self.show_region_dialog = true;
                        }
                        
                        ui.add_space(8.0);
                        
                        // Update region display from self.region, not profile directly
                        // as profile might only update when the dialog confirms
                        let (x, y, width, height) = self.region;
                        
                        ui.label(
                            RichText::new(format!("({}, {}, {}x{})", x, y, width, height))
                                .monospace()
                        );
                    }
                });
            });
        });
    }
    
    /// Show the settings tab
    fn show_settings_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.heading("Settings");
            ui.add_space(16.0);
            
            // Upscaling settings
            Self::card_frame().show(ui, |ui| {
                ui.strong(RichText::new("Upscaling Settings").size(16.0).color(ACCENT_COLOR));
                ui.add_space(12.0);
                
                // Scale factor
                ui.horizontal(|ui| {
                    ui.label("Scale Factor:");
                    ui.add_space(8.0);
                    let slider = Slider::new(&mut self.profile.scale_factor, 1.0..=4.0)
                        .step_by(0.1)
                        .text("×");
                    let _response = ui.add_sized([300.0, 20.0], slider);
                    
                    ui.label(format!("{:.1}×", self.profile.scale_factor));
                });
                
                ui.add_space(8.0);
                
                // Upscaling technology
                ui.horizontal(|ui| {
                    ui.label("Upscaling Technology:");
                    ui.add_space(8.0);
                    
                    // Map usize back to string for display
                    let tech_text = match self.profile.upscaling_tech {
                         0 => "Auto", // Assuming 0 is Auto/None
                         1 => "AMD FSR",
                         2 => "NVIDIA DLSS",
                         3 => "GPU (Vulkan)",  // Changed from CUDA
                         4 => "Fallback/Basic",
                         _ => "Unknown",
                    };
                    
                    egui::ComboBox::from_id_source("upscale_tech")
                        .selected_text(tech_text) // Display mapped text
                        .width(300.0)
                        .show_ui(ui, |ui| {
                            // Use usize values directly
                            ui.selectable_value(&mut self.profile.upscaling_tech, 0, "Auto");
                            ui.selectable_value(&mut self.profile.upscaling_tech, 1, "AMD FSR");
                            ui.selectable_value(&mut self.profile.upscaling_tech, 2, "NVIDIA DLSS");
                            ui.selectable_value(&mut self.profile.upscaling_tech, 3, "GPU (Vulkan)");  // Changed from CUDA
                            ui.selectable_value(&mut self.profile.upscaling_tech, 4, "Fallback/Basic");
                        });
                });
                
                ui.add_space(8.0);
                
                // Upscaling quality
                ui.horizontal(|ui| {
                    ui.label("Upscaling Quality:");
                    ui.add_space(8.0);
                    
                    // Map usize back to string for display
                    let quality_text = match self.profile.upscaling_quality {
                        0 => "Ultra",
                        1 => "Quality",
                        2 => "Balanced",
                        3 => "Performance",
                        _ => "Unknown",
                    };
                    
                    egui::ComboBox::from_id_source("upscale_quality")
                        .selected_text(quality_text) // Display mapped text
                        .width(300.0)
                        .show_ui(ui, |ui| {
                             // Use usize values directly
                            ui.selectable_value(&mut self.profile.upscaling_quality, 0, "Ultra Quality");
                            ui.selectable_value(&mut self.profile.upscaling_quality, 1, "Quality");
                            ui.selectable_value(&mut self.profile.upscaling_quality, 2, "Balanced");
                            ui.selectable_value(&mut self.profile.upscaling_quality, 3, "Performance");
                        });
                });
                
                // Only show algorithm selection for GPU or Fallback upscaling (index 3 or 4)
                if self.profile.upscaling_tech == 3 || self.profile.upscaling_tech == 4 {
                    ui.add_space(8.0);
                    
                    // Upscaling algorithm is usize
                    ui.horizontal(|ui| {
                        ui.label("Upscaling Algorithm:");
                        ui.add_space(8.0);
                        
                        // Map usize back to string for display
                        let algo_text = match self.profile.upscaling_algorithm {
                             0 => "Lanczos (a=3)",
                             1 => "Bicubic",
                             2 => "Bilinear",
                             3 => "Nearest-Neighbor",
                             _ => "Unknown", // Or default to Lanczos3 text
                        };
                        let mut current_algorithm_index = self.profile.upscaling_algorithm;
                        
                        egui::ComboBox::from_id_source("upscale_algorithm")
                            .selected_text(algo_text) // Use mapped text
                            .width(300.0)
                            .show_ui(ui, |ui| {
                                // Use usize values
                                ui.selectable_value(&mut current_algorithm_index, 3, "Nearest-Neighbor"); // Index 3
                                ui.selectable_value(&mut current_algorithm_index, 2, "Bilinear");       // Index 2
                                ui.selectable_value(&mut current_algorithm_index, 1, "Bicubic");        // Index 1
                                ui.selectable_value(&mut current_algorithm_index, 0, "Lanczos (a=3)");  // Index 0
                                // Add other algorithms if defined in Profile struct with corresponding indices
                            });
                        // Update the profile if the index changed
                        if current_algorithm_index != self.profile.upscaling_algorithm {
                             self.profile.upscaling_algorithm = current_algorithm_index;
                        }
                    });
                    
                    // Add algorithm description based on the usize index
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add_space(138.0); // Align with dropdown content
                        
                        let description = match self.profile.upscaling_algorithm {
                             3 => "Copies each input pixel to an N×N block. Zero smoothing, zero blur, but aliased.",
                             2 => "Computes a weighted average of the four nearest input pixels. Fast and smooth, but tends to blur sharp edges.",
                             1 => "Uses cubic convolution on a 4×4 neighborhood to preserve more edge sharpness than bilinear, at moderate cost.",
                             0 => "Windowed sinc filter over a 6×6 kernel. Best edge preservation among traditional kernels, heavier compute.",
                             _ => "", // Default or handle unknown index
                        };
                        
                        ui.label(RichText::new(description).weak().italics());
                    });
                    
                }
                // No else needed, algorithm index remains as is when tech is not Fallback
            });
            
            // Hotkey settings - REMOVED as `hotkey` field doesn't exist on Profile
            // Self::card_frame().show(ui, |ui| { ... });
            // Use settings fields for hotkeys instead
            Self::card_frame().show(ui, |ui| {
                ui.strong(RichText::new("Hotkey Settings").size(16.0).color(ACCENT_COLOR));
                ui.add_space(12.0);
                
                // Toggle capture hotkey
                ui.horizontal(|ui| {
                    ui.label("Start/Stop Capture:");
                    ui.add_space(8.0);
                    // Use the field from AppSettings
                    let text_edit = TextEdit::singleline(&mut self.settings.toggle_capture_hotkey)
                        .desired_width(200.0)
                        .hint_text("Enter hotkey (e.g., Ctrl+Shift+C)");
                    ui.add(text_edit);
                });
                
                ui.add_space(4.0);
                
                // Single frame hotkey
                ui.horizontal(|ui| {
                    ui.label("Capture Single Frame:");
                    ui.add_space(8.0);
                     // Use the field from AppSettings
                    let text_edit = TextEdit::singleline(&mut self.settings.capture_frame_hotkey)
                        .desired_width(200.0)
                        .hint_text("Enter hotkey (e.g., Ctrl+Shift+F)");
                    ui.add(text_edit);
                });
                
                ui.add_space(4.0);
                
                // Overlay toggle hotkey
                ui.horizontal(|ui| {
                    ui.label("Toggle Overlay:");
                    ui.add_space(8.0);
                    // Use the field from AppSettings
                    let text_edit = TextEdit::singleline(&mut self.settings.toggle_overlay_hotkey)
                        .desired_width(200.0)
                        .hint_text("Enter hotkey (e.g., Ctrl+Shift+O)");
                    ui.add(text_edit);
                });
                 ui.label("Note: Hotkey changes might require restart."); // Add note
            });
            
            // FPS settings
            Self::card_frame().show(ui, |ui| {
                ui.strong(RichText::new("Capture FPS").size(16.0).color(ACCENT_COLOR));
                ui.add_space(12.0);
                
                ui.horizontal(|ui| {
                    ui.label("Target FPS:");
                    ui.add_space(8.0);
                     // Fix slider range type
                    let slider = Slider::new(&mut self.profile.fps, 1.0..=240.0)
                        .text("fps");
                    let _response = ui.add_sized([300.0, 20.0], slider);
                    ui.label(format!("{} fps", self.profile.fps));
                });
            });
        });
    }
    
    /// Show the advanced tab
    fn show_advanced_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.heading("Advanced");
            ui.add_space(16.0);
            
            // Application settings
            Self::card_frame().show(ui, |ui| {
                ui.strong(RichText::new("Application Settings").size(16.0).color(ACCENT_COLOR));
                ui.add_space(12.0);
                
                // Checkboxes for application settings
                ui.vertical(|ui| {
                    // REMOVED: ui.checkbox(&mut self.settings.start_minimized, "Start Minimized");
                    // REMOVED: ui.checkbox(&mut self.settings.start_with_system, "Start with System");
                    // REMOVED: ui.checkbox(&mut self.settings.check_for_updates, "Check for Updates");
                    // Corrected field name:
                    ui.checkbox(&mut self.settings.auto_save_frames, "Auto-save Captured Frames");
                     ui.add_space(4.0);
                     ui.checkbox(&mut self.settings.show_fps_counter, "Show FPS counter");
                     ui.add_space(4.0);
                     ui.checkbox(&mut self.settings.show_notifications, "Show notifications");
                });
                
                ui.add_space(8.0);
                
                // Theme selection
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    ui.add_space(8.0);
                    
                    egui::ComboBox::from_id_source("theme")
                        .selected_text(&self.settings.theme) // Theme is already String
                        .width(200.0)
                        .show_ui(ui, |ui| {
                            // Theme field is String, no need for System value? Assuming dark/light
                            // ui.selectable_value(&mut self.settings.theme, "system".to_string(), "System");
                            ui.selectable_value(&mut self.settings.theme, "light".to_string(), "Light");
                            ui.selectable_value(&mut self.settings.theme, "dark".to_string(), "Dark");
                        });
                });
            });
            
            // Advanced options placeholder
            Self::card_frame().show(ui, |ui| {
                ui.strong(RichText::new("Advanced Options").size(16.0).color(ACCENT_COLOR));
                ui.add_space(12.0);
                
                ui.vertical_centered(|ui| {
                    ui.label("Advanced settings will be available in future versions.");
                });
            });
            
             // Save settings button for this tab
             if ui.button("Save Application Settings").clicked() {
                 if let Err(e) = self.settings.save() {
                     self.status_message = format!("Failed to save settings: {}", e);
                     self.status_message_type = StatusMessageType::Error;
                 } else {
                     self.status_message = "Application settings saved".to_string();
                     self.status_message_type = StatusMessageType::Success;
                 }
             }
        });
    }

    /// Toggle fullscreen mode
    pub fn toggle_fullscreen_mode(&mut self, ctx: &egui::Context) -> Result<()> {
        self.is_fullscreen = !self.is_fullscreen;
        
        // Use eframe's API to toggle fullscreen mode
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.is_fullscreen));
        
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if self.is_fullscreen {
                        if let Some(element) = document.document_element() {
                            let _ = element.request_fullscreen();
                        }
                    } else {
                        let _ = document.exit_fullscreen();
                    }
                }
            }
        }
        
        log::info!("Toggled fullscreen mode: {}", self.is_fullscreen);
        Ok(())
    }
    
    /// Start upscaling mode
    pub fn start_upscaling_mode(
        &mut self,
        source: CaptureTarget,
        technology: UpscalingTechnology,
        quality: UpscalingQuality,
        fps: u32,
        algorithm: Option<UpscalingAlgorithm>,
    ) -> Result<()> {
        // Create buffer with capacity for 10 frames and a stop signal
        let buffer = Arc::new(FrameBuffer::new(10)); 
        let stop_signal = Arc::new(AtomicBool::new(false));
        
        // Use crate:: path
        let _capture_handle = crate::capture::common::start_live_capture_thread(
            source.clone(),
            fps,
            buffer.clone(),
            stop_signal.clone(),
        )?;
        
        log::info!("Capture thread started for source: {:?}", source);
        
        // Create the upscaler using the factory function
        // No need for separate CUDA checks here anymore
        let upscaler = create_upscaler(technology, quality, algorithm)?;
        log::info!("Upscaler created: {}", upscaler.name());
        
        // Store references for cleanup and use
        self.upscaling_buffer = Some(buffer);
        self.upscaling_stop_signal = Some(stop_signal);
        
        // Create entry in AppState to track the upscaler
        self.upscaler = Some(upscaler);
        
        // Set state
        self.is_upscaling = true;
        
        // Maximize the window - in an actual implementation, we would maximize the window
        // using the eframe API. Since eframe::Frame::open() doesn't exist, we'll use a comment
        // to indicate what should happen here
        // #[cfg(not(target_arch = "wasm32"))]
        // Maximize current window via appropriate platform API
        
        Ok(())
    }
    
    /// Update the application upscaling mode state
    /// Renders the captured frames with the upscaler
    fn update_upscaling_mode(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for ESC key to exit fullscreen mode
        if ctx.input(|i| i.key_pressed(eframe::egui::Key::Escape)) {
            log::info!("ESC pressed, exiting fullscreen mode");
            
            // Exit fullscreen mode if active
            if self.is_fullscreen {
                if let Err(e) = self.toggle_fullscreen_mode(ctx) {
                    log::error!("Failed to exit fullscreen mode: {}", e);
                }
            }
            
            // Stop upscaling
            if let Some(stop_signal) = &self.upscaling_stop_signal {
                stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            
            return;
        }
        
        // Clear the entire screen and ensure we paint over everything with a solid background
        ctx.set_visuals(eframe::egui::Visuals::dark());
        
        // Create a full-screen frame with black background
        eframe::egui::CentralPanel::default()
            .frame(eframe::egui::Frame::none()
                .fill(eframe::egui::Color32::BLACK)
                .stroke(eframe::egui::Stroke::NONE))
            .show(ctx, |ui| {
                let available_size = ui.available_size();
                log::debug!("Available UI size: {}x{}", available_size.x, available_size.y);
                
                // Force opaque painting
                ui.set_clip_rect(ui.max_rect());
                
                // Clone buffer to avoid borrowing issues
                let buffer_option = self.upscaling_buffer.clone();
                
                if let Some(buffer) = buffer_option {
                    log::debug!("Upscaling buffer exists, trying to get latest frame");
                    
                    // Check if we already have a pending upscaled frame
                    let have_pending_frame = match self.pending_upscaled_frame.lock() {
                        Ok(guard) => guard.is_some(),
                        Err(_) => false,
                    };
                    
                    // First, check if there's an upscale in progress
                    let upscale_in_progress = self.upscale_in_progress.load(Ordering::SeqCst);
                    
                    if have_pending_frame {
                        // We have a ready frame from a previous upscale operation
                        let upscaled_frame = match self.pending_upscaled_frame.lock() {
                            Ok(mut guard) => guard.take(),
                            Err(_) => None,
                        };
                        
                        if let Some(upscaled) = upscaled_frame {
                            // We have an upscaled frame ready to display
                            log::info!("Using previously upscaled frame: {}x{}", 
                                     upscaled.width(), upscaled.height());
                            
                            let size = [upscaled.width() as usize, upscaled.height() as usize];
                            let pixels = upscaled.as_raw();
                            
                            self.render_frame_to_screen(ui, available_size, size, pixels);
                            
                            // Track frame processing
                            self.frames_processed += 1;
                            log::debug!("Frames processed: {}", self.frames_processed);
                            
                            // Schedule next upscale operation if not already in progress
                            if !upscale_in_progress {
                                // Clone the buffer to avoid borrowing issues
                                let buffer_clone = buffer.clone();
                                self.schedule_next_upscale(buffer_clone);
                            }
                        }
                    } else if !upscale_in_progress {
                        // No upscaling in progress and no pending frame, schedule a new upscale
                        let buffer_clone = buffer.clone();
                        self.schedule_next_upscale(buffer_clone);
                    } else {
                        // Upscaling is in progress, render the last frame if available
                        if let Some(texture) = &self.frame_texture {
                            log::debug!("Upscaling in progress, rendering last frame");
                            
                            // Calculate dimensions to fit the screen
                            let texture_size = texture.size_vec2();
                            let aspect_ratio = texture_size.x / texture_size.y;
                            let max_width = available_size.x;
                            let max_height = available_size.y;
                            
                            let (width, height) = if aspect_ratio > max_width / max_height {
                                // Width constrained
                                let width = max_width;
                                let height = width / aspect_ratio;
                                (width, height)
                            } else {
                                // Height constrained
                                let height = max_height;
                                let width = height * aspect_ratio;
                                (width, height)
                            };
                            
                            let rect = eframe::egui::Rect::from_center_size(
                                ui.available_rect_before_wrap().center(),
                                eframe::egui::vec2(width, height)
                            );
                            
                            ui.put(rect, eframe::egui::Image::new((texture.id(), eframe::egui::vec2(width, height))));
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.heading("Initializing upscaler...");
                                ui.label("Please wait while the upscaler initializes.");
                            });
                        }
                    }
                } else {
                    log::warn!("No upscaling buffer available");
                    ui.centered_and_justified(|ui| {
                        ui.heading("No upscaling buffer");
                        ui.label("Upscaling mode is active but no frame buffer is available.");
                        ui.label("This is likely a bug in the application.");
                    });
                }
            });
        
        // Use smarter repaint strategy based on frame state and upscaling status
        let repaint_after = if self.upscale_in_progress.load(Ordering::SeqCst) {
            // If upscaling is in progress, check more often
            Duration::from_millis(8)  // ~120 fps check rate 
        } else if self.frames_processed < 5 {
            // During startup, check frequently
            Duration::from_millis(16) // ~60 fps
        } else {
            // Normal operation, based on target FPS
            let target_fps = self.profile.fps.max(30.0);
            Duration::from_secs_f32(1.0 / target_fps)
        };
        
        ctx.request_repaint_after(repaint_after);
    }
    
    /// Schedule the next upscale operation on the thread pool
    fn schedule_next_upscale(&mut self, frame: Arc<FrameBuffer>) {
        // Clone necessary data to avoid borrowing issues
        let upscale_sender = self.upscaled_frame_sender.clone();
        let upscale_in_progress = self.upscale_in_progress.clone();
        let pending_frame = self.pending_upscaled_frame.clone();
        let profile = self.profile.clone();
        
        // Set the upscale in progress flag
        self.upscale_in_progress.store(true, Ordering::SeqCst);
        self.last_upscale_request = Some(Instant::now());
        self.upscale_start_time = Some(Instant::now());
        
        // Queue the upscaling work in the thread pool
        self.upscale_threadpool.execute(move || {
            // FIXED: Get the quality based on index, not by enum matching
            // Assuming ProfileUpscalingQuality is just an enum with numeric indices
            let quality = match profile.upscaling_quality {
                0 => crate::upscale::UpscalingQuality::Quality,
                1 => crate::upscale::UpscalingQuality::Balanced,
                2 => crate::upscale::UpscalingQuality::Performance,
                _ => crate::upscale::UpscalingQuality::Balanced, // Default
            };
            
            // FIXED: Match on technology index
            let result = match profile.upscaling_tech {
                0 => { // FSR
                    if let Ok(upscaler) = crate::upscale::fsr::FsrUpscaler::new(quality) {
                        // FIXED: Convert FrameBuffer to RgbaImage
                        match frame_buffer_to_rgba_image(&frame) {
                            Ok(image) => upscaler.upscale(&image),
                            Err(e) => Err(e),
                        }
                    } else {
                        Err(anyhow::anyhow!("Failed to create FSR upscaler"))
                    }
                },
                1 => { // DLSS
                    if let Ok(upscaler) = crate::upscale::dlss::DlssUpscaler::new(quality) {
                        // FIXED: Convert FrameBuffer to RgbaImage
                        match frame_buffer_to_rgba_image(&frame) {
                            Ok(image) => upscaler.upscale(&image),
                            Err(e) => Err(e),
                        }
                    } else {
                        Err(anyhow::anyhow!("Failed to create DLSS upscaler"))
                    }
                },
                // Add other technologies as needed
                _ => {
                    // Default to simple scaling
                    Err(anyhow::anyhow!("Unsupported upscaling technology"))
                }
            };
            
            // Process the result
            match result {
                Ok(upscaled_frame) => {
                    // Send upscaled frame back to UI thread
                    let _ = upscale_sender.send(upscaled_frame);
                },
                Err(err) => {
                    // Log the error
                    log::error!("Upscaling failed: {}", err);
                }
            }
            
            // Mark upscaling as complete
            upscale_in_progress.store(false, Ordering::SeqCst);
            
            // Clear the pending frame
            if let Ok(mut pending) = pending_frame.lock() {
                *pending = None;
            }
        });
    }
    
    /// Render a frame to the screen with proper texture management
    fn render_frame_to_screen(&mut self, ui: &mut Ui, available_size: Vec2, size: [usize; 2], pixels: &[u8]) {
        // Use our texture cache instead of storing directly in self.frame_texture
        let texture = self.texture_cache.get_texture(
            ui.ctx(), 
            (size[0] as u32, size[1] as u32), 
            pixels
        );
        
        // Store the texture for reference (only valid until next time this is called)
        self.frame_texture = Some(texture.clone());
        
        // Calculate dimensions to fit the screen
        let aspect_ratio = size[0] as f32 / size[1] as f32;
        let max_width = available_size.x;
        let max_height = available_size.y;
        
        let (width, height) = if aspect_ratio > max_width / max_height {
            (max_width, max_width / aspect_ratio)
        } else {
            (max_height * aspect_ratio, max_height)
        };
        
        // Create a centered rectangle
        let rect = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2((max_width - width) * 0.5, (max_height - height) * 0.5),
            eframe::egui::vec2(width, height)
        );
        
        log::debug!("Drawing image at rect: pos=({}, {}), size={}x{}", 
                 rect.min.x, rect.min.y, rect.width(), rect.height());
        
        // Draw the image
        // Use the TextureHandle directly with egui::Image::new
        let img_widget = egui::Image::new(&texture); // Pass TextureHandle directly
        ui.put(rect, img_widget);
        
        // Display performance counter in the corner if enabled
        if self.settings.show_fps_counter {
            let memory_text = if self.gpu_memory_warning {
                format!("| MEM: HIGH!")
            } else {
                format!("| MEM: {:.1} MB", self.texture_cache.get_memory_usage_mb())
            };
            
            ui.put(
                eframe::egui::Rect::from_min_size(
                    eframe::egui::pos2(10.0, 10.0),
                    eframe::egui::vec2(200.0, 20.0)
                ),
                eframe::egui::Label::new(
                    eframe::egui::RichText::new(format!("Frame: {} | FPS: {:.1} {}", 
                                                     self.frames_processed,
                                                     self.calculate_current_fps(),
                                                     memory_text))
                        .size(14.0)
                        .color(if self.gpu_memory_warning {
                            eframe::egui::Color32::RED
                        } else {
                            eframe::egui::Color32::GREEN
                        })
                )
            );
        }
        
        // Check GPU memory status
        self.check_gpu_memory(ui.ctx());
    }
    
    /// Calculate current FPS
    fn calculate_current_fps(&self) -> f32 {
        if let Some(last_request) = self.last_upscale_request {
            let elapsed = last_request.elapsed();
            if elapsed.as_secs_f32() > 0.0 {
                1.0 / elapsed.as_secs_f32()
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
    
    /// Launch the fullscreen upscaling mode with current profile settings
    fn launch_fullscreen_mode(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        log::info!("Entering fullscreen mode via eframe command");
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(true)); // Use ViewportCommand
        self.is_fullscreen = true;
        // Fix: Access profile fields directly, add TODO for mapping usize to enum/string
        // TODO: Map self.profile.upscaling_tech (usize) to expected enum/string for map_tech
        let upscaling_tech = self.map_tech(&ProfileUpscalingTechnology::Fallback); // Placeholder mapping
        // TODO: Map self.profile.upscaling_quality (usize) to expected enum/string for map_quality
        let upscaling_quality = self.map_quality(&ProfileUpscalingQuality::Balanced); // Placeholder mapping
        // let upscaling_tech = self.map_tech(&self.profile.upscaling_tech); // Original attempt assuming direct access works
        // let upscaling_quality = self.map_quality(&self.profile.upscaling_quality); // Original attempt

        let title = format!(
            "NU_Scaler - Upscaling with {:?} at {:?} quality",
            upscaling_tech, upscaling_quality
        );
        ctx.send_viewport_cmd(ViewportCommand::Title(title)); // Use ViewportCommand
        self.status_bar.set_message("Fullscreen mode activated".to_string(), StatusMessageType::Success);
        // NOTE: The original logic spawning a separate process is removed.
        // If separate process is desired, reinstate that logic and remove ViewportCommand::Fullscreen.
    }

    /// Sets the title of the window to capture in the profile and updates the window title.
    fn set_capture_target_window_title(&mut self, ctx: &Context, title: &str) {
        // Fix: Access profile field directly
        self.profile.window_title = title.to_string();
        log::info!("Set capture target window title to: '{}'", title);
        let display_title = if title.is_empty() {
            "NU Scale".to_string()
        } else {
            format!("NU Scale - Capturing: {}", title)
        };
        ctx.send_viewport_cmd(ViewportCommand::Title(display_title)); // Ensure ctx is passed and used
        if self.is_capturing {
            log::warn!("Capture target changed while capturing is active. Restart capture to apply.");
        }
    }

    // ... show_source_window_list needs ctx passed to set_capture_target_window_title ...
    fn show_source_window_list(&mut self, ctx: &Context, ui: &mut Ui) {
         if ui.button("Set Capture Window").clicked() {
             if let Some(name) = self.available_windows.get(self.selected_window_index) {
                 let name = name.clone();
                 self.set_capture_target_window_title(ctx, &name);
             }
         }
         if ui.button("Reset Window").clicked() {
             self.set_capture_target_window_title(ctx, "");
         }
         // Fix borrow checker: clone window names before the loop
         let window_names: Vec<String> = self.available_windows.clone();
         for (index, name) in window_names.iter().enumerate() {
             let response = ui.selectable_label(self.selected_window_index == index, name);
             if response.clicked() {
                 self.selected_window_index = index;
                 let name = name.clone();
                 self.set_capture_target_window_title(ctx, &name);
             }
             if response.double_clicked() {
                 self.selected_window_index = index;
                 let name = name.clone();
                 self.set_capture_target_window_title(ctx, &name);
             }
         }
     }

    /// Start a scaling process in a separate application instance
    fn start_scaling_process(&mut self) -> Result<()> {
        // First, kill any existing scaling process
        self.kill_scaling_process();

        // Get the capture source string
        let source_str = match self.profile.capture_source {
            0 => "fullscreen".to_string(),
            1 => format!("window:{}", self.profile.window_title),
            2 => format!("region:{},{},{},{}", 
                self.profile.region_x, 
                self.profile.region_y, 
                self.profile.region_width, 
                self.profile.region_height
            ),
            _ => "fullscreen".to_string(), // Default fallback
        };

        // Map tech to string
        let tech_str = match self.profile.upscaling_tech {
            0 => "fsr",        // Auto defaults to FSR
            1 => "fsr",        // FSR
            2 => "dlss",       // DLSS
            3 => "gpu",        // Changed from cuda
            4 => "fallback",   // Fallback
            _ => "fallback",
        };

        // Map quality to string
        let quality_str = match self.profile.upscaling_quality {
            0 => "ultra",
            1 => "quality",
            2 => "balanced",
            3 => "performance",
            _ => "balanced",
        };

        // Get algorithm string if needed
        // Only add if Fallback or GPU is selected (index 3 or 4)
        let mut alg_arg = Vec::new();
        if self.profile.upscaling_tech == 3 || self.profile.upscaling_tech == 4 {
            let alg_str = match self.profile.upscaling_algorithm {
                0 => "lanczos3",
                1 => "nearest",
                2 => "bilinear",
                3 => "bicubic",
                _ => "lanczos3",
            };
            alg_arg.push("--algorithm");
            alg_arg.push(alg_str);
        }

        // Get current executable path
        let exe_path = std::env::current_exe().unwrap_or_else(|_| {
            log::error!("Failed to get current executable path");
            PathBuf::from("nu_scaler.exe")
        });

        // Build and start the process
        let mut cmd = std::process::Command::new(exe_path);
        cmd.arg("fullscreen")
           .arg("--source").arg(&source_str)
           .arg("--tech").arg(tech_str)
           .arg("--quality").arg(quality_str)
           .arg("--fps").arg(self.profile.fps.to_string());

        // Add algorithm argument if needed
        if !alg_arg.is_empty() {
            cmd.arg(alg_arg[0]).arg(alg_arg[1]);
        }

        // Run the process
        match cmd.spawn() {
            Ok(child) => {
                log::info!("Started scaling process with PID: {}", child.id());
                self.scaling_process = Some(child);
                self.status_message = "Scaling started in separate window.".to_string();
                self.status_message_type = StatusMessageType::Success;
                Ok(()) // Fix: Add Ok(())
            },
            Err(e) => {
                log::error!("Failed to start scaling process: {}", e);
                self.status_message = format!("Failed to start scaling: {}", e);
                self.status_message_type = StatusMessageType::Error;
                Err(e.into()) // Fix: Return Err
            }
        }
    }

    /// Kill an existing scaling process if one exists
    fn kill_scaling_process(&mut self) {
        if let Some(mut child) = self.scaling_process.take() {
            log::info!("Killing existing scaling process");
            
            // Try to kill the process gracefully
            #[cfg(windows)]
            {
                // On Windows, we call the Win32 API to try to send WM_CLOSE first
                let process_id = child.id();
                unsafe {
                    let handle = HANDLE(process_id as isize);
                    let _ = TerminateProcess(handle, 0);
                }
            }

            // Fallback to kill() if terminate isn't available or didn't work
            match child.kill() {
                Ok(_) => {
                    let _ = child.wait(); // Clean up zombie process
                    log::info!("Successfully killed scaling process");
                },
                Err(e) => {
                    log::warn!("Failed to kill scaling process: {}", e);
                    // Process might have already terminated
                }
            }
        }
    }

    // Map profile UpscalingTechnology to library UpscalingTechnology
    fn map_tech(&self, tech: &ProfileUpscalingTechnology) -> UpscalingTechnology {
        match tech {
            ProfileUpscalingTechnology::None => UpscalingTechnology::Fallback,
            ProfileUpscalingTechnology::FSR => UpscalingTechnology::FSR,
            ProfileUpscalingTechnology::DLSS => UpscalingTechnology::DLSS,
            ProfileUpscalingTechnology::Fallback => UpscalingTechnology::Fallback,
            ProfileUpscalingTechnology::Custom => UpscalingTechnology::Fallback,
        }
    }

    // Map profile UpscalingQuality to library UpscalingQuality
    fn map_quality(&self, quality: &ProfileUpscalingQuality) -> UpscalingQuality {
        match quality {
            ProfileUpscalingQuality::Ultra => UpscalingQuality::Ultra,
            ProfileUpscalingQuality::Quality => UpscalingQuality::Quality,
            ProfileUpscalingQuality::Balanced => UpscalingQuality::Balanced,
            ProfileUpscalingQuality::Performance => UpscalingQuality::Performance,
        }
    }

    /// Get the window under the cursor
    fn get_window_under_cursor(&self) -> Option<String> {
        if let Ok(capturer) = crate::capture::create_capturer() {
            if let Ok(windows) = capturer.list_windows() {
                // Get the current cursor position
                #[cfg(target_os = "windows")]
                let cursor_pos = unsafe {
                    use windows::Win32::Foundation::POINT;
                    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
                    
                    let mut point = POINT::default();
                    if GetCursorPos(&mut point).as_bool() {
                        Some((point.x, point.y))
                    } else {
                        None
                    }
                };
                
                #[cfg(not(target_os = "windows"))]
                let cursor_pos = None;
                
                if let Some((x, y)) = cursor_pos {
                    // Find the window at the cursor position
                    for window in &windows {
                        let geom = window.geometry;
                        if x >= geom.x && 
                           y >= geom.y && 
                           x < geom.x + geom.width as i32 && 
                           y < geom.y + geom.height as i32 {
                            // Found the window under cursor
                            log::info!("Found window under cursor: {}", window.title);
                            return Some(window.title.clone());
                        }
                    }
                }
                
                // If we couldn't find by position, try to get the foreground window
                #[cfg(target_os = "windows")]
                {
                    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
                    
                    let hwnd = unsafe { GetForegroundWindow() };
                    for window in &windows {
                        if let crate::capture::platform::WindowId::Windows(id) = window.id {
                            if id == hwnd.0 as usize {
                                log::info!("Found active window: {}", window.title);
                                return Some(window.title.clone());
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Launch upscaling for the window under cursor (triggered by shortcut)
    fn upscale_window_under_cursor(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        log::info!("Upscaling window under cursor");
        
        // Get the window under cursor
        if let Some(window_title) = self.get_window_under_cursor() {
            // Fix: Clone window_title before pushing it to avoid move
            let window_title_clone = window_title.clone();
            
            // Update the profile to use this window
            self.profile.capture_source = 1; // Window mode
            self.profile.window_title = window_title.clone();
            
            // Find the window in our list and update the selection
            for (i, title) in self.available_windows.iter().enumerate() {
                if title == &window_title {
                    self.selected_window_index = i;
                    break;
                }
            }
            
            // If not found, add it to the list
            if !self.available_windows.contains(&window_title) {
                self.available_windows.push(window_title_clone);
                self.selected_window_index = self.available_windows.len() - 1;
            }
            
            // Launch fullscreen upscaling for this window
            log::info!("Launching fullscreen upscaling for selected window: {}", window_title);
            self.launch_fullscreen_mode(ctx, frame);
        } else {
            log::warn!("No window found under cursor");
            self.status_message = "No window found under cursor".to_string();
            self.status_message_type = StatusMessageType::Error;
        }
    }

    /// Check GPU memory pressure
    fn check_gpu_memory(&mut self, ctx: &egui::Context) -> bool {
        // Check memory pressure and clean up if needed
        if Instant::now().duration_since(self.last_memory_check.unwrap_or_else(Instant::now)) 
            > Duration::from_secs(5) 
        {
            self.last_memory_check = Some(Instant::now());
            
            // Pass ctx for cleanup
            self.texture_cache.cleanup_old_textures(ctx);
            
            // Check GPU memory usage
            let memory_usage_mb = self.texture_cache.get_memory_usage_mb();
            if memory_usage_mb > 500.0 {
                self.gpu_memory_warning = true;
                return true;
            }
        }
        
        self.gpu_memory_warning = false;
        false
    }

    fn process_input(&mut self, ctx: &egui::Context) {
        // Fix input_mut usage and scope issues
        ctx.input(|input| {
            // Space key to toggle upscaling
            if input.key_pressed(egui::Key::Space) {
                self.toggle_upscaling();
            }
            
            // Process drag and drop - fix scope issue
            if !input.raw.dropped_files.is_empty() {
                let file = &input.raw.dropped_files[0];
                if let Some(path) = &file.path {
                    log::info!("File dropped: {:?}", path);
                    self.load_image(path);
                }
            }
        });
    }

    fn check_gpu_memory_pressure(&mut self, ctx: &egui::Context) {
        // Check if we need to clean up GPU memory
        if self.texture_cache.get_memory_usage_mb() > 1000.0 {
            // Fix: Pass ctx instead of Duration
            self.texture_cache.cleanup_old_textures(ctx);
            log::info!("Cleaned up GPU memory. Current usage: {} MB", 
                      self.texture_cache.get_memory_usage_mb());
        }
    }
    
    fn load_image(&mut self, path: &Path) {
        match image::open(path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let width = rgba.width() as usize;
                let height = rgba.height() as usize;
                let _raw_data = rgba.into_raw(); // Prefixed raw_data
                log::info!("Loaded image {} ({}x{})", path.display(), width, height);
                // TODO: Use raw_data
            }
            Err(e) => {
                self.status_message = format!("Failed to load image: {}", e);
                self.status_message_type = StatusMessageType::Error;
            }
        }
    }
    
    fn toggle_upscaling(&mut self) {
        self.auto_upscale = !self.auto_upscale;
        log::info!("Auto upscaling: {}", self.auto_upscale);
        
        // Fix: Check pending frame properly
        if self.auto_upscale && !self.is_upscaling {
            let has_pending = match self.pending_upscaled_frame.lock() {
                Ok(guard) => guard.is_some(),
                Err(_) => false,
            };
            
            if !self.upscale_in_progress.load(Ordering::SeqCst) && !has_pending {
                if let Some(frame) = &self.current_frame {
                    // Clone to avoid borrow checker issues
                    let frame_clone = frame.clone();
                    self.schedule_next_upscale(frame_clone);
                }
            }
        }
    }
    
    fn should_reload_textures(&self) -> bool {
        // Check timing and state to determine if textures should be reloaded
        if let Some(last_request) = self.last_upscale_request {
            if last_request.elapsed() > Duration::from_secs(5) {
                // Only reload if we're not currently upscaling
                let in_progress = self.upscale_in_progress.load(Ordering::SeqCst);
                
                // Fix: Check for pending frame properly
                let has_pending = match self.pending_upscaled_frame.lock() {
                    Ok(guard) => guard.is_some(),
                    Err(_) => false,
                };
                
                return !in_progress && !has_pending;
            }
        }
        
        false
    }
}

// Add a cleanup function to ensure we kill the scaling process on exit
impl Drop for AppState {
    fn drop(&mut self) {
        self.kill_scaling_process();
    }
}

// Implement Drop trait to ensure texture cleanup on exit
impl Drop for TextureCache {
    fn drop(&mut self) {
        log::info!("Cleaning up texture cache: {} textures, {:.1} MB", 
                 self.textures.len(), self.get_memory_usage_mb());
    }
}

/// Run the egui application
pub fn run_app() -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_transparent(false), // Changed from true to false
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "NU Scaler", 
        native_options,
        Box::new(|cc| {
            // Use default egui fonts (no custom font loading)
            Box::new(AppState::default())
        }),
    )
    .map_err(|e| anyhow!("Application failed: {}", e))?;

    Ok(())
} 

// Fix the duplicate helper function with a proper implementation
fn frame_buffer_to_rgba_image(frame: &Arc<FrameBuffer>) -> Result<image::RgbaImage> {
    // We need to extract the width, height, and data from the FrameBuffer
    // First, get the frame data - adjust these calls based on your FrameBuffer API
    let frame_data = match frame.get_latest_frame() {
        Ok(data) => data,
        Err(e) => return Err(anyhow::anyhow!("Failed to get latest frame: {}", e)),
    };
    
    // The frame_data is an Option, so unwrap it
    let image = match frame_data {
        Some(img) => img,
        None => return Err(anyhow::anyhow!("No frame data available")),
    };
    
    // Now access properties on the actual image
    let width = image.width();
    let height = image.height();
    
    if width == 0 || height == 0 {
        return Err(anyhow::anyhow!("Invalid frame dimensions"));
    }
    
    // Get raw pixel data - image is already an RgbaImage, so just clone it
    Ok(image.clone())
}
