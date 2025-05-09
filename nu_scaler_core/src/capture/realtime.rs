use anyhow::{Result /*, anyhow*/};
// use image::ImageFormat;
use scrap::{Capturer, Display /*, Frame as ScrapFrame*/};
use std::io::ErrorKind;
// std::sync::mpsc is still used for the final channel to Python
use std::sync::mpsc::{self, Receiver as StdReceiver, Sender as StdSender};
// use std::sync::Mutex; // This was unused, removing for now. Add back if needed for other parts.
use std::thread::{self, JoinHandle};
use std::time::Instant;
use std::sync::Arc;
use std::sync::atomic::Ordering;

// +++ Added imports +++
use std::cell::Cell; 
// Removed unused imports based on compiler warnings
// use std::cell::RefCell; 
// use std::fs::OpenOptions;
// use std::io::Write;

// For crossbeam channel
use crossbeam_channel::{Receiver as CrossbeamReceiver, Sender as CrossbeamSender};

// For thread priority and affinity
#[cfg(target_os = "windows")]
use windows::{
    Win32::System::Threading::{
        GetCurrentThread, SetThreadAffinityMask,
        THREAD_PRIORITY_ABOVE_NORMAL, THREAD_PRIORITY_HIGHEST,
    },
    Win32::Foundation::BOOL,
};

// use std::sync::mpsc::{Receiver, Sender}; // Remove unused Receiver, Sender (mpsc itself covers usage)
// use std::fs; // Removed unused import

// Windows API imports (needed for list_windows)
// use windows::core::{Error, Result as WindowsResult}; // Unused
// Removed BOOL from this import as it's defined above in the cfg block
use windows::Win32::Foundation::{HWND, LPARAM}; 
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, IsWindowVisible, /*, FindWindowW*/
}; // FindWindowW unused

// windows-capture integration (v1.4)
use windows_capture::capture::{Context, GraphicsCaptureApiHandler};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::settings::{ColorFormat, Settings, CursorCaptureSettings, DrawBorderSettings};
use windows_capture::window::Window;
use windows_capture::monitor::Monitor;

/* // Remove block of unused windows imports
use windows::Graphics::Capture::{
    Direct3D11CaptureFramePool, // Marked as unused by cargo check
    GraphicsCaptureItem,        // Marked as unused by cargo check
};
use windows::Win32::Graphics::Direct3D11::{ID3D11Texture2D}; // Marked as unused
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM; // Marked as unused
*/

// Constants for thread affinity (optional)
const CAPTURE_CORE_ID: Option<usize> = None; // Example: Some(2) to pin to core 2
const WORKER_CORE_ID: Option<usize> = None;  // Example: Some(3) to pin to core 3

// Channel packet type update
type FramePacket = (Vec<u8>, u32, u32); // (data, width, height)

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

// Handles capture events for Windows Graphics Capture (v1.4.3 API)
pub struct CaptureHandler {
    frame_sender: CrossbeamSender<FramePacket>,
}

impl GraphicsCaptureApiHandler for CaptureHandler {
    type Flags = CrossbeamSender<FramePacket>; // Context uses this
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    // v1.4.3: `new` receives Context<Self::Flags>
    fn new(context: Context<Self::Flags>) -> Result<Self, Self::Error> {
        println!("[CaptureHandler] Created (v1.4.3 API).");
        Ok(Self { frame_sender: context.flags() })
    }

    // v1.4.3: `frame` is &mut Frame
    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame, // Changed back to &mut Frame
        _capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        thread_local! {
            static LAST_FRAME_ARRIVAL_TIME_CONSOLE_LOG: Cell<Option<Instant>> = Cell::new(None);
        }
        let now = Instant::now();
        LAST_FRAME_ARRIVAL_TIME_CONSOLE_LOG.with(|last_time_cell| {
            if let Some(last_time) = last_time_cell.get() {
                let delta = now.duration_since(last_time);
                if delta.as_secs_f64() > 0.0 {
                    println!(
                        "RUST_CONSOLE_LOG [CaptureHandler::on_frame_arrived] Interval: {:?}, Approx FPS: {:.2}",
                        delta,
                        1.0 / delta.as_secs_f64()
                    );
                }
            }
            last_time_cell.set(Some(now));
        });

