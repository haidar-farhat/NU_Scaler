use anyhow::{Result, anyhow};
use image::{RgbaImage, Rgba};
use std::mem::size_of;
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::UI::WindowsAndMessaging::*,
    core::PCWSTR,
};
use crate::capture::{ScreenCapture, CaptureTarget, CaptureError};
use super::{WindowId, WindowInfo, WindowGeometry};
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Windows implementation of screen capture
pub struct PlatformScreenCapture {
    /// Device context for the screen
    screen_dc: HDC,
    /// List of windows cached from last enumeration
    cached_windows: Vec<WindowInfo>,
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
    let mut screen_rect = RECT::default();
    
    if unsafe { GetWindowRect(hwnd, &mut window_rect) }.as_bool() {
        let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
        let mut monitor_info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        
        if unsafe { GetMonitorInfoW(monitor, &mut monitor_info) }.as_bool() {
            screen_rect = monitor_info.rcMonitor;
            
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
        // Get window DC
        let window_dc = unsafe { GetDC(hwnd) };
        if window_dc.is_invalid() {
            return Err(anyhow!(CaptureError::CaptureFailed("Failed to get window DC".into())));
        }
        
        // Get window dimensions
        let mut rect = RECT::default();
        unsafe { GetClientRect(hwnd, &mut rect) };
        let width = rect.right as u32;
        let height = rect.bottom as u32;
        
        if width == 0 || height == 0 {
            unsafe { ReleaseDC(hwnd, window_dc) };
            return Err(anyhow!(CaptureError::CaptureFailed("Window has zero size".into())));
        }
        
        // Create compatible DC and bitmap
        let compatible_dc = unsafe { CreateCompatibleDC(window_dc) };
        if compatible_dc.is_invalid() {
            unsafe { ReleaseDC(hwnd, window_dc) };
            return Err(anyhow!(CaptureError::CaptureFailed("Failed to create compatible DC".into())));
        }
        
        let bitmap = unsafe { CreateCompatibleBitmap(window_dc, width as i32, height as i32) };
        if bitmap.is_invalid() {
            unsafe { DeleteDC(compatible_dc) };
            unsafe { ReleaseDC(hwnd, window_dc) };
            return Err(anyhow!(CaptureError::CaptureFailed("Failed to create compatible bitmap".into())));
        }
        
        // Select bitmap into DC
        let old_bitmap = unsafe { SelectObject(compatible_dc, bitmap) };
        
        // Copy window content to bitmap
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
            return Err(anyhow!(CaptureError::CaptureFailed("BitBlt failed".into())));
        }
        
        // Get bitmap data
        let mut bitmap_info = BITMAPINFOHEADER {
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
            unsafe { ReleaseDC(hwnd, window_dc) };
            return Err(anyhow!(CaptureError::CaptureFailed("GetDIBits failed".into())));
        }
        
        // Clean up
        unsafe { SelectObject(compatible_dc, old_bitmap) };
        unsafe { DeleteObject(bitmap) };
        unsafe { DeleteDC(compatible_dc) };
        unsafe { ReleaseDC(hwnd, window_dc) };
        
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
        let mut bitmap_info = BITMAPINFOHEADER {
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
        let mut bitmap_info = BITMAPINFOHEADER {
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
            self.cached_windows = WINDOW_LIST.lock().unwrap().clone();
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