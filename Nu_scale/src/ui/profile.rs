use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use dirs;
use std::fmt;

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

// Implement Display for UpscalingTechnology
impl fmt::Display for UpscalingTechnology {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpscalingTechnology::None => write!(f, "Auto"),
            UpscalingTechnology::FSR => write!(f, "AMD FSR"),
            UpscalingTechnology::DLSS => write!(f, "NVIDIA DLSS"),
            UpscalingTechnology::Fallback => write!(f, "Fallback/Basic"),
            UpscalingTechnology::Custom => write!(f, "GPU (Vulkan)"),
        }
    }
}

/// Upscaling quality
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

// Implement Display for UpscalingQuality
impl fmt::Display for UpscalingQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpscalingQuality::Ultra => write!(f, "Ultra Quality"),
            UpscalingQuality::Quality => write!(f, "Quality"),
            UpscalingQuality::Balanced => write!(f, "Balanced"),
            UpscalingQuality::Performance => write!(f, "Performance"),
        }
    }
}

/// A profile for capturing and upscaling settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Name of the profile
    pub name: String,
    /// Capture source (0 = full screen, 1 = window, 2 = region)
    pub capture_source: usize,
    /// Window title (for window capture)
    pub window_title: String,
    /// Region coordinates (for region capture)
    pub region_x: i32,
    pub region_y: i32,
    pub region_width: u32,
    pub region_height: u32,
    /// Upscaling technology (0 = auto, 1 = FSR, 2 = DLSS, 3 = basic)
    pub upscaling_tech: usize,
    /// Upscaling quality (0 = ultra quality, 1 = quality, 2 = balanced, 3 = performance)
    pub upscaling_quality: usize,
    /// Upscaling algorithm (0 = lanczos, 1 = bicubic, 2 = bilinear, 3 = nearest)
    pub upscaling_algorithm: usize,
    /// Target FPS
    pub fps: f32,
    /// Scale factor
    pub scale_factor: f32,
    /// Post-processing options
    pub sharpening: bool,
    pub sharpening_amount: f32,
    pub noise_reduction: bool,
    pub noise_reduction_amount: f32,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            capture_source: 0,
            window_title: String::new(),
            region_x: 0,
            region_y: 0,
            region_width: 1280,
            region_height: 720,
            upscaling_tech: 0,
            upscaling_quality: 1,
            upscaling_algorithm: 0,
            fps: 60.0,
            scale_factor: 2.0,
            sharpening: true,
            sharpening_amount: 0.5,
            noise_reduction: false,
            noise_reduction_amount: 0.3,
        }
    }
}

impl Profile {
    /// Create a new profile with the given name
    pub fn new(name: &str) -> Self {
        let mut profile = Self::default();
        profile.name = name.to_string();
        profile
    }

    /// Save the profile to a file
    pub fn save(&self, path: Option<&str>) -> Result<(), std::io::Error> {
        let file_name = format!("{}.json", self.name);
        let path = path.unwrap_or(&file_name);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }

    /// Load a profile from a file
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let profile = serde_json::from_str(&json)?;
        Ok(profile)
    }

    /// Load all profiles from a directory
    pub fn load_all(dir: &str) -> Vec<Self> {
        let mut profiles = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(extension) = entry.path().extension() {
                            if extension == "json" {
                                if let Ok(profile) = Self::load(entry.path().to_str().unwrap_or_default()) {
                                    profiles.push(profile);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if profiles.is_empty() {
            profiles.push(Self::default());
        }
        
        profiles
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