        let width = frame.width();
        let height = frame.height();

        match frame.buffer() { // On &mut Frame
            Ok(mut fb) => { // fb is Buffer<'frame> which is mutable implicitly
                match fb.as_nopadding_buffer() { // This should be available on v1.4.3 Buffer
                    Ok(nopadding_byte_slice) => {
                        if self.frame_sender.try_send((nopadding_byte_slice.to_vec(), width, height)).is_err() {
                            // eprintln!("[CaptureHandler] Failed to send frame (v1.4.3): channel full or disconnected.");
                        }
                    }
                    Err(e) => {
                        eprintln!("[CaptureHandler] Failed to get no-padding buffer (v1.4.3): {:?}", e);
                        return Err(Box::new(e)); // Propagate error
                    }
                }
            }
            Err(e) => {
                eprintln!("[CaptureHandler] Failed to get frame buffer (v1.4.3): {:?}", e);
                return Err(Box::new(e)); // Propagate error
            }
        }
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("[CaptureHandler] Capture session closed (v1.4.3 API).");
        Ok(())
    }
}

pub struct ScreenCapture {
    running: bool,
    scrap_capturer: Option<Capturer>,
    
    // WGC related fields
    wgc_capture_thread_handle: Option<JoinHandle<()>>, 
    wgc_worker_thread_handle: Option<JoinHandle<()>>,  
    
    // Receiver for frames from Python's perspective, type updated
    python_frame_receiver: Option<StdReceiver<Option<FramePacket>>>, 
    
    // Used to signal the WGC capture and worker threads to stop
    // The cb_sender (from start_wgc_capture) is used by CaptureHandler, not directly stored here for stop signal usually.
    // Dropping the capture (which stops the CaptureHandler::start loop) or other mechanisms handle stop.
    // We might need a way to signal the CaptureHandler::start loop to exit if it doesn't by itself.
    // For now, focusing on joining threads.

    // Store the crossbeam sender if needed to signal worker to stop, or rely on channel disconnect
    wgc_control_sender: Option<CrossbeamSender<FramePacket>>, // This is the cb_sender from start_wgc_capture
    stop_event: Arc<std::sync::atomic::AtomicBool>, // For graceful shutdown signal

