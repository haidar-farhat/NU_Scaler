use anyhow::{anyhow, Result};
use egui::{
    epaint::ahash::{HashMap, HashMapExt},
    widgets::*,
    TextureHandle,
    *,
};
// Standard library imports
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, 
    thread,
    time::{Duration, Instant},
    marker::PhantomData
};

// Windows API for process management (cfg guard added in implementation)
#[cfg(windows)]
use windows::{
    Win32::System::Threading::TerminateProcess,
    Win32::Foundation::HANDLE,
};

// Use crate:: for lib modules
use crate::capture::{CaptureError, CaptureTarget, ScreenCapture};
use crate::capture::common::FrameBuffer;
use crate::upscale::{
    create_upscaler, Upscaler, UpscalingQuality, UpscalingTechnology,
    common::UpscalingAlgorithm,
};
use crate::renderer;

// UI-internal imports (using super::)
use super::profile::{Profile, CaptureSource, UpscalingTechnology as ProfileUpscalingTechnology, UpscalingQuality as ProfileUpscalingQuality};
use super::settings::AppSettings;
use super::hotkeys::{register_global_hotkey, KEY_TOGGLE_CAPTURE, KEY_CAPTURE_FRAME, KEY_TOGGLE_OVERLAY};
use super::components::{self, StatusBar, StatusMessageType};
use super::region_dialog::RegionDialog;
use super::tabs::{self, TabState};

// External crate imports were removed

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
}

// Type definition for upscaling buffer
type UpscalingBufferType = Arc<FrameBuffer>;

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
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.configure_fonts(ctx);

        // Set dark mode for the UI
        ctx.set_visuals(egui::Visuals::dark());
        
        // Handle ESC key to exit fullscreen mode
        if self.is_fullscreen && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.toggle_fullscreen_mode().ok();
        }
        
        // Handle upscaling mode if active
        if self.is_upscaling {
            self.update_upscaling_mode(ctx);
            return;
        }
        
        // Main app layout
        egui::CentralPanel::default().show(ctx, |_ui| {
            // Top panel with app name and main actions
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                self.show_top_bar(ui);
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
                    TabState::Capture => self.show_capture_tab(ui),
                    TabState::Settings => self.show_settings_tab(ui),
                    TabState::Advanced => self.show_advanced_tab(ui),
                }
            });
        });
        
        // Region selection dialog
        if self.show_region_dialog {
            self.show_region_dialog(ctx);
        }
    }
}

impl AppState {
    /// Configure custom fonts
    fn configure_fonts(&self, ctx: &eframe::egui::Context) {
        let fonts = eframe::egui::FontDefinitions::default();
        // Could add custom fonts here
        ctx.set_fonts(fonts);
    }

