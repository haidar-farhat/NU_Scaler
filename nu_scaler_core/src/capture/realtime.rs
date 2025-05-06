use scrap::{Capturer, Display};
use std::io::ErrorKind;
use std::sync::mpsc; // For sending frames from callback
use std::sync::Mutex;
use std::thread;
use image::ImageFormat; // Keep, used in workaround
use std::env;
use std::fs;
use uuid::Uuid;
use std::time::Duration;
use std::path::PathBuf;

// Windows API imports (needed for list_windows)
use windows::core::{Error, Result as WindowsResult};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, IsWindowVisible/*, FindWindowW*/}; // FindWindowW unused

// windows-capture integration (v1.4)
use windows_capture::capture::{GraphicsCaptureApiHandler, Context};
use windows_capture::frame::{Frame, FrameBuffer/*, ImageFormat as CaptureImageFormat*/}; // ImageFormat unused here
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::settings::{Settings, ColorFormat, CursorCaptureSettings, DrawBorderSettings};
use windows_capture::window::Window;

#[derive(Debug, Clone)]
pub enum CaptureTarget {
    FullScreen,
    WindowByTitle(String),
    Region { x: i32, y: i32, width: u32, height: u32 }, // Region not yet handled by this refactor
}

pub trait RealTimeCapture {
    fn start(&mut self, target: CaptureTarget) -> std::result::Result<(), String>;
    fn stop(&mut self);
    fn get_frame(&mut self) -> Option<(Vec<u8>, usize, usize)>; 
    fn list_windows() -> Vec<String> where Self: Sized;
}

// --- windows-capture Handler Implementation ---
struct CaptureHandler {
    frame_sender: Mutex<mpsc::Sender<Option<(Vec<u8>, usize, usize)>>>,
}

// Implement the correct trait from windows-capture v1.4
impl GraphicsCaptureApiHandler for CaptureHandler {
    type Flags = mpsc::Sender<Option<(Vec<u8>, usize, usize)>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    // Use the required `new` method signature
    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self { frame_sender: Mutex::new(ctx.flags) })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _capture_control: InternalCaptureControl
    ) -> Result<(), Self::Error>
    {
        let width = frame.width() as usize;
        let height = frame.height() as usize;

        // --- WORKAROUND: Save to temp file and read back --- 
        let temp_dir = env::temp_dir();
        let unique_id = Uuid::new_v4();
        let mut temp_path = temp_dir;
        temp_path.push(format!("nu_scaler_frame_{}.bmp", unique_id));

        frame.save_as_image(&temp_path, windows_capture::frame::ImageFormat::Bmp)
             .map_err(|e| Box::new(e) as Self::Error)?;
        let buffer = fs::read(&temp_path).map_err(|e| Box::new(e) as Self::Error)?;
        let _ = fs::remove_file(&temp_path); // Ignore remove error
        // --- End WORKAROUND ---

        // Send the frame data (read from BMP file)
        match self.frame_sender.lock() {
            Ok(sender) => {
                if sender.send(Some((buffer, width, height))).is_err() {
                    eprintln!("[CaptureHandler] Receiver disconnected. Stopping capture implicitly.");
                }
            },
            Err(poison_error) => {
                let msg = format!("Mutex poisoned: {}", poison_error);
                eprintln!("[CaptureHandler] {}", msg);
                return Err(Box::new(std::io::Error::new(ErrorKind::Other, msg)) as Self::Error);
            }
        }
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("[CaptureHandler] Capture session closed (on_closed called).");
        match self.frame_sender.lock() {
            Ok(sender) => { let _ = sender.send(None); },
            Err(poison_error) => { 
                let msg = format!("Mutex poisoned on close: {}", poison_error);
                eprintln!("[CaptureHandler] {}", msg);
                return Err(Box::new(std::io::Error::new(ErrorKind::Other, msg)) as Self::Error);
             }
        }
        Ok(())
    }
}
// --- End Handler ---

