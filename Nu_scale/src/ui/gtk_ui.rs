use anyhow::{Result, anyhow};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Builder, Button, ComboBoxText, 
          Entry, Label, Scale, Switch, Box as GtkBox, Orientation};
use glib::clone;
use std::{
    path::{PathBuf, Path},
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    thread,
    time::{Duration, Instant},
    rc::Rc,
    cell::RefCell,
};

use crate::capture::{CaptureTarget, ScreenCapture};
use crate::capture::common::{FrameBuffer};
use crate::upscale::{
    create_upscaler, Upscaler, UpscalingQuality, UpscalingTechnology,
    common::UpscalingAlgorithm,
};

// Import profile and settings modules
use super::profile::{Profile, UpscalingTechnology as ProfileUpscalingTechnology, UpscalingQuality as ProfileUpscalingQuality};
use super::settings::AppSettings;

// Define message types for UI updates
enum UIMessage {
    FrameCaptured(Arc<FrameBuffer>),
    StatusUpdate(String, MessageType),
    WindowListUpdated(Vec<String>),
    UpscalingStarted,
    UpscalingFinished,
    Error(String),
}

// Define status message types
#[derive(Clone, Copy, PartialEq)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

// Application state structure
pub struct AppState {
    // Core application state
    profile: Profile,
    settings: AppSettings,
    is_capturing: Arc<AtomicBool>,
    is_upscaling: Arc<AtomicBool>,
    
    // Window properties
    available_windows: Vec<String>,
    selected_window_index: usize,
    capture_source_index: usize,
    region: (i32, i32, u32, u32),
    
    // GTK UI elements
    window: ApplicationWindow,
    status_label: Label,
    capture_button: Button,
    scale_button: Button,
    window_combo: ComboBoxText,
    tech_combo: ComboBoxText,
    quality_combo: ComboBoxText,
    algorithm_combo: ComboBoxText,
    scale_factor_scale: Scale,
    fps_scale: Scale,
    
    // Capture and processing
    frame_buffer: Arc<FrameBuffer>,
    stop_signal: Arc<AtomicBool>,
    capture_status: Arc<Mutex<String>>,
    
    // Upscaling
    upscaling_buffer: Option<Arc<FrameBuffer>>,
    upscaling_stop_signal: Option<Arc<AtomicBool>>,
    upscaler: Option<Box<dyn Upscaler>>,
    
    // UI State
    current_tab: TabState,
}

// Tab state enum
#[derive(Clone, Copy, PartialEq)]
pub enum TabState {
    Capture,
    Settings,
    Advanced,
}