    /// Show the application top bar
    fn show_top_bar(&mut self, ui: &mut Ui) {
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
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                
                // Scale button - combined scaling and fullscreen
                let scale_button = ui.add(egui::Button::new(
                    RichText::new("🔍 Scale").size(14.0))
                        .fill(Color32::from_rgb(180, 100, 240)));
                
                if scale_button.clicked() {
                    self.start_scaling_process();
                }
                
                ui.add_space(8.0);

                // Original fullscreen mode button
                let fullscreen_button = ui.add(egui::Button::new(
                    RichText::new("🖥️ Fullscreen Mode").size(14.0))
                        .fill(Color32::from_rgb(0, 120, 215)));
                
                if fullscreen_button.clicked() {
                    self.launch_fullscreen_mode();
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
    fn show_region_dialog(&mut self, ctx: &eframe::egui::Context) {
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
    fn show_capture_tab(&mut self, ui: &mut Ui) {
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
                                    if ui.selectable_label(self.selected_window_index == i, window_name).clicked() {
                                        if self.selected_window_index != i {
                                            self.selected_window_index = i;
                                            changed = true;
                                        }
                                    }
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
                         3 => "CUDA GPU",  // Add this option
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
                            ui.selectable_value(&mut self.profile.upscaling_tech, 3, "CUDA GPU");  // Add this option
                            ui.selectable_value(&mut self.profile.upscaling_tech, 4, "Fallback/Basic");
                            // Removed Custom as it wasn't in the Profile struct definition
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
                
                // Only show algorithm selection for Traditional/Fallback upscaling (index 3)
                if self.profile.upscaling_tech == 3 {
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
    pub fn toggle_fullscreen_mode(&mut self) -> Result<()> {
        self.is_fullscreen = !self.is_fullscreen;
        
        // For web builds this would use web_sys, but for native we'll use the window handling of eframe
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
            source,
            fps,
            buffer.clone(),
            stop_signal.clone(),
        )?;
        
        // Store references for cleanup
        self.upscaling_buffer = Some(buffer);
        self.upscaling_stop_signal = Some(stop_signal);
        
        // Set state
        self.is_upscaling = true;
        
        Ok(())
    }
    
    /// Update the application upscaling mode state
    /// Renders the captured frames with the upscaler
    fn update_upscaling_mode(&mut self, ctx: &eframe::egui::Context) {
        // Clear the UI and display only the upscaled frame
        eframe::egui::CentralPanel::default()
            .frame(eframe::egui::Frame::none().fill(eframe::egui::Color32::BLACK))
            .show(ctx, |ui| {
                // Check if we have a buffer
                if let Some(buffer) = &self.upscaling_buffer {
                    // Get latest frame if available
                    if let Ok(Some(frame)) = buffer.get_latest_frame() {
                        // Display the frame (upscaling would happen here)
                        // For now, just display the raw frame
                        let size = [frame.width() as _, frame.height() as _];
                        
                        // Convert frame to egui format
                        let flat_samples = frame.as_flat_samples();
                        let pixels = flat_samples.as_slice();
                        
                        // Create or update texture
                        let texture = self.frame_texture.get_or_insert_with(|| {
                            ui.ctx().load_texture(
                                "captured_frame",
                                eframe::egui::ColorImage::from_rgba_unmultiplied(size, pixels),
                                eframe::egui::TextureOptions::LINEAR
                            )
                        });
                        
                        // Update existing texture if we already had one
                        if texture.size() != size {
                            *texture = ui.ctx().load_texture(
                                "captured_frame",
                                eframe::egui::ColorImage::from_rgba_unmultiplied(size, pixels),
                                eframe::egui::TextureOptions::LINEAR
                            );
                        } else {
                            texture.set(eframe::egui::ColorImage::from_rgba_unmultiplied(size, pixels), eframe::egui::TextureOptions::LINEAR);
                        }
                        
                        // Display the texture full screen
                        let available_size = ui.available_size();
                        ui.image(texture, available_size);
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label(eframe::egui::RichText::new("Waiting for frames...").size(24.0).color(eframe::egui::Color32::WHITE));
                        });
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(eframe::egui::RichText::new("No frame buffer available").size(24.0).color(eframe::egui::Color32::RED));
                    });
                }
            });
        
        let mut should_stop = false;
        if let Some(stop_signal_atomic) = &self.upscaling_stop_signal {
            // Load the AtomicBool directly
            should_stop = stop_signal_atomic.load(Ordering::Relaxed);
             // Optionally use Ordering::SeqCst for stronger guarantees
        } // No lock needed

        if should_stop {
            self.is_upscaling = false;
            self.upscaling_buffer = None;
            self.upscaling_stop_signal = None;
            self.status_message = "Upscaling stopped.".to_string();
            self.status_message_type = StatusMessageType::Info;
            return;
        }
        
        // Request continuous repainting
        ctx.request_repaint();
    }
    
    /// Launch the fullscreen upscaling mode with current profile settings
    fn launch_fullscreen_mode(&mut self) {
        // Get the capture target based on the profile configuration
        let target = match self.profile.capture_source {
            0 => CaptureTarget::FullScreen,
            1 => CaptureTarget::WindowByTitle(self.profile.window_title.clone()),
            2 => CaptureTarget::Region {
                x: self.profile.region_x,
                y: self.profile.region_y,
                width: self.profile.region_width,
                height: self.profile.region_height,
            },
            _ => CaptureTarget::FullScreen, // Default fallback
        };
        
        // Map technology and quality
        let tech = match self.profile.upscaling_tech {
            0 => UpscalingTechnology::FSR,       // Auto defaults to FSR
            1 => UpscalingTechnology::FSR,       // FSR
            2 => UpscalingTechnology::DLSS,      // DLSS
            3 => UpscalingTechnology::CUDA,      // CUDA
            4 => UpscalingTechnology::Fallback,  // Fallback
            _ => UpscalingTechnology::Fallback,  // Default
        };
        
        let quality = match self.profile.upscaling_quality {
            0 => UpscalingQuality::Ultra,
            1 => UpscalingQuality::Quality,
            2 => UpscalingQuality::Balanced,
            3 => UpscalingQuality::Performance,
            _ => UpscalingQuality::Balanced,
        };
        
        // Determine algorithm based on technology
        let algorithm = if tech == UpscalingTechnology::Fallback {
            match self.profile.upscaling_algorithm {
                0 => Some(UpscalingAlgorithm::Lanczos3),
                1 => Some(UpscalingAlgorithm::NearestNeighbor),
                2 => Some(UpscalingAlgorithm::Bilinear),
                4 => Some(UpscalingAlgorithm::Bicubic),
                _ => Some(UpscalingAlgorithm::Lanczos3),
            }
        } else {
            None
        };
        
        // Create the FrameBuffer and the LOCAL AtomicBool stop signal for fullscreen
        let frame_buffer = Arc::new(FrameBuffer::new(60));
        let stop_signal = Arc::new(AtomicBool::new(false));

        // Launch the fullscreen UI thread
        let thread_frame_buffer = frame_buffer.clone();
        let thread_stop_signal = stop_signal.clone();
        let thread_tech = tech;
        let thread_quality = quality;
        let thread_algorithm = algorithm;

        let _app_handle = std::thread::spawn(move || {
            // Use crate:: path
            if let Err(e) = crate::renderer::fullscreen::run_fullscreen_upscaler(
                thread_frame_buffer,
                thread_stop_signal,
                thread_tech,
                thread_quality,
                thread_algorithm,
            ) {
                eprintln!("Failed to run fullscreen renderer: {}", e);
            }
        });

        self.is_fullscreen = true;
        self.status_message = "Launched fullscreen mode.".to_string();
        self.status_message_type = StatusMessageType::Info;
    }

    /// Start a scaling process in a separate application instance
    fn start_scaling_process(&mut self) {
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
            3 => "cuda",       // CUDA
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
        let mut alg_arg = Vec::new();
        if self.profile.upscaling_tech == 3 { // Fallback mode uses algorithm
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
            },
            Err(e) => {
                log::error!("Failed to start scaling process: {}", e);
                self.status_message = format!("Failed to start scaling: {}", e);
                self.status_message_type = StatusMessageType::Error;
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
}

// Add a cleanup function to ensure we kill the scaling process on exit
impl Drop for AppState {
    fn drop(&mut self) {
        self.kill_scaling_process();
    }
}

/// Run the egui application
pub fn run_app() -> Result<()> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(1024.0, 768.0)),
        min_window_size: Some(eframe::egui::vec2(800.0, 600.0)),
        vsync: true,
        decorated: true,
        centered: true,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        renderer: eframe::Renderer::Wgpu,
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };
    
    eframe::run_native(
        "NU Scale",
        options,
        Box::new(|cc| {
            let mut app_state = AppState::default();
            Box::new(app_state)
        }),
    )
    .map_err(|e| anyhow!("Failed to run application: {}", e))?;

    Ok(())
} 