    width: usize, // Note: scrap uses usize, WGC uses u32. Consider consistency or conversion.
    height: usize,
    pub target: Option<CaptureTarget>,
    is_capturing: Arc<std::sync::atomic::AtomicBool>, // Used by multiple threads for status
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self {
            running: false, // This will be managed by is_capturing primarily
            scrap_capturer: None,
            wgc_capture_thread_handle: None,
            wgc_worker_thread_handle: None,
            python_frame_receiver: None, 
            wgc_control_sender: None,
            stop_event: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            width: 0,
            height: 0,
            target: None,
            is_capturing: Arc::new(std::sync::atomic::AtomicBool::new(false)),
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

    fn stop_wgc_threads(&mut self) { // Renamed from stop_wgc for clarity
        self.debug_print("Stopping WGC threads...");

        // Signal the capture loop to stop (if it checks stop_event)
        self.stop_event.store(true, Ordering::SeqCst);

        // Drop the main crossbeam sender. This will cause the worker to exit its recv() loop.
        // The CaptureHandler::start loop might also exit if it detects its sender (flags) is broken or all receivers dropped.
        if let Some(sender) = self.wgc_control_sender.take() {
            drop(sender);
            self.debug_print("Dropped WGC control sender (cb_sender).");
        }

        // Join the WGC capture thread (runs windows-capture's event loop)
        if let Some(handle) = self.wgc_capture_thread_handle.take() {
            self.debug_print("Joining WGC capture thread...");
            if handle.join().is_err() {
                eprintln!("[ScreenCapture] WGC capture thread panicked or error during join.");
            }
            self.debug_print("WGC capture thread joined.");
        }

        // Join the worker thread
        if let Some(handle) = self.wgc_worker_thread_handle.take() {
            self.debug_print("Joining WGC worker thread...");
            if handle.join().is_err() {
                eprintln!("[ScreenCapture] WGC worker thread panicked or error during join.");
            }
            self.debug_print("WGC worker thread joined.");
        }
        self.python_frame_receiver = None; // Clear the receiver as threads are stopped
        self.debug_print("WGC threads stopped.");
    }

    #[cfg(target_os = "windows")]
    fn start_wgc_main_logic(&mut self, target: CaptureTarget, capture_core_id: Option<usize>) -> Result<(), String> {
        let item_title_for_debug = match &target {
            CaptureTarget::WindowByTitle(t) => t.clone(),
            CaptureTarget::FullScreen => "FullScreen".to_string(),
            _ => "UnknownTarget".to_string(),
        };
        self.debug_print(&format!("Starting WGC for Target: {}", item_title_for_debug));
        
        let capture_item = target.clone().into_internal_type()?; // Use the helper

        // MPSC channel for worker to send to Python consumer (get_frame)
        let (py_sender, py_receiver): (StdSender<Option<FramePacket>>, StdReceiver<Option<FramePacket>>) = mpsc::channel();
        self.python_frame_receiver = Some(py_receiver);

        // Call the free function start_wgc_capture (its signature needs to accept CaptureItem)
        // Let's assume start_wgc_capture is refactored to accept a CaptureItem directly.
        // pub fn start_wgc_capture(
        //    capture_item: windows_capture::capture::CaptureItem,
        //    python_frame_sender: StdSender<Option<FramePacket>>,
        // ) -> Result<(JoinHandle<()>, CrossbeamSender<FramePacket>, Settings<CrossbeamSender<FramePacket>>), String>
        
        // This free function sets up cb_channels, spawns worker, creates Settings struct
        let (worker_handle, cb_sender_for_handler_flags, wgc_settings) = 
            start_wgc_capture_internal_setup(capture_item, py_sender)?;

        self.wgc_worker_thread_handle = Some(worker_handle);
        self.wgc_control_sender = Some(cb_sender_for_handler_flags); // This is the cb_sender

        let capture_thread_settings = wgc_settings; // Settings struct already has the cb_sender flag
        let local_stop_event = self.stop_event.clone();

        self.wgc_capture_thread_handle = Some(thread::Builder::new()
            .name("wgc_capture_thread".into())
            .spawn(move || {
                println!("[WGC Capture Thread] Starting using windows-capture v2.0.0...");
                #[cfg(target_os = "windows")]
                unsafe {
                    if windows::Win32::System::Threading::SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST).is_err() {
                        println!("[WGC Capture Thread] Failed to set thread priority to HIGHEST.");
                    }
                    if let Some(core_id) = capture_core_id.or(CAPTURE_CORE_ID) {
                        if SetThreadAffinityMask(GetCurrentThread(), 1 << core_id) == 0 {
                            println!("[WGC Capture Thread] Failed to set thread affinity to core {}.", core_id);
                        }
                    }
                }
                
                if let Err(e) = CaptureHandler::start(capture_thread_settings) {
                    eprintln!("[WGC Capture Thread] Capture failed/stopped: {:?}", e);
                }
                println!("[WGC Capture Thread] Capture loop exited.");
                local_stop_event.store(true, Ordering::SeqCst); 
            })
            .map_err(|e| format!("Failed to spawn WGC capture thread: {:?}", e))?);
        
        self.is_capturing.store(true, Ordering::SeqCst);
        Ok(())
    }
}

