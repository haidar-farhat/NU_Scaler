use anyhow::{Result, anyhow};
use image::{RgbaImage, Rgba};
use std::slice;
use std::collections::HashMap;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::composite::{ConnectionExt as CompositeConnectionExt, RedirectAutomatic};
use x11rb::protocol::xfixes::{ConnectionExt as XfixesConnectionExt};
use x11rb::protocol::randr::{ConnectionExt as RandrConnectionExt};
use x11rb::image::{Image, ImageFormat};
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;
use once_cell::sync::Lazy;

use crate::capture::{ScreenCapture, CaptureTarget, CaptureError};
use super::{WindowId, WindowInfo, WindowGeometry};

/// Linux (X11) implementation of screen capture
pub struct PlatformScreenCapture {
    /// X11 connection
    conn: std::sync::Arc<x11rb::xcb_ffi::XCBConnection>,
    /// Root window
    root: Window,
    /// Screen dimensions
    dimensions: (u32, u32),
    /// List of windows cached from last enumeration
    cached_windows: Vec<WindowInfo>,
    /// Map of window IDs to window information for quick lookup
    window_map: HashMap<u32, WindowInfo>,
    /// Atoms for window properties
    atoms: Atoms,
}

/// X11 atoms used for various window properties
#[derive(Debug, Clone)]
struct Atoms {
    /// WM_CLASS atom
    wm_class: Atom,
    /// _NET_WM_NAME atom
    net_wm_name: Atom,
    /// WM_NAME atom
    wm_name: Atom,
    /// _NET_WM_STATE atom
    net_wm_state: Atom,
    /// _NET_WM_STATE_FULLSCREEN atom
    net_wm_state_fullscreen: Atom,
    /// _NET_WM_WINDOW_TYPE atom
    net_wm_window_type: Atom,
    /// _NET_WM_WINDOW_TYPE_NORMAL atom
    net_wm_window_type_normal: Atom,
    /// UTF8_STRING atom
    utf8_string: Atom,
}

impl PlatformScreenCapture {
    /// Initialize X11 atoms
    fn init_atoms(&self) -> Result<Atoms> {
        let wm_class = self.conn.intern_atom(false, b"WM_CLASS")?.reply()?.atom;
        let net_wm_name = self.conn.intern_atom(false, b"_NET_WM_NAME")?.reply()?.atom;
        let wm_name = self.conn.intern_atom(false, b"WM_NAME")?.reply()?.atom;
        let net_wm_state = self.conn.intern_atom(false, b"_NET_WM_STATE")?.reply()?.atom;
        let net_wm_state_fullscreen = self.conn.intern_atom(false, b"_NET_WM_STATE_FULLSCREEN")?.reply()?.atom;
        let net_wm_window_type = self.conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE")?.reply()?.atom;
        let net_wm_window_type_normal = self.conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_NORMAL")?.reply()?.atom;
        let utf8_string = self.conn.intern_atom(false, b"UTF8_STRING")?.reply()?.atom;
        
        Ok(Atoms {
            wm_class,
            net_wm_name,
            wm_name,
            net_wm_state,
            net_wm_state_fullscreen,
            net_wm_window_type,
            net_wm_window_type_normal,
            utf8_string,
        })
    }
    
    /// Get window title using _NET_WM_NAME or WM_NAME
    fn get_window_title(&self, window: Window) -> Result<String> {
        // Try _NET_WM_NAME first (UTF-8)
        let net_wm_name_cookie = self.conn.get_property(
            false,
            window,
            self.atoms.net_wm_name,
            self.atoms.utf8_string,
            0,
            1024,
        )?;
        
        if let Ok(net_wm_name_reply) = net_wm_name_cookie.reply() {
            if net_wm_name_reply.value.len() > 0 {
                if let Ok(title) = String::from_utf8(net_wm_name_reply.value) {
                    return Ok(title);
                }
            }
        }
        
        // Fall back to WM_NAME
        let wm_name_cookie = self.conn.get_property(
            false,
            window,
            self.atoms.wm_name,
            AtomEnum::STRING,
            0,
            1024,
        )?;
        
        if let Ok(wm_name_reply) = wm_name_cookie.reply() {
            if wm_name_reply.value.len() > 0 {
                return Ok(String::from_utf8_lossy(&wm_name_reply.value).to_string());
            }
        }
        
        Ok(String::new())
    }
    
