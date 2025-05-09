use anyhow::{Result /*, anyhow*/};
// use image::ImageFormat;
use scrap::{Capturer, Display /*, Frame as ScrapFrame*/};
use std::io::ErrorKind;
use std::sync::mpsc; // Keep
use std::sync::Mutex;
use std::thread;
// use std::sync::mpsc::{Receiver, Sender}; // Remove unused Receiver, Sender (mpsc itself covers usage)
use std::fs;
// use uuid::Uuid; // Remove unused Uuid

// Windows API imports (needed for list_windows)
// use windows::core::{Error, Result as WindowsResult}; // Unused
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, IsWindowVisible, /*, FindWindowW*/
}; // FindWindowW unused

// windows-capture integration (v1.4)
use windows_capture::capture::{Context, GraphicsCaptureApiHandler};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings};
use windows_capture::window::Window;

/* // Remove block of unused windows imports
use windows::Graphics::Capture::{
    Direct3D11CaptureFramePool, // Marked as unused by cargo check
    GraphicsCaptureItem,        // Marked as unused by cargo check
};
use windows::Win32::Graphics::Direct3D11::{ID3D11Texture2D}; // Marked as unused
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM; // Marked as unused
*/

#[derive(Debug, Clone)]
pub enum CaptureTarget {
    FullScreen,
    WindowByTitle(String),
    Region {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    }, // Region not yet handled by this refactor
}

pub trait RealTimeCapture {
    fn start(&mut self, target: CaptureTarget) -> std::result::Result<(), String>;
    fn stop(&mut self);
    fn get_frame(&mut self) -> Option<(Vec<u8>, usize, usize)>;
    fn list_windows() -> Vec<String>
    where
        Self: Sized;
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

        // Access the frame buffer directly
        match frame.buffer() {
            Ok(mut fb) => {
                // Use as_nopadding_buffer() to get tightly packed pixel data.
                match fb.as_nopadding_buffer() {
                    Ok(nopadding_byte_slice) => {
                        let frame_data_to_send = nopadding_byte_slice.to_vec(); // Convert to owned Vec<u8>

                        // Send the raw frame data (BGRA, tightly packed)
                        match self.frame_sender.lock() {
                            Ok(sender) => {
                                if sender.send(Some((frame_data_to_send, width, height))).is_err() {
                                    eprintln!("[CaptureHandler] Receiver disconnected during frame send.");
                                }
                            },
                            Err(poison_error) => {
                                let msg = format!("Mutex poisoned during frame send: {}", poison_error);
                                eprintln!("[CaptureHandler] FATAL: {}", msg);
                                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, msg)));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[CaptureHandler] Failed to get no-padding buffer: {:?}", e);
                        // Optionally send nothing or an error indicator, or just skip the frame.
                        // For now, just skip if we can't get the buffer correctly.
                    }
                }
            }
            Err(e) => {
                // Failed to get buffer from the frame. Log and return error.
                eprintln!("[CaptureHandler] Failed to get frame buffer: {:?}", e);
                // Convert the windows_capture error into the required boxed error type.
                // Assuming the error type implements std::error::Error + Send + Sync.
                return Err(Box::new(e));
            }
        }

        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("[CaptureHandler] Capture session closed (on_closed called).");
        match self.frame_sender.lock() {
            Ok(sender) => {
                let _ = sender.send(None);
            }
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
            let _ = EnumWindows(
                Some(enum_windows_proc),
                LPARAM(&mut titles as *mut _ as isize),
            );
        }
        titles
    }
    #[cfg(not(target_os = "windows"))]
    fn enum_windows_internal() -> Vec<String> {
        vec![]
    }

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
    fn start(&mut self, target: CaptureTarget) -> std::result::Result<(), String> {
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
            CaptureTarget::Region { .. } => Err("Region capture not implemented yet".to_string()),
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
                                rgba.push(chunk[2]);
                                rgba.push(chunk[1]);
                                rgba.push(chunk[0]);
                                rgba.push(chunk[3]);
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
                        // --- Modified logic: Drain channel, return last frame --- 
                        let mut last_frame_data: Option<(Vec<u8>, usize, usize)> = None;
                        
                        // Loop to consume all pending messages
                        loop {
                            match rx.try_recv() {
                                Ok(Some(frame_tuple)) => {
                                    println!("[ScreenCapture::get_frame] WGC Handler: Received frame from channel. Size: {}x{}", frame_tuple.1, frame_tuple.2); // DEBUG PRINT
                                    last_frame_data = Some(frame_tuple);
                                }
                                Ok(None) => {
                                    println!("[ScreenCapture::get_frame] WGC Handler: Received STOP signal (None) from channel."); // DEBUG PRINT
                                    self.debug_print("Received stop signal (None sentinel) from WGC handler.");
                                    last_frame_data = None; // Ensure we don't return a frame if stop was received
                                    self.stop(); // Stop capture immediately
                                    break; // Exit loop
                                }
                                Err(mpsc::TryRecvError::Empty) => {
                                    // Channel is empty, stop draining
                                    println!("[ScreenCapture::get_frame] WGC Handler: Channel empty."); // DEBUG PRINT
                                    break;
                                }
                                Err(mpsc::TryRecvError::Disconnected) => {
                                    println!("[ScreenCapture::get_frame] WGC Handler: Channel DISCONNECTED."); // DEBUG PRINT
                                    self.debug_print("WGC channel disconnected.");
                                    last_frame_data = None; // Ensure no frame is returned
                                    self.stop();
                                    break; // Exit loop
                                }
                            }
                        }
                        
                        // Return the last frame we processed from the channel drain (if any)
                        // Update width/height if we got a frame
                        if let Some((_, width, height)) = &last_frame_data {
                             self.width = *width;
                             self.height = *height;
                        }
                        last_frame_data // This is Option<(Vec<u8>, usize, usize)> 
                    } else {
                        None // No receiver exists
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    None
                }
            }
            Some(CaptureTarget::Region { .. }) => {
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