// The into_internal_type() helper would be on CaptureTarget enum if start_wgc_capture expects a specific type
// For now, assuming CaptureTarget::WindowByTitle(title) is directly usable or adapted in start_wgc_capture.
// The `start_wgc_capture` needs to be adapted to take `CaptureTarget` or its decomposed parts.

// Placeholder for CaptureTarget::into_internal_type() - this needs to be implemented on CaptureTarget
impl CaptureTarget {
    fn into_internal_type(self) -> Result<windows_capture::capture::CaptureItem, String> {
        match self {
            CaptureTarget::WindowByTitle(title) => Window::from_name(&title).map(Into::into).map_err(|e| e.to_string()),
            CaptureTarget::FullScreen => Monitor::primary().map(Into::into).map_err(|e| e.to_string()),
            CaptureTarget::Region{..} => Err("Region capture not yet supported for WGC direct item".to_string()),
        }
    }
}

impl RealTimeCapture for ScreenCapture {
    fn start(&mut self, target: CaptureTarget) -> std::result::Result<(), String> {
        self.debug_print(&format!("Starting capture: {:?}", target));
        if self.is_capturing.load(Ordering::Relaxed) {
            self.debug_print("Capture already running. Stopping first.");
            self.stop()?; // Ensure previous capture is fully stopped
        }
        
        self.target = Some(target.clone());
        self.stop_event.store(false, Ordering::SeqCst); // Reset stop event

        match target {
            CaptureTarget::FullScreen => {
                // Use scrap for fullscreen
                let display = Display::primary().map_err(|e| e.to_string())?;
                self.width = display.width();
                self.height = display.height();
                let capturer = Capturer::new(display).map_err(|e| e.to_string())?;
                self.scrap_capturer = Some(capturer);
                self.is_capturing.store(true, Ordering::SeqCst);
                self.running = true; // old flag, can be removed if is_capturing is sole source of truth
                self.debug_print(&format!("FullScreen capture started with scrap: {}x{}", self.width, self.height));
                Ok(())
            }
            CaptureTarget::WindowByTitle(title) => {
                #[cfg(target_os = "windows")]
                {
                    self.start_wgc_main_logic(target, None) // Pass None for core_id for now
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Err("Window capture not implemented for this OS".to_string())
                }
            }
            CaptureTarget::Region { .. } => Err("Region capture not implemented yet".to_string()),
        }
    }

    fn stop(&mut self) -> std::result::Result<(), String> {
        if !self.is_capturing.load(Ordering::SeqCst) && !self.running /* old flag */ {
            self.debug_print("Stop called but capture not running.");
            return Ok(());
        }
        self.debug_print("Stopping capture (RealTimeCapture::stop)...");
        
        if self.scrap_capturer.is_some() {
            self.scrap_capturer = None;
            self.debug_print("Scrap capture stopped.");
        }
        
        self.stop_wgc_threads(); // Handles WGC related threads and sender
        
        self.is_capturing.store(false, Ordering::SeqCst);
        self.running = false; // old flag
        self.debug_print("Capture fully stopped (RealTimeCapture::stop).");
        Ok(())
    }