impl AppState {
    // Create a new application state with GTK window
    pub fn new(app: &Application) -> Result<Self> {
        println!("AppState::new() called");
        
        // Load settings and profile
        let settings = AppSettings::load().unwrap_or_default();
        let profile_path = format!("{}.json", settings.current_profile);
        let profile = Profile::load(&profile_path).unwrap_or_default();
        
        // Determine capture source index and region from profile
        let capture_source_index = profile.capture_source;
        let region = (
            profile.region_x,
            profile.region_y,
            profile.region_width,
            profile.region_height
        );
        
        // Get available windows
        let available_windows = crate::capture::common::list_available_windows()
            .map(|windows| windows.iter().map(|w| w.title.clone()).collect())
            .unwrap_or_default();
        
        // Create frame buffer and signals
        let frame_buffer = Arc::new(FrameBuffer::new(10));
        let stop_signal = Arc::new(AtomicBool::new(false));
        let capture_status = Arc::new(Mutex::new("Idle".to_string()));
        
        // Create the main window
        let window = ApplicationWindow::new(app);
        window.set_title("NU Scaler");
        window.set_default_size(1200, 800);
        
        // Create basic UI components
        let status_label = Label::new(Some("Ready"));
        let capture_button = Button::with_label("Start Capture");
        let scale_button = Button::with_label("Scale");
        
        // Create dropdown components
        let window_combo = ComboBoxText::new();
        let tech_combo = ComboBoxText::new();
        let quality_combo = ComboBoxText::new();
        let algorithm_combo = ComboBoxText::new();
        
        // Create slider components
        let scale_factor_scale = Scale::with_range(Orientation::Horizontal, 1.0, 4.0, 0.1);
        let fps_scale = Scale::with_range(Orientation::Horizontal, 1.0, 240.0, 1.0);
        
        // Initialize sliders with profile values
        scale_factor_scale.set_value(profile.scale_factor as f64);
        fps_scale.set_value(profile.fps as f64);
        
        // Setup dropdowns
        for window_name in &available_windows {
            window_combo.append_text(window_name);
        }
        
        // Upscaling technology options
        tech_combo.append_text("Auto");
        tech_combo.append_text("AMD FSR");
        tech_combo.append_text("NVIDIA DLSS");
        tech_combo.append_text("GPU (Vulkan)");
        tech_combo.append_text("Fallback/Basic");
        tech_combo.set_active(Some(profile.upscaling_tech as u32));
        
        // Upscaling quality options
        quality_combo.append_text("Ultra Quality");
        quality_combo.append_text("Quality");
        quality_combo.append_text("Balanced");
        quality_combo.append_text("Performance");
        quality_combo.set_active(Some(profile.upscaling_quality as u32));
        
        // Scaling algorithm options
        algorithm_combo.append_text("Lanczos (a=3)");
        algorithm_combo.append_text("Bicubic");
        algorithm_combo.append_text("Bilinear");
        algorithm_combo.append_text("Nearest-Neighbor");
        algorithm_combo.set_active(Some(profile.upscaling_algorithm as u32));
        
        // Create the app state
        let app_state = AppState {
            profile,
            settings,
            is_capturing: Arc::new(AtomicBool::new(false)),
            is_upscaling: Arc::new(AtomicBool::new(false)),
            available_windows,
            selected_window_index: 0,
            capture_source_index,
            region,
            window,
            status_label,
            capture_button,
            scale_button,
            window_combo,
            tech_combo,
            quality_combo,
            algorithm_combo,
            scale_factor_scale,
            fps_scale,
            frame_buffer,
            stop_signal,
            capture_status,
            upscaling_buffer: None,
            upscaling_stop_signal: None,
            upscaler: None,
            current_tab: TabState::Capture,
        };
        
        Ok(app_state)
    }
    
    // Build the UI layout
    pub fn build_ui(&self) {
        // Create the main layout
        let main_box = GtkBox::new(Orientation::Vertical, 10);
        
        // Add the header bar
        main_box.append(&self.build_header_bar());
        
        // Create horizontal layout for sidebar and content
        let content_box = GtkBox::new(Orientation::Horizontal, 10);
        
        // Add sidebar
        content_box.append(&self.build_sidebar());
        
        // Add content area (initially showing capture tab)
        let tab_content = self.build_capture_tab();
        content_box.append(&tab_content);
        
        // Add content box to main layout
        main_box.append(&content_box);
        
        // Add status bar at the bottom
        main_box.append(&self.build_status_bar());
        
        // Add the main layout to the window
        self.window.set_child(Some(&main_box));
    }
    
    // Build the header bar
    fn build_header_bar(&self) -> GtkBox {
        let header_box = GtkBox::new(Orientation::Horizontal, 10);
        
        // Add title/logo
        let title_label = Label::new(Some("NU Scale"));
        title_label.set_markup("<span size='x-large' weight='bold'>NU Scale</span>");
        header_box.append(&title_label);
        
        // Add save profile button
        let save_button = Button::with_label("💾 Save Profile");
        header_box.append(&save_button);
        
        // Add new profile button
        let new_profile_button = Button::with_label("+ New Profile");
        header_box.append(&new_profile_button);
        
        // Add capture buttons on the right
        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header_box.append(&spacer);
        
        // Add buttons
        header_box.append(&self.capture_button.clone());
        
        let capture_frame_button = Button::with_label("📷 Capture Frame");
        header_box.append(&capture_frame_button);
        
        // Add fullscreen and scale buttons
        let fullscreen_button = Button::with_label("🖥️ Fullscreen Mode");
        header_box.append(&fullscreen_button);
        
        header_box.append(&self.scale_button.clone());
        
        header_box.set_margin_all(10);
        
        header_box
    }
    
