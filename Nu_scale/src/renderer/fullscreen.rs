use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering, AtomicUsize}};
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use anyhow::Result;
use eframe::{self, egui};
use egui::Vec2;
use image::RgbaImage;
use std::time::{Instant, Duration};
use log;
use egui_wgpu::WgpuConfiguration;
use winit::window::Window;
use anyhow::anyhow;

use crate::capture::common::FrameBuffer;
use crate::upscale::{Upscaler, UpscalingTechnology, UpscalingQuality};
use crate::upscale::common::UpscalingAlgorithm;
use crate::capture::CaptureTarget;
use crate::capture::ScreenCapture;

// Constants for texture size limits
const MAX_TEXTURE_SIZE: u32 = 16384; // Maximum dimension for a texture (width or height)
const MAX_TEXTURE_MEMORY_MB: u64 = 2048; // Maximum memory allowed for a texture in MB

// Define a constant for the lock file path
const LOCK_FILE_PATH: &str = "nu_scaler_fullscreen.lock";

/// WGPU render resources for direct rendering
struct WgpuRenderResources {
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    vertex_buffer: wgpu::Buffer,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    texture_size: (u32, u32),
}

/// Fullscreen shader module
const FULLSCREEN_SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vert_idx: u32) -> @builtin(position) vec4<f32> {
    let pos = array(
        vec2(-1.0, -1.0),
        vec2(3.0, -1.0),
        vec2(-1.0, 3.0),
    );
    return vec4(pos[vert_idx], 0.0, 1.0);
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(tex, sampler, uv);
}
"#;

/// Create a lock file to ensure only one instance can run fullscreen mode
fn create_lock_file() -> std::io::Result<Option<File>> {
    // Try to get the app data directory
    let lock_path = if let Some(data_dir) = dirs::data_dir() {
        let app_dir = data_dir.join("NU_Scaler");
        // Create the directory if it doesn't exist
        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir)?;
        }
        app_dir.join(LOCK_FILE_PATH)
    } else {
        std::path::PathBuf::from(LOCK_FILE_PATH)
    };
    
    // If lock file exists, check if it's stale
    if lock_path.exists() {
        let is_stale = match std::fs::read_to_string(&lock_path) {
            Ok(content) => {
                // Read PID from lock file
                if let Ok(pid) = content.trim().parse::<u32>() {
                    // On Windows, check if the process exists
                    #[cfg(windows)]
                    {
                        use std::process::Command;
                        // Try to query the process - if it doesn't exist, this will fail
                        let output = Command::new("tasklist")
                            .args(&["/FI", &format!("PID eq {}", pid), "/NH"])
                            .output();
                        
                        match output {
                            Ok(output) => {
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                // If the process is not in the list, the lock is stale
                                let is_stale = !output_str.contains(&pid.to_string());
                                if is_stale {
                                    log::info!("Detected stale lock file from non-existent process {}", pid);
                                }
                                is_stale
                            },
                            Err(_) => {
                                // If we can't check, assume it's not stale
                                false
                            }
                        }
                    }
                    
                    // On Unix systems, check differently
                    #[cfg(unix)]
                    {
                        use std::process::Command;
                        // Check if the process exists
                        let output = Command::new("ps")
                            .args(&["-p", &pid.to_string()])
                            .output();
                            
                        match output {
                            Ok(output) => {
                                // The process doesn't exist if ps returns no lines beyond the header
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                let lines = output_str.lines().count();
                                let is_stale = lines <= 1;
                                if is_stale {
                                    log::info!("Detected stale lock file from non-existent process {}", pid);
                                }
                                is_stale
                            },
                            Err(_) => false
                        }
                    }
                    
                    // Default for other platforms
                    #[cfg(not(any(windows, unix)))]
                    {
                        // Can't check on other platforms, assume it's not stale
                        false
                    }
                } else {
                    // Invalid PID in lock file, consider it stale
                    log::warn!("Invalid PID in lock file, treating as stale");
                    true
                }
            },
            Err(_) => {
                // Can't read lock file, assume it's stale
                log::warn!("Couldn't read lock file, treating as stale");
                true
            }
        };
        
        // Remove stale lock file
        if is_stale {
            log::info!("Removing stale lock file at {:?}", lock_path);
            let _ = std::fs::remove_file(&lock_path);
        } else {
            log::warn!("Lock file is active (not stale) at {:?}", lock_path);
            return Ok(None);
        }
    }
    
    // Try to create the lock file with exclusive access
    match OpenOptions::new().write(true).create_new(true).open(&lock_path) {
        Ok(file) => {
            log::info!("Created lock file at {:?}", lock_path);
            // Write the current process ID to the lock file
            if let Err(e) = std::io::Write::write_all(&mut std::io::BufWriter::new(&file), 
                                                     format!("{}", std::process::id()).as_bytes()) {
                log::warn!("Failed to write PID to lock file: {}", e);
            }
            Ok(Some(file))
        },
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            log::warn!("Lock file already exists at {:?}, another instance may be running", lock_path);
            Ok(None)
        },
        Err(e) => {
            log::error!("Failed to create lock file: {}", e);
            Err(e)
        }
    }
}

