use anyhow::{Result /*, anyhow*/};
// use image::ImageFormat;
use scrap::{Capturer, Display /*, Frame as ScrapFrame*/};
use std::io::ErrorKind;
// std::sync::mpsc is still used for the final channel to Python
use std::sync::mpsc::{self, Receiver as StdReceiver, Sender as StdSender};
// use std::sync::Mutex; // This was unused, removing for now. Add back if needed for other parts.
use std::thread::{self, JoinHandle};
use std::time::Instant;

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

// Define a type for the data passed through the crossbeam channel
// Option is used to signal shutdown (None)
type FramePacket = Option<(Vec<u8>, usize, usize)>;

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
    // This sender is now a crossbeam_channel sender for high-speed, lock-free handoff
    frame_sender: CrossbeamSender<FramePacket>,
}

// Implement the correct trait from windows-capture v1.4
impl GraphicsCaptureApiHandler for CaptureHandler {
    // The Flags type for windows-capture context is now our crossbeam_channel Sender
    type Flags = CrossbeamSender<FramePacket>;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    // Use the required `new` method signature
    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        // ctx.flags is the CrossbeamSender passed in during Settings creation
        Ok(Self { frame_sender: ctx.flags })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _capture_control: InternalCaptureControl
    ) -> Result<(), Self::Error>
    {
        // +++ Start of on_frame_arrived interval logging (to console via println!) +++
        thread_local! {
            static LAST_FRAME_ARRIVAL_TIME_CONSOLE_LOG: Cell<Option<Instant>> = Cell::new(None);
        }
        let now = Instant::now();
        LAST_FRAME_ARRIVAL_TIME_CONSOLE_LOG.with(|last_time_cell| {
            if let Some(last_time) = last_time_cell.get() {
                let delta = now.duration_since(last_time);
                // Ensure this println! is very unlikely to be missed or buffered away indefinitely.
                // Adding a more prominent tag.
                println!("RUST_CONSOLE_LOG [CaptureHandler::on_frame_arrived] Interval since last call: {:?}", delta);
            } else {
                println!("RUST_CONSOLE_LOG [CaptureHandler::on_frame_arrived] First call.");
            }
            last_time_cell.set(Some(now));
        });
        // +++ End of on_frame_arrived interval logging (to console via println!) +++

        let total_start = Instant::now(); // This is for timing the work *within* on_frame_arrived

        let width = frame.width() as usize;
        let height = frame.height() as usize;

        let get_buffer_start = Instant::now();
        match frame.buffer() {
            Ok(mut fb) => {
                let get_buffer_duration = get_buffer_start.elapsed();
                let as_nopadding_start = Instant::now();
                match fb.as_nopadding_buffer() {
                    Ok(nopadding_byte_slice) => {
                        let as_nopadding_duration = as_nopadding_start.elapsed();
                        let to_vec_start = Instant::now();
                        // This to_vec() is still here, but the subsequent send is much faster.
                        // Ideally, if windows-capture could give an owned buffer that is Send,
                        // we could defer this. For now, let's assume to_vec() is necessary
                        // to make FramePacket Send-able.
                        let frame_data = nopadding_byte_slice.to_vec();
                        let to_vec_duration = to_vec_start.elapsed();

                        let send_start = Instant::now();
                        // Send to the crossbeam channel. This should be very fast and non-blocking (if unbounded).
                        if self.frame_sender.send(Some((frame_data, width, height))).is_err() {
                            // Receiver likely dropped, means worker thread shut down or WGC is stopping.
                            eprintln!("[CaptureHandler] Crossbeam receiver disconnected during frame send.");
                            // This might signal that the capture session should terminate.
                            // The windows-capture crate might handle this by eventually calling on_closed.
                        }
                        let send_duration = send_start.elapsed();
                        let total_duration = total_start.elapsed();
                        println!(
                            "[CaptureHandler] TIMING: get_buffer: {:?}, as_nopadding: {:?}, to_vec: {:?}, crossbeam_send: {:?}, TOTAL: {:?}",
                            get_buffer_duration, as_nopadding_duration, to_vec_duration, send_duration, total_duration
                        );
                    }
                    Err(e) => {
                        let as_nopadding_duration = as_nopadding_start.elapsed();
                        let total_duration = total_start.elapsed();
                        eprintln!(
                            "[CaptureHandler] TIMING: get_buffer: {:?}, as_nopadding_ERR: {:?}, TOTAL_ERR: {:?}",
                            get_buffer_duration, as_nopadding_duration, total_duration
                        );
                        eprintln!("[CaptureHandler] Failed to get no-padding buffer: {:?}", e);
                    }
                }
            }
            Err(e) => {
                let get_buffer_duration = get_buffer_start.elapsed();
                let total_duration = total_start.elapsed();
                eprintln!(
                    "[CaptureHandler] TIMING: get_buffer_ERR: {:?}, TOTAL_ERR: {:?}",
                    get_buffer_duration, total_duration
                );
                eprintln!("[CaptureHandler] Failed to get frame buffer: {:?}", e);
                return Err(Box::new(e));
            }
        }
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("[CaptureHandler] Capture session closed (on_closed called).");
        // Signal worker thread to stop by sending None through the crossbeam channel
        if self.frame_sender.send(None).is_err() {
            eprintln!("[CaptureHandler] Crossbeam receiver already disconnected on_closed.");
        }
        Ok(())
    }
}
// --- End Handler ---