    // Build the sidebar
    fn build_sidebar(&self) -> GtkBox {
        let sidebar_box = GtkBox::new(Orientation::Vertical, 10);
        sidebar_box.set_margin_all(10);
        sidebar_box.set_width_request(200);
        
        // Add navigation heading
        let nav_label = Label::new(Some("Navigation"));
        nav_label.set_margin_top(20);
        nav_label.set_margin_bottom(10);
        sidebar_box.append(&nav_label);
        
        // Add separator
        let separator = gtk::Separator::new(Orientation::Horizontal);
        sidebar_box.append(&separator);
        
        // Add navigation buttons
        let capture_button = Button::with_label("📷 Capture");
        capture_button.set_margin_top(10);
        sidebar_box.append(&capture_button);
        
        let settings_button = Button::with_label("⚙️ Settings");
        settings_button.set_margin_top(5);
        sidebar_box.append(&settings_button);
        
        let advanced_button = Button::with_label("🔧 Advanced");
        advanced_button.set_margin_top(5);
        sidebar_box.append(&advanced_button);
        
        // Add version info at bottom
        let version_box = GtkBox::new(Orientation::Vertical, 5);
        version_box.set_margin_top(20);
        version_box.set_valign(gtk::Align::End);
        version_box.set_vexpand(true);
        
        let version_label = Label::new(Some("NU Scale"));
        version_box.append(&version_label);
        
        let version_number = Label::new(Some("v1.0.0"));
        version_box.append(&version_number);
        
        sidebar_box.append(&version_box);
        
        sidebar_box
    }
    
    // Build the capture tab
    fn build_capture_tab(&self) -> GtkBox {
        let content_box = GtkBox::new(Orientation::Vertical, 10);
        content_box.set_margin_all(10);
        content_box.set_hexpand(true);
        
        // Add heading
        let heading = Label::new(Some("Capture Settings"));
        heading.set_margin_bottom(16);
        heading.set_halign(gtk::Align::Start);
        content_box.append(&heading);
        
        // Profile selection card
        let profile_frame = gtk::Frame::new(Some("Profile Selection"));
        let profile_box = GtkBox::new(Orientation::Vertical, 10);
        profile_box.set_margin_all(16);
        
        // Profile selection row
        let profile_row = GtkBox::new(Orientation::Horizontal, 10);
        let profile_label = Label::new(Some("Current Profile:"));
        profile_row.append(&profile_label);
        
        let profile_combo = ComboBoxText::new();
        for profile_name in &super::profile::Profile::list_profiles().unwrap_or_default() {
            profile_combo.append_text(profile_name);
        }
        profile_combo.set_active_id(Some(&self.profile.name));
        profile_row.append(&profile_combo);
        
        profile_box.append(&profile_row);
        
        // Profile buttons row
        let buttons_row = GtkBox::new(Orientation::Horizontal, 10);
        let save_button = Button::with_label("Save");
        let new_button = Button::with_label("📋 New");
        let delete_button = Button::with_label("❌ Delete");
        
        buttons_row.append(&save_button);
        buttons_row.append(&new_button);
        buttons_row.append(&delete_button);
        
        profile_box.append(&buttons_row);
        profile_frame.set_child(Some(&profile_box));
        
        content_box.append(&profile_frame);
        
        // Capture source card
        let capture_frame = gtk::Frame::new(Some("Capture Source"));
        let capture_box = GtkBox::new(Orientation::Vertical, 10);
        capture_box.set_margin_all(16);
        
        // Fullscreen option
        let fullscreen_row = GtkBox::new(Orientation::Horizontal, 10);
        let fullscreen_radio = gtk::CheckButton::with_label("🖥️ Fullscreen");
        fullscreen_row.append(&fullscreen_radio);
        capture_box.append(&fullscreen_row);
        
        // Window option
        let window_row = GtkBox::new(Orientation::Horizontal, 10);
        let window_radio = gtk::CheckButton::with_label("🪟 Window");
        window_radio.set_group(Some(&fullscreen_radio));
        window_row.append(&window_radio);
        
        // Window dropdown
        let window_dropdown = self.window_combo.clone();
        window_dropdown.set_margin_start(16);
        window_row.append(&window_dropdown);
        
        // Refresh button
        let refresh_button = Button::with_label("🔄 Refresh");
        window_row.append(&refresh_button);
        
        capture_box.append(&window_row);
        
        // Region option
        let region_row = GtkBox::new(Orientation::Horizontal, 10);
        let region_radio = gtk::CheckButton::with_label("📏 Region");
        region_radio.set_group(Some(&fullscreen_radio));
        region_row.append(&region_radio);
        
        // Region selection button
        let select_region_button = Button::with_label("Select Region");
        select_region_button.set_margin_start(16);
        region_row.append(&select_region_button);
        
        // Region coordinates
        let region_coords_label = Label::new(Some(
            &format!("({}, {}, {}x{})", 
                    self.region.0, self.region.1, self.region.2, self.region.3)
        ));
        region_coords_label.set_margin_start(8);
        region_row.append(&region_coords_label);
        
        capture_box.append(&region_row);
        
        // Set initial active radio button based on capture_source_index
        match self.capture_source_index {
            0 => fullscreen_radio.set_active(true),
            1 => window_radio.set_active(true),
            2 => region_radio.set_active(true),
            _ => fullscreen_radio.set_active(true),
        }
        
        capture_frame.set_child(Some(&capture_box));
        content_box.append(&capture_frame);
        
        content_box
    }
    
