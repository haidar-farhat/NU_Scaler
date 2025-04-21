use egui::{Ui, RichText};
use anyhow::Result;

use crate::ui::components::{card_frame, StatusMessageType};
use crate::ui::profile::{CaptureSource, Profile};
use crate::capture::CaptureTarget;

/// Capture tab implementation
pub struct CaptureTab {
    /// Current profile
    profile: Profile,
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
    /// Is capturing active
    is_capturing: bool,
}

impl CaptureTab {
    /// Create a new capture tab
    pub fn new(
        profile: Profile,
        available_windows: Vec<String>,
    ) -> Self {
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
        
        Self {
            profile,
            available_windows,
            selected_window_index: 0,
            capture_source_index,
            region,
            show_region_dialog: false,
            status_message: "Ready".to_string(),
            status_message_type: StatusMessageType::Info,
            is_capturing: false,
        }
    }
    
    /// Show the capture tab UI
    pub fn show(&mut self, ui: &mut Ui) -> Result<()> {
        ui.heading("Capture");
        
        ui.add_space(10.0);
        
        // Capture source selection
        self.show_capture_source_section(ui);
        
        ui.add_space(10.0);
        
        // Upscaling options
        self.show_upscaling_section(ui);
        
        ui.add_space(10.0);
        
        // Output settings
        self.show_output_section(ui);
        
        ui.add_space(10.0);
        
        // Capture control buttons
        self.show_capture_controls(ui);
        
        Ok(())
    }
    
