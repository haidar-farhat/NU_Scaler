#![allow(dead_code)]
use anyhow::{Result, anyhow};
use image::{RgbaImage, Rgba};
use std::mem::size_of;
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::UI::WindowsAndMessaging::*,
};
use crate::capture::{ScreenCapture, CaptureTarget, CaptureError};
use super::{WindowId, WindowInfo, WindowGeometry, CaptureBackend};
use once_cell::sync::Lazy;
use std::sync::Mutex;

// For WGPU implementation
use wgpu::{self, Instance, Adapter, Device, Queue};

/// Windows implementation of screen capture
pub struct PlatformScreenCapture {
    /// Device context for the screen
    screen_dc: HDC,
    /// List of windows cached from last enumeration
    cached_windows: Vec<WindowInfo>,
}

/// WGPU-based Windows capture implementation
pub struct WgpuWindowsCapture {
    /// WGPU instance
    instance: Instance,
    /// WGPU adapter
    adapter: Option<Adapter>,
    /// WGPU device
    device: Option<Device>,
    /// WGPU queue
    queue: Option<Queue>,
    /// List of windows cached from last enumeration
    cached_windows: Vec<WindowInfo>,
    /// Last captured frame for debugging
    last_frame: Option<RgbaImage>,
}

// Thread-safe window list for enumerator callback
static WINDOW_LIST: Lazy<Mutex<Vec<WindowInfo>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Callback for EnumWindows
unsafe extern "system" fn enum_windows_callback(hwnd: HWND, _: LPARAM) -> BOOL {
    let mut title = [0u16; 512];
    let mut class_name = [0u16; 512];
    
    // Check if window is visible
    if unsafe { GetWindowTextW(hwnd, &mut title) } == 0 || !unsafe { IsWindowVisible(hwnd) }.as_bool() {
        return TRUE;
    }
    
    // Get window class
    let class_len = unsafe { GetClassNameW(hwnd, &mut class_name) } as usize;
    
    // Get window rect
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect) };
    
    // Get title text
    let title_len = unsafe { GetWindowTextLengthW(hwnd) } as usize;
    let title = String::from_utf16_lossy(&title[..title_len]);
    let class = String::from_utf16_lossy(&class_name[..class_len]);
    
    // Get window placement info (for minimized state)
    let mut placement = WINDOWPLACEMENT {
        length: size_of::<WINDOWPLACEMENT>() as u32,
        ..Default::default()
    };
    unsafe { GetWindowPlacement(hwnd, &mut placement) };
    
    // Create window info
    let window_info = WindowInfo {
        id: WindowId::Windows(hwnd.0 as usize),
        title,
        class: if class.len() > 0 {
            Some(class)
        } else {
            None
        },
        geometry: WindowGeometry::new(
            rect.left, 
            rect.top, 
            (rect.right - rect.left) as u32,
            (rect.bottom - rect.top) as u32
        ),
        visible: unsafe { IsWindowVisible(hwnd) }.as_bool(),
        minimized: placement.showCmd.0 == SW_SHOWMINIMIZED.0,
        fullscreen: unsafe { is_fullscreen(hwnd) },
    };
    
    // Add to window list in a thread-safe way
    if let Ok(mut list) = WINDOW_LIST.lock() {
        list.push(window_info);
    }
    
    TRUE
}

/// Checks if a window is in fullscreen mode
unsafe fn is_fullscreen(hwnd: HWND) -> bool {
    let mut window_rect = RECT::default();
    
    if unsafe { GetWindowRect(hwnd, &mut window_rect) }.as_bool() {
        let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
        let mut monitor_info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        
        if unsafe { GetMonitorInfoW(monitor, &mut monitor_info) }.as_bool() {
            let screen_rect = monitor_info.rcMonitor;
            
            // Compare if window covers the entire monitor
            return window_rect.left == screen_rect.left
                && window_rect.top == screen_rect.top
                && window_rect.right == screen_rect.right
                && window_rect.bottom == screen_rect.bottom;
        }
    }
    
    false
}

