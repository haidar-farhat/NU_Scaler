use egui::{Ui, RichText};
use anyhow::Result;

use crate::ui::components::card_frame;
use crate::ui::settings::AppSettings;
use crate::ui::profile::Profile;

/// Settings tab implementation
pub struct SettingsTab {
    /// Application settings
    settings: AppSettings,
    /// Available profiles
    available_profiles: Vec<String>,
    /// Current profile index
    current_profile_index: usize,
    /// New profile name input
    new_profile_name: String,
    /// Settings were modified flag
    modified: bool,
}

impl SettingsTab {
    /// Create a new settings tab
    pub fn new(
        settings: AppSettings,
        available_profiles: Vec<String>,
    ) -> Self {
        // Find the index of the current profile
        let current_profile_name = settings.current_profile.clone();
        let current_profile_index = available_profiles
            .iter()
            .position(|profile| profile == &current_profile_name)
            .unwrap_or(0);
        
        Self {
            settings,
            available_profiles,
            current_profile_index,
            new_profile_name: String::new(),
            modified: false,
        }
    }
    
    /// Show the settings tab UI
    pub fn show(&mut self, ui: &mut Ui) -> Result<()> {
        ui.heading("Settings");
        
        ui.add_space(10.0);
        
        // Profile selection
        self.show_profile_section(ui);
        
        ui.add_space(10.0);
        
        // Interface settings
        self.show_interface_section(ui);
        
        ui.add_space(10.0);
        
        // Hotkey settings
        self.show_hotkey_section(ui);
        
        ui.add_space(20.0);
        
        // Save and reset buttons
        ui.horizontal(|ui| {
            if ui.button("Save Settings").clicked() {
                self.save_settings()?;
            }
            
            if ui.button("Reset to Defaults").clicked() {
                self.reset_settings();
            }
        });
        
        Ok(())
    }
    
    /// Show profile selection section
    fn show_profile_section(&mut self, ui: &mut Ui) {
        card_frame().show(ui, |ui| {
            ui.strong(RichText::new("Profiles").size(16.0));
            ui.separator();
            
            // Profile selector
            ui.horizontal(|ui| {
                ui.label("Current Profile:");
                egui::ComboBox::from_id_source("profile_selector")
                    .selected_text(if self.available_profiles.is_empty() {
                        "Default Profile"
                    } else if self.current_profile_index < self.available_profiles.len() {
                        &self.available_profiles[self.current_profile_index]
                    } else {
                        "Select a profile"
                    })
                    .show_ui(ui, |ui| {
                        for (i, profile) in self.available_profiles.iter().enumerate() {
                            ui.selectable_value(
                                &mut self.current_profile_index,
                                i,
                                profile
                            );
                        }
                    });
                
                if !self.available_profiles.is_empty() && self.current_profile_index < self.available_profiles.len() {
                    if ui.button("Delete").clicked() {
                        // Delete the selected profile
                        if let Some(profile_name) = self.available_profiles.get(self.current_profile_index) {
                            let _ = Profile::delete_profile(profile_name);
                            
                            // Reload available profiles
                            self.reload_profiles();
                            
                            // Mark as modified
                            self.modified = true;
                        }
                    }
                }
            });
            
            // Create new profile
            ui.horizontal(|ui| {
                ui.label("New Profile:");
                ui.text_edit_singleline(&mut self.new_profile_name);
                
                if ui.button("Create").clicked() && !self.new_profile_name.is_empty() {
                    // Create a new profile
                    let mut profile = Profile::default();
                    profile.name = self.new_profile_name.clone();
                    
                    // Save the new profile
                    if profile.save().is_ok() {
                        // Reload available profiles
                        self.reload_profiles();
                        
                        // Clear the input field
                        self.new_profile_name.clear();
                        
                        // Mark as modified
                        self.modified = true;
                    }
                }
            });
        });
    }
    
    /// Show interface settings section
    fn show_interface_section(&mut self, ui: &mut Ui) {
        card_frame().show(ui, |ui| {
            ui.strong(RichText::new("Interface").size(16.0));
            ui.separator();
            
            // Theme selection
            ui.horizontal(|ui| {
                ui.label("Theme:");
                egui::ComboBox::from_id_source("theme_selector")
                    .selected_text(match self.settings.theme.as_str() {
                        "dark" => "Dark",
                        "light" => "Light",
                        _ => "Dark",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.settings.theme,
                            "dark".to_string(),
                            "Dark"
                        );
                        ui.selectable_value(
                            &mut self.settings.theme,
                            "light".to_string(),
                            "Light"
                        );
                    });
            });
            
            // Auto-save captured frames
            ui.checkbox(&mut self.settings.auto_save_frames, "Auto-save captured frames");
            
            // Show FPS counter
            ui.checkbox(&mut self.settings.show_fps_counter, "Show FPS counter");
            
            // Show notifications
            ui.checkbox(&mut self.settings.show_notifications, "Show notifications");
        });
    }
    
    /// Show hotkey settings section
    fn show_hotkey_section(&mut self, ui: &mut Ui) {
        card_frame().show(ui, |ui| {
            ui.strong(RichText::new("Hotkeys").size(16.0));
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Toggle Capture:");
                ui.text_edit_singleline(&mut self.settings.toggle_capture_hotkey);
            });
            
            ui.horizontal(|ui| {
                ui.label("Capture Frame:");
                ui.text_edit_singleline(&mut self.settings.capture_frame_hotkey);
            });
            
            ui.horizontal(|ui| {
                ui.label("Toggle Overlay:");
                ui.text_edit_singleline(&mut self.settings.toggle_overlay_hotkey);
            });
            
            ui.add_space(5.0);
            ui.label("Note: Restart required for hotkey changes to take effect.");
        });
    }
    
    /// Save the current settings
    fn save_settings(&mut self) -> Result<()> {
        // Update current profile from selection
        if !self.available_profiles.is_empty() && self.current_profile_index < self.available_profiles.len() {
            self.settings.current_profile = self.available_profiles[self.current_profile_index].clone();
        }
        
        // Save settings
        self.settings.save()?;
        
        // Reset modified flag
        self.modified = false;
        
        Ok(())
    }
    
    /// Reset settings to defaults
    fn reset_settings(&mut self) {
        self.settings = AppSettings::default();
        self.modified = true;
    }
    
    /// Reload available profiles
    fn reload_profiles(&mut self) {
        if let Ok(profiles) = Profile::list_profiles() {
            self.available_profiles = profiles;
            
            // Find the index of the current profile again
            let current_profile_name = self.settings.current_profile.clone();
            self.current_profile_index = self.available_profiles
                .iter()
                .position(|profile| profile == &current_profile_name)
                .unwrap_or(0);
        }
    }
} 