    /// Show capture source selection section
    fn show_capture_source_section(&mut self, ui: &mut Ui) {
        egui::Frame::group(ui.style())
            .fill(card_frame().fill)
            .rounding(card_frame().rounding)
            .stroke(card_frame().stroke)
            .show(ui, |ui| {
                ui.heading(RichText::new("Source").size(16.0));
                ui.separator();
                
                // Radio button for fullscreen
                ui.radio_value(
                    &mut self.capture_source_index,
                    0,
                    "Fullscreen"
                );
                
                // Radio button for window capture
                ui.radio_value(
                    &mut self.capture_source_index,
                    1,
                    "Window"
                );
                
                // Window selector dropdown
                if self.capture_source_index == 1 {
                    ui.indent("window_selector", |ui| {
                        egui::ComboBox::from_id_source("window_selector")
                            .selected_text(if self.available_windows.is_empty() {
                                "No windows available"
                            } else if self.selected_window_index < self.available_windows.len() {
                                &self.available_windows[self.selected_window_index]
                            } else {
                                "Select a window"
                            })
                            .show_ui(ui, |ui| {
                                for (i, window) in self.available_windows.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut self.selected_window_index,
                                        i,
                                        window
                                    );
                                }
                            });
                    });
                }
                
                // Radio button for region capture
                ui.radio_value(
                    &mut self.capture_source_index,
                    2,
                    "Region"
                );
                
                // Region selector
                if self.capture_source_index == 2 {
                    ui.indent("region_selector", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Position:");
                            ui.label(format!("X: {}, Y: {}", self.region.0, self.region.1));
                            ui.label(format!("Size: {}x{}", self.region.2, self.region.3));
                            if ui.button("Select Region").clicked() {
                                self.show_region_dialog = true;
                            }
                        });
                    });
                }
                
                // Update profile based on selection
                self.update_profile_from_ui();
            });
    }
    
    /// Show upscaling options section
    fn show_upscaling_section(&mut self, ui: &mut Ui) {
        egui::Frame::group(ui.style())
            .fill(card_frame().fill)
            .rounding(card_frame().rounding)
            .stroke(card_frame().stroke)
            .show(ui, |ui| {
                ui.heading(RichText::new("Upscaling").size(16.0));
                ui.separator();
                
                // Technology selection
                ui.horizontal(|ui| {
                    ui.label("Technology:");
                    egui::ComboBox::from_id_source("tech_selector")
                        .selected_text(format!("{:?}", self.profile.upscaling_technology))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.profile.upscaling_technology,
                                crate::ui::profile::UpscalingTechnology::None,
                                "None"
                            );
                            ui.selectable_value(
                                &mut self.profile.upscaling_technology,
                                crate::ui::profile::UpscalingTechnology::FSR,
                                "AMD FSR"
                            );
                            ui.selectable_value(
                                &mut self.profile.upscaling_technology,
                                crate::ui::profile::UpscalingTechnology::DLSS,
                                "NVIDIA DLSS"
                            );
                            ui.selectable_value(
                                &mut self.profile.upscaling_technology,
                                crate::ui::profile::UpscalingTechnology::Fallback,
                                "Basic Upscaling"
                            );
                        });
                });
                
                // Quality selection
                if self.profile.upscaling_technology != crate::ui::profile::UpscalingTechnology::None {
                    ui.horizontal(|ui| {
                        ui.label("Quality:");
                        egui::ComboBox::from_id_source("quality_selector")
                            .selected_text(format!("{:?}", self.profile.upscaling_quality))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.profile.upscaling_quality,
                                    crate::ui::profile::UpscalingQuality::Ultra,
                                    "Ultra"
                                );
                                ui.selectable_value(
                                    &mut self.profile.upscaling_quality,
                                    crate::ui::profile::UpscalingQuality::Quality,
                                    "Quality"
                                );
                                ui.selectable_value(
                                    &mut self.profile.upscaling_quality,
                                    crate::ui::profile::UpscalingQuality::Balanced,
                                    "Balanced"
                                );
                                ui.selectable_value(
                                    &mut self.profile.upscaling_quality,
                                    crate::ui::profile::UpscalingQuality::Performance,
                                    "Performance"
                                );
                            });
                    });
                }
            });
    }
    
    /// Show output settings section
    fn show_output_section(&mut self, ui: &mut Ui) {
        egui::Frame::group(ui.style())
            .fill(card_frame().fill)
            .rounding(card_frame().rounding)
            .stroke(card_frame().stroke)
            .show(ui, |ui| {
                ui.heading(RichText::new("Output").size(16.0));
                ui.separator();
                
                // FPS setting
                ui.horizontal(|ui| {
                    ui.label("Target FPS:");
                    ui.add(egui::Slider::new(&mut self.profile.fps, 15..=240));
                });
                
                // Scale setting
                ui.horizontal(|ui| {
                    ui.label("Scale Factor:");
                    ui.add(egui::Slider::new(&mut self.profile.scale_factor, 1.0..=4.0)
                        .step_by(0.1));
                });
            });
    }
    
    /// Show capture control buttons
    fn show_capture_controls(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let button_text = if self.is_capturing {
                "Stop Capture"
            } else {
                "Start Capture"
            };
            
            if ui.button(button_text).clicked() {
                self.is_capturing = !self.is_capturing;
                self.status_message = if self.is_capturing {
                    "Capture started".to_string()
                } else {
                    "Capture stopped".to_string()
                };
                self.status_message_type = if self.is_capturing {
                    StatusMessageType::Success
                } else {
                    StatusMessageType::Info
                };
            }
            
            if ui.button("Capture Frame").clicked() {
                self.status_message = "Frame captured".to_string();
                self.status_message_type = StatusMessageType::Success;
            }
            
            if ui.button("Fullscreen Mode").clicked() {
                // Launch fullscreen mode
                self.status_message = "Launching fullscreen mode".to_string();
                self.status_message_type = StatusMessageType::Info;
            }
        });
    }
    
    /// Update the profile based on UI selections
    fn update_profile_from_ui(&mut self) {
        // Update source based on selection
        self.profile.source = match self.capture_source_index {
            0 => CaptureSource::Fullscreen,
            1 => {
                if !self.available_windows.is_empty() && self.selected_window_index < self.available_windows.len() {
                    CaptureSource::Window(self.available_windows[self.selected_window_index].clone())
                } else {
                    CaptureSource::Window("No window selected".to_string())
                }
            },
            2 => CaptureSource::Region {
                x: self.region.0,
                y: self.region.1,
                width: self.region.2,
                height: self.region.3,
            },
            _ => CaptureSource::Fullscreen,
        };
    }
    
    /// Get the current capture target
    pub fn get_capture_target(&self) -> CaptureTarget {
        match self.profile.source {
            CaptureSource::Fullscreen => CaptureTarget::FullScreen,
            CaptureSource::Window(ref title) => CaptureTarget::WindowByTitle(title.clone()),
            CaptureSource::Region { x, y, width, height } => CaptureTarget::Region {
                x, y, width, height,
            },
        }
    }
    
    /// Get the profile
    pub fn profile(&self) -> &Profile {
        &self.profile
    }
    
    /// Get a mutable reference to the profile
    pub fn profile_mut(&mut self) -> &mut Profile {
        &mut self.profile
    }
    
    /// Check if region dialog should be shown
    pub fn show_region_dialog(&self) -> bool {
        self.show_region_dialog
    }
    
    /// Set the show_region_dialog flag
    pub fn set_show_region_dialog(&mut self, show: bool) {
        self.show_region_dialog = show;
    }
    
    /// Set the region parameters
    pub fn set_region(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.region = (x, y, width, height);
        self.update_profile_from_ui();
    }
} 