impl PlatformScreenCapture {
    /// Finds a window by title (partial match)
    fn find_window_by_title(&self, title: &str) -> Result<WindowInfo> {
        let title_lower = title.to_lowercase();
        self.cached_windows.iter()
            .find(|w| w.title.to_lowercase().contains(&title_lower))
            .cloned()
            .ok_or_else(|| anyhow!(CaptureError::WindowNotFound))
    }
    
    /// Finds a window by ID
    fn find_window_by_id(&self, id: &WindowId) -> Result<WindowInfo> {
        match id {
            WindowId::Windows(hwnd) => {
                self.cached_windows.iter()
                    .find(|w| match w.id {
                        WindowId::Windows(h) => h == *hwnd,
                        _ => false,
                    })
                    .cloned()
                    .ok_or_else(|| anyhow!(CaptureError::WindowNotFound))
            },
            _ => Err(anyhow!(CaptureError::InvalidParameters)),
        }
    }
    
    /// Captures a window by HWND
    unsafe fn capture_window(&self, hwnd: HWND) -> Result<RgbaImage> {
        log::debug!("Capturing window with HWND: {:?}", hwnd.0);
        
        // Check if window is valid and visible
        if !unsafe { IsWindow(hwnd) }.as_bool() {
            return Err(anyhow!(CaptureError::WindowNotFound));
        }
        
        // Check if the window is minimized, as we can't capture minimized windows
        let mut placement = WINDOWPLACEMENT {
            length: size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };
        unsafe { GetWindowPlacement(hwnd, &mut placement) };
        
        if placement.showCmd.0 == SW_SHOWMINIMIZED.0 {
            log::warn!("Window is minimized, can't capture content");
            return Err(anyhow!(CaptureError::CaptureFailed("Window is minimized".into())));
        }
        
        // Get window dimensions - try both client rect and window rect
        let mut client_rect = RECT::default();
        let mut window_rect = RECT::default();
        unsafe { GetClientRect(hwnd, &mut client_rect) };
        unsafe { GetWindowRect(hwnd, &mut window_rect) };
        
        let client_width = client_rect.right as u32;
        let client_height = client_rect.bottom as u32;
        let window_width = (window_rect.right - window_rect.left) as u32;
        let window_height = (window_rect.bottom - window_rect.top) as u32;
        
        // Use the larger of the two to ensure we capture everything
        let width = client_width.max(window_width);
        let height = client_height.max(window_height);
        
        log::debug!("Window dimensions: client={}x{}, window={}x{}, using={}x{}", 
                  client_width, client_height, window_width, window_height, width, height);
        
        if width == 0 || height == 0 {
            log::warn!("Window has zero size: {}x{}", width, height);
            return Err(anyhow!(CaptureError::CaptureFailed("Window has zero size".into())));
        }
        
        // Get window DC
        let window_dc = unsafe { GetDC(hwnd) };
        if window_dc.is_invalid() {
            log::error!("Failed to get window DC");
            return Err(anyhow!(CaptureError::CaptureFailed("Failed to get window DC".into())));
        }
        
        // Get desktop DC as backup
        let desktop_dc = unsafe { GetDC(HWND(0)) };
        
        // Create compatible DC and bitmap
        let compatible_dc = unsafe { CreateCompatibleDC(window_dc) };
        if compatible_dc.is_invalid() {
            unsafe { ReleaseDC(hwnd, window_dc) };
            unsafe { ReleaseDC(HWND(0), desktop_dc) };
            log::error!("Failed to create compatible DC");
            return Err(anyhow!(CaptureError::CaptureFailed("Failed to create compatible DC".into())));
        }
        
        // Create bitmap compatible with desktop to ensure proper color depth
        let bitmap = unsafe { CreateCompatibleBitmap(desktop_dc, width as i32, height as i32) };
        if bitmap.is_invalid() {
            unsafe { DeleteDC(compatible_dc) };
            unsafe { ReleaseDC(hwnd, window_dc) };
            unsafe { ReleaseDC(HWND(0), desktop_dc) };
            log::error!("Failed to create compatible bitmap");
            return Err(anyhow!(CaptureError::CaptureFailed("Failed to create compatible bitmap".into())));
        }
        
        // Select bitmap into DC
        let old_bitmap = unsafe { SelectObject(compatible_dc, bitmap) };
        
        // Copy window content to bitmap using BitBlt
        log::debug!("Attempting to capture using BitBlt");
        if !unsafe { 
            BitBlt(
                compatible_dc,
                0, 0,
                width as i32, height as i32,
                window_dc,
                0, 0,
                SRCCOPY,
            )
        }.as_bool() {
            unsafe { SelectObject(compatible_dc, old_bitmap) };
            unsafe { DeleteObject(bitmap) };
            unsafe { DeleteDC(compatible_dc) };
            unsafe { ReleaseDC(hwnd, window_dc) };
            unsafe { ReleaseDC(HWND(0), desktop_dc) };
            log::error!("BitBlt failed");
            return Err(anyhow!(CaptureError::CaptureFailed("Window capture failed (BitBlt)".into())));
        }
        
        // Get bitmap data
        let bitmap_info = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32), // Negative height for top-down image
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0 as u32,
            ..Default::default()
        };
        
        // Create buffer for pixel data
        let mut buffer = vec![0u8; (width * height * 4) as usize];
        
        // Get bitmap bits
        let dibit_result = unsafe {
            GetDIBits(
                compatible_dc,
                bitmap,
                0,
                height,
                Some(buffer.as_mut_ptr() as *mut std::ffi::c_void),
                &mut BITMAPINFO {
                    bmiHeader: bitmap_info,
                    bmiColors: [RGBQUAD::default()],
                },
                DIB_RGB_COLORS,
            )
        };
        
        if dibit_result == 0 {
            unsafe { SelectObject(compatible_dc, old_bitmap) };
            unsafe { DeleteObject(bitmap) };
            unsafe { DeleteDC(compatible_dc) };
            unsafe { ReleaseDC(hwnd, window_dc) };
            unsafe { ReleaseDC(HWND(0), desktop_dc) };
            log::error!("GetDIBits failed");
            return Err(anyhow!(CaptureError::CaptureFailed("GetDIBits failed".into())));
        }
        
        // Clean up GDI objects
        unsafe { SelectObject(compatible_dc, old_bitmap) };
        unsafe { DeleteObject(bitmap) };
        unsafe { DeleteDC(compatible_dc) };
        unsafe { ReleaseDC(hwnd, window_dc) };
        unsafe { ReleaseDC(HWND(0), desktop_dc) };
        
        // Check if the buffer is all black
        let sample_count = 1000.min(buffer.len() / 4);
        let all_black = buffer.chunks(4)
            .take(sample_count)
            .all(|pixel| pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0);
        
        if all_black {
            log::warn!("Captured image appears to be all black (from {} samples)", sample_count);
            
            // Log some diagnostic information
            let foreground_hwnd = unsafe { GetForegroundWindow() };
            log::debug!("Current foreground window: {:?}, target window: {:?}", foreground_hwnd.0, hwnd.0);
            
            // Check if this is a layered window
            let style = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) };
            let is_layered = (style & (WS_EX_LAYERED.0 as i32)) != 0;
            log::debug!("Window has layered style: {}", is_layered);
        }
        
        // Convert the buffer to an image
        let mut image = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                if idx + 3 < buffer.len() {
                    let blue = buffer[idx];
                    let green = buffer[idx + 1];
                    let red = buffer[idx + 2];
                    let alpha = buffer[idx + 3];
                    
                    image.put_pixel(x, y, Rgba([red, green, blue, alpha]));
                }
            }
        }
        
        log::debug!("Window capture completed successfully: {}x{}", width, height);
        Ok(image)
    }
    
    /// Captures the full screen
    unsafe fn capture_screen(&self) -> Result<RgbaImage> {
        // Get screen dimensions
        let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        
        // Create compatible DC and bitmap
        let compatible_dc = unsafe { CreateCompatibleDC(self.screen_dc) };
        if compatible_dc.is_invalid() {
            return Err(anyhow!(CaptureError::CaptureFailed("Failed to create compatible DC".into())));
        }
        
        let bitmap = unsafe { CreateCompatibleBitmap(self.screen_dc, screen_width, screen_height) };
        if bitmap.is_invalid() {
            unsafe { DeleteDC(compatible_dc) };
            return Err(anyhow!(CaptureError::CaptureFailed("Failed to create compatible bitmap".into())));
        }
        
        // Select bitmap into DC
        let old_bitmap = unsafe { SelectObject(compatible_dc, bitmap) };
        
        // Copy screen content to bitmap
        if !unsafe {
            BitBlt(
                compatible_dc,
                0, 0,
                screen_width, screen_height,
                self.screen_dc,
                0, 0,
                SRCCOPY,
            )
        }.as_bool() {
            unsafe { SelectObject(compatible_dc, old_bitmap) };
            unsafe { DeleteObject(bitmap) };
            unsafe { DeleteDC(compatible_dc) };
            return Err(anyhow!(CaptureError::CaptureFailed("BitBlt failed".into())));
        }
        
        // Get bitmap data
        let bitmap_info = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: screen_width,
            biHeight: -screen_height, // Negative height for top-down image
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0 as u32,
            ..Default::default()
        };
        
        // Create buffer for pixel data
        let width = screen_width as u32;
        let height = screen_height as u32;
        let mut buffer = vec![0u8; (width * height * 4) as usize];
        
        // Get bitmap bits
        if unsafe {
            GetDIBits(
                compatible_dc,
                bitmap,
                0,
                height,
                Some(buffer.as_mut_ptr() as *mut std::ffi::c_void),
                &mut BITMAPINFO {
                    bmiHeader: bitmap_info,
                    bmiColors: [RGBQUAD::default()],
                },
                DIB_RGB_COLORS,
            )
        } == 0 {
            unsafe { SelectObject(compatible_dc, old_bitmap) };
            unsafe { DeleteObject(bitmap) };
            unsafe { DeleteDC(compatible_dc) };
            return Err(anyhow!(CaptureError::CaptureFailed("GetDIBits failed".into())));
        }
        
        // Clean up
        unsafe { SelectObject(compatible_dc, old_bitmap) };
        unsafe { DeleteObject(bitmap) };
        unsafe { DeleteDC(compatible_dc) };
        
        // Convert to image::RgbaImage (BGRA -> RGBA)
        let mut image = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                let blue = buffer[idx];
                let green = buffer[idx + 1];
                let red = buffer[idx + 2];
                let alpha = buffer[idx + 3];
                
                image.put_pixel(x, y, Rgba([red, green, blue, alpha]));
            }
        }
        
        Ok(image)
    }
    
    /// Captures a specific region of the screen
    unsafe fn capture_region(&self, x: i32, y: i32, width: u32, height: u32) -> Result<RgbaImage> {
        // Create compatible DC and bitmap
        let compatible_dc = unsafe { CreateCompatibleDC(self.screen_dc) };
        if compatible_dc.is_invalid() {
            return Err(anyhow!(CaptureError::CaptureFailed("Failed to create compatible DC".into())));
        }
        
        let bitmap = unsafe { CreateCompatibleBitmap(self.screen_dc, width as i32, height as i32) };
        if bitmap.is_invalid() {
            unsafe { DeleteDC(compatible_dc) };
            return Err(anyhow!(CaptureError::CaptureFailed("Failed to create compatible bitmap".into())));
        }
        
        // Select bitmap into DC
        let old_bitmap = unsafe { SelectObject(compatible_dc, bitmap) };
        
        // Copy screen content to bitmap
        if !unsafe { BitBlt(
            compatible_dc,
            0, 0,
            width as i32, height as i32,
            self.screen_dc,
            x, y,
            SRCCOPY,
        ) }.as_bool() {
            unsafe { SelectObject(compatible_dc, old_bitmap) };
            unsafe { DeleteObject(bitmap) };
            unsafe { DeleteDC(compatible_dc) };
            return Err(anyhow!(CaptureError::CaptureFailed("BitBlt failed".into())));
        }
        
        // Get bitmap data
        let bitmap_info = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32), // Negative height for top-down image
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0 as u32,
            ..Default::default()
        };
        
        // Create buffer for pixel data
        let mut buffer = vec![0u8; (width * height * 4) as usize];
        
        // Get bitmap bits
        if unsafe { GetDIBits(
            compatible_dc,
            bitmap,
            0,
            height,
            Some(buffer.as_mut_ptr() as *mut std::ffi::c_void),
            &mut BITMAPINFO {
                bmiHeader: bitmap_info,
                bmiColors: [RGBQUAD::default()],
            },
            DIB_RGB_COLORS,
        ) } == 0 {
            unsafe { SelectObject(compatible_dc, old_bitmap) };
            unsafe { DeleteObject(bitmap) };
            unsafe { DeleteDC(compatible_dc) };
            return Err(anyhow!(CaptureError::CaptureFailed("GetDIBits failed".into())));
        }
        
        // Clean up
        unsafe { SelectObject(compatible_dc, old_bitmap) };
        unsafe { DeleteObject(bitmap) };
        unsafe { DeleteDC(compatible_dc) };
        
        // Convert to image::RgbaImage (BGRA -> RGBA)
        let mut image = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                let blue = buffer[idx];
                let green = buffer[idx + 1];
                let red = buffer[idx + 2];
                let alpha = buffer[idx + 3];
                
                image.put_pixel(x, y, Rgba([red, green, blue, alpha]));
            }
        }
        
        Ok(image)
    }
    
    /// Updates the cached window list
    fn update_window_list(&mut self) -> Result<()> {
        unsafe {
            WINDOW_LIST.lock().unwrap().clear();
            EnumWindows(Some(enum_windows_callback), LPARAM(0));
            if let Ok(list) = WINDOW_LIST.lock() {
                self.cached_windows = list.clone();
            }
        }
        Ok(())
    }
}

