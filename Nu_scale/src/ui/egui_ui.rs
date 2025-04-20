use anyhow::Result;
use eframe::{self, egui};
use egui::{Color32, RichText, Slider, TextEdit, Ui, Stroke, Rounding, Vec2, Frame};
use std::sync::{Arc, Mutex};
use image::RgbaImage;
use egui::{TextureOptions, TextureHandle, ColorImage};

use crate::capture;
use super::profile::{Profile, CaptureSource, SystemPlatform, UpscalingTechnology, UpscalingQuality};
use super::settings::AppSettings;
use crate::capture::common::FrameBuffer;
use crate::upscale::{Upscaler, UpscalingAlgorithm};

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
    upscaling_buffer: Option<Arc<crate::capture::common::FrameBuffer>>,
    /// Stop signal for upscaling mode
    upscaling_stop_signal: Option<Arc<Mutex<bool>>>,
    /// Current frame texture
    frame_texture: Option<TextureHandle>,
}

#[derive(PartialEq)]
enum StatusMessageType {
    Info,
    Success,
    #[allow(dead_code)]
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
            is_fullscreen: false,
            is_upscaling: false,
            _toggle_capture_hotkey: "Ctrl+Alt+C".to_string(),
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
            upscaling_buffer: None,
            upscaling_stop_signal: None,
            frame_texture: None,
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
                
