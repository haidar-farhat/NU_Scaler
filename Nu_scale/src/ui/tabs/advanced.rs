use egui::{Ui, RichText, Color32};
use anyhow::Result;

use crate::ui::components::card_frame;

const ACCENT_COLOR: Color32 = Color32::from_rgb(0, 120, 215); // Blue accent

/// Advanced tab implementation
pub struct AdvancedTab {
    // Advanced settings
    /// Enable experimental features
    enable_experimental: bool,
    /// Enable developer mode
    enable_developer_mode: bool,
    /// Enable API access
    enable_api_access: bool,
    /// API port
    api_port: u16,
    /// Custom upscaler path
    custom_upscaler_path: String,
    /// Logging level
    logging_level: LogLevel,
    /// Frame buffer size
    frame_buffer_size: usize,
    /// Maximum threads
    max_threads: usize,
    /// GPU memory limit (in MB)
    gpu_memory_limit: usize,
}

/// Logging level enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

impl AdvancedTab {
    /// Create a new advanced tab
    pub fn new() -> Self {
        Self {
            enable_experimental: false,
            enable_developer_mode: false,
            enable_api_access: false,
            api_port: 8080,
            custom_upscaler_path: String::new(),
            logging_level: LogLevel::default(),
            frame_buffer_size: 5,
            max_threads: num_cpus::get(),
            gpu_memory_limit: 1024, // Default 1GB
        }
    }
    
    /// Show the advanced tab UI
    pub fn show(&mut self, ui: &mut Ui) -> Result<()> {
        ui.heading("Advanced Settings");
        ui.add_space(5.0);
        ui.label("Warning: These settings are for advanced users only. Changes may affect application stability.");
        
        ui.add_space(20.0);
        
        // Feature flags
        self.show_feature_flags(ui);
        
        ui.add_space(10.0);
        
        // Performance settings
        self.show_performance_settings(ui);
        
        ui.add_space(10.0);
        
        // API settings
        self.show_api_settings(ui);
        
        ui.add_space(10.0);
        
        // Developer options
        self.show_developer_options(ui);
        
        ui.add_space(20.0);
        
        // Apply button
        if ui.button("Apply Advanced Settings").clicked() {
            // TODO: Actually apply settings
            self.apply_settings();
        }
        
        Ok(())
    }
    
    /// Show feature flags section
    fn show_feature_flags(&mut self, ui: &mut Ui) {
        card_frame().show(ui, |ui| {
            ui.strong(RichText::new("Feature Flags").size(16.0).color(ACCENT_COLOR));
            ui.add_space(5.0);
            
            ui.checkbox(&mut self.enable_experimental, "Enable experimental features");
            if self.enable_experimental {
                ui.indent("experimental", |ui| {
                    ui.label(RichText::new("Warning: Experimental features may cause instability").color(Color32::YELLOW));
                });
            }
        });
    }
    
    /// Show performance settings section
    fn show_performance_settings(&mut self, ui: &mut Ui) {
        card_frame().show(ui, |ui| {
            ui.strong(RichText::new("Performance").size(16.0).color(ACCENT_COLOR));
            ui.add_space(5.0);
            
            // Frame buffer size
            ui.horizontal(|ui| {
                ui.label("Frame buffer size:");
                ui.add(egui::Slider::new(&mut self.frame_buffer_size, 1..=30));
            });
            
            // Max threads
            ui.horizontal(|ui| {
                ui.label("Maximum threads:");
                let max_cpu = num_cpus::get();
                ui.add(egui::Slider::new(&mut self.max_threads, 1..=max_cpu).text("threads"));
                ui.label(format!("(System has {} logical cores)", max_cpu));
            });
            
            // GPU memory limit
            ui.horizontal(|ui| {
                ui.label("GPU memory limit:");
                ui.add(egui::Slider::new(&mut self.gpu_memory_limit, 512..=8192).text("MB"));
            });
        });
    }
    
    /// Show API settings section
    fn show_api_settings(&mut self, ui: &mut Ui) {
        card_frame().show(ui, |ui| {
            ui.strong(RichText::new("API Settings").size(16.0).color(ACCENT_COLOR));
            ui.add_space(5.0);
            
            ui.checkbox(&mut self.enable_api_access, "Enable API access");
            
            if self.enable_api_access {
                ui.indent("api_settings", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("API port:");
                        ui.add(egui::DragValue::new(&mut self.api_port).speed(1.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("API URL:");
                        ui.label(format!("http://localhost:{}/api", self.api_port));
                    });
                    
                    ui.label(RichText::new("Note: Restart required for API changes to take effect").color(Color32::YELLOW));
                });
            }
        });
    }
    
    /// Show developer options section
    fn show_developer_options(&mut self, ui: &mut Ui) {
        card_frame().show(ui, |ui| {
            ui.strong(RichText::new("Developer Options").size(16.0).color(ACCENT_COLOR));
            ui.add_space(5.0);
            
            ui.checkbox(&mut self.enable_developer_mode, "Enable developer mode");
            
            if self.enable_developer_mode {
                ui.indent("dev_options", |ui| {
                    // Custom upscaler path
                    ui.horizontal(|ui| {
                        ui.label("Custom upscaler path:");
                        ui.text_edit_singleline(&mut self.custom_upscaler_path);
                        if ui.button("Browse...").clicked() {
                            // Open file dialog
                            // TODO: Implement file dialog
                        }
                    });
                    
                    // Logging level
                    ui.horizontal(|ui| {
                        ui.label("Logging level:");
                        egui::ComboBox::from_id_source("logging_level")
                            .selected_text(format!("{:?}", self.logging_level))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.logging_level, LogLevel::Error, "Error");
                                ui.selectable_value(&mut self.logging_level, LogLevel::Warning, "Warning");
                                ui.selectable_value(&mut self.logging_level, LogLevel::Info, "Info");
                                ui.selectable_value(&mut self.logging_level, LogLevel::Debug, "Debug");
                                ui.selectable_value(&mut self.logging_level, LogLevel::Trace, "Trace");
                            });
                    });
                    
                    // Buttons for developer actions
                    ui.horizontal(|ui| {
                        if ui.button("Dump GPU Info").clicked() {
                            // TODO: Implement GPU info dump
                        }
                        
                        if ui.button("Clear Cache").clicked() {
                            // TODO: Implement cache clearing
                        }
                    });
                });
            }
        });
    }
    
    /// Apply the advanced settings
    pub fn apply_settings(&self) {
        // Implementation to apply advanced settings
        // Could call into settings management system
    }
} 