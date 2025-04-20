use anyhow::Result;
use eframe::{self, egui};
use egui::{Color32, RichText, Slider, TextEdit, Ui, FontId, Label, Stroke, Rounding, Vec2, Frame};
use std::sync::{Arc, Mutex};

use crate::capture;
use super::profile::{Profile, CaptureSource, SystemPlatform, UpscalingTechnology};
use super::settings::AppSettings;
use super::hotkeys::{HotkeyManager, HotkeyAction};

const ACCENT_COLOR: Color32 = Color32::from_rgb(0, 120, 215); // Blue accent
const SUCCESS_COLOR: Color32 = Color32::from_rgb(25, 170, 88); // Green
const WARNING_COLOR: Color32 = Color32::from_rgb(235, 165, 0); // Amber
const ERROR_COLOR: Color32 = Color32::from_rgb(209, 43, 43);   // Red
const SPACING: f32 = 8.0;

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
    /// Status message type
    status_message_type: StatusMessageType,
    /// Current selected tab
    selected_tab: TabState,
}

#[derive(PartialEq)]
enum StatusMessageType {
    Info,
    Success,
    Warning,
    Error,
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
            status_message_type: StatusMessageType::Info,
            selected_tab: TabState::Capture,
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.configure_fonts(ctx);

        // Set dark mode for the UI
        ctx.set_visuals(egui::Visuals::dark());
        
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

enum TabState {
    Capture,
    Settings,
    Advanced,
}

impl AppState {
    /// Configure custom fonts
    fn configure_fonts(&self, ctx: &egui::Context) {
        let fonts = egui::FontDefinitions::default();
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
                if let Err(e) = self.profile.save() {
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
    fn show_region_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Select Region")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(RichText::new("Select Capture Region").size(20.0));
                });
                
                ui.add_space(16.0);
                
                // X coordinate
                ui.horizontal(|ui| {
                    ui.label(RichText::new("X:").strong());
                    ui.add(
                        Slider::new(&mut self.region.0, -2000..=2000)
                            .text("px")
                            .fixed_decimals(0)
                    );
                });
                