                // Fullscreen mode button
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
                            ui.selectable_value(&mut self.profile.upscaling_tech, UpscalingTechnology::FSR, "AMD FidelityFX Super Resolution (FSR)");
                            ui.selectable_value(&mut self.profile.upscaling_tech, UpscalingTechnology::DLSS, "NVIDIA Deep Learning Super Sampling (DLSS)");
                            ui.selectable_value(&mut self.profile.upscaling_tech, UpscalingTechnology::Fallback, "Traditional Algorithms");
                            ui.selectable_value(&mut self.profile.upscaling_tech, UpscalingTechnology::Custom, "Custom");
                        });
                });
                
                ui.add_space(8.0);
                
                // Upscaling quality
                ui.horizontal(|ui| {
                    ui.label("Upscaling Quality:");
                    ui.add_space(8.0);
                    
                    egui::ComboBox::from_id_source("upscale_quality")
                        .selected_text(format!("{:?}", self.profile.upscaling_quality))
                        .width(300.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.profile.upscaling_quality, UpscalingQuality::Ultra, "Ultra Quality");
                            ui.selectable_value(&mut self.profile.upscaling_quality, UpscalingQuality::Quality, "Quality");
                            ui.selectable_value(&mut self.profile.upscaling_quality, UpscalingQuality::Balanced, "Balanced");
                            ui.selectable_value(&mut self.profile.upscaling_quality, UpscalingQuality::Performance, "Performance");
                        });
                });
                
                // Only show algorithm selection for Traditional/Fallback upscaling
                if self.profile.upscaling_tech == UpscalingTechnology::Fallback {
                    ui.add_space(8.0);
                    
                    // Initialize algorithm if not set
                    if self.profile.upscaling_algorithm.is_none() {
                        self.profile.upscaling_algorithm = Some("Lanczos3".to_string());
                    }
                    
                    // Upscaling algorithm 
                    ui.horizontal(|ui| {
                        ui.label("Upscaling Algorithm:");
                        ui.add_space(8.0);
                        
                        let mut current_algorithm = self.profile.upscaling_algorithm.clone().unwrap_or_else(|| "Lanczos3".to_string());
                        
                        egui::ComboBox::from_id_source("upscale_algorithm")
                            .selected_text(match current_algorithm.as_str() {
                                "NearestNeighbor" => "Nearest-Neighbor",
                                "Bilinear" => "Bilinear",
                                "Bicubic" => "Bicubic",
                                "Lanczos2" => "Lanczos (a=2)",
                                "Lanczos3" => "Lanczos (a=3)",
                                "Mitchell" => "Mitchell-Netravali",
                                "Area" => "Area (Box) Resample",
                                _ => "Lanczos (a=3)",
                            })
                            .width(300.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut current_algorithm, "NearestNeighbor".to_string(), 
                                    "Nearest-Neighbor - Zero smoothing, zero blur, but aliased");
                                ui.selectable_value(&mut current_algorithm, "Bilinear".to_string(), 
                                    "Bilinear - Fast and smooth, but tends to blur sharp edges");
                                ui.selectable_value(&mut current_algorithm, "Bicubic".to_string(), 
                                    "Bicubic - Preserves more edge sharpness than bilinear");
                                ui.selectable_value(&mut current_algorithm, "Lanczos2".to_string(), 
                                    "Lanczos (a=2) - Good edge preservation with 4×4 kernel");
                                ui.selectable_value(&mut current_algorithm, "Lanczos3".to_string(), 
                                    "Lanczos (a=3) - Best edge preservation with 6×6 kernel");
                                ui.selectable_value(&mut current_algorithm, "Mitchell".to_string(), 
                                    "Mitchell-Netravali - Tunable cubic filter for balanced results");
                                ui.selectable_value(&mut current_algorithm, "Area".to_string(), 
                                    "Area (Box) - Excellent for downscaling, useful for upscaling to avoid overshoot");
                            });
                            
                        self.profile.upscaling_algorithm = Some(current_algorithm);
                    });
                    
                    // Add algorithm description
                    if let Some(algorithm) = &self.profile.upscaling_algorithm {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add_space(138.0); // Align with dropdown content
                            
                            let description = match algorithm.as_str() {
                                "NearestNeighbor" => 
                                    "Copies each input pixel to an N×N block. Zero smoothing, zero blur, but aliased.",
                                "Bilinear" => 
                                    "Computes a weighted average of the four nearest input pixels. Fast and smooth, but tends to blur sharp edges.",
                                "Bicubic" => 
                                    "Uses cubic convolution on a 4×4 neighborhood to preserve more edge sharpness than bilinear, at moderate cost.",
                                "Lanczos2" => 
                                    "Windowed sinc filter over a 4×4 kernel. Good edge preservation with moderate compute.",
                                "Lanczos3" => 
                                    "Windowed sinc filter over a 6×6 kernel. Best edge preservation among traditional kernels, heavier compute.",
                                "Mitchell" => 
                                    "Tunable two-parameter cubic filters that trade off ringing vs. smoothness.",
                                "Area" => 
                                    "Averages all pixels covered by the destination pixel's footprint. Excellent for downscaling, sometimes used for upscaling.",
                                _ => "",
                            };
                            
                            ui.label(RichText::new(description).weak().italics());
                        });
                    }
                } else {
                    // Reset algorithm when not using traditional upscaling
                    self.profile.upscaling_algorithm = None;
                }
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

    /// Toggle fullscreen mode
    pub fn toggle_fullscreen_mode(&mut self) -> Result<()> {
        self.is_fullscreen = !self.is_fullscreen;
        
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
        
        Ok(())
    }
    
    /// Start upscaling mode
    pub fn start_upscaling_mode(
        &mut self,
        source: crate::capture::CaptureTarget,
        technology: crate::upscale::UpscalingTechnology,
        quality: crate::upscale::UpscalingQuality,
        fps: u32,
        algorithm: Option<crate::UpscalingAlgorithm>,
    ) -> Result<()> {
        // Create frame buffer and stop signal
        let buffer = Arc::new(crate::capture::common::FrameBuffer::new(5));
        let stop_signal = Arc::new(Mutex::new(false));
        
        // Start capture thread
        let capture_buffer = Arc::clone(&buffer);
        let capture_stop = Arc::clone(&stop_signal);
        let capture_handle = crate::capture::common::start_live_capture_thread(
            source.clone(),
            fps,
            capture_buffer,
            capture_stop,
        )?;
        
        // Create upscaler and wrap in Arc<Mutex<>> for thread safety
        let mut upscaler = crate::upscale::create_upscaler(technology, quality, algorithm)?;
        
        // Get screen dimensions
        let capturer = crate::capture::create_capturer()?;
        let (screen_width, screen_height) = capturer.get_primary_screen_dimensions()?;
        
        // Initialize upscaler with target dimensions
        upscaler.initialize(screen_width, screen_height, screen_width, screen_height)?;
        
        // Move upscaler to an Arc<Mutex<>> for sharing between threads
        let upscaler = Arc::new(Mutex::new(upscaler));
        
        // Store the buffer and stop signal
        self.upscaling_buffer = Some(Arc::clone(&buffer));
        self.upscaling_stop_signal = Some(Arc::clone(&stop_signal));
        
        // Set upscaling mode flag
        self.is_upscaling = true;
        
        // Update status
        self.status_message = "Upscaling mode active. Press ESC to exit.".to_string();
        self.status_message_type = StatusMessageType::Success;
        
        Ok(())
    }
    
    /// Update upscaling mode UI
    fn update_upscaling_mode(&mut self, ctx: &egui::Context) {
        // If upscaling buffer or stop signal is missing, exit upscaling mode
        let buffer = match &self.upscaling_buffer {
            Some(buf) => Arc::clone(buf),
            None => {
                self.is_upscaling = false;
                return;
            }
        };
        
        let stop_signal = match &self.upscaling_stop_signal {
            Some(sig) => Arc::clone(sig),
            None => {
                self.is_upscaling = false;
                return;
            }
        };
        
        // Get the latest frame
        if let Ok(Some(frame)) = buffer.get_latest_frame() {
            // Convert to egui format for display
            let size = [frame.width() as _, frame.height() as _];
            let flat_samples = frame.as_flat_samples();
            let pixels = flat_samples.as_slice();
            
            // Create egui image
            let color_image = ColorImage::from_rgba_unmultiplied(size, pixels);
            
            // Create or update texture
            self.frame_texture = Some(ctx.load_texture(
                "upscaled_frame",
                color_image,
                TextureOptions::LINEAR
            ));
        }
        
        // Display the frame fullscreen
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::BLACK))
            .show(ctx, |ui| {
                if let Some(texture) = &self.frame_texture {
                    let available_size = ui.available_size();
                    ui.image(texture, available_size);
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("Waiting for frames...").size(24.0).color(Color32::WHITE));
                    });
                }
            });
        
        // Show stats overlay
        egui::Window::new("Statistics")
            .anchor(egui::Align2::RIGHT_TOP, [0.0, 0.0])
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Press ESC to exit upscaling mode");
            });
        
        // Handle ESC to exit upscaling mode
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            // Signal to stop capture
            if let Ok(mut guard) = stop_signal.lock() {
                *guard = true;
            }
            
            // Exit upscaling mode
            self.is_upscaling = false;
            self.upscaling_buffer = None;
            self.upscaling_stop_signal = None;
            self.frame_texture = None;
            
            // Exit fullscreen mode
            if self.is_fullscreen {
                self.toggle_fullscreen_mode().ok();
            }
            
            // Update status
            self.status_message = "Exited upscaling mode".to_string();
            self.status_message_type = StatusMessageType::Info;
        }
        
        // Request continuous repaint
        ctx.request_repaint();
    }
    
    /// Launch the fullscreen upscaling mode with current profile settings
    fn launch_fullscreen_mode(&mut self) {
        // Create capture target from profile
        let target = match &self.profile.source {
            CaptureSource::Fullscreen => crate::capture::CaptureTarget::FullScreen,
            CaptureSource::Window(title) => crate::capture::CaptureTarget::WindowByTitle(title.clone()),
            CaptureSource::Region { x, y, width, height } => crate::capture::CaptureTarget::Region {
                x: *x,
                y: *y,
                width: *width,
                height: *height,
            },
        };
        
        // Convert upscaling technology
        let technology = match self.profile.upscaling_tech {
            UpscalingTechnology::None => crate::upscale::UpscalingTechnology::None,
            UpscalingTechnology::FSR => crate::upscale::UpscalingTechnology::FSR,
            UpscalingTechnology::DLSS => crate::upscale::UpscalingTechnology::DLSS,
            UpscalingTechnology::Fallback => crate::upscale::UpscalingTechnology::Fallback,
            _ => crate::upscale::UpscalingTechnology::Fallback,
        };
        
        // Convert quality setting
        let quality = match self.profile.upscaling_quality {
            UpscalingQuality::Ultra => crate::upscale::UpscalingQuality::Ultra,
            UpscalingQuality::Quality => crate::upscale::UpscalingQuality::Quality,
            UpscalingQuality::Balanced => crate::upscale::UpscalingQuality::Balanced,
            UpscalingQuality::Performance => crate::upscale::UpscalingQuality::Performance,
        };
        
        // Convert algorithm if using fallback technology
        let algorithm = if self.profile.upscaling_tech == UpscalingTechnology::Fallback {
            if let Some(alg_str) = &self.profile.upscaling_algorithm {
                crate::string_to_algorithm(alg_str)
            } else {
                None
            }
        } else {
            None
        };
        
        // Save the profile before launching fullscreen mode
        if let Err(e) = self.profile.save() {
            self.status_message = format!("Warning: Failed to save profile before launching fullscreen: {}", e);
            self.status_message_type = StatusMessageType::Warning;
        }
        
        // Start upscaling mode directly within the current window
        match self.toggle_fullscreen_mode()
            .and_then(|_| self.start_upscaling_mode(
                target,
                technology,
                quality,
                self.profile.fps,
                algorithm,
            ))
        {
            Ok(_) => {
                self.status_message = "Upscaling mode started. Press ESC to exit.".to_string();
                self.status_message_type = StatusMessageType::Success;
            },
            Err(e) => {
                self.status_message = format!("Failed to start upscaling mode: {}", e);
                self.status_message_type = StatusMessageType::Error;
                
                // Exit fullscreen mode if we entered it
                if self.is_fullscreen {
                    self.toggle_fullscreen_mode().ok();
                }
            }
        }
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

/// Run a fullscreen renderer using wgpu and egui for hardware-accelerated upscaling
pub fn run_fullscreen_renderer(
    buffer: Arc<crate::capture::common::FrameBuffer>,
    stop_signal: Arc<Mutex<bool>>,
    processor: impl FnMut(&RgbaImage) -> anyhow::Result<RgbaImage> + Send + 'static,
) -> anyhow::Result<()> {
    // Create a custom application state for fullscreen rendering
    struct FullscreenRenderer {
        buffer: Arc<crate::capture::common::FrameBuffer>,
        stop_signal: Arc<Mutex<bool>>,
        processor: Box<dyn FnMut(&RgbaImage) -> anyhow::Result<RgbaImage> + Send + 'static>,
        texture: Option<TextureHandle>,
        last_frame: Option<RgbaImage>,
        fps: f32,
        frame_count: usize,
        last_fps_update: std::time::Instant,
        show_stats: bool,
    }

    impl eframe::App for FullscreenRenderer {
        fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            // Handle ESC key to exit
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                // Signal to stop
                if let Ok(mut stop) = self.stop_signal.lock() {
                    *stop = true;
                }
                frame.close();
                return;
            }
            
            // Toggle stats display with F1
            if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
                self.show_stats = !self.show_stats;
            }
            
            // Get latest frame from buffer
            if let Ok(Some(frame_img)) = self.buffer.get_latest_frame() {
                // Process the frame (upscale)
                match (self.processor)(&frame_img) {
                    Ok(processed) => {
                        // Convert to egui format for display
                        let size = [processed.width() as _, processed.height() as _];
                        
                        // Create a copy of the pixel data to avoid temporary value issues
                        let flat_samples = processed.as_flat_samples();
                        let pixels = flat_samples.as_slice();
                        
                        // Create egui image
                        let color_image = ColorImage::from_rgba_unmultiplied(size, pixels);
                        
                        // Load or update texture with egui 
                        self.texture = Some(ctx.load_texture(
                            "upscaled_frame",
                            color_image,
                            TextureOptions::LINEAR
                        ));
                        
                        // Store processed frame
                        self.last_frame = Some(processed);
                        
                        // Update FPS counter
                        self.frame_count += 1;
                        let now = std::time::Instant::now();
                        let elapsed = now.duration_since(self.last_fps_update);
                        if elapsed.as_secs_f32() >= 1.0 {
                            self.fps = self.frame_count as f32 / elapsed.as_secs_f32();
                            self.frame_count = 0;
                            self.last_fps_update = now;
                        }
                    },
                    Err(e) => {
                        eprintln!("Error processing frame: {}", e);
                    }
                }
            }
            
            // Render the texture full-screen
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(Color32::BLACK))
                .show(ctx, |ui| {
                    if let Some(texture) = &self.texture {
                        let available_size = ui.available_size();
                        ui.image(texture, available_size);
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("Waiting for frames...").size(24.0).color(Color32::WHITE));
                        });
                    }
                });
            
            // Show stats if enabled
            if self.show_stats {
                egui::Window::new("Statistics")
                    .anchor(egui::Align2::RIGHT_TOP, [0.0, 0.0])
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(format!("FPS: {:.1}", self.fps));
                        
                        if let Some(frame) = &self.last_frame {
                            ui.label(format!("Resolution: {}x{}", frame.width(), frame.height()));
                        }
                        
                        ui.label("Press ESC to exit, F1 to toggle stats");
                    });
            }
            
            // Request continuous repaint for smooth animation
            ctx.request_repaint();
        }
    }
    
    // Create app state
    let app = FullscreenRenderer {
        buffer,
        stop_signal,
        processor: Box::new(processor),
        texture: None,
        last_frame: None,
        fps: 0.0,
        frame_count: 0,
        last_fps_update: std::time::Instant::now(),
        show_stats: true,
    };
    
    // Run with native options for fullscreen
    let native_options = eframe::NativeOptions {
        initial_window_size: None, // Will use fullscreen
        maximized: true,
        fullscreen: true,
        renderer: eframe::Renderer::Wgpu,
        vsync: false, // Disable vsync for maximum performance
        hardware_acceleration: eframe::HardwareAcceleration::Required,
        ..Default::default()
    };
    
    // Run the app
    eframe::run_native(
        "NU Scale - Fullscreen",
        native_options,
        Box::new(|_cc| Box::new(app)),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run fullscreen renderer: {}", e))
}