    /// Get window class using WM_CLASS
    fn get_window_class(&self, window: Window) -> Result<Option<String>> {
        let class_cookie = self.conn.get_property(
            false,
            window,
            self.atoms.wm_class,
            AtomEnum::STRING,
            0,
            1024,
        )?;
        
        if let Ok(class_reply) = class_cookie.reply() {
            if class_reply.value.len() > 0 {
                // WM_CLASS returns two null-terminated strings
                // We'll take the second one which is the class name
                let null_pos = class_reply.value.iter().position(|&b| b == 0);
                if let Some(pos) = null_pos {
                    if pos + 1 < class_reply.value.len() {
                        let class = String::from_utf8_lossy(&class_reply.value[pos + 1..]).to_string();
                        // Remove null terminator if present
                        let class = class.trim_matches(char::from(0));
                        return Ok(Some(class.to_string()));
                    }
                }
                return Ok(Some(String::from_utf8_lossy(&class_reply.value).to_string()));
            }
        }
        
        Ok(None)
    }
    
    /// Check if window is in fullscreen state
    fn is_fullscreen(&self, window: Window) -> Result<bool> {
        let state_cookie = self.conn.get_property(
            false,
            window,
            self.atoms.net_wm_state,
            AtomEnum::ATOM,
            0,
            1024,
        )?;
        
        if let Ok(state_reply) = state_cookie.reply() {
            if state_reply.value.len() > 0 {
                let atoms = state_reply.value.chunks(4).map(|c| {
                    u32::from_ne_bytes([c[0], c[1], c[2], c[3]])
                });
                
                for atom in atoms {
                    if atom == self.atoms.net_wm_state_fullscreen {
                        return Ok(true);
                    }
                }
            }
        }
        
        Ok(false)
    }
    
    /// Check if window is visible
    fn is_window_visible(&self, window: Window) -> Result<bool> {
        // Get window attributes
        let attr_cookie = self.conn.get_window_attributes(window)?;
        if let Ok(attr_reply) = attr_cookie.reply() {
            // Check if window is mapped and not InputOnly
            return Ok(attr_reply.map_state == MapState::VIEWABLE 
                   && attr_reply.class != WindowClass::INPUT_ONLY);
        }
        
        Ok(false)
    }
    
    /// Get window geometry
    fn get_window_geometry(&self, window: Window) -> Result<WindowGeometry> {
        let geom_cookie = self.conn.get_geometry(window)?;
        let geom_reply = geom_cookie.reply()?;
        
        // Get window position relative to root window
        let translate_cookie = self.conn.translate_coordinates(
            window,
            self.root,
            0, 0,
        )?;
        let translate_reply = translate_cookie.reply()?;
        
        let x = translate_reply.dst_x;
        let y = translate_reply.dst_y;
        
        Ok(WindowGeometry::new(
            x as i32,
            y as i32,
            geom_reply.width as u32,
            geom_reply.height as u32,
        ))
    }
    
    /// Update window list
    fn update_window_list(&mut self) -> Result<()> {
        self.cached_windows.clear();
        self.window_map.clear();
        
        // Get the Window ID of the root window
        let tree_cookie = self.conn.query_tree(self.root)?;
        let tree_reply = tree_cookie.reply()?;
        
        for &window in &tree_reply.children {
            // Skip certain windows that are not relevant
            
            // Check if window is visible
            if !self.is_window_visible(window)? {
                continue;
            }
            
            // Get window title
            let title = self.get_window_title(window)?;
            if title.is_empty() {
                continue;
            }
            
            // Get window class
            let class = self.get_window_class(window)?;
            
            // Get window geometry
            let geometry = self.get_window_geometry(window)?;
            
            // Check if window is fullscreen
            let fullscreen = self.is_fullscreen(window)?;
            
            // Create WindowInfo
            let window_info = WindowInfo {
                id: WindowId::X11(window),
                title,
                class,
                geometry,
                visible: true,
                minimized: false, // TODO: Check if window is minimized
                fullscreen,
            };
            
            self.cached_windows.push(window_info.clone());
            self.window_map.insert(window, window_info);
        }
        
        Ok(())
    }
    
    /// Find window by title (partial match)
    fn find_window_by_title(&self, title: &str) -> Result<WindowInfo> {
        let title_lower = title.to_lowercase();
        self.cached_windows.iter()
            .find(|w| w.title.to_lowercase().contains(&title_lower))
            .cloned()
            .ok_or_else(|| anyhow!(CaptureError::WindowNotFound))
    }
    
    /// Find window by ID
    fn find_window_by_id(&self, id: &WindowId) -> Result<WindowInfo> {
        match id {
            WindowId::X11(window_id) => {
                self.window_map.get(window_id)
                    .cloned()
                    .ok_or_else(|| anyhow!(CaptureError::WindowNotFound))
            },
            _ => Err(anyhow!(CaptureError::InvalidParameters)),
        }
    }
    
