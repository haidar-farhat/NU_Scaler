use scrap::{Capturer, Display};
use std::io::ErrorKind;
use windows::core::PCWSTR;
use windows::Win32::Foundation::BOOL;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, IsWindowVisible, FindWindowW, GetWindowRect};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC, CreateCompatibleDC, CreateCompatibleBitmap, SelectObject, BitBlt, DeleteObject, DeleteDC, GetDIBits, SRCCOPY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS};

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
    #[cfg(target_os = "windows")]
    hwnd: Option<isize>,
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self {
            running: false,
            capturer: None,
            width: 0,
            height: 0,
            target: None,
            #[cfg(target_os = "windows")]
            hwnd: None,
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
                #[cfg(target_os="windows")]
                { self.hwnd = None; }
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
                    if unsafe { GetWindowRect(hwnd, &mut rect).is_ok() } == false {
                         return Err(format!("Could not get window rect for '{}'", title));
                    }
                    let width = (rect.right - rect.left).max(0) as usize;
                    let height = (rect.bottom - rect.top).max(0) as usize;
                    if width == 0 || height == 0 {
                        return Err(format!("Window '{}' has zero width or height", title));
                    }

                    self.width = width;
                    self.height = height;
                    self.hwnd = Some(hwnd.0);
                    self.capturer = None;
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
             self.capturer = None;
             #[cfg(target_os = "windows")]
             { self.hwnd = None; }
        }
    }

    fn get_frame(&mut self) -> Option<(Vec<u8>, usize, usize)> {
        if !self.running {
            return None;
        }
        let width = self.width;
        let height = self.height;

        match &self.target {
            Some(CaptureTarget::FullScreen) => {
                if let Some(capturer) = self.capturer.as_mut() {
                    match capturer.frame() {
                        Ok(frame) => {
                            let expected_len = width * height * 4;
                            if frame.len() != expected_len {
                                let error_msg = format!("Frame size mismatch (FullScreen)! Expected: {}, Got: {}", expected_len, frame.len());
                                self.debug_print(&error_msg);
                                return None;
                            }
                            let mut rgba = Vec::with_capacity(expected_len);
                            for chunk in frame.chunks_exact(4) {
                                rgba.push(chunk[2]); rgba.push(chunk[1]); rgba.push(chunk[0]); rgba.push(chunk[3]);
                            }
                            let success_msg = format!("Captured fullscreen frame: {} bytes ({}x{}) via scrap", rgba.len(), width, height);
                            self.debug_print(&success_msg);
                            Some((rgba, width, height))
                        }
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => None,
                        Err(e) => {
                            self.debug_print(&format!("Frame capture error (FullScreen): {}", e));
                            self.stop();
                            None
                        }
                    }
                } else {
                    self.debug_print("get_frame called for FullScreen but capturer is None");
                    None
                }
            }
            Some(CaptureTarget::WindowByTitle(_)) => {
                #[cfg(target_os = "windows")]
                {
                    self.hwnd.and_then(|hwnd_isize| {
                        unsafe {
                            let hwnd = HWND(hwnd_isize);
                            self.debug_print(&format!("Attempting GDI capture for HWND: {:?}", hwnd));
                            let hdc_window = GetDC(hwnd);
                            if hdc_window.is_invalid() { self.debug_print("GetDC failed"); return None; }
                            self.debug_print(&format!("Got window DC: {:?}", hdc_window));

                            let hdc_mem = CreateCompatibleDC(hdc_window);
                            if hdc_mem.is_invalid() { self.debug_print("CreateCompatibleDC failed"); ReleaseDC(hwnd, hdc_window); return None; }
                            self.debug_print(&format!("Created memory DC: {:?}", hdc_mem));

                            let hbm = CreateCompatibleBitmap(hdc_window, width as i32, height as i32);
                            if hbm.is_invalid() { self.debug_print("CreateCompatibleBitmap failed"); DeleteDC(hdc_mem); ReleaseDC(hwnd, hdc_window); return None; }
                            self.debug_print(&format!("Created compatible bitmap: {:?}", hbm));

                            let old_obj = SelectObject(hdc_mem, hbm); // Select bitmap into memory DC
                            if old_obj.is_invalid() { self.debug_print("SelectObject (new) failed"); DeleteObject(hbm); DeleteDC(hdc_mem); ReleaseDC(hwnd, hdc_window); return None; }
                            self.debug_print("Selected bitmap into memory DC");

                            // Copy window content to memory DC
                            let blt_result = BitBlt(hdc_mem, 0, 0, width as i32, height as i32, hdc_window, 0, 0, SRCCOPY);
                            // Check if the Result is an error
                            if blt_result.is_err() { 
                                self.debug_print(&format!("BitBlt failed: {:?}", blt_result.err())); 
                                // Decide whether to return None or continue
                                // Let's return None for now if BitBlt fails
                                DeleteObject(hbm); // Cleanup needed before returning
                                DeleteDC(hdc_mem);
                                ReleaseDC(hwnd, hdc_window);
                                return None;
                            } else {
                                 self.debug_print("BitBlt succeeded");
                            }

                            // Prepare bitmap info
                            let mut bmi = BITMAPINFO {
                                bmiHeader: BITMAPINFOHEADER {
                                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                                    biWidth: width as i32,
                                    biHeight: -(height as i32),
                                    biPlanes: 1,
                                    biBitCount: 32,
                                    biCompression: BI_RGB.0 as u32,
                                    ..Default::default()
                                },
                                ..Default::default()
                            };

                            // Allocate buffer for pixel data (BGRA)
                            let mut buf = vec![0u8; width * height * 4];

                            // Get pixel data
                            let result = GetDIBits(hdc_mem, hbm, 0, height as u32, Some(buf.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS);
                             self.debug_print(&format!("GetDIBits result: {}", result));

                            // Clean up GDI objects
                            SelectObject(hdc_mem, old_obj); // Select old object back
                            DeleteObject(hbm);
                            DeleteDC(hdc_mem);
                            ReleaseDC(hwnd, hdc_window);
                            self.debug_print("Cleaned up GDI objects");

                            if result == 0 {
                                self.debug_print("GetDIBits failed, returning None");
                                return None;
                            }

                            let mut rgba = Vec::with_capacity(width * height * 4);
                            for chunk in buf.chunks_exact(4) {
                                rgba.push(chunk[2]);
                                rgba.push(chunk[1]);
                                rgba.push(chunk[0]);
                                rgba.push(chunk[3]);
                            }

                            self.debug_print(&format!("Captured window frame: {} bytes ({}x{}) via GDI", rgba.len(), width, height));
                            Some((rgba, width, height))
                        }
                    })
                }
                #[cfg(not(target_os = "windows"))]
                {
                    None
                }
            }
            _ => {
                self.debug_print("get_frame called for unsupported target");
                None
            }
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