/// Runs a fullscreen egui application for upscaling images
pub fn run_fullscreen_upscaler(
    frame_buffer: Arc<FrameBuffer>,
    stop_signal: Arc<Mutex<bool>>,
    upscaler: Box<dyn Upscaler>,
    algorithm: Option<UpscalingAlgorithm>,
) -> Result<()> {
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::vec2(1280.0, 720.0));
    native_options.maximized = true;
    native_options.decorated = false;
    native_options.transparent = false;
    native_options.fullscreen = true;
    
    eframe::run_native(
        "NU_Scaler Fullscreen",
        native_options,
        Box::new(|cc| {
            let mut app = FullscreenUpscalerUi::new(
                cc,
                frame_buffer,
                stop_signal,
                upscaler,
                algorithm,
            );
            Box::new(app)
        }),
    )
    .map_err(|e| e.to_string().into())
}

// Implementation of the fullscreen upscaler UI
struct FullscreenUpscalerUi {
    frame_buffer: Arc<FrameBuffer>,
    stop_signal: Arc<Mutex<bool>>,
    upscaler: Box<dyn Upscaler>,
    algorithm: Option<UpscalingAlgorithm>,
    texture: Option<egui::TextureHandle>,
    last_frame_time: std::time::Instant,
    fps: f32,
    frames_processed: u64,
}