    // Build the settings tab
    fn build_settings_tab(&self) -> GtkBox {
        let content_box = GtkBox::new(Orientation::Vertical, 10);
        content_box.set_margin_all(10);
        content_box.set_hexpand(true);
        
        // Add heading
        let heading = Label::new(Some("Settings"));
        heading.set_margin_bottom(16);
        heading.set_halign(gtk::Align::Start);
        content_box.append(&heading);
        
        // Upscaling settings card
        let upscaling_frame = gtk::Frame::new(Some("Upscaling Settings"));
        let upscaling_box = GtkBox::new(Orientation::Vertical, 10);
        upscaling_box.set_margin_all(16);
        
        // Scale factor row
        let scale_factor_row = GtkBox::new(Orientation::Horizontal, 10);
        let scale_factor_label = Label::new(Some("Scale Factor:"));
        scale_factor_row.append(&scale_factor_label);
        
        let scale_factor_slider = self.scale_factor_scale.clone();
        scale_factor_slider.set_hexpand(true);
        scale_factor_row.append(&scale_factor_slider);
        
        let scale_factor_value = Label::new(Some(&format!("{:.1}×", self.profile.scale_factor)));
        scale_factor_row.append(&scale_factor_value);
        
        upscaling_box.append(&scale_factor_row);
        
        // Upscaling technology row
        let tech_row = GtkBox::new(Orientation::Horizontal, 10);
        let tech_label = Label::new(Some("Upscaling Technology:"));
        tech_row.append(&tech_label);
        
        let tech_combo = self.tech_combo.clone();
        tech_combo.set_hexpand(true);
        tech_row.append(&tech_combo);
        
        upscaling_box.append(&tech_row);
        
        // Upscaling quality row
        let quality_row = GtkBox::new(Orientation::Horizontal, 10);
        let quality_label = Label::new(Some("Upscaling Quality:"));
        quality_row.append(&quality_label);
        
        let quality_combo = self.quality_combo.clone();
        quality_combo.set_hexpand(true);
        quality_row.append(&quality_combo);
        
        upscaling_box.append(&quality_row);
        
        // Upscaling algorithm row (only shown for GPU or Fallback)
        let algorithm_row = GtkBox::new(Orientation::Horizontal, 10);
        let algorithm_label = Label::new(Some("Upscaling Algorithm:"));
        algorithm_row.append(&algorithm_label);
        
        let algorithm_combo = self.algorithm_combo.clone();
        algorithm_combo.set_hexpand(true);
        algorithm_row.append(&algorithm_combo);
        
        upscaling_box.append(&algorithm_row);
        
        // Algorithm description
        let description_row = GtkBox::new(Orientation::Horizontal, 10);
        let description_label = Label::new(Some(""));
        description_label.set_margin_start(138);
        description_row.append(&description_label);
        
        upscaling_box.append(&description_row);
        
        upscaling_frame.set_child(Some(&upscaling_box));
        content_box.append(&upscaling_frame);
        
        // FPS settings card
        let fps_frame = gtk::Frame::new(Some("Capture FPS"));
        let fps_box = GtkBox::new(Orientation::Vertical, 10);
        fps_box.set_margin_all(16);
        
        // FPS row
        let fps_row = GtkBox::new(Orientation::Horizontal, 10);
        let fps_label = Label::new(Some("Target FPS:"));
        fps_row.append(&fps_label);
        
        let fps_slider = self.fps_scale.clone();
        fps_slider.set_hexpand(true);
        fps_row.append(&fps_slider);
        
        let fps_value = Label::new(Some(&format!("{} fps", self.profile.fps)));
        fps_row.append(&fps_value);
        
        fps_box.append(&fps_row);
        
        fps_frame.set_child(Some(&fps_box));
        content_box.append(&fps_frame);
        
        content_box
    }
    