/// Remove the lock file when the application exits
fn remove_lock_file() {
    let lock_path = if let Some(data_dir) = dirs::data_dir() {
        data_dir.join("NU_Scaler").join(LOCK_FILE_PATH)
    } else {
        std::path::PathBuf::from(LOCK_FILE_PATH)
    };
    
    if let Err(e) = std::fs::remove_file(&lock_path) {
        log::warn!("Failed to remove lock file: {}", e);
    } else {
        log::info!("Removed lock file at {:?}", lock_path);
    }
}

/// Performance metrics for the fullscreen upscaler
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    /// Time taken to capture the frame
    capture_time: Duration,
    /// Time taken to upscale the frame
    upscale_time: Duration,
    /// Time taken to render the frame
    render_time: Duration,
    /// Total time for processing a frame
    total_frame_time: Duration,
    /// Number of frames processed
    frame_count: u64,
    /// Number of black frames detected in a row
    black_frame_count: u32,
    /// Number of consecutive errors
    error_count: u32,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            capture_time: Duration::from_millis(0),
            upscale_time: Duration::from_millis(0),
            render_time: Duration::from_millis(0),
            total_frame_time: Duration::from_millis(0),
            frame_count: 0,
            black_frame_count: 0,
            error_count: 0,
        }
    }
}

/// State for WGPU rendering
struct WgpuState {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    format: wgpu::TextureFormat,
    render_resources: Option<WgpuRenderResources>,
}

impl WgpuState {
    fn from_render_state(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            device,
            queue,
            format,
            render_resources: None,
        }
    }

    fn create_render_resources(&mut self, width: u32, height: u32) -> Result<()> {
        if self.render_resources.is_none() {
            let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Fullscreen Shader"),
                source: wgpu::ShaderSource::Wgsl(FULLSCREEN_SHADER.into()),
            });
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Upscaled Frame Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Frame Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });
            let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Frame Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Frame Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
            let render_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Fullscreen Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
            let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Fullscreen Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });
            let vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: 0,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
            let new_resources = WgpuRenderResources {
                render_pipeline,
                bind_group,
                bind_group_layout,
                vertex_buffer,
                texture,
                texture_view,
                sampler,
                texture_size: (width, height),
            };
            self.render_resources = Some(new_resources);
        }
        Ok(())
    }

    fn update_texture(&mut self, frame: &RgbaImage) -> Result<()> {
        let (width, height) = frame.dimensions();
        let needs_resize = {
            if let Some(resources) = &self.render_resources {
                resources.texture_size != (width, height)
            } else {
                true
            }
        };
        if needs_resize {
            self.create_render_resources(width, height)?;
        }
        if let Some(resources) = &mut self.render_resources {
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &resources.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                frame.as_raw(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }
        Ok(())
    }
}