pub struct ScreenCapture {
    running: bool,
    scrap_capturer: Option<Capturer>,
    
    // WGC related fields
    wgc_capture_thread: Option<JoinHandle<()>>, // Thread for windows-capture event loop
    wgc_worker_thread: Option<JoinHandle<()>>,  // Thread for processing frames from crossbeam channel
    
    // Receiver for frames from Python's perspective (fed by wgc_worker_thread)
    python_frame_receiver: Option<StdReceiver<FramePacket>>, 
    
    // To signal the WGC capture and worker threads to stop
    // The crossbeam_frame_sender is moved into CaptureHandler, but we need a way to signal its drop maybe.
    // Or rely on dropping crossbeam_frame_receiver in the worker to signal the handler's sends to fail.
    // Let's store the crossbeam_frame_sender used to start CaptureHandler to drop it explicitly if needed.
    // Actually, the crossbeam_channel sender is passed to windows-capture. Its lifetime is tied there.
    // The worker thread's crossbeam_receiver drop will signal the CaptureHandler.
    // The std_frame_sender is moved into the worker thread.
    
    width: usize,
    height: usize,
    pub target: Option<CaptureTarget>,
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self {
            running: false,
            scrap_capturer: None,
            wgc_capture_thread: None,
            wgc_worker_thread: None,
            python_frame_receiver: None, // This will be the std mpsc receiver
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
        self.debug_print("Stopping WGC...");

        // The CaptureHandler's on_closed should send a None to the crossbeam channel.
        // The worker thread will see this None and shut down, dropping the python_frame_sender.
        // Python's get_frame will then see the channel disconnected.

        // We need to ensure the windows-capture session is properly stopped.
        // The `windows-capture` crate implies that dropping the `Settings` object
        // or the context signals stop, but it's usually event-driven.
        // The `InternalCaptureControl` in `on_frame_arrived` has `stop()`, but we don't store it.
        // Relying on on_closed to propagate is the main path.

        // Join the WGC capture thread (runs windows-capture's event loop)
        if let Some(handle) = self.wgc_capture_thread.take() {
            self.debug_print("Joining WGC capture thread...");
            if let Err(e) = handle.join() {
                eprintln!("[ScreenCapture] WGC capture thread panicked: {:?}", e);
            } else {
                self.debug_print("WGC capture thread joined.");
            }
        }

        // Join the worker thread
        if let Some(handle) = self.wgc_worker_thread.take() {
            self.debug_print("Joining WGC worker thread...");
            if let Err(e) = handle.join() {
                eprintln!("[ScreenCapture] WGC worker thread panicked: {:?}", e);
            } else {
                self.debug_print("WGC worker thread joined.");
            }
        }
        // python_frame_receiver is dropped when ScreenCapture is dropped or on next start.
        self.python_frame_receiver = None;
        self.debug_print("WGC stopped.");
    }