pub struct ScreenCapture {
    running: bool,
    scrap_capturer: Option<Capturer>,
    wgc_capture_thread: Option<thread::JoinHandle<()>>, // Renamed for clarity
    wgc_frame_receiver: Option<mpsc::Receiver<Option<(Vec<u8>, usize, usize)>>>,
    width: usize,
    height: usize,
    pub target: Option<CaptureTarget>, // Make public again
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self {
            running: false,
            scrap_capturer: None,
            wgc_capture_thread: None,
            wgc_frame_receiver: None,
            width: 0,
            height: 0,
            target: None,
        }
    }

    #[cfg(target_os = "windows")]
    fn enum_windows_internal() -> Vec<String> {
        // ... (existing GDI EnumWindows logic) ...
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        let mut titles = Vec::new();
        unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            const MAX_TITLE_LEN: usize = 512;
            let mut title_buffer: Vec<u16> = vec![0; MAX_TITLE_LEN];
            let title_len = GetWindowTextW(hwnd, &mut title_buffer);
            if title_len > 0 && IsWindowVisible(hwnd).as_bool() {
                let title = OsString::from_wide(&title_buffer[..title_len as usize]);
                if let Some(title_str) = title.to_str() {
                    if !title_str.is_empty() { 
                         let vec_ptr = lparam.0 as *mut Vec<String>;
                         (*vec_ptr).push(title_str.to_string());
                    }
                }
            }
            BOOL(1) 
        }
        unsafe {
            let _ = EnumWindows(Some(enum_windows_proc), LPARAM(&mut titles as *mut _ as isize));
        }
        titles
    }
     #[cfg(not(target_os = "windows"))]
     fn enum_windows_internal() -> Vec<String> { vec![] }

    pub fn list_windows() -> Vec<String> {
        ScreenCapture::enum_windows_internal()
    }

    pub fn debug_print(&self, msg: &str) {
        println!("[ScreenCapture] {}", msg);
    }

    fn stop_wgc(&mut self) {
         // Dropping the receiver is the primary way to signal the handler to stop
         if let Some(receiver) = self.wgc_frame_receiver.take() {
             drop(receiver);
             self.debug_print("WGC receiver dropped.");
        }
         // Join the thread to ensure it cleans up
         if let Some(handle) = self.wgc_capture_thread.take() {
            if let Err(e) = handle.join() {
                 eprintln!("[ScreenCapture] WGC capture thread panicked: {:?}", e);
            } else {
                 self.debug_print("WGC capture thread joined successfully.");
            }
        }
    }

    fn start_wgc(&mut self, title: String) -> Result<(), String> {
        self.debug_print(&format!("Starting WGC for Window Title: {}", title));

        let window = Window::from_name(&title)
            .map_err(|e| format!("Window '{}' not found or error: {:?}", title, e))?;

        let (tx, rx) = mpsc::channel::<Option<(Vec<u8>, usize, usize)>>();
        self.wgc_frame_receiver = Some(rx);

        let capture_flags = tx;
        let settings = Settings::new(
            window, 
            CursorCaptureSettings::Default, 
            DrawBorderSettings::Default, 
            ColorFormat::Bgra8, 
            capture_flags, 
        );

        let capture_thread = thread::spawn(move || {
            println!("[WGC Thread] Starting capture...");
            // This call takes control of the thread until capture stops/fails
            if let Err(e) = CaptureHandler::start(settings) {
                eprintln!("[WGC Thread] Capture failed: {}", e);
                // The handler's on_error or on_closed should send None via the channel
            } else {
                 println!("[WGC Thread] Capture finished gracefully.");
            }
        });

        self.wgc_capture_thread = Some(capture_thread);
        self.running = true;
        Ok(())
     }
}

