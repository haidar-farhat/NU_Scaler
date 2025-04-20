use anyhow::Result;
use eframe::{self, egui};
use egui::{Color32, RichText, Slider, TextEdit, Ui, FontId, Label};
use std::sync::{Arc, Mutex};

use crate::capture::{self, CaptureTarget};
use super::profile::{Profile, CaptureSource, SystemPlatform, UpscalingTechnology};
use super::settings::AppSettings;
use super::hotkeys::{HotkeyManager, HotkeyAction};

/// The main application state
struct AppState {
    /// Current profile
    profile: Profile,
    /// Application settings
    settings: AppSettings,
    /// Is capturing active
    is_capturing: bool,
    /// Hotkey string for toggle capture
    toggle_capture_hotkey: String,
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
}

impl Default for AppState {
    fn default() -> Self {
        // Load settings
        let settings = AppSettings::load().unwrap_or_default();
        
        // Load profile
        let profile = settings.get_current_profile().unwrap_or_default();
        
        // Determine capture source index
        let capture_source_index = match profile.source {
            CaptureSource::Fullscreen => 0,
            CaptureSource::Window(_) => 1,
            CaptureSource::Region { .. } => 2,
        };
        
        // Get region
        let region = match profile.source {
            CaptureSource::Region { x, y, width, height } => (x, y, width, height),
            _ => (0, 0, 800, 600),
        };
        
        // Get available profiles
        let available_profiles = Profile::list_profiles().unwrap_or_default();
        
        // Get available windows
        let available_windows = capture::common::list_available_windows()
            .map(|windows| windows.iter().map(|w| w.title.clone()).collect())
            .unwrap_or_default();
        
        Self {
            profile,
            settings,
            is_capturing: false,
            toggle_capture_hotkey: "Ctrl+Alt+C".to_string(),
            single_frame_hotkey: "Ctrl+Alt+S".to_string(),
            toggle_overlay_hotkey: "Ctrl+Alt+O".to_string(),
            available_profiles,
            available_windows,
            selected_window_index: 0,
            capture_source_index,
            region,
            show_region_dialog: false,
            status_message: "Ready".to_string(),
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Central panel with tabs
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("NU Scale");
                    ui.separator();
                    
                    if ui.button("Save Profile").clicked() {
                        if let Err(e) = self.profile.save() {
                            self.status_message = format!("Error saving profile: {}", e);
                        } else {
                            self.status_message = "Profile saved".to_string();
                        }
                    }
                    
                    if ui.button("New Profile").clicked() {
                        // Show dialog to create new profile (simplified)
                        let new_name = format!("Profile_{}", self.available_profiles.len() + 1);
                        self.profile = Profile::new(&new_name);
                        self.available_profiles.push(new_name);
                        self.status_message = "New profile created".to_string();
                    }
                    
                    if self.is_capturing {
                        if ui.button("Stop Capture").clicked() {
                            self.is_capturing = false;
                            self.status_message = "Capture stopped".to_string();
                        }
                    } else {
                        if ui.button("Start Capture").clicked() {
                            self.is_capturing = true;
                            self.status_message = "Capture started".to_string();
                        }
                    }
                    
                    if ui.button("Capture Frame").clicked() {
                        self.status_message = "Frame captured".to_string();
                    }
                });
            });
            
            // Status bar at bottom
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.status_message).monospace());
                    
                    // Show capture status
                    if self.is_capturing {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("CAPTURING").color(Color32::GREEN).strong());
                        });
                    }
                });
            });
            
            // Main content
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::TabBar::new(&mut 0)
                    .tab(&mut TabState::Capture, "Capture", |ui| self.show_capture_tab(ui))
                    .tab(&mut TabState::Settings, "Settings", |ui| self.show_settings_tab(ui))
                    .tab(&mut TabState::Advanced, "Advanced", |ui| self.show_advanced_tab(ui))
                    .ui(ui);
            });
        });
        
        // Region selection dialog
        if self.show_region_dialog {
            egui::Window::new("Select Region")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Select Region");
                    ui.add(egui::Slider::new(&mut self.region.0, -2000..=2000).text("X"));
                    ui.add(egui::Slider::new(&mut self.region.1, -2000..=2000).text("Y"));
                    ui.add(egui::Slider::new(&mut self.region.2 as &mut i32, 100..=3840).text("Width"));
                    ui.add(egui::Slider::new(&mut self.region.3 as &mut i32, 100..=2160).text("Height"));
                    
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            self.show_region_dialog = false;
                            self.profile.source = CaptureSource::Region {
                                x: self.region.0,
                                y: self.region.1,
                                width: self.region.2,
                                height: self.region.3,
                            };
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_region_dialog = false;
                        }
                    });
                });
        }
    }
}

enum TabState {
    Capture,
    Settings,
    Advanced,
}