/// Fullscreen upscaler UI
pub struct FullscreenUpscalerUi<'a> {
    /// Frame buffer for capturing frames
    frame_buffer: Arc<FrameBuffer>,
    /// Stop signal for capture thread
    stop_signal: Arc<AtomicBool>,
    /// Upscaler implementation
    upscaler: Box<dyn Upscaler + Send + Sync>,
    /// Upscaling algorithm
    algorithm: Option<UpscalingAlgorithm>,
    /// Processing thread for offloading heavy operations
    processing_thread: Option<std::thread::JoinHandle<()>>,
    /// Time of last frame
    last_frame_time: std::time::Instant,
    /// FPS counter
    fps: f32,
    /// Number of frames processed
    frames_processed: u64,
    /// Current upscaler name
    upscaler_name: String,
    /// Current upscaling quality
    upscaler_quality: UpscalingQuality,
    /// Show performance overlay
    show_overlay: bool,
    /// Performance metrics history
    fps_history: Vec<f32>,
    /// Upscaling time history (ms)
    upscale_time_history: Vec<f32>,
    /// Last upscale time (ms)
    last_upscale_time: f32,
    /// Input size
    input_size: (u32, u32),
    /// Output size 
    output_size: (u32, u32),
    /// Source window position (x, y, width, height)
    source_window_info: Option<(i32, i32, u32, u32)>,
    /// Capture target used for this upscaling session
    capture_target: Option<CaptureTarget>,
    /// Performance metrics
    performance_metrics: PerformanceMetrics,
    /// Last update time
    last_update_time: Option<Instant>,
    /// Memory pressure counter
    memory_pressure_counter: Option<u32>,
    /// Flag to reinitialize on next update
    requires_reinitialization: bool,
    /// Flag to use a different capture method
    fallback_capture: bool,
    /// Flag to enable frame skipping when lagging
    enable_frame_skipping: bool,
    /// Time budget for frame processing in ms
    frame_time_budget: f32,
    /// Pending frame to be processed
    pending_frame: Option<RgbaImage>,
    /// WGPU state for rendering
    wgpu_state: Option<WgpuState>,
    /// Triple buffer for frame data
    triple_buffer: [Arc<Mutex<Option<RgbaImage>>>; 3],
    /// Current buffer index
    current_buffer_index: AtomicUsize,
    /// Surface for WGPU rendering
    surface: Option<wgpu::Surface<'a>>,
    /// Window reference
    window: Option<Arc<Window>>,
    /// egui::TextureId for the WGPU texture
    egui_texture_id: Option<egui::TextureId>,
    /// egui_wgpu::Renderer for rendering
    egui_wgpu_renderer: Option<egui_wgpu::Renderer>,
}

impl<'a> FullscreenUpscalerUi<'a> {
    /// Create a new fullscreen upscaler UI
    fn new(
        cc: &eframe::CreationContext<'_>,
        frame_buffer: Arc<FrameBuffer>,
        stop_signal: Arc<AtomicBool>,
        upscaler: Box<dyn Upscaler + Send + Sync>,
        algorithm: Option<UpscalingAlgorithm>,
        capture_target: CaptureTarget,
    ) -> Self {
        let wgpu_state = cc.wgpu_render_state.as_ref().map(|rs| {
            WgpuState::from_render_state(
                rs.device.clone(),
                rs.queue.clone(),
                rs.target_format,
            )
        });
        let egui_wgpu_renderer = if let Some(ref state) = wgpu_state {
            Some(egui_wgpu::Renderer::new(
                &state.device,
                state.format,
                None,
                1,
            ))
        } else {
            None
        };
        Self {
            frame_buffer: frame_buffer.clone(),
            stop_signal: stop_signal.clone(),
            upscaler,
            algorithm,
            processing_thread: None,
            last_frame_time: std::time::Instant::now(),
            fps: 0.0,
            frames_processed: 0,
            upscaler_name: upscaler.name().to_string(),
            upscaler_quality: upscaler.quality(),
            show_overlay: true,
            fps_history: Vec::with_capacity(120),
            upscale_time_history: Vec::with_capacity(120),
            last_upscale_time: 0.0,
            input_size: (0, 0),
            output_size: (0, 0),
            source_window_info: None,
            capture_target: Some(capture_target),
            performance_metrics: PerformanceMetrics::new(),
            last_update_time: None,
            memory_pressure_counter: None,
            requires_reinitialization: false,
            fallback_capture: false,
            enable_frame_skipping: true,
            frame_time_budget: 16.0, // ~60 FPS
            pending_frame: None,
            wgpu_state,
            triple_buffer: [
                Arc::new(Mutex::new(None)),
                Arc::new(Mutex::new(None)),
                Arc::new(Mutex::new(None)),
            ],
            current_buffer_index: AtomicUsize::new(0),
            surface: None, // Not used directly
            window: None, // Not used directly
            egui_texture_id: None,
            egui_wgpu_renderer,
        }
    }
    