impl ScreenCapture for PlatformScreenCapture {
    fn new() -> Result<Self> {
        unsafe {
            let screen_dc = GetDC(HWND(0));
            if screen_dc.is_invalid() {
                return Err(anyhow!(CaptureError::CaptureFailed("Failed to get screen DC".into())));
            }
            
            let mut capturer = Self {
                screen_dc,
                cached_windows: Vec::new(),
            };
            
            capturer.update_window_list()?;
            
            Ok(capturer)
        }
    }
    
    fn capture_frame(&mut self, target: &CaptureTarget) -> Result<RgbaImage> {
        unsafe {
            match target {
                CaptureTarget::FullScreen => self.capture_screen(),
                CaptureTarget::WindowByTitle(title) => {
                    let window_info = self.find_window_by_title(title)?;
                    match window_info.id {
                        WindowId::Windows(hwnd) => self.capture_window(HWND(hwnd as isize)),
                        _ => Err(anyhow!(CaptureError::InvalidParameters)),
                    }
                },
                CaptureTarget::WindowById(id) => {
                    let window_info = self.find_window_by_id(id)?;
                    match window_info.id {
                        WindowId::Windows(hwnd) => self.capture_window(HWND(hwnd as isize)),
                        _ => Err(anyhow!(CaptureError::InvalidParameters)),
                    }
                },
                CaptureTarget::Region { x, y, width, height } => {
                    self.capture_region(*x, *y, *width, *height)
                },
            }
        }
    }
    
    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        Ok(self.cached_windows.clone())
    }
    
    fn get_primary_screen_dimensions(&self) -> Result<(u32, u32)> {
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN) as u32;
            let height = GetSystemMetrics(SM_CYSCREEN) as u32;
            Ok((width, height))
        }
    }
}

