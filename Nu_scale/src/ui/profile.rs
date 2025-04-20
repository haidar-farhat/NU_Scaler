use serde::{Serialize, Deserialize};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use dirs;
use crate::upscale::common::UpscalingAlgorithm;

/// Source type for capture
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaptureSource {
    /// Capture fullscreen
    Fullscreen,
    /// Capture a window by title
    Window(String),
    /// Capture a specific region
    Region { x: i32, y: i32, width: u32, height: u32 },
}

/// System platform for capture
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SystemPlatform {
    /// X11 (Linux)
    X11,
    /// Wayland (Linux)
    Wayland,
    /// Windows
    Windows,
    /// Auto-detect
    Auto,
}

/// Upscaling technology
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpscalingTechnology {
    /// No upscaling
    None,
    /// AMD FidelityFX Super Resolution (FSR)
    FSR,
    /// NVIDIA Deep Learning Super Sampling (DLSS)
    DLSS,
    /// Fallback to basic algorithms
    Fallback,
    /// Custom implementation
    Custom,
}

/// Upscaling quality
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum UpscalingQuality {
    /// Ultra quality (minimal upscaling)
    Ultra,
    /// Quality (good balance)
    Quality,
    /// Balanced (medium quality, better performance)
    Balanced,
    /// Performance (focus on performance)
    Performance,
}

/// Profile for capture and upscaling settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Profile name
    pub name: String,
    /// Capture source
    pub source: CaptureSource,
    /// System platform
    pub platform: SystemPlatform,
    /// Upscaling factor (1.0 = no upscaling)
    pub scale_factor: f32,
    /// Upscaling technology
    pub upscaling_tech: UpscalingTechnology,
    /// Upscaling quality preset
    pub upscaling_quality: UpscalingQuality,
    /// Specific upscaling algorithm when using fallback technology
    pub upscaling_algorithm: Option<UpscalingAlgorithm>,
    /// Enable overlay
    pub enable_overlay: bool,
    /// Hotkey for starting/stopping capture
    pub hotkey: String,
    /// Capture FPS
    pub fps: u32,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            source: CaptureSource::Fullscreen,
            platform: SystemPlatform::Auto,
            scale_factor: 1.5,
            upscaling_tech: UpscalingTechnology::None,
            upscaling_quality: UpscalingQuality::Balanced,
            upscaling_algorithm: None, // Let quality determine the algorithm
            enable_overlay: false,
            hotkey: "Ctrl+Alt+C".to_string(),
            fps: 30,
        }
    }
}

impl Profile {
    /// Create a new profile with default settings
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
    
    /// Get profiles directory
    pub fn get_profiles_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not find config directory"))?;
        let profiles_dir = config_dir.join("nu_scale").join("profiles");
        
        // Create directory if it doesn't exist
        if !profiles_dir.exists() {
            fs::create_dir_all(&profiles_dir)?;
        }
        
        Ok(profiles_dir)
    }
    
    /// Save profile to disk
    pub fn save(&self) -> Result<()> {
        let profiles_dir = Self::get_profiles_dir()?;
        let file_path = profiles_dir.join(format!("{}.json", self.name));
        
        let json = serde_json::to_string_pretty(self)?;
        fs::write(file_path, json)?;
        
        Ok(())
    }
    
    /// Load profile from disk
    pub fn load(name: &str) -> Result<Self> {
        let profiles_dir = Self::get_profiles_dir()?;
        let file_path = profiles_dir.join(format!("{}.json", name));
        
        let json = fs::read_to_string(file_path)?;
        let profile = serde_json::from_str(&json)?;
        
        Ok(profile)
    }
    
    /// List all available profiles
    pub fn list_profiles() -> Result<Vec<String>> {
        let profiles_dir = Self::get_profiles_dir()?;
        
        let mut profiles = Vec::new();
        for entry in fs::read_dir(profiles_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Some(name) = path.file_stem() {
                    if let Some(name_str) = name.to_str() {
                        profiles.push(name_str.to_string());
                    }
                }
            }
        }
        
        Ok(profiles)
    }
    
    /// Delete profile from disk
    pub fn delete(name: &str) -> Result<()> {
        let profiles_dir = Self::get_profiles_dir()?;
        let file_path = profiles_dir.join(format!("{}.json", name));
        
        fs::remove_file(file_path)?;
        
        Ok(())
    }
} 