    fn get_frame(&mut self) -> Option<(Vec<u8>, usize, usize)> { // Return type may need to be Option<FramePacket>
        if !self.is_capturing.load(Ordering::Relaxed) {
            return None;
        }

        match self.target.as_ref() {
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
                                rgba.push(chunk[2]); // B
                                rgba.push(chunk[1]); // G
                                rgba.push(chunk[0]); // R
                                rgba.push(chunk[3]); // A
                            }
                            Some((rgba, self.width, self.height))
                        }
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => None, // No new frame
                        Err(e) => {
                            eprintln!("[ScreenCapture] Frame capture error (FullScreen): {}", e);
                            self.stop().ok();
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
                    if let Some(rx) = self.python_frame_receiver.as_ref() {
                        let mut last_frame_data: Option<FramePacket> = None;
                        loop {
                            match rx.try_recv() {
                                Ok(Some(frame_packet)) => { // frame_packet is FramePacket (Vec<u8>, u32, u32)
                                    last_frame_data = Some(frame_packet);
                                }
                                Ok(None) => { 
                                    self.debug_print("[get_frame] WGC Path: Received STOP signal (None) from Python channel.");
                                    self.stop().ok(); // Attempt to stop if not already
                                    return None; // Return None as capture is stopping/stopped
                                }
                                Err(mpsc::TryRecvError::Empty) => break,
                                Err(mpsc::TryRecvError::Disconnected) => {
                                    self.debug_print("[get_frame] WGC Path: Python channel DISCONNECTED.");
                                    self.stop().ok();
                                    return None;
                                }
                            }
                        }
                        // Convert (Vec<u8>, u32, u32) to (Vec<u8>, usize, usize) for consistent return type
                        return last_frame_data.map(|(data, w, h)| (data, w as usize, h as usize));
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    return None;
                }
            }
            Some(CaptureTarget::Region { .. }) => {
                eprintln!("[ScreenCapture] Region capture not implemented yet");
                None
            }
            None => {
                // eprintln!("[ScreenCapture::get_frame] No target set."); // DEBUG
                None
            }
        }
    }

    fn list_windows() -> Vec<String> {
        ScreenCapture::enum_windows_internal()
    }
}

impl Drop for ScreenCapture {
    fn drop(&mut self) {
        self.debug_print("[ScreenCapture Drop] Dropping ScreenCapture instance.");
        if self.is_capturing.load(Ordering::Relaxed) {
            if let Err(e) = self.stop() {
                eprintln!("[ScreenCapture Drop] Error stopping capture: {}", e);
            }
        }
    }
}

// For Linux: Scaffold X11 window capture (not implemented yet)
#[cfg(target_os = "linux")]
mod x11_capture {
    // use x11::xlib::*;
    // TODO: Implement X11 window capture
}

// Free function for WGC setup (v1.4.3 API for Settings)
fn start_wgc_capture_internal_setup(
    capture_item: windows_capture::capture::CaptureItem,
    python_frame_sender: StdSender<Option<FramePacket>>,
) -> Result<(JoinHandle<()>, CrossbeamSender<FramePacket>, Settings<CrossbeamSender<FramePacket>>), String> {
    let (cb_sender, cb_receiver): (
        CrossbeamSender<FramePacket>,
        CrossbeamReceiver<FramePacket>,
    ) = crossbeam_channel::unbounded();

    let worker_thread_py_sender = python_frame_sender.clone();
    let worker_thread_handle = thread::Builder::new()
        .name("wgc_worker_thread".into())
        .spawn(move || {
            // ... (worker thread logic as before, it receives FramePacket) ...
            println!("[WGC Worker Thread] Started. Waiting for frames.");
            loop {
                match cb_receiver.recv() {
                    Ok(packet) => { 
                        if worker_thread_py_sender.send(Some(packet)).is_err() {
                            eprintln!("[WGC Worker Thread] Python mpsc receiver disconnected. Stopping.");
                            break;
                        }
                    }
                    Err(_) => {
                        eprintln!("[WGC Worker Thread] Crossbeam channel disconnected. Stopping worker.");
                        break;
                    }
                }
            }
            println!("[WGC Worker Thread] Stopped.");
        })
        .map_err(|e| format!("Failed to spawn WGC worker thread: {:?}", e))?;

    let capture_handler_flags = cb_sender.clone(); 
    
    // Settings::new for windows-capture v1.4.3 (5 args, no Result)
    let settings = Settings::new(
        capture_item,                     
        CursorCaptureSettings::Disabled, // Using Enum, hoping this compiles now with correct windows features
        DrawBorderSettings::Disabled,    // Using Enum
        ColorFormat::Bgra8,            
        capture_handler_flags // This is CrossbeamSender<FramePacket>, used by Context for Handler::new
    ); 
    // No .map_err() or ? needed here for v1.4.3 as it's a const fn returning Self

    Ok((worker_thread_handle, cb_sender, settings))
}