    /// Write a frame to the triple buffer
    fn write_frame(&self, frame: RgbaImage) {
        let next_index = (self.current_buffer_index.load(Ordering::Acquire) + 1) % 3;
        if let Ok(mut buffer) = self.triple_buffer[next_index].lock() {
            *buffer = Some(frame);
            self.current_buffer_index.store(next_index, Ordering::Release);
        }
    }
    
    /// Read the current frame from the triple buffer
    fn read_frame(&self) -> Option<RgbaImage> {
        let index = self.current_buffer_index.load(Ordering::Acquire);
        if let Ok(buffer) = self.triple_buffer[index].lock() {
            buffer.clone()
        } else {
            None
        }
    }
    
    /// Update the texture with a new frame
    fn update_texture(&mut self, frame: &RgbaImage) -> Result<()> {
        if let (Some(wgpu_state), Some(renderer)) = (&mut self.wgpu_state, &mut self.egui_wgpu_renderer) {
            wgpu_state.update_texture(frame)?;
            if let Some(resources) = &wgpu_state.render_resources {
                let texture_id = renderer.register_native_texture(
                    &wgpu_state.device,
                    &resources.texture_view,
                    wgpu::FilterMode::Linear,
                );
                self.egui_texture_id = Some(texture_id);
            }
        }
        Ok(())
    }
    
    /// Render the current frame
    fn render(&mut self) -> Result<()> {
        // Check for surface and read the frame first
        let frame = if let Some(_surface) = &self.surface {
            self.read_frame().ok_or(anyhow!("Frame not available"))?
        } else {
            return Ok(());
        };
        // Now handle wgpu_state mutably
        if let Some(wgpu_state) = &mut self.wgpu_state {
            wgpu_state.update_texture(&frame)?;
        }
        Ok(())
    }

    fn create_texture(&self, device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Input Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }

    fn check_and_reinitialize_upscaler(&mut self) {
        // Implementation for reinitializing the upscaler
        if let Some(wgpu_state) = &mut self.wgpu_state {
            if let Err(e) = wgpu_state.create_render_resources(self.input_size.0, self.input_size.1) {
                log::error!("Failed to reinitialize upscaler: {}", e);
            }
        }
    }

    fn draw_performance_overlay(&mut self, ui: &mut egui::Ui) {
        // Implementation for drawing performance metrics
        ui.vertical(|ui| {
            ui.label(format!("FPS: {:.1}", self.fps));
            ui.label(format!("Frame Time: {:.1}ms", self.last_upscale_time));
            ui.label(format!("Input: {}x{}", self.input_size.0, self.input_size.1));
            ui.label(format!("Output: {}x{}", self.output_size.0, self.output_size.1));
        });
    }

    fn update_source_window_info(&mut self, ctx: &egui::Context) {
        if let Some(rect) = ctx.input(|i| i.viewport().inner_rect) {
            self.source_window_info = Some((
                rect.left() as i32,
                rect.top() as i32,
                rect.width() as u32,
                rect.height() as u32,
            ));
        } else {
            log::warn!("Could not get viewport rectangle");
            self.source_window_info = Some((0, 0, 1280, 720));
        }
    }

    fn cleanup(&mut self) {
        // Implementation for cleanup
        if let Some(wgpu_state) = &mut self.wgpu_state {
            wgpu_state.render_resources = None;
        }
        self.upscaler.cleanup();
    }

    fn start_processing_thread(&mut self) {
        let frame_buffer = self.frame_buffer.clone();
        let stop_signal = self.stop_signal.clone();
        let sender = self.triple_buffer.clone();
        self.processing_thread = Some(std::thread::spawn(move || {
            while !stop_signal.load(Ordering::SeqCst) {
                if let Ok(Some(frame)) = frame_buffer.get_latest_frame() {
                    let next_index = 0; // Replace with correct index logic
                    if let Ok(mut buf) = sender[next_index].lock() {
                        *buf = Some(frame);
                    }
                }
            }
        }));
    }

