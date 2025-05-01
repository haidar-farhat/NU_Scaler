use scrap::{Capturer, Display};
use std::io::ErrorKind;

// Windows API / Graphics Capture imports
use windows::core::{ComInterface, HSTRING, PCWSTR};
use windows::Foundation::Metadata::ApiInformation;
use windows::Graphics::Capture::{GraphicsCaptureItem, Direct3D11CaptureFramePool, GraphicsCaptureSession};
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_11_0;
use windows::Win32::Graphics::Direct3D11::*
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED, IGraphicsCaptureItemInterop};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, IsWindowVisible, FindWindowW, GetWindowRect};
// GDI imports might still be needed for list_windows or fallbacks, keep for now
use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC, CreateCompatibleDC, CreateCompatibleBitmap, SelectObject, BitBlt, DeleteObject, DeleteDC, GetDIBits, SRCCOPY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS};

// windows-capture integration
use windows_capture::capture::WindowsCaptureHandler;
use windows_capture::frame::Frame;
use windows_capture::monitor::Monitor;
use windows_capture::window::Window;

// Use image crate imports here too
// use image::ImageBuffer; // No longer saving debug image here
// use image::Bgra;

#[derive(Debug, Clone)]
pub enum CaptureTarget {
    FullScreen,
    WindowByTitle(String),
    // Add WindowByHandle for potential future use with windows-capture
    // WindowByHandle(isize), // Representing HWND
    Region { x: i32, y: i32, width: u32, height: u32 },
}

pub trait RealTimeCapture {
    fn start(&mut self, target: CaptureTarget) -> Result<(), String>;
    fn stop(&mut self);
    // Return will likely be BGRA from texture copy
    fn get_frame(&mut self) -> Option<(Vec<u8>, usize, usize)>; 
    fn list_windows() -> Vec<String> where Self: Sized;
}

// Structure to hold state for Windows Graphics Capture
struct WgcCaptureState {
    session: Option<GraphicsCaptureSession>,
    frame_pool: Option<Direct3D11CaptureFramePool>,
    d3d_device: Option<ID3D11Device>,
    d3d_context: Option<ID3D11DeviceContext>,
    capture_width: u32,
    capture_height: u32,
    // We need a way to receive frames, maybe use a channel or callback later
    // For now, try_get_next_frame might block or require polling
    // last_frame: std::sync::Mutex<Option<Vec<u8>>> // Example: Needs more thought
}

pub struct ScreenCapture {
    running: bool,
    // Fullscreen Capture (scrap)
    scrap_capturer: Option<Capturer>,
    // Window Capture (WGC)
    wgc_state: Option<WgcCaptureState>,
    // Common state
    width: usize,
    height: usize,
    target: Option<CaptureTarget>,
}

