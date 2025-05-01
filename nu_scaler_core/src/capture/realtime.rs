use scrap::{Capturer, Display};
use std::io::ErrorKind;
use windows::core::PCWSTR;
use windows::Win32::Foundation::BOOL;

// Add raw_window_handle imports
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, Win32WindowHandle};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, IsWindowVisible, FindWindowW, GetWindowRect};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC, CreateCompatibleDC, CreateCompatibleBitmap, SelectObject, BitBlt, DeleteObject, DeleteDC, GetDIBits, SRCCOPY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{RECT};

#[derive(Debug, Clone)]
pub enum CaptureTarget {
    FullScreen,
    WindowByTitle(String),
    Region { x: i32, y: i32, width: u32, height: u32 },
}

pub trait RealTimeCapture {
    fn start(&mut self, target: CaptureTarget) -> Result<(), String>;
    fn stop(&mut self);
    fn get_frame(&mut self) -> Option<(Vec<u8>, usize, usize)>;
    fn list_windows() -> Vec<String> where Self: Sized;
}

pub struct ScreenCapture {
    running: bool,
    capturer: Option<Capturer>,
    width: usize,
    height: usize,
    target: Option<CaptureTarget>,
    // Remove HWND, scrap handles it internally via RawWindowHandle
    // #[cfg(target_os = "windows")]
    // hwnd: Option<isize>,
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self {
            running: false,
            capturer: None,
            width: 0,
            height: 0,
            target: None,
            // Remove HWND
            // #[cfg(target_os = "windows")]
            // hwnd: None,
        }
    }
    pub fn list_windows() -> Vec<String> {
        #[cfg(target_os = "windows")]
        {
            use std::ptr;
            use std::ffi::OsString;
            use std::os::windows::ffi::OsStringExt;
            let mut titles = Vec::new();
            unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
                let mut buf = [0u16; 512];
                let len = GetWindowTextW(hwnd, &mut buf);
                if len > 0 && IsWindowVisible(hwnd).as_bool() {
                    let title = OsString::from_wide(&buf[..len as usize]).to_string_lossy().to_string();
                    if !title.is_empty() {
                        let titles = &mut *(lparam.0 as *mut Vec<String>);
                        titles.push(title);
                    }
                }
                BOOL(1)
            }
            unsafe {
                EnumWindows(Some(enum_windows_proc), LPARAM(&mut titles as *mut _ as isize));
            }
            titles
        }
        #[cfg(not(target_os = "windows"))]
        {
            vec![]
        }
    }
    pub fn debug_print(&self, msg: &str) {
        println!("[ScreenCapture] {}", msg);
    }
}

// Implement HasRawWindowHandle for HWND to pass to scrap
#[cfg(target_os = "windows")]
struct HwndWrapper(HWND);

#[cfg(target_os = "windows")]
unsafe impl HasRawWindowHandle for HwndWrapper {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = Win32WindowHandle::empty();
        handle.hwnd = self.0 .0 as *mut std::ffi::c_void; // HWND -> *mut c_void
        RawWindowHandle::Win32(handle)
    }
}

