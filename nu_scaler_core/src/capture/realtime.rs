use scrap::{Capturer, Display};
use std::io::ErrorKind;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, IsWindowVisible};

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
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self {
            running: false,
            capturer: None,
            width: 0,
            height: 0,
            target: None,
        }
    }
    pub fn list_windows() -> Vec<String> {
        #[cfg(target_os = "windows")]
        {
            use std::ptr;
            use std::ffi::OsString;
            use std::os::windows::ffi::OsStringExt;
            let mut titles = Vec::new();
            unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
                let mut buf = [0u16; 512];
                let len = GetWindowTextW(hwnd, &mut buf);
                if len > 0 && IsWindowVisible(hwnd).as_bool() {
                    let title = OsString::from_wide(&buf[..len as usize]).to_string_lossy().to_string();
                    if !title.is_empty() {
                        let titles = &mut *(lparam.0 as *mut Vec<String>);
                        titles.push(title);
                    }
                }
                1
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
                    Ok(())
                }
                #[cfg(not(target_os = "windows"))]
                {
                    // TODO: Implement for Linux (X11/Wayland)
                    Err("Screen capture not implemented for this OS".to_string())
                }
            }
            CaptureTarget::WindowByTitle(_title) => {
                // TODO: Implement window capture by title (Windows: user32, Linux: X11)
                Err("Window capture not implemented yet".to_string())
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
    }
    fn get_frame(&mut self) -> Option<Vec<u8>> {
        if !self.running {
            return None;
        }
        let capturer = self.capturer.as_mut()?;
        match capturer.frame() {
            Ok(frame) => {
                // Convert BGRA to RGB
                let mut rgb = Vec::with_capacity(self.width * self.height * 3);
                for chunk in frame.chunks(4) {
                    if chunk.len() == 4 {
                        rgb.push(chunk[2]); // R
                        rgb.push(chunk[1]); // G
                        rgb.push(chunk[0]); // B
                    }
                }
                Some(rgb)
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => None, // No new frame yet
            Err(_) => None,
        }
    }
    fn list_windows() -> Vec<String> {
        ScreenCapture::list_windows()
    }
} 