    #[cfg(target_os = "windows")]
    fn start_wgc(&mut self, title: String, capture_core_id: Option<usize>) -> Result<(), String> {
        self.debug_print(&format!("Starting WGC for Window Title: {}", title));

        let window = Window::from_name(&title)
            .map_err(|e| format!("Window '{}' not found or error: {:?}", title, e))?;

        // Channel for CaptureHandler (fast, lock-free) -> WGC Worker Thread
        let (cb_sender, cb_receiver): (CrossbeamSender<FramePacket>, CrossbeamReceiver<FramePacket>) = crossbeam_channel::unbounded();
        
        // Channel for WGC Worker Thread -> Python consumer (standard mpsc)
        let (py_sender, py_receiver): (StdSender<FramePacket>, StdReceiver<FramePacket>) = mpsc::channel();
        self.python_frame_receiver = Some(py_receiver);

        // --- Spawn WGC Worker Thread ---
        let worker_thread_py_sender = py_sender.clone(); // Clone sender for the worker thread
        let worker_thread_handle = thread::Builder::new()
            .name("wgc_worker_thread".to_string())
            .spawn(move || {
                #[cfg(target_os = "windows")]
                {
                    // Set worker thread priority
                    unsafe {
                        if windows::Win32::System::Threading::SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_ABOVE_NORMAL).is_err() {
                            println!("[WorkerThread] Failed to set thread priority to ABOVE_NORMAL (call returned error).");
                        } else {
                            println!("[WorkerThread] Thread priority set to ABOVE_NORMAL (call returned success).");
                        }
                    }
                }
                println!("[WGC Worker Thread] Started. Waiting for frames from crossbeam channel...");
                loop {
                    match cb_receiver.recv() { // Blocking receive from crossbeam
                        Ok(Some((frame_data, width, height))) => {
                            // Process/forward to Python consumer
                            // println!("[WGC Worker Thread] Received frame {}x{}, sending to Python channel.", width, height);
                            if worker_thread_py_sender.send(Some((frame_data, width, height))).is_err() {
                                eprintln!("[WGC Worker Thread] Python mpsc receiver disconnected. Stopping.");
                                break;
                            }
                        }
                        Ok(None) => { // Shutdown signal from CaptureHandler::on_closed
                            println!("[WGC Worker Thread] Received shutdown signal (None). Sending to Python channel and stopping.");
                            let _ = worker_thread_py_sender.send(None); // Signal Python consumer
                            break;
                        }
                        Err(_) => { // Crossbeam channel disconnected
                            eprintln!("[WGC Worker Thread] Crossbeam channel disconnected. Assuming shutdown. Stopping.");
                            let _ = worker_thread_py_sender.send(None); // Attempt to signal Python consumer
                            break;
                        }
                    }
                }
                println!("[WGC Worker Thread] Stopped.");
            })
            .map_err(|e| format!("Failed to spawn WGC worker thread: {}", e))?;
        self.wgc_worker_thread = Some(worker_thread_handle);


        // --- Prepare and Start windows-capture API ---
        // The `cb_sender` is given to `CaptureHandler` via `Settings` flags.
        let capture_handler_flags = cb_sender; 

        // Removed the separate CaptureSettings struct construction.
        // Directly use the 5-argument Settings::new constructor from windows-capture v1.4.3
        let settings = Settings::new(
            window, // Argument 1: GraphicsCaptureItem (Window implements Into<GraphicsCaptureItem>)
            CursorCaptureSettings::Disabled, // Argument 2
            DrawBorderSettings::Disabled,    // Argument 3
            ColorFormat::Bgra8,              // Argument 4
            capture_handler_flags            // Argument 5: ContextFlags (our CrossbeamSender)
        ); // Settings::new is a const fn, so it doesn't return a Result, remove .map_err().
           // If type errors persist for ::Disabled, it means the windows crate features are still not aligning.

        // Example for capture_area if you want to test cropping later:
        // This would need to be done via a method on the `settings` object if supported,
        // or by different constructor if available.
        // use windows_capture::settings::CaptureArea;
        // settings.set_capture_area(CaptureArea::ClientArea); // Or specific rect