// WGPU implementation
impl WgpuWindowsCapture {
    /// Creates a new WGPU Windows capture
    pub fn new() -> Result<Self> {
        // Create WGPU instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::empty(),
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });
        
        // Initialize cached windows
        let mut _cached_windows = Vec::new();
        unsafe {
            WINDOW_LIST.lock().unwrap().clear();
            EnumWindows(Some(enum_windows_callback), LPARAM(0));
            _cached_windows = WINDOW_LIST.lock().unwrap().clone();
        }
        
        // We'll initialize adapter, device, queue when needed
        Ok(Self {
            instance,
            adapter: None,
            device: None,
            queue: None,
            cached_windows: _cached_windows,
            last_frame: None,
        })
    }
    
    /// Initialize WGPU resources (now public so you can call it from `main`)
    pub async fn initialize_wgpu(&mut self) -> Result<()> {
        if self.adapter.is_none() {
            // Request adapter
            self.adapter = self.instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            }).await;
            
            if let Some(adapter) = &self.adapter {
                // Request device
                let (device, queue) = adapter.request_device(
                    &wgpu::DeviceDescriptor {
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                        label: Some("NU_Scaler Capture Device"),
                    },
                    None,
                ).await?;
                
                self.device = Some(device);
                self.queue = Some(queue);
            }
        }
        
        Ok(())
    }
    
    /// Capture a window with WGPU
    async fn capture_window_wgpu(&mut self, hwnd: HWND) -> Result<RgbaImage> {
        // Initialize WGPU if not already done
        self.initialize_wgpu().await?;
        
        if self.device.is_none() || self.queue.is_none() {
            return Err(anyhow!("WGPU not initialized"));
        }
        
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        
        // Create a surface for the window
        // In a real implementation, this would use the Windows HWND to create a surface
        // But for simplicity, we'll capture using GDI and then use WGPU for processing
        
        // First capture with GDI
        let gdi_capture = unsafe {
            let capture = PlatformScreenCapture::new()?;
            capture.capture_window(hwnd)
        }?;
        
        // Process with WGPU (in a real implementation)
        // This is a placeholder that just returns the GDI capture
        // A real implementation would upload to a texture, process with shaders, and download
        
        // Create a texture
        let texture_size = wgpu::Extent3d {
            width: gdi_capture.width(),
            height: gdi_capture.height(),
            depth_or_array_layers: 1,
        };
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING 
                | wgpu::TextureUsages::COPY_DST 
                | wgpu::TextureUsages::COPY_SRC,
            label: Some("capture_texture"),
            view_formats: &[],
        });
        
        // Upload data to texture
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            gdi_capture.as_raw(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * gdi_capture.width()),
                rows_per_image: Some(gdi_capture.height()),
            },
            texture_size,
        );
        
        // In a real implementation, we would now apply shaders for processing
        // But for this example, we just return the original image
        
        self.last_frame = Some(gdi_capture.clone());
        Ok(gdi_capture)
    }
    
    /// Find a window by title
    fn find_window_by_title(&self, title: &str) -> Result<WindowInfo> {
        let title_lower = title.to_lowercase();
        self.cached_windows.iter()
            .find(|w| w.title.to_lowercase().contains(&title_lower))
            .cloned()
            .ok_or_else(|| anyhow!(CaptureError::WindowNotFound))
    }
    
    /// Update the cached window list
    fn update_window_list(&mut self) -> Result<()> {
        unsafe {
            WINDOW_LIST.lock().unwrap().clear();
            EnumWindows(Some(enum_windows_callback), LPARAM(0));
            self.cached_windows = WINDOW_LIST.lock().unwrap().clone();
        }
        Ok(())
    }
}

impl CaptureBackend for WgpuWindowsCapture {
    fn process_frame(&mut self, frame: &Option<RgbaImage>) -> Option<RgbaImage> {
        // If we have a frame, just return it
        // In a real implementation, we would process it with WGPU
        if let Some(image) = frame {
            // Store a copy for debugging
            self.last_frame = Some(image.clone());
            Some(image.clone())
        } else {
            // Return the last frame if available
            self.last_frame.clone()
        }
    }
    
    fn backend_name(&self) -> String {
        "WGPU Windows".to_string()
    }
} 