impl RealTimeCapture for ScreenCapture {
    fn start(&mut self, target: CaptureTarget) -> Result<(), String> {
        self.debug_print(&format!("Starting capture: {:?}", target));
        self.target = Some(target.clone());
        self.stop(); 

        match target {
            CaptureTarget::FullScreen => {
                // Use scrap for fullscreen
                let display = Display::primary().map_err(|e| e.to_string())?;
                let width = display.width();
                let height = display.height();
                let capturer = Capturer::new(display).map_err(|e| e.to_string())?;
                self.width = width;
                self.height = height;
                self.scrap_capturer = Some(capturer);
                self.running = true;
                self.debug_print(&format!("FullScreen capture started: {}x{}", width, height));
                Ok(())
            }
            CaptureTarget::WindowByTitle(title) => {
                #[cfg(target_os = "windows")]
                {
                    self.start_wgc(title)
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
             self.scrap_capturer = None;
             self.stop_wgc();
        }
    }

    fn get_frame(&mut self) -> Option<(Vec<u8>, usize, usize)> { 
        if !self.running {
            return None;
        }

        match self.target {
            Some(CaptureTarget::FullScreen) => {
                if let Some(capturer) = self.scrap_capturer.as_mut() {
                    match capturer.frame() {
                        Ok(frame) => {
                            if self.width == 0 || self.height == 0 { 
                                eprintln!("[ScreenCapture] Fullscreen dimensions not set!"); 
                                return None; 
                            } 
                            let expected_len = self.width * self.height * 4;
                            if frame.len() != expected_len {
                                eprintln!("[ScreenCapture] Frame size mismatch (FullScreen)! Expected: {}, Got: {}", expected_len, frame.len());
                                return None;
                            }
                            let mut rgba = Vec::with_capacity(expected_len);
                            for chunk in frame.chunks_exact(4) {
                                rgba.push(chunk[2]); rgba.push(chunk[1]); rgba.push(chunk[0]); rgba.push(chunk[3]);
                            }
                            Some((rgba, self.width, self.height))
                        }
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => None,
                        Err(e) => {
                            eprintln!("[ScreenCapture] Frame capture error (FullScreen): {}", e);
                            self.stop();
                            None
                        }
                    }
                } else {
                    None
                }
            }
            Some(CaptureTarget::WindowByTitle(_)) => {
                 #[cfg(target_os = "windows")]
                 {
                    if let Some(rx) = self.wgc_frame_receiver.as_ref() {
                        match rx.try_recv() { 
                             Ok(Some((bgra_buffer, width, height))) => {
                                 // Update dimensions based on received frame
                                 self.width = width;
                                 self.height = height;
                                 // Convert BGRA from windows-capture to RGBA
                                 let mut rgba = Vec::with_capacity(bgra_buffer.len());
                                  for chunk in bgra_buffer.chunks_exact(4) {
                                    rgba.push(chunk[2]); rgba.push(chunk[1]); rgba.push(chunk[0]); rgba.push(chunk[3]);
                                }
                                Some((rgba, width, height))
                             },
                             Ok(None) => { 
                                 self.debug_print("Received stop signal from WGC handler.");
                                 self.stop(); 
                                 None
                             },
                             Err(mpsc::TryRecvError::Empty) => None, 
                             Err(mpsc::TryRecvError::Disconnected) => {
                                 self.debug_print("WGC channel disconnected.");
                                 self.stop();
                                 None
                             }
                        }
                    } else {
                        None 
                    }
                 }
                 #[cfg(not(target_os = "windows"))]
                 { None }
            }
             Some(CaptureTarget::Region { .. })=> {
                 eprintln!("[ScreenCapture] Region capture not implemented yet");
                 None
             }
            None => None, // No target set
        }
    }

    fn list_windows() -> Vec<String> {
        ScreenCapture::enum_windows_internal()
    }
}

// For Linux: Scaffold X11 window capture (not implemented yet)
#[cfg(target_os = "linux")]
mod x11_capture {
    // use x11::xlib::*;
    // TODO: Implement X11 window capture
} 