        let capture_thread_handle = thread::Builder::new()
            .name("wgc_capture_api_thread".to_string())
            .spawn(move || {
                #[cfg(target_os = "windows")]
                {
                    // Set capture thread priority and affinity
                    unsafe {
                        if windows::Win32::System::Threading::SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST).is_err() {
                            println!("[WGC_CaptureThread] Failed to set thread priority to HIGHEST (call returned error).");
                        } else {
                            println!("[WGC_CaptureThread] Thread priority set to HIGHEST (call returned success).");
                        }
                        if let Some(core_id) = capture_core_id {
                            if core_id < (std::mem::size_of::<usize>() * 8) { // Max 64 cores for usize mask
                                let affinity_mask = 1usize << core_id;
                                if SetThreadAffinityMask(GetCurrentThread(), affinity_mask) == 0 {
                                    eprintln!("[WGC Capture Thread] Failed to set thread affinity to core {}. Error: {:?}", core_id, windows::core::Error::from_win32());
                                } else {
                                    println!("[WGC Capture Thread] Affinity set to core {}.", core_id);
                                }
                            } else {
                                eprintln!("[WGC Capture Thread] Invalid core_id {} for affinity mask.", core_id);
                            }
                        }
                    }
                }
                println!("[WGC_CaptureThread] Starting capture event loop.");
                if let Err(e) = CaptureHandler::start(settings) { // This blocks until capture stops
                    eprintln!("[WGC_CaptureThread] Capture failed/stopped: {}", e);
                    // If CaptureHandler::start errors out, on_closed might not be called.
                    // We need to ensure the worker thread is signaled to stop.
                    // The cb_sender (capture_handler_flags) is moved into `settings`.
                    // If this thread exits, the sender might be dropped, which should disconnect the channel.
                } else {
                    println!("[WGC_CaptureThread] Capture session finished gracefully.");
                }
                // Note: If CaptureHandler::start returns, it means the capture session ended.
                // on_closed should have sent None via cb_sender to the worker.
            })
            .map_err(|e| format!("Failed to spawn WGC capture API thread: {}", e))?;
        
        self.wgc_capture_thread = Some(capture_thread_handle);
        self.running = true;
        Ok(())
    }
}

impl RealTimeCapture for ScreenCapture {
    fn start(&mut self, target: CaptureTarget) -> std::result::Result<(), String> {
        self.debug_print(&format!("Starting capture: {:?}", target));
        self.target = Some(target.clone()); // Clone target for later use
        self.stop(); // Stop any existing capture first

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
                    // Example: Pin WGC capture thread to core 2. Adjust as needed.
                    let core_to_pin_capture_thread = Some(2); 
                    self.start_wgc(title, core_to_pin_capture_thread)
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
            self.debug_print("Stopping capture...");
            if self.scrap_capturer.is_some() {
                self.scrap_capturer = None;
                self.debug_print("Scrap capture stopped.");
            }
            // Stop WGC path if it was running
            self.stop_wgc(); // This now handles joining both WGC threads
            self.running = false; // Set running to false after all stop logic
            self.debug_print("Capture fully stopped.");
        }
    }

    fn get_frame(&mut self) -> Option<(Vec<u8>, usize, usize)> {
        if !self.running {
            // println!("[ScreenCapture::get_frame] Not running, returning None."); // DEBUG
            return None;
        }

        match self.target.as_ref() { // Use as_ref to avoid moving self.target
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
                    if let Some(rx) = self.python_frame_receiver.as_ref() {
                        // This is now reading from the std::sync::mpsc channel fed by the WGC worker thread
                        let mut last_frame_data: Option<(Vec<u8>, usize, usize)> = None;
                        
                        // Drain all immediately available frames, returning the latest.
                        // This is similar to the previous logic but on the python_frame_receiver.
                        loop {
                            match rx.try_recv() {
                                Ok(Some(frame_tuple)) => {
                                    // println!("[ScreenCapture::get_frame] WGC Path: Received frame from Python channel. Size: {}x{}", frame_tuple.1, frame_tuple.2);
                                    last_frame_data = Some(frame_tuple);
                                }
                                Ok(None) => { // Shutdown signal from worker
                                    self.debug_print("[ScreenCapture::get_frame] WGC Path: Received STOP signal (None) from Python channel.");
                                    last_frame_data = None; 
                                    self.stop(); 
                                    break; 
                                }
                                Err(mpsc::TryRecvError::Empty) => {
                                    // println!("[ScreenCapture::get_frame] WGC Path: Python channel empty.");
                                    break;
                                }
                                Err(mpsc::TryRecvError::Disconnected) => {
                                    self.debug_print("[ScreenCapture::get_frame] WGC Path: Python channel DISCONNECTED.");
                                    last_frame_data = None;
                                    self.stop();
                                    break;
                                }
                            }
                        }
                        if let Some((_, width, height)) = &last_frame_data {
                             self.width = *width; // Update dimensions based on received frame
                             self.height = *height;
                        }
                        last_frame_data
                    } else {
                        // eprintln!("[ScreenCapture::get_frame] WGC Path: No python_frame_receiver."); // DEBUG
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

// For Linux: Scaffold X11 window capture (not implemented yet)
#[cfg(target_os = "linux")]
mod x11_capture {
    // use x11::xlib::*;
    // TODO: Implement X11 window capture
}