impl ScreenCapture {
    pub fn new() -> Self {
        // Initialize COM and WinRT
        unsafe {
            // Apartment threaded is often needed for UI/Capture APIs
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            // WinRT initialization
            let _ = RoInitialize(RO_INIT_MULTITHREADED); 
        }
        Self {
            running: false,
            scrap_capturer: None,
            wgc_state: None,
            width: 0,
            height: 0,
            target: None,
        }
    }
    // list_windows remains the same for now (uses GDI/EnumWindows)
    pub fn list_windows() -> Vec<String> {
         #[cfg(target_os = "windows")]
         {
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
                let _ = EnumWindows(Some(enum_windows_proc), LPARAM(&mut titles as *mut _ as isize));
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

    fn stop_wgc(&mut self) {
         if let Some(mut state) = self.wgc_state.take() {
            if let Some(session) = state.session.take() {
                let _ = session.Close(); // Close the capture session
                self.debug_print("WGC session closed.");
            }
             if let Some(frame_pool) = state.frame_pool.take() {
                 frame_pool.Close().ok(); // Close the frame pool
                 self.debug_print("WGC frame pool closed.");
             }
            // D3D device/context are typically managed elsewhere or dropped automatically
            state.d3d_device = None;
            state.d3d_context = None;
        }
    }

     fn start_wgc(&mut self, hwnd: HWND) -> Result<(), String> {
        self.debug_print(&format!("Starting WGC for HWND: {:?}", hwnd));
        // Check if Graphics Capture is supported
        if !ApiInformation::IsApiContractPresentByMajor("Windows.Foundation.UniversalApiContract", 7).unwrap_or(false) {
            return Err("Windows Graphics Capture requires Windows 10 version 1903 (Build 18362) or later.".to_string());
        }

        // Get CaptureItem using Interop
        let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
            .map_err(|e| format!("Failed to get IGraphicsCaptureItemInterop factory: {:?}", e))?;
        let item = unsafe { interop.CreateForWindow(hwnd) }
            .map_err(|e| format!("Failed to create GraphicsCaptureItem for HWND: {:?}", e))?;
        let item_size = item.Size().map_err(|e| format!("Failed to get capture item size: {:?}", e))?;
        let width = item_size.Width as u32;
        let height = item_size.Height as u32;
        if width == 0 || height == 0 {
            return Err(format!("Target window for WGC has zero dimensions ({}x{}).", width, height));
        }
        self.width = width as usize;
        self.height = height as usize;
        self.debug_print(&format!("WGC Target Size: {}x{}", width, height));

        // Create D3D11 Device and Context
        let mut d3d_device: Option<ID3D11Device> = None;
        let mut d3d_context: Option<ID3D11DeviceContext> = None;
        unsafe {
             D3D11CreateDevice(
                None, // Adapter: None for default
                D3D11_DRIVER_TYPE_HARDWARE,
                None, // Software module handle
                D3D11_CREATE_DEVICE_BGRA_SUPPORT, // Flags: BGRA support is crucial
                Some(&[D3D_FEATURE_LEVEL_11_0]), // Feature levels
                D3D11_SDK_VERSION,
                Some(&mut d3d_device),
                None, // Feature level out
                Some(&mut d3d_context),
            ).map_err(|e| format!("D3D11CreateDevice failed: {:?}", e))?;
        }
        let d3d_device = d3d_device.ok_or_else(|| "D3D11CreateDevice succeeded but returned null device".to_string())?;
        let d3d_context = d3d_context.ok_or_else(|| "D3D11CreateDevice succeeded but returned null context".to_string())?;
        self.debug_print("Created D3D11 device and context.");

        // Get IDirect3DDevice interface (WinRT type) from ID3D11Device (COM type)
        let dxgi_device: IDXGIDevice = d3d_device.cast()
            .map_err(|e| format!("Failed to cast D3D11Device to IDXGIDevice: {:?}", e))?;
        let d3d_winrt_device: IDirect3DDevice = dxgi_device.cast()
             .map_err(|e| format!("Failed to cast IDXGIDevice to IDirect3DDevice: {:?}", e))?;

        // Create FramePool and Session
        let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &d3d_winrt_device,
            DirectXPixelFormat::B8G8R8A8UIntNormalized, // Output format BGRA
            1, // Number of buffers
            item_size,
        ).map_err(|e| format!("Failed to create frame pool: {:?}", e))?;
        let session = frame_pool.CreateCaptureSession(&item)
             .map_err(|e| format!("Failed to create capture session: {:?}", e))?;
        
        // Set cursor capture (optional)
        session.SetIsCursorCaptureEnabled(false).ok(); 

        // Start capture (Note: Actual frames might arrive on a callback or need polling)
        session.StartCapture().map_err(|e| format!("Failed to start WGC session: {:?}", e))?;
        self.debug_print("WGC session started.");

        // Store state
        self.wgc_state = Some(WgcCaptureState {
            session: Some(session),
            frame_pool: Some(frame_pool),
            d3d_device: Some(d3d_device),
            d3d_context: Some(d3d_context),
            capture_width: width,
            capture_height: height,
        });
        self.running = true;
        Ok(())
     }
}

impl RealTimeCapture for ScreenCapture {
    fn start(&mut self, target: CaptureTarget) -> Result<(), String> {
        self.debug_print(&format!("Starting capture: {:?}", target));
        self.target = Some(target.clone());
        self.stop(); // Stop previous capture

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
            CaptureTarget::WindowByTitle(ref title) => {
                #[cfg(target_os = "windows")]
                {
                    // Use WGC for window
                    use std::ffi::OsStr;
                    use std::os::windows::ffi::OsStrExt;
                    let wide: Vec<u16> = OsStr::new(&title).encode_wide().chain(Some(0)).collect();
                    let hwnd = unsafe { FindWindowW(None, PCWSTR::from_raw(wide.as_ptr())) };
                    if hwnd.0 == 0 {
                        return Err(format!("Window '{}' not found", title));
                    }
                    self.start_wgc(hwnd)
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
                // Use scrap capturer for fullscreen
                if let Some(capturer) = self.scrap_capturer.as_mut() {
                    match capturer.frame() {
                        Ok(frame) => {
                            // scrap gives BGRA, convert to RGBA
                            let expected_len = self.width * self.height * 4;
                            if frame.len() != expected_len {
                                self.debug_print(&format!("Frame size mismatch (FullScreen)! Expected: {}, Got: {}", expected_len, frame.len()));
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
                            self.debug_print(&format!("Frame capture error (FullScreen): {}", e));
                            self.stop();
                            None
                        }
                    }
                } else {
                    None
                }
            }
            Some(CaptureTarget::WindowByTitle(_)) |
            Some(CaptureTarget::Region { .. })=> { // Assuming Region uses WGC if implemented
                 #[cfg(target_os = "windows")]
                 {
                    if let Some(state) = self.wgc_state.as_mut() {
                        if let Some(frame_pool) = &state.frame_pool {
                            match frame_pool.TryGetNextFrame() { // Use TryGetNextFrame for polling
                                Ok(frame) => {
                                    // Access the D3D11 texture surface
                                    let surface = frame.Surface().ok()?.cast::<IDXGISurface>().ok()?;
                                    let texture: ID3D11Texture2D = surface.cast().ok()?;

                                    // --- Copy texture to CPU --- (This is the complex part)
                                    // 1. Get texture description
                                    let mut desc = D3D11_TEXTURE2D_DESC::default();
                                    unsafe { texture.GetDesc(&mut desc) };

                                    // 2. Create a staging texture description (CPU readable)
                                    let staging_desc = D3D11_TEXTURE2D_DESC {
                                        Width: desc.Width,
                                        Height: desc.Height,
                                        MipLevels: 1,
                                        ArraySize: 1,
                                        Format: desc.Format, // Should be BGRA8 normally
                                        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                                        Usage: D3D11_USAGE_STAGING,
                                        BindFlags: D3D11_BIND_FLAG(0),
                                        CPUAccessFlags: D3D11_CPU_ACCESS_READ,
                                        MiscFlags: D3D11_RESOURCE_MISC_FLAG(0),
                                    };

                                    // 3. Create the staging texture
                                    let mut staging_texture: Option<ID3D11Texture2D> = None;
                                    if unsafe { state.d3d_device.as_ref()?.CreateTexture2D(&staging_desc, None, Some(&mut staging_texture)) }.is_err() {
                                        self.debug_print("Failed to create staging texture");
                                        return None;
                                    }
                                    let staging_texture = staging_texture?;

                                    // 4. Copy the frame texture to the staging texture
                                    unsafe {
                                        state.d3d_context.as_ref()?.CopyResource(
                                            Some(&staging_texture.cast().unwrap()), 
                                            Some(&texture.cast().unwrap()),
                                        );
                                    }

                                    // 5. Map the staging texture
                                    let mut mapped_resource = D3D11_MAPPED_SUBRESOURCE::default();
                                    if unsafe { state.d3d_context.as_ref()?.Map(Some(&staging_texture.cast().unwrap()), 0, D3D11_MAP_READ, 0, Some(&mut mapped_resource)) }.is_err() {
                                        self.debug_print("Failed to map staging texture");
                                        return None;
                                    }
                                    
                                    // 6. Read the data
                                    let width = desc.Width as usize;
                                    let height = desc.Height as usize;
                                    let data_slice: &[u8] = unsafe { 
                                        std::slice::from_raw_parts(
                                            mapped_resource.pData as *const u8, 
                                            height * mapped_resource.RowPitch as usize // Use RowPitch for correct size
                                        )
                                    };
                                    
                                    // Copy data row by row if RowPitch != width * 4
                                    let bytes_per_pixel = 4; // Assuming BGRA8
                                    let mut cpu_buffer = vec![0u8; width * height * bytes_per_pixel];
                                    let src_pitch = mapped_resource.RowPitch as usize;
                                    let dst_pitch = width * bytes_per_pixel;

                                    for row in 0..height {
                                        let src_offset = row * src_pitch;
                                        let dst_offset = row * dst_pitch;
                                        cpu_buffer[dst_offset..dst_offset + dst_pitch]
                                            .copy_from_slice(&data_slice[src_offset..src_offset + dst_pitch]);
                                    }
                                    
                                    // 7. Unmap the texture
                                    unsafe { state.d3d_context.as_ref()?.Unmap(Some(&staging_texture.cast().unwrap()), 0) };

                                    // cpu_buffer now contains the BGRA data
                                    Some((cpu_buffer, width, height))
                                },
                                Err(_) => None, // No frame available yet
                            }
                        } else {
                            None // Frame pool not initialized
                        }
                    } else {
                        None // WGC state not initialized
                    }
                 }
                 #[cfg(not(target_os = "windows"))]
                 { None }
            }
            None => None, // No target set
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