impl AppState {
    /// Show the capture tab
    fn show_capture_tab(&mut self, ui: &mut Ui) {
        // Profile selection
        ui.group(|ui| {
            ui.heading("Profile");
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Current Profile")
                    .selected_text(self.profile.name.clone())
                    .show_ui(ui, |ui| {
                        for profile_name in &self.available_profiles {
                            ui.selectable_value(&mut self.profile.name, profile_name.clone(), profile_name);
                        }
                    });
            });
        });
        
        // Capture source
        ui.group(|ui| {
            ui.heading("Capture Source");
            
            // Fullscreen
            ui.radio_value(&mut self.capture_source_index, 0, "Fullscreen");
            if self.capture_source_index == 0 {
                self.profile.source = CaptureSource::Fullscreen;
            }
            
            // Window
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.capture_source_index, 1, "Window");
                if self.capture_source_index == 1 {
                    egui::ComboBox::from_id_source("window_selector")
                        .selected_text(
                            self.available_windows.get(self.selected_window_index)
                                .cloned()
                                .unwrap_or_else(|| "Select Window".to_string())
                        )
                        .show_ui(ui, |ui| {
                            for (i, window_name) in self.available_windows.iter().enumerate() {
                                if ui.selectable_value(&mut self.selected_window_index, i, window_name).changed() {
                                    self.profile.source = CaptureSource::Window(window_name.clone());
                                }
                            }
                        });
                        
                    if ui.button("Refresh").clicked() {
                        // Refresh window list
                        self.available_windows = capture::common::list_available_windows()
                            .map(|windows| windows.iter().map(|w| w.title.clone()).collect())
                            .unwrap_or_default();
                    }
                }
            });
            
            // Region
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.capture_source_index, 2, "Region");
                if self.capture_source_index == 2 {
                    if ui.button("Select Region").clicked() {
                        self.show_region_dialog = true;
                    }
                    
                    let (x, y, width, height) = match self.profile.source {
                        CaptureSource::Region { x, y, width, height } => (x, y, width, height),
                        _ => self.region,
                    };
                    ui.label(format!("({}, {}, {}x{})", x, y, width, height));
                }
            });
        });
        
        // Platform selection
        ui.group(|ui| {
            ui.heading("System Platform");
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Platform")
                    .selected_text(format!("{:?}", self.profile.platform))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.profile.platform, SystemPlatform::Auto, "Auto");
                        ui.selectable_value(&mut self.profile.platform, SystemPlatform::Windows, "Windows");
                        ui.selectable_value(&mut self.profile.platform, SystemPlatform::X11, "X11");
                        ui.selectable_value(&mut self.profile.platform, SystemPlatform::Wayland, "Wayland");
                    });
            });
        });
    }
    
    /// Show the settings tab
    fn show_settings_tab(&mut self, ui: &mut Ui) {
        // Upscaling settings
        ui.group(|ui| {
            ui.heading("Upscaling Settings");
            
            // Scale factor
            ui.horizontal(|ui| {
                ui.label("Scale Factor:");
                ui.add(Slider::new(&mut self.profile.scale_factor, 1.0..=4.0)
                    .step_by(0.1)
                    .text("x"));
            });
            
            // Upscaling technology
            ui.horizontal(|ui| {
                ui.label("Upscaling Technology:");
                egui::ComboBox::from_id_source("upscale_tech")
                    .selected_text(format!("{:?}", self.profile.upscaling_tech))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.profile.upscaling_tech, UpscalingTechnology::None, "None");
                        ui.selectable_value(&mut self.profile.upscaling_tech, UpscalingTechnology::FSR, "FSR (AMD FidelityFX)");
                        ui.selectable_value(&mut self.profile.upscaling_tech, UpscalingTechnology::NIS, "NIS (NVIDIA Image Scaling)");
                        ui.selectable_value(&mut self.profile.upscaling_tech, UpscalingTechnology::Custom, "Custom");
                    });
            });
        });
        
        // Hotkey settings
        ui.group(|ui| {
            ui.heading("Hotkey Settings");
            
            // Toggle capture hotkey
            ui.horizontal(|ui| {
                ui.label("Start/Stop Capture:");
                let response = ui.add(TextEdit::singleline(&mut self.profile.hotkey));
                if response.changed() {
                    // Update hotkey
                }
            });
            
            // Single frame hotkey
            ui.horizontal(|ui| {
                ui.label("Capture Single Frame:");
                let response = ui.add(TextEdit::singleline(&mut self.single_frame_hotkey));
                if response.changed() {
                    // Update hotkey
                }
            });
            
            // Overlay toggle hotkey
            ui.horizontal(|ui| {
                ui.label("Toggle Overlay:");
                let response = ui.add(TextEdit::singleline(&mut self.toggle_overlay_hotkey));
                if response.changed() {
                    // Update hotkey
                }
            });
        });
        
        // FPS settings
        ui.group(|ui| {
            ui.heading("Capture FPS");
            ui.horizontal(|ui| {
                ui.label("Target FPS:");
                ui.add(egui::DragValue::new(&mut self.profile.fps).speed(1.0).clamp_range(1..=240));
            });
        });
        
        // Overlay settings
        ui.group(|ui| {
            ui.heading("Overlay");
            ui.checkbox(&mut self.profile.enable_overlay, "Enable Overlay");
        });
    }
    
    /// Show the advanced tab
    fn show_advanced_tab(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Advanced Settings");
            ui.label("Advanced settings will be available in future versions.");
        });
        
        // Add some application settings
        ui.group(|ui| {
            ui.heading("Application Settings");
            ui.checkbox(&mut self.settings.start_minimized, "Start Minimized");
            ui.checkbox(&mut self.settings.start_with_system, "Start with System");
            ui.checkbox(&mut self.settings.check_for_updates, "Check for Updates");
            ui.checkbox(&mut self.settings.auto_save_profiles, "Auto-save Profiles");
            
            ui.horizontal(|ui| {
                ui.label("Theme:");
                egui::ComboBox::from_id_source("theme")
                    .selected_text(&self.settings.theme)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.settings.theme, "system".to_string(), "System");
                        ui.selectable_value(&mut self.settings.theme, "light".to_string(), "Light");
                        ui.selectable_value(&mut self.settings.theme, "dark".to_string(), "Dark");
                    });
            });
        });
    }
}

/// Run the egui application
pub fn run_app() -> Result<()> {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        min_window_size: Some(egui::vec2(640.0, 480.0)),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    
    eframe::run_native(
        "NU Scale",
        native_options,
        Box::new(|_cc| Box::new(AppState::default())),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))
} 