    // Build the advanced tab
    fn build_advanced_tab(&self) -> GtkBox {
        let content_box = GtkBox::new(Orientation::Vertical, 10);
        content_box.set_margin_all(10);
        content_box.set_hexpand(true);
        
        // Add heading
        let heading = Label::new(Some("Advanced"));
        heading.set_margin_bottom(16);
        heading.set_halign(gtk::Align::Start);
        content_box.append(&heading);
        
        // Application settings card
        let app_settings_frame = gtk::Frame::new(Some("Application Settings"));
        let app_settings_box = GtkBox::new(Orientation::Vertical, 10);
        app_settings_box.set_margin_all(16);
        
        // Auto-save frames checkbox
        let auto_save_row = GtkBox::new(Orientation::Horizontal, 10);
        let auto_save_switch = Switch::new();
        auto_save_switch.set_active(self.settings.auto_save_frames);
        auto_save_row.append(&auto_save_switch);
        
        let auto_save_label = Label::new(Some("Auto-save Captured Frames"));
        auto_save_row.append(&auto_save_label);
        
        app_settings_box.append(&auto_save_row);
        
        // Show FPS counter checkbox
        let fps_counter_row = GtkBox::new(Orientation::Horizontal, 10);
        let fps_counter_switch = Switch::new();
        fps_counter_switch.set_active(self.settings.show_fps_counter);
        fps_counter_row.append(&fps_counter_switch);
        
        let fps_counter_label = Label::new(Some("Show FPS counter"));
        fps_counter_row.append(&fps_counter_label);
        
        app_settings_box.append(&fps_counter_row);
        
        // Show notifications checkbox
        let notifications_row = GtkBox::new(Orientation::Horizontal, 10);
        let notifications_switch = Switch::new();
        notifications_switch.set_active(self.settings.show_notifications);
        notifications_row.append(&notifications_switch);
        
        let notifications_label = Label::new(Some("Show notifications"));
        notifications_row.append(&notifications_label);
        
        app_settings_box.append(&notifications_row);
        
        // Theme selection
        let theme_row = GtkBox::new(Orientation::Horizontal, 10);
        let theme_label = Label::new(Some("Theme:"));
        theme_row.append(&theme_label);
        
        let theme_combo = ComboBoxText::new();
        theme_combo.append_text("Light");
        theme_combo.append_text("Dark");
        theme_combo.set_active_id(Some(&self.settings.theme));
        theme_row.append(&theme_combo);
        
        app_settings_box.append(&theme_row);
        
        app_settings_frame.set_child(Some(&app_settings_box));
        content_box.append(&app_settings_frame);
        
        // Advanced options card
        let advanced_frame = gtk::Frame::new(Some("Advanced Options"));
        let advanced_box = GtkBox::new(Orientation::Vertical, 10);
        advanced_box.set_margin_all(16);
        
        let advanced_label = Label::new(Some("Advanced settings will be available in future versions."));
        advanced_box.append(&advanced_label);
        
        advanced_frame.set_child(Some(&advanced_box));
        content_box.append(&advanced_frame);
        
        // Save settings button
        let save_settings_button = Button::with_label("Save Application Settings");
        save_settings_button.set_margin_top(10);
        content_box.append(&save_settings_button);
        
        content_box
    }
    
