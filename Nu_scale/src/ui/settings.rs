use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use dirs;

use super::profile::Profile;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Last used profile
    pub last_profile: Option<String>,
    /// Start minimized
    pub start_minimized: bool,
    /// Start with system
    pub start_with_system: bool,
    /// Check for updates
    pub check_for_updates: bool,
    /// Auto-save profiles
    pub auto_save_profiles: bool,
    /// Theme (light, dark, or system)
    pub theme: String,
    /// Language
    pub language: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            last_profile: None,
            start_minimized: false,
            start_with_system: false,
            check_for_updates: true,
            auto_save_profiles: true,
            theme: "system".to_string(),
            language: "en".to_string(),
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
        match &self.last_profile {
            Some(name) => Profile::load(name),
            None => Ok(Profile::default()),
        }
    }
    
    /// Set the current profile
    pub fn set_current_profile(&mut self, name: &str) -> Result<()> {
        self.last_profile = Some(name.to_string());
        self.save()
    }
} 