    /// Capture a window by Window ID
    fn capture_window(&self, window: Window) -> Result<RgbaImage> {
        // Get window geometry
        let geom_cookie = self.conn.get_geometry(window)?;
        let geom_reply = geom_cookie.reply()?;
        
        let width = geom_reply.width as u32;
        let height = geom_reply.height as u32;
        
        // Get window image
        let image_cookie = self.conn.get_image(
            ImageFormat::Z_PIXMAP,
            window,
            0, 0,
            width, height,
            !0,
        )?;
        
        let image_reply = image_cookie.reply()?;
        let data = image_reply.data;
        
        // Create RgbaImage
        let mut rgba_image = RgbaImage::new(width, height);
        
        // Process image data based on depth and format
        let bpp = match image_reply.depth {
            24 => 4, // 32-bit RGB(A)
            16 => 2, // 16-bit RGB
            8 => 1,  // 8-bit indexed
            _ => return Err(anyhow!(CaptureError::CaptureFailed(format!(
                "Unsupported image depth: {}", image_reply.depth
            )))),
        };
        
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * bpp) as usize;
                if idx + 3 < data.len() {
                    // Assuming format is BGRA or BGR
                    let blue = data[idx];
                    let green = data[idx + 1];
                    let red = data[idx + 2];
                    let alpha = if bpp == 4 { data[idx + 3] } else { 255 };
                    
                    rgba_image.put_pixel(x, y, Rgba([red, green, blue, alpha]));
                }
            }
        }
        
        Ok(rgba_image)
    }
    
    /// Capture the root window (full screen)
    fn capture_screen(&self) -> Result<RgbaImage> {
        self.capture_window(self.root)
    }
    
    /// Capture a specific region of the screen
    fn capture_region(&self, x: i32, y: i32, width: u32, height: u32) -> Result<RgbaImage> {
        // Get image for the specified region
        let image_cookie = self.conn.get_image(
            ImageFormat::Z_PIXMAP,
            self.root,
            x, y,
            width, height,
            !0,
        )?;
        
        let image_reply = image_cookie.reply()?;
        let data = image_reply.data;
        
        // Create RgbaImage
        let mut rgba_image = RgbaImage::new(width, height);
        
        // Process image data based on depth and format
        let bpp = match image_reply.depth {
            24 => 4, // 32-bit RGB(A)
            16 => 2, // 16-bit RGB
            8 => 1,  // 8-bit indexed
            _ => return Err(anyhow!(CaptureError::CaptureFailed(format!(
                "Unsupported image depth: {}", image_reply.depth
            )))),
        };
        
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * bpp) as usize;
                if idx + 3 < data.len() {
                    // Assuming format is BGRA or BGR
                    let blue = data[idx];
                    let green = data[idx + 1];
                    let red = data[idx + 2];
                    let alpha = if bpp == 4 { data[idx + 3] } else { 255 };
                    
                    rgba_image.put_pixel(x, y, Rgba([red, green, blue, alpha]));
                }
            }
        }
        
        Ok(rgba_image)
    }
}

impl ScreenCapture for PlatformScreenCapture {
    fn new() -> Result<Self> {
        // Connect to X server
        let (conn, screen_num) = x11rb::connect(None)?;
        let conn = std::sync::Arc::new(conn);
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;
        
        // Get screen dimensions
        let dimensions = (screen.width_in_pixels as u32, screen.height_in_pixels as u32);
        
        // Create capturer
        let mut capturer = Self {
            conn,
            root,
            dimensions,
            cached_windows: Vec::new(),
            window_map: HashMap::new(),
            atoms: Atoms {
                wm_class: 0,
                net_wm_name: 0,
                wm_name: 0,
                net_wm_state: 0,
                net_wm_state_fullscreen: 0,
                net_wm_window_type: 0,
                net_wm_window_type_normal: 0,
                utf8_string: 0,
            },
        };
        
        // Initialize atoms
        capturer.atoms = capturer.init_atoms()?;
        
        // Initialize window list
        capturer.update_window_list()?;
        
        Ok(capturer)
    }
    
    fn capture_frame(&mut self, target: &CaptureTarget) -> Result<RgbaImage> {
        match target {
            CaptureTarget::FullScreen => self.capture_screen(),
            CaptureTarget::WindowByTitle(title) => {
                let window_info = self.find_window_by_title(title)?;
                match window_info.id {
                    WindowId::X11(window_id) => self.capture_window(window_id),
                    _ => Err(anyhow!(CaptureError::InvalidParameters)),
                }
            },
            CaptureTarget::WindowById(id) => {
                let window_info = self.find_window_by_id(id)?;
                match window_info.id {
                    WindowId::X11(window_id) => self.capture_window(window_id),
                    _ => Err(anyhow!(CaptureError::InvalidParameters)),
                }
            },
            CaptureTarget::Region { x, y, width, height } => {
                self.capture_region(*x, *y, *width, *height)
            },
        }
    }
    
    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        Ok(self.cached_windows.clone())
    }
    
    fn get_primary_screen_dimensions(&self) -> Result<(u32, u32)> {
        Ok(self.dimensions)
    }
} 