    // Build the status bar
    fn build_status_bar(&self) -> GtkBox {
        let status_box = GtkBox::new(Orientation::Horizontal, 10);
        status_box.set_margin_all(10);
        
        // Add status label
        status_box.append(&self.status_label.clone());
        
        // Add capturing indicator on right side
        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        status_box.append(&spacer);
        
        // This will be shown/hidden based on capture state
        let capturing_label = Label::new(Some("● CAPTURING"));
        capturing_label.set_visible(false); // Initially hidden
        status_box.append(&capturing_label);
        
        status_box
    }
    
    // Connect signals to UI elements
    pub fn connect_signals(&self) {
        // Start/stop capture button
        let is_capturing = self.is_capturing.clone();
        let status_label = self.status_label.clone();
        let capture_button = self.capture_button.clone();
        
        capture_button.connect_clicked(move |button| {
            let currently_capturing = is_capturing.load(Ordering::SeqCst);
            is_capturing.store(!currently_capturing, Ordering::SeqCst);
            
            if !currently_capturing {
                // Starting capture
                button.set_label("⏹ Stop Capture");
                status_label.set_text("Capture started");
            } else {
                // Stopping capture
                button.set_label("▶ Start Capture");
                status_label.set_text("Capture stopped");
            }
        });
        
        // Connect other signals (scale button, dropdowns, etc.) here
    }
    
    // Update the UI based on the current state
    pub fn update_ui(&self) {
        // Update UI elements based on current state
        let currently_capturing = self.is_capturing.load(Ordering::SeqCst);
        
        if currently_capturing {
            self.capture_button.set_label("⏹ Stop Capture");
        } else {
            self.capture_button.set_label("▶ Start Capture");
        }
        
        // Update other UI elements
    }
    
    // Set a status message
    pub fn set_status(&self, message: &str, message_type: MessageType) {
        self.status_label.set_text(message);
        
        // You could also set colors based on message type
        match message_type {
            MessageType::Info => {
                // Regular color
            }
            MessageType::Success => {
                // Green color
            }
            MessageType::Warning => {
                // Yellow/orange color
            }
            MessageType::Error => {
                // Red color
            }
        }
    }
}

/// Run the GTK application
pub fn run_app() -> Result<()> {
    println!("GTK run_app() started");
    
    // Create the GTK application
    let app = Application::builder()
        .application_id("com.nu_scaler.app")
        .build();
    
    // Connect to the activate signal
    app.connect_activate(|app| {
        // Create app state
        match AppState::new(app) {
            Ok(app_state) => {
                // Build UI
                app_state.build_ui();
                
                // Connect signals
                app_state.connect_signals();
                
                // Show the window
                app_state.window.present();
            }
            Err(e) => {
                eprintln!("Failed to create app state: {}", e);
                std::process::exit(1);
            }
        }
    });
    
    // Run the application
    let status = app.run();
    if status == 0 {
        Ok(())
    } else {
        Err(anyhow!("GTK application exited with status: {}", status))
    }
} 