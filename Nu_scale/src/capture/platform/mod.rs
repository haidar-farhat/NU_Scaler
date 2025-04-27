#[cfg(windows)]
pub mod windows;
#[cfg(unix)]
pub mod linux;

use image::RgbaImage;

/// Common platform-agnostic window identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WindowId {
    /// X11 window ID
    X11(u32),
    /// Wayland surface ID (opaque handle)
    Wayland(String),
    /// Windows HWND (as usize)
    Windows(usize),
    /// Generic ID for other platforms
    Other(String),
}

/// Information about a window
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Window identifier
    pub id: WindowId,
    /// Window title
    pub title: String,
    /// Window class or application name
    pub class: Option<String>,
    /// Window position and size
    pub geometry: WindowGeometry,
    /// Whether the window is visible
    pub visible: bool,
    /// Whether the window is minimized
    pub minimized: bool,
    /// Whether the window is fullscreen
    pub fullscreen: bool,
}

/// Window geometry
#[derive(Debug, Clone, Copy)]
pub struct WindowGeometry {
    /// X coordinate
    pub x: i32,
    /// Y coordinate
    pub y: i32,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
}

impl WindowGeometry {
    /// Create a new window geometry
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
}

/// Trait for capture backend implementations
pub trait CaptureBackend: Send + Sync {
    /// Process a captured frame, possibly applying transformations
    fn process_frame(&mut self, frame: &Option<RgbaImage>) -> Option<RgbaImage>;
    
    /// Get the name of this capture backend
    fn backend_name(&self) -> String;
} 