                // Y coordinate
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Y:").strong());
                    ui.add(
                        Slider::new(&mut self.region.1, -2000..=2000)
                            .text("px")
                            .fixed_decimals(0)
                    );
                });
                
                ui.add_space(8.0);
                
                // Convert to i32 for the slider UI
                let mut width_i32 = self.region.2 as i32;
                let mut height_i32 = self.region.3 as i32;
                
                // Add sliders for width and height
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Width:").strong());
                    ui.add(
                        Slider::new(&mut width_i32, 100..=3840)
                            .text("px")
                            .fixed_decimals(0)
                    );
                });
                
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Height:").strong());
                    ui.add(
                        Slider::new(&mut height_i32, 100..=2160)
                            .text("px")
                            .fixed_decimals(0)
                    );
                });
                
                // Store values back 
                self.region.2 = width_i32 as u32;
                self.region.3 = height_i32 as u32;
                
                ui.add_space(16.0);
                
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                        if ui.button(RichText::new("OK")).clicked() {
                            self.show_region_dialog = false;
                            self.profile.source = CaptureSource::Region {
                                x: self.region.0,
                                y: self.region.1,
                                width: self.region.2,
                                height: self.region.3,
                            };
                        }
                        if ui.button(RichText::new("Cancel")).clicked() {
                            self.show_region_dialog = false;
                        }
                    });
                });
            });
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
                            for profile_name in &self.available_profiles {
                                ui.selectable_value(
                                    &mut self.profile.name, 
                                    profile_name.clone(), 
                                    profile_name
                                );
                            }
                        });
                });
            });
            
            // Capture source
            Self::card_frame().show(ui, |ui| {
                ui.strong(RichText::new("Capture Source").size(16.0).color(ACCENT_COLOR));
                ui.add_space(12.0);
                
                // Fullscreen
                let fullscreen_selected = ui.radio_value(
                    &mut self.capture_source_index, 
                    0, 
                    RichText::new("🖥️ Fullscreen").size(14.0)
                ).clicked();
                
                if fullscreen_selected || self.capture_source_index == 0 {
                    self.profile.source = CaptureSource::Fullscreen;
                }
                
                ui.add_space(4.0);
                
                // Window
                ui.horizontal(|ui| {
                    let _window_selected = ui.radio_value(
                        &mut self.capture_source_index, 
                        1, 
                        RichText::new("🪟 Window").size(14.0)
                    ).clicked();
                    
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
                                for (i, window_name) in self.available_windows.iter().enumerate() {
                                    if ui.selectable_value(&mut self.selected_window_index, i, window_name).changed() {
                                        self.profile.source = CaptureSource::Window(window_name.clone());
                                    }
                                }
                            });
                            
                        ui.add_space(8.0);
                        
                        if ui.button(RichText::new("🔄 Refresh").size(14.0)).clicked() {
                            // Refresh window list
                            self.available_windows = capture::common::list_available_windows()
                                .map(|windows| windows.iter().map(|w| w.title.clone()).collect())
                                .unwrap_or_default();
                        }
                    }
                });
                
                ui.add_space(4.0);
                
                // Region
                ui.horizontal(|ui| {
                    let _region_selected = ui.radio_value(
                        &mut self.capture_source_index, 
                        2, 
                        RichText::new("📏 Region").size(14.0)
                    ).clicked();
                    
                    if self.capture_source_index == 2 {
                        ui.add_space(16.0);
                        
                        if ui.button(RichText::new("Select Region").size(14.0)).clicked() {
                            self.show_region_dialog = true;
                        }
                        
                        ui.add_space(8.0);
                        
                        let (x, y, width, height) = match self.profile.source {
                            CaptureSource::Region { x, y, width, height } => (x, y, width, height),
                            _ => self.region,
                        };
                        
                        ui.label(
                            RichText::new(format!("({}, {}, {}x{})", x, y, width, height))
                                .monospace()
                        );
                    }
                });
            });
            
            // Platform selection
            Self::card_frame().show(ui, |ui| {
                ui.strong(RichText::new("System Platform").size(16.0).color(ACCENT_COLOR));
                ui.add_space(12.0);
                
                ui.horizontal(|ui| {
                    ui.label("Platform:");
                    ui.add_space(8.0);
                    
                    egui::ComboBox::from_id_source("platform_selector")
                        .selected_text(format!("{:?}", self.profile.platform))
                        .width(200.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.profile.platform, SystemPlatform::Auto, "Auto");
                            ui.selectable_value(&mut self.profile.platform, SystemPlatform::Windows, "Windows");
                            ui.selectable_value(&mut self.profile.platform, SystemPlatform::X11, "X11");
                            ui.selectable_value(&mut self.profile.platform, SystemPlatform::Wayland, "Wayland");
                        });
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
                    
                    egui::ComboBox::from_id_source("upscale_tech")
                        .selected_text(format!("{:?}", self.profile.upscaling_tech))
                        .width(300.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.profile.upscaling_tech, UpscalingTechnology::None, "None");
                            ui.selectable_value(&mut self.profile.upscaling_tech, UpscalingTechnology::FSR, "FSR (AMD FidelityFX)");
                            ui.selectable_value(&mut self.profile.upscaling_tech, UpscalingTechnology::NIS, "NIS (NVIDIA Image Scaling)");
                            ui.selectable_value(&mut self.profile.upscaling_tech, UpscalingTechnology::Custom, "Custom");
                        });
                });
            });
            
            // Hotkey settings
            Self::card_frame().show(ui, |ui| {
                ui.strong(RichText::new("Hotkey Settings").size(16.0).color(ACCENT_COLOR));
                ui.add_space(12.0);
                
                // Toggle capture hotkey
                ui.horizontal(|ui| {
                    ui.label("Start/Stop Capture:");
                    ui.add_space(8.0);
                    let text_edit = TextEdit::singleline(&mut self.profile.hotkey)
                        .desired_width(200.0)
                        .hint_text("Enter hotkey (e.g., Ctrl+Alt+C)");
                    ui.add(text_edit);
                });
                
                ui.add_space(4.0);
                
                // Single frame hotkey
                ui.horizontal(|ui| {
                    ui.label("Capture Single Frame:");
                    ui.add_space(8.0);
                    let text_edit = TextEdit::singleline(&mut self.single_frame_hotkey)
                        .desired_width(200.0)
                        .hint_text("Enter hotkey (e.g., Ctrl+Alt+S)");
                    ui.add(text_edit);
                });
                
                ui.add_space(4.0);
                
                // Overlay toggle hotkey
                ui.horizontal(|ui| {
                    ui.label("Toggle Overlay:");
                    ui.add_space(8.0);
                    let text_edit = TextEdit::singleline(&mut self.toggle_overlay_hotkey)
                        .desired_width(200.0)
                        .hint_text("Enter hotkey (e.g., Ctrl+Alt+O)");
                    ui.add(text_edit);
                });
            });
            
            // FPS settings
            Self::card_frame().show(ui, |ui| {
                ui.strong(RichText::new("Capture FPS").size(16.0).color(ACCENT_COLOR));
                ui.add_space(12.0);
                
                ui.horizontal(|ui| {
                    ui.label("Target FPS:");
                    ui.add_space(8.0);
                    let slider = Slider::new(&mut self.profile.fps, 1..=240)
                        .text("fps");
                    let _response = ui.add_sized([300.0, 20.0], slider);
                    ui.label(format!("{} fps", self.profile.fps));
                });
            });
            
            // Overlay settings
            Self::card_frame().show(ui, |ui| {
                ui.strong(RichText::new("Overlay").size(16.0).color(ACCENT_COLOR));
                ui.add_space(12.0);
                
                ui.checkbox(&mut self.profile.enable_overlay, "Enable Overlay");
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
                    ui.checkbox(&mut self.settings.start_minimized, "Start Minimized");
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.settings.start_with_system, "Start with System");
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.settings.check_for_updates, "Check for Updates");
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.settings.auto_save_profiles, "Auto-save Profiles");
                });
                
                ui.add_space(8.0);
                
                // Theme selection
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    ui.add_space(8.0);
                    
                    egui::ComboBox::from_id_source("theme")
                        .selected_text(&self.settings.theme)
                        .width(200.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.settings.theme, "system".to_string(), "System");
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
        });
    }
}

/// Run the egui application
pub fn run_app() -> Result<()> {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 768.0)),
        min_window_size: Some(egui::vec2(800.0, 600.0)),
        renderer: eframe::Renderer::Wgpu,
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };
    
    eframe::run_native(
        "NU Scale",
        native_options,
        Box::new(|_cc| Box::new(AppState::default())),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))
} 