    fn set_capture_target(&mut self, target: CaptureTarget) {
        self.capture_target = Some(target);
        // Additional logic if needed
    }

    fn cleanup_textures(&mut self) {
        if let Some(wgpu_state) = &mut self.wgpu_state {
            wgpu_state.render_resources = None;
        }
    }

    pub fn render_upscaled_content(&self, ui: &mut egui::Ui) -> bool {
        self.egui_texture_id.map_or(false, |texture_id| {
            ui.image((texture_id, ui.available_size()));
            true
        })
    }
}

impl<'a> eframe::App for FullscreenUpscalerUi<'a> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if upscaler needs to be reinitialized
        if self.requires_reinitialization {
            // Clean up existing resources
            self.cleanup();
            // Reinitialize the upscaler
            self.check_and_reinitialize_upscaler();
        }
        
        // Update the texture with the latest frame
        // Use frame budget for skipping if necessary
        let frame_budget = Duration::from_millis(self.frame_time_budget as u64);
        let update_start = Instant::now();
        
        // For gaming PCs, enable adaptive frame rendering based on system performance
        let adaptive_frame_skipping = {
            if self.fps > 0.0 {
                // If we're maintaining good FPS, we can be less aggressive with skipping
                self.enable_frame_skipping && self.fps < 45.0
            } else {
                // Default to the user setting
                self.enable_frame_skipping
            }
        };
        
        // If we're already using too much time, consider skipping this frame update
        // This is especially relevant for gaming PCs that might be running other demanding apps
        let skip_processing = adaptive_frame_skipping && 
                             update_start.elapsed() > Duration::from_millis(frame_budget.as_millis() as u64 / 4);
        
        // Safe error handling to avoid crashes
        if !skip_processing {
            let latest_frame = self.read_frame();
            if let Some(ref frame_data) = latest_frame {
                match self.update_texture(frame_data) {
                    Ok(_) => {
                        // Measure frame processing time and check if we're lagging
                        let frame_time = update_start.elapsed();
                        if frame_time > frame_budget && self.enable_frame_skipping {
                            log::warn!("Frame processing took {}ms (budget: {}ms), consider adjusting settings",
                                    frame_time.as_millis(), frame_budget.as_millis());
                        }
                        
                        // Calculate FPS with higher precision for gaming PCs
                        let now = std::time::Instant::now();
                        let frame_time = now.duration_since(self.last_frame_time);
                        self.last_frame_time = now;
                        
                        // Calculate rolling FPS average - more responsive for gaming PCs
                        let current_fps = 1.0 / frame_time.as_secs_f32().max(0.0001); // Prevent division by zero
                        
                        // Update FPS history with adaptive smoothing based on stability
                        let smooth_factor = if self.fps_history.len() > 10 {
                            // Calculate FPS variance to determine smoothing factor
                            let sum: f32 = self.fps_history.iter().sum();
                            let mean = sum / self.fps_history.len() as f32;
                            let variance: f32 = self.fps_history.iter()
                                .map(|x| (x - mean).powi(2))
                                .sum::<f32>() / self.fps_history.len() as f32;
                            
                            // More stable FPS = less smoothing needed
                            if variance < 5.0 { 0.8 } else { 0.95 }
                        } else {
                            // Default smoothing for initial values
                            0.9
                        };
                        
                        self.fps = if self.fps == 0.0 {
                            current_fps
                        } else {
                            self.fps * smooth_factor + current_fps * (1.0 - smooth_factor)
                        };
                        
                        // Keep the last 120 frames of history for the graph
                        self.fps_history.push(self.fps);
                        if self.fps_history.len() > 120 {
                            self.fps_history.remove(0);
                        }
                        
                        // Update upscale time history
                        let upscale_time_ms = self.performance_metrics.upscale_time.as_secs_f32() * 1000.0;
                        self.last_upscale_time = upscale_time_ms;
                        self.upscale_time_history.push(upscale_time_ms);
                        if self.upscale_time_history.len() > 120 {
                            self.upscale_time_history.remove(0);
                        }
                        
                        // Update frame counter
                        self.frames_processed += 1;
                        
                        // Update input/output size for display
                        let index = self.current_buffer_index.load(Ordering::Acquire);
                        if let Ok(texture_guard) = self.triple_buffer[index].lock() {
                            if let Some(texture) = texture_guard.as_ref() {
                                let (w, h) = texture.dimensions();
                                self.output_size = (w, h);
                            }
                        }
                        
                        // Update input size from the latest frame
                        if let Ok(Some(frame)) = self.frame_buffer.get_latest_frame() {
                            self.input_size = (frame.width(), frame.height());
                        }
                    },
                    Err(e) => {
                        log::error!("Error updating texture: {}", e);
                        
                        // Increment error counter and trigger recovery if needed
                        self.performance_metrics.error_count += 1;
                        if self.performance_metrics.error_count > 5 {
                            log::warn!("Multiple texture update errors, triggering recovery");
                            self.cleanup();
                            self.requires_reinitialization = true;
                            self.performance_metrics.error_count = 0;
                        }
                    }
                }
            }
        } else {
            // Log skipped frame processing
            log::debug!("Skipped frame processing due to time constraints");
        }
        
        // Check for ESC key to exit fullscreen mode
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            // Signal the capture thread to stop and clean up resources
            self.stop_signal.store(true, Ordering::SeqCst);
            self.cleanup();
            
            // Close the application
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        
        // Check for F1 key to toggle performance overlay
        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            self.show_overlay = !self.show_overlay;
        }
        
        // Check for F2 key to toggle frame skipping
        if ctx.input(|i| i.key_pressed(egui::Key::F2)) {
            self.enable_frame_skipping = !self.enable_frame_skipping;
            log::info!("Frame skipping {}", if self.enable_frame_skipping { "enabled" } else { "disabled" });
        }
        
        // Force the window to be opaque black instead of transparent
        ctx.set_visuals(egui::Visuals::dark());
        
        // Use a dark background instead of transparent
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(10, 10, 10)))
            .show(ctx, |ui| {
                let texture_available = if let Ok(texture_guard) = self.triple_buffer[self.current_buffer_index.load(Ordering::Acquire)].lock() {
                    texture_guard.is_some()
                } else {
                    false
                };
                
                if texture_available {
                    // Get the texture under lock
                    if let Ok(texture_guard) = self.triple_buffer[self.current_buffer_index.load(Ordering::Acquire)].lock() {
                        if let Some(texture) = texture_guard.as_ref() {
                            // Get available size
                            let available_size = ui.available_size();
                            let (w, h) = texture.dimensions();
                            let texture_size = egui::Vec2::new(w as f32, h as f32);
                            
                            // Calculate the scaling to fit in the available space
                            // while maintaining aspect ratio
                            let aspect_ratio = texture_size.x / texture_size.y;
                            let width = available_size.x;
                            let height = width / aspect_ratio;
                            
                            // Center the image if it's smaller than the available space
                            let rect = if height <= available_size.y {
                                let y_offset = (available_size.y - height) / 2.0;
                                egui::Rect::from_min_size(
                                    egui::pos2(0.0, y_offset),
                                    Vec2::new(width, height)
                                )
                            } else {
                                let height = available_size.y;
                                let width = height * aspect_ratio;
                                let x_offset = (available_size.x - width) / 2.0;
                                egui::Rect::from_min_size(
                                    egui::pos2(x_offset, 0.0),
                                    Vec2::new(width, height)
                                )
                            };
                            
                            // Simple rendering with error handling
                            if let Some(texture_id) = self.egui_texture_id {
                                ui.put(rect, egui::Image::new((texture_id, rect.size())));
                            }
                        }
                    }
                    
                    // Draw performance overlay in the top-right corner only if enabled
                    if self.show_overlay {
                        let overlay_width = 250.0;
                        let overlay_rect = egui::Rect::from_min_size(
                            egui::pos2(ui.available_rect_before_wrap().right() - overlay_width - 10.0, 10.0),
                            Vec2::new(overlay_width, 0.0) // Height will be determined by content
                        );
                        
                        ui.allocate_ui_at_rect(overlay_rect, |ui| {
                            self.draw_performance_overlay(ui);
                        });
                    }
                } else {
                    // Show loading message if no texture is available
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("Waiting for frames...");
                            ui.add_space(10.0);
                            ui.label("If you don't see any content, please ensure the source window is visible and not minimized.");
                            ui.add_space(5.0);
                            ui.label("Press ESC to exit and try again.");
                        });
                    });
                }
            });
        
        // Use a smarter repaint strategy for gaming PCs
        let next_frame_time = if self.fps > 0.0 {
            // For high-end systems, aim for high refresh rates
            // Calculate time dynamically based on whether we're GPU or CPU bound
            if self.fps > 100.0 {
                // Very high performance - can minimize delay further
                Duration::from_micros(500)
            } else if self.fps > 75.0 {
                // Prioritize high refresh rate display
                Duration::from_millis(5)
            } else if self.fps > 45.0 {
                // Good performance, aim for 60 FPS
                Duration::from_millis(10)
            } else {
                // Lower performance, be more conservative
                Duration::from_millis(1000 / self.fps.max(30.0) as u64)
            }
        } else {
            // Default for gaming PC - try to hit 120 FPS initially
            Duration::from_millis(8)
        };
        
        // Request repaint based on performance metrics for gaming PC
        ctx.request_repaint_after(next_frame_time);
        
        // Safe window position update
        self.update_source_window_info(ctx);
    }
    
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Clean up resources when the app exits
        log::info!("Fullscreen upscaler exiting, cleaning up resources");
        self.cleanup();
    }
}