impl RealTimeCapture for ScreenCapture {
    fn start(&mut self, target: CaptureTarget) -> Result<(), String> {
        self.debug_print(&format!("Starting capture: {:?}", target));
        self.target = Some(target.clone());
        self.stop(); // Stop previous capture if any

        match target {
            CaptureTarget::FullScreen => {
                let display = Display::primary().map_err(|e| e.to_string())?;
                let width = display.width();
                let height = display.height();
                let capturer = Capturer::new(display).map_err(|e| e.to_string())?;
                self.width = width;
                self.height = height;
                self.capturer = Some(capturer);
                self.running = true;
                self.debug_print(&format!("FullScreen capture started: {}x{}", width, height));
                Ok(())
            }
            CaptureTarget::WindowByTitle(ref title) => {
                #[cfg(target_os = "windows")]
                {
                    use std::ffi::OsStr;
                    use std::os::windows::ffi::OsStrExt;
                    let wide: Vec<u16> = OsStr::new(&title).encode_wide().chain(Some(0)).collect();
                    let hwnd = unsafe { FindWindowW(None, PCWSTR::from_raw(wide.as_ptr())) };
                    if hwnd.0 == 0 {
                        return Err(format!("Window '{}' not found", title));
                    }
                    let mut rect = RECT::default();
                    if unsafe { GetWindowRect(hwnd, &mut rect) } == false {
                         return Err(format!("Could not get window rect for '{}'", title));
                    }
                    let width = (rect.right - rect.left).max(0) as usize; // Ensure non-negative
                    let height = (rect.bottom - rect.top).max(0) as usize;
                    if width == 0 || height == 0 {
                        return Err(format!("Window '{}' has zero width or height", title));
                    }

                    // Use scrap::Capturer::new with the window handle
                    let wrapper = HwndWrapper(hwnd);
                    let capturer = Capturer::new(wrapper).map_err(|e| e.to_string())?;

                    self.width = width;
                    self.height = height;
                    self.capturer = Some(capturer);
                    self.running = true;
                    self.debug_print(&format!("WindowByTitle capture started: '{}' {}x{}", title, width, height));
                    Ok(())
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Err("Window capture not implemented for this OS".to_string())
                }
            }
            CaptureTarget::Region { .. } => {
                Err("Region capture not implemented yet".to_string())
            }
        }
    }

    fn stop(&mut self) {
        if self.running {
             self.debug_print("Stopping capture");
             self.running = false;
             self.capturer = None; // Drop the capturer
        }
    }

    fn get_frame(&mut self) -> Option<(Vec<u8>, usize, usize)> {
        if !self.running {
            // self.debug_print("get_frame called but not running"); // Too noisy
            return None;
        }
        let width = self.width;
        let height = self.height;

        // Use the capturer regardless of target type (if start succeeded)
        if let Some(capturer) = self.capturer.as_mut() {
            match capturer.frame() {
                Ok(frame) => {
                    // Check frame dimensions match expected (scrap might return full screen?)
                    // Although for window capture, it *should* return the window size.
                    // Let's assume scrap gives BGRA bytes for the correct dimensions for now.
                    let expected_len = width * height * 4;
                    if frame.len() != expected_len {
                        self.debug_print(&format!(
                            "Frame size mismatch! Expected: {}x{}={} bytes, Got: {} bytes. Target: {:?}",
                            width, height, expected_len, frame.len(), self.target
                        ));
                        // Attempt to process anyway if possible, might be stride issue?
                        // Or return None?
                        return None; // Safer to return None for now
                    }

                    // Convert BGRA to RGBA
                    let mut rgba = Vec::with_capacity(expected_len);
                    for chunk in frame.chunks_exact(4) {
                        // chunk[0]=B, chunk[1]=G, chunk[2]=R, chunk[3]=A
                        rgba.push(chunk[2]); // R
                        rgba.push(chunk[1]); // G
                        rgba.push(chunk[0]); // B
                        rgba.push(chunk[3]); // A
                    }
                    self.debug_print(&format!("Captured frame: {} bytes ({}x{}) via scrap", rgba.len(), width, height));
                    Some((rgba, width, height))
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // self.debug_print("No frame available yet (WouldBlock)"); // Too noisy
                    None
                }
                Err(e) => {
                    self.debug_print(&format!("Frame capture error: {}", e));
                    self.stop(); // Stop capture on error
                    None
                }
            }
        } else {
            self.debug_print("get_frame called but capturer is None");
            None
        }
    }

    fn list_windows() -> Vec<String> {
        ScreenCapture::list_windows()
    }
}

// For Linux: Scaffold X11 window capture (not implemented yet)
#[cfg(target_os = "linux")]
mod x11_capture {
    // use x11::xlib::*;
    // TODO: Implement X11 window capture
} 