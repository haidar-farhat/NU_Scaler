use scrap::{Capturer, Display};
use std::io::ErrorKind;
use windows::core::PCWSTR;
use windows::Win32::Foundation::BOOL;

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
    fn get_frame(&mut self) -> Option<Vec<u8>>; // Returns raw RGB frame
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
}

impl RealTimeCapture for ScreenCapture {
    fn start(&mut self, target: CaptureTarget) -> Result<(), String> {
        self.target = Some(target.clone());
        match target {
            CaptureTarget::FullScreen => {
                #[cfg(target_os = "windows")]
                {
                    let display = Display::primary().map_err(|e| e.to_string())?;
                    let width = display.width();
                    let height = display.height();
                    let capturer = Capturer::new(display).map_err(|e| e.to_string())?;
                    self.width = width as usize;
                    self.height = height as usize;
                    self.capturer = Some(capturer);
                    self.running = true;
                    self.hwnd = None;
                    Ok(())
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Err("Screen capture not implemented for this OS".to_string())
                }
            }
            CaptureTarget::WindowByTitle(title) => {
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
                    unsafe { GetWindowRect(hwnd, &mut rect) };
                    let width = (rect.right - rect.left) as usize;
                    let height = (rect.bottom - rect.top) as usize;
                    self.width = width;
                    self.height = height;
                    self.hwnd = Some(hwnd.0);
                    self.running = true;
                    Ok(())
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Err("Window capture not implemented for this OS".to_string())
                }
            }
            CaptureTarget::Region { .. } => {
                // TODO: Implement region capture (Windows: BitBlt, Linux: X11)
                Err("Region capture not implemented yet".to_string())
            }
        }
    }
    fn stop(&mut self) {
        self.running = false;
        self.capturer = None;
        #[cfg(target_os = "windows")]
        {
            self.hwnd = None;
        }
    }
    fn get_frame(&mut self) -> Option<Vec<u8>> {
        if !self.running {
            return None;
        }
        match &self.target {
            Some(CaptureTarget::FullScreen) => {
                let capturer = self.capturer.as_mut()?;
                match capturer.frame() {
                    Ok(frame) => {
                        let mut rgb = Vec::with_capacity(self.width * self.height * 3);
                        for chunk in frame.chunks(4) {
                            if chunk.len() == 4 {
                                rgb.push(chunk[2]);
                                rgb.push(chunk[1]);
                                rgb.push(chunk[0]);
                            }
                        }
                        Some(rgb)
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => None,
                    Err(_) => None,
                }
            }
            Some(CaptureTarget::WindowByTitle(_)) => {
                #[cfg(target_os = "windows")]
                {
                    let hwnd = self.hwnd?;
                    unsafe {
                        let hdc_window = GetDC(HWND(hwnd));
                        let hdc_mem = CreateCompatibleDC(hdc_window);
                        let hbm = CreateCompatibleBitmap(hdc_window, self.width as i32, self.height as i32);
                        SelectObject(hdc_mem, hbm);
                        BitBlt(hdc_mem, 0, 0, self.width as i32, self.height as i32, hdc_window, 0, 0, SRCCOPY);
                        let mut bmi = BITMAPINFO::default();
                        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
                        bmi.bmiHeader.biWidth = self.width as i32;
                        bmi.bmiHeader.biHeight = -(self.height as i32); // top-down
                        bmi.bmiHeader.biPlanes = 1;
                        bmi.bmiHeader.biBitCount = 24;
                        bmi.bmiHeader.biCompression = BI_RGB;
                        let mut buf = vec![0u8; self.width * self.height * 3];
                        GetDIBits(hdc_mem, hbm, 0, self.height as u32, Some(buf.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS);
                        DeleteObject(hbm);
                        DeleteDC(hdc_mem);
                        ReleaseDC(HWND(hwnd), hdc_window);
                        Some(buf)
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    None
                }
            }
            _ => None,
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