/// Create an upscaler for the given technology and quality
fn create_upscaler(
    technology: UpscalingTechnology,
    quality: UpscalingQuality,
    algorithm: Option<UpscalingAlgorithm>,
) -> Result<Box<dyn Upscaler + Send + Sync>> {
    // Special case for FSR3 since it requires extra setup
    if technology == UpscalingTechnology::FSR3 {
        if crate::upscale::fsr3::Fsr3Upscaler::is_supported() {
            log::info!("Using FSR3 with frame generation for upscaling");
            return crate::upscale::fsr3::Fsr3Upscaler::new(quality, true)
                .map(|upscaler| Box::new(upscaler) as Box<dyn Upscaler + Send + Sync>);
        } else {
            log::warn!("FSR3 not supported, falling back to alternative upscaler");
            // Fall through to standard upscaler creation
        }
    }
    
    crate::upscale::create_upscaler(technology, quality, algorithm)
}

/// Run the fullscreen upscaler UI
pub fn run_fullscreen_upscaler(
    frame_buffer: Arc<FrameBuffer>,
    stop_signal: Arc<AtomicBool>,
    technology: UpscalingTechnology,
    quality: UpscalingQuality,
    algorithm: Option<UpscalingAlgorithm>,
    capture_target: CaptureTarget,
) -> Result<(), String> {
    // Try to create a lock file to ensure only one instance runs
    let lock_file_result = create_lock_file();
    
    // Check if we got a lock
    if let Ok(None) = lock_file_result {
        return Err("Another instance of NU_Scaler is already running in fullscreen mode. Please close it before starting a new session.".to_string());
    }
    
    // Handle error cases but continue
    if let Err(e) = &lock_file_result {
        log::error!("Failed to check for running instances: {}", e);
        // Continue anyway, but log the error
    }
    
    // Create an upscaler with the given technology and quality
    let upscaler = match create_upscaler(technology, quality, algorithm) {
        Ok(u) => u,
        Err(e) => {
            // Release the lock if we fail to create the upscaler
            remove_lock_file();
            return Err(format!("Failed to create upscaler: {}", e));
        }
    };
    
    // Log the upscaler we're actually using
    log::info!("Using upscaler: {} with quality: {:?}", upscaler.name(), upscaler.quality());
    
    // Get the window info from the capture target
    let mut window_info = None;
    
    if let CaptureTarget::WindowByTitle(title) = &capture_target {
        if let Ok(capturer) = crate::capture::create_capturer() {
            if let Ok(windows) = capturer.list_windows() {
                // Find window with matching title
                if let Some(window) = windows.iter().find(|w| w.title.contains(title)) {
                    // Store window position and size
                    window_info = Some((
                        window.geometry.x,
                        window.geometry.y,
                        window.geometry.width,
                        window.geometry.height,
                    ));
                    log::info!("Found source window: {} at position {:?}", title, window_info);
                }
            }
        }
    }
    
    // If we couldn't get window info from the capture target, try getting it from a frame
    if window_info.is_none() {
        match frame_buffer.get_latest_frame() {
            Ok(Some(frame)) => {
                // Since we don't have position info, just use the dimensions
                log::info!("Using frame dimensions: {}x{}", frame.width(), frame.height());
                window_info = Some((0, 0, frame.width(), frame.height()));
            },
            _ => {
                // Try one more direct capture attempt
                if let CaptureTarget::WindowByTitle(title) = &capture_target {
                    if let Ok(mut capturer) = crate::capture::create_capturer() {
                        if let Ok(windows) = capturer.list_windows() {
                            if let Some(window) = windows.iter().find(|w| w.title.contains(title)) {
                                if let Ok(frame) = capturer.capture_frame(&CaptureTarget::WindowById(window.id.clone())) {
                                    log::info!("Direct capture successful: {}x{}", frame.width(), frame.height());
                                    window_info = Some((
                                        window.geometry.x,
                                        window.geometry.y,
                                        frame.width(),
                                        frame.height(),
                                    ));
                                }
                            }
                        }
                    }
                }
                
                // If we still have no info, use default dimensions
                if window_info.is_none() {
                    log::warn!("Could not get frame dimensions, using default 1280x720");
                    window_info = Some((0, 0, 1280, 720));
                }
            }
        };
    }
    
    // Get final window dimensions
    let (win_x, win_y, win_width, win_height) = window_info.unwrap_or((0, 0, 1280, 720));
    
    // Register cleanup handler
    let cleanup_lock = std::sync::Arc::new(());
    let cleanup_lock_weak = std::sync::Arc::downgrade(&cleanup_lock);
    std::thread::spawn(move || {
        // Wait for the lock to be dropped (when the main thread exits)
        while cleanup_lock_weak.upgrade().is_some() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        // Clean up the lock file when the app exits
        remove_lock_file();
    });

    // Create window options
    let native_options = eframe::NativeOptions {
        vsync: true,
        multisampling: 0, // Disable multisampling for performance
        depth_buffer: 0, // No depth buffer needed
        stencil_buffer: 0, // No stencil buffer needed
        hardware_acceleration: eframe::HardwareAcceleration::Required,
        renderer: eframe::Renderer::Wgpu,
        
        // Configure viewport using ViewportBuilder
        viewport: egui::ViewportBuilder::default()
            .with_title("NU_Scaler Fullscreen")
            .with_position(egui::pos2(win_x as f32, win_y as f32))
            .with_inner_size([win_width as f32, win_height as f32])
            .with_resizable(false)
            .with_decorations(false) // No decorations for fullscreen
            .with_fullscreen(true)
            .with_transparent(false),
        
        // Enable GPU features needed for upscaling
        wgpu_options: WgpuConfiguration {
            ..Default::default()
        },
        
        ..Default::default()
    };
    
    // Create clones of variables that will be moved into the closure
    let frame_buffer_clone = frame_buffer.clone();
    let stop_signal_clone = stop_signal.clone();
    let capture_target_clone = capture_target.clone();
    
    // Store any algorithm for the closure
    let algorithm_copy = algorithm.clone();
    
    // Clone capture_target_clone for use inside closure
    let capture_target_clone2 = capture_target_clone.clone();
    
    // Run the fullscreen upscaler
    eframe::run_native(
        "NU_Scaler Fullscreen",
        native_options,
        Box::new(move |cc| {
            let mut ui = FullscreenUpscalerUi::new(
                cc,
                frame_buffer_clone,
                stop_signal_clone,
                upscaler,
                algorithm_copy,
                capture_target_clone2,
            );
            ui.set_capture_target(capture_target_clone);
            Box::new(ui)
        }),
    ).map_err(|e| e.to_string())
}