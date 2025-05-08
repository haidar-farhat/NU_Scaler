use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use dirs;

use super::profile::Profile;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Current profile
    pub current_profile: String,
    /// Theme (dark/light)
    pub theme: String,
    /// Auto-save captured frames
    pub auto_save_frames: bool,
    /// Show FPS counter
    pub show_fps_counter: bool,
    /// Show notifications
    pub show_notifications: bool,
    /// Toggle capture hotkey
    pub toggle_capture_hotkey: String,
    /// Capture frame hotkey
    pub capture_frame_hotkey: String,
    /// Toggle overlay hotkey
    pub toggle_overlay_hotkey: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            current_profile: "Default".to_string(),
            theme: "dark".to_string(),
            auto_save_frames: false,
            show_fps_counter: true,
            show_notifications: true,
            toggle_capture_hotkey: "Ctrl+Shift+C".to_string(),
            capture_frame_hotkey: "Ctrl+Shift+F".to_string(),
            toggle_overlay_hotkey: "Ctrl+Shift+O".to_string(),
        }
    }
}

impl AppSettings {
    /// Get settings file path
    pub fn get_settings_file() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not find config directory"))?;
        let settings_dir = config_dir.join("nu_scale");
        
        // Create directory if it doesn't exist
        if !settings_dir.exists() {
            fs::create_dir_all(&settings_dir)?;
        }
        
        Ok(settings_dir.join("settings.json"))
    }
    
    /// Load settings from disk
    pub fn load() -> Result<Self> {
        let settings_file = Self::get_settings_file()?;
        
        if settings_file.exists() {
            let json = fs::read_to_string(settings_file)?;
            let settings = serde_json::from_str(&json)?;
            Ok(settings)
        } else {
            // Create default settings
            let settings = Self::default();
            settings.save()?;
            Ok(settings)
        }
    }
    
    /// Save settings to disk
    pub fn save(&self) -> Result<()> {
        let settings_file = Self::get_settings_file()?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(settings_file, json)?;
        Ok(())
    }
    
    /// Get the current profile
    pub fn get_current_profile(&self) -> Result<Profile> {
        Ok(Profile::default())
    }
    
    /// Set the current profile
    pub fn set_current_profile(&mut self, name: &str) -> Result<()> {
        self.current_profile = name.to_string();
        self.save()
    }
} 