impl FullscreenUpscalerUi {
    fn new(
        cc: &eframe::CreationContext<'_>,
        frame_buffer: Arc<FrameBuffer>,
        stop_signal: Arc<Mutex<bool>>,
        upscaler: Box<dyn Upscaler>,
        algorithm: Option<UpscalingAlgorithm>,
    ) -> Self {
        // Set up the theme
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::new(22.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Body, egui::FontId::new(18.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Monospace, egui::FontId::new(16.0, egui::FontFamily::Monospace)),
            (egui::TextStyle::Button, egui::FontId::new(18.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Small, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
        ].into();
        cc.egui_ctx.set_style(style);
        
        // Configure keyboard focus for ESC key detection
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        
        Self {
            frame_buffer,
            stop_signal,
            upscaler,
            algorithm,
            texture: None,
            last_frame_time: std::time::Instant::now(),
            fps: 0.0,
            frames_processed: 0,
        }
    }
    
    fn update_texture(&mut self, ctx: &egui::Context) {
        if let Some(frame) = self.frame_buffer.read_frame() {
            // Measure time for FPS calculation
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(self.last_frame_time).as_secs_f32();
            self.last_frame_time = now;
            self.fps = 0.9 * self.fps + 0.1 * (1.0 / elapsed);
            self.frames_processed += 1;
            
            // Process the frame with the upscaler
            let upscaled_frame = match self.algorithm {
                Some(algorithm) => {
                    let (width, height) = frame.dimensions();
                    let (output_width, output_height) = (width * 2, height * 2); // Default 2x upscaling
                    let mut output = image::RgbaImage::new(output_width, output_height);
                    
                    if let Err(e) = self.upscaler.upscale(
                        &frame,
                        &mut output,
                        algorithm,
                    ) {
                        eprintln!("Upscaling error: {}", e);
                        return;
                    }
                    output
                },
                None => {
                    let (width, height) = frame.dimensions();
                    let (output_width, output_height) = (width * 2, height * 2); // Default 2x upscaling
                    let mut output = image::RgbaImage::new(output_width, output_height);
                    
                    if let Err(e) = self.upscaler.upscale(
                        &frame,
                        &mut output,
                        UpscalingAlgorithm::Balanced,
                    ) {
                        eprintln!("Upscaling error: {}", e);
                        return;
                    }
                    output
                }
            };
            
            // Create or update texture
            let size = [upscaled_frame.width() as _, upscaled_frame.height() as _];
            let pixels = upscaled_frame.as_raw();
            
            let texture = self.texture.get_or_insert_with(|| {
                ctx.load_texture(
                    "upscaled-frame",
                    egui::ColorImage::from_rgba_unmultiplied(size, pixels),
                    egui::TextureFilter::Linear,
                )
            });
            
            texture.set(egui::ColorImage::from_rgba_unmultiplied(size, pixels), egui::TextureFilter::Linear);
        }
    }
}

impl eframe::App for FullscreenUpscalerUi {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Handle ESC key to exit
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if let Ok(mut stop) = self.stop_signal.lock() {
                *stop = true;
            }
            frame.close();
            return;
        }
        
        // Update the texture with the latest captured frame
        self.update_texture(ctx);
        
        // Draw the upscaled frame fullscreen
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                if let Some(texture) = &self.texture {
                    let available_size = ui.available_size();
                    let texture_size = texture.size_vec2();
                    
                    // Calculate scale to fit screen while maintaining aspect ratio
                    let scale = (available_size.x / texture_size.x)
                        .min(available_size.y / texture_size.y);
                    
                    let scaled_size = texture_size * scale;
                    let position = egui::pos2(
                        (available_size.x - scaled_size.x) * 0.5,
                        (available_size.y - scaled_size.y) * 0.5,
                    );
                    
                    ui.put(
                        egui::Rect::from_min_size(position, scaled_size),
                        egui::Image::new(texture)
                            .fit_to_exact_size(scaled_size)
                    );
                    
                    // Display FPS in the corner
                    ui.put(
                        egui::Rect::from_min_size(
                            egui::pos2(10.0, 10.0),
                            egui::vec2(100.0, 20.0),
                        ),
                        egui::Label::new(
                            egui::RichText::new(format!("FPS: {:.1}", self.fps))
                                .color(egui::Color32::GREEN)
                                .background_color(egui::Color32::from_black_alpha(150))
                        ),
                    );
                    
                    // Press ESC to exit message
                    ui.put(
                        egui::Rect::from_min_size(
                            egui::pos2(10.0, ui.available_height() - 30.0),
                            egui::vec2(200.0, 20.0),
                        ),
                        egui::Label::new(
                            egui::RichText::new("Press ESC to exit")
                                .color(egui::Color32::WHITE)
                                .background_color(egui::Color32::from_black_alpha(150))
                        ),
                    );
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Waiting for frames...");
                    });
                }
            });
        
        // Request continuous repaints for smooth animation
        ctx.request_repaint();
    }
} 