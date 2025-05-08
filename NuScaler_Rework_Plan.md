# Nu Scaler Rework Plan

## 1. Current Architecture Assessment

The current Nu_scale codebase is a Rust-based implementation with the following components:

- **Capture Module**: Platform-specific capture (Windows and Linux) using DXGI/WGC on Windows, X11/PipeWire on Linux
- **Upscale Module**: Multiple upscaling technologies (FSR, DLSS, XeSS, etc.) with fallback algorithms
- **Renderer Module**: Handles display and rendering of upscaled content
- **UI Module**: Multiple UI implementations (egui, Iced, GTK)
- **GPU Module**: GPU acceleration support

## 2. Rework Architecture

The rework will maintain a high-performance Rust core while adding Python integration for improved UI and rapid development capabilities, aligned with the architectural vision in the requirements.

### System Components:

| Component | Language | Responsibility |
|-----------|----------|----------------|
| Core Engine | Rust | Screen capture, GPU interaction, upscaling, rendering |
| User Interface | Python | Configuration UI, settings management, control flow |
| Platform Bridge | Rust/Python | Cross-language communication (PyO3 or IPC) |

## 3. Implementation Approach

### 3.1 GPU Acceleration

Leverage WGPU as the primary graphics API for improved cross-platform compatibility:

```rust
// Transition from multiple APIs to consolidated WGPU implementation
pub struct WgpuRenderer {
    surface: wgpu::Surface,
    device: wgpu::Device, 
    queue: wgpu::Queue,
    // Additional fields for configuration
}
```

WGPU provides:
- Cross-platform abstraction over Vulkan, Metal, DirectX, and OpenGL
- Pure Rust implementation with safe memory management
- Modern GPU access patterns for compute and graphics operations

### 3.2 Screen Capture Enhancement

Improve the existing capture module:
- Windows: Continue using DXGI Desktop Duplication and Windows Graphics Capture
- Linux: Enhance X11 capture and add robust PipeWire support for Wayland
- Performance optimizations: Zero-copy pathways where possible

Consider integration with the `scap` Rust crate to simplify cross-platform capturing.

### 3.3 Rust-Python Integration

Two integration options with a recommendation:

#### Option A: Python Extension (PyO3) - RECOMMENDED
- Use PyO3 to build a native Python module from the Rust core
- Create Python bindings for core functionality
- Package the extension as a Python wheel

```rust
#[pymodule]
fn nu_scaler(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyScreenCapture>()?;
    m.add_class::<PyUpscaler>()?;
    m.add_function(wrap_pyfunction!(start_upscaling, m)?)?;
    // Additional bindings
    Ok(())
}
```

#### Option B: IPC-based Communication
- Run Rust as a subprocess from Python
- Communicate via sockets, named pipes, or shared memory
- Define a simple JSON-based protocol for commands

### 3.4 Python GUI

Implement a modern Python GUI using one of:

- **DearPyGui**: GPU-accelerated, immediate mode GUI with excellent performance
- **Qt (PySide/PyQt)**: Rich widget toolkit with strong platform integration

The Python GUI will provide:
- Configuration interface for upscaling parameters
- Profile management
- Overlay controls
- Performance monitoring

## 4. Development Phases

### Phase 1: Core Engine Refactoring (2-3 weeks)
- Consolidate GPU backends to WGPU
- Improve capture pipeline performance
- Prepare Rust codebase for Python binding
- Enhance cross-platform compatibility

### Phase 2: Python Integration (1-2 weeks)
- Implement PyO3 bindings
- Create Python module structure
- Establish communication patterns
- Test performance across language boundary

### Phase 3: Python GUI Development (2-3 weeks)
- Implement configuration interface
- Build real-time controls and monitoring
- Add profile management
- Design overlay UI

### Phase 4: Performance Optimization (1-2 weeks)
- Profile and optimize critical paths
- Reduce latency in capture-to-display pipeline
- Implement multi-threading optimizations
- Add hardware-specific acceleration paths

### Phase 5: Packaging and Distribution (1 week)
- Create installer for Windows
- Build packages for Linux (AppImage/Flatpak)
- Documentation and user guides

## 5. Technical Implementation Details

### 5.1 Screen Capture

```rust
// Enhanced capture module with cross-platform support
pub trait EnhancedCapture: Send + Sync {
    fn capture_frame(&mut self) -> Result<Frame>;
    fn get_available_displays(&self) -> Vec<Display>;
    fn get_available_windows(&self) -> Vec<Window>;
    fn set_target(&mut self, target: CaptureTarget);
    fn get_settings(&self) -> CaptureSettings;
    fn set_settings(&mut self, settings: CaptureSettings);
}

// Platform-specific implementations
#[cfg(windows)]
pub struct WindowsCapture {
    dxgi_capturer: Option<DxgiCapturer>,
    wgc_capturer: Option<WgcCapturer>,
    // Additional fields
}

#[cfg(target_os = "linux")]
pub struct LinuxCapture {
    x11_capturer: Option<X11Capturer>,
    pipewire_capturer: Option<PipeWireCapture>,
    // Additional fields
}
```

### 5.2 Upscaling Pipeline

```rust
// Unified upscaling pipeline
pub struct UpscalePipeline {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    upscale_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    // Additional fields
}

impl UpscalePipeline {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Result<Self> {
        // Initialize upscale pipeline with compute shader
    }
    
    pub fn upscale(&self, input_texture: &wgpu::Texture, output_texture: &wgpu::Texture) -> Result<()> {
        // Execute upscaling operation
    }
}
```

### 5.3 Python Bindings

```python
# Example Python API
import nu_scaler

# Initialize the engine
engine = nu_scaler.Engine()

# Configure capture source
capture = nu_scaler.Capture()
capture.set_source("window", "Game Title")

# Configure upscaling
upscaler = nu_scaler.Upscaler()
upscaler.set_technology("FSR")
upscaler.set_quality("Quality")
upscaler.set_scale_factor(1.5)

# Start upscaling
engine.start(capture, upscaler)

# Access engine status
stats = engine.get_stats()
print(f"FPS: {stats.fps}, Frame time: {stats.frame_time}ms")
```

### 5.4 GUI Implementation (DearPyGui)

```python
import dearpygui.dearpygui as dpg
import nu_scaler

def start_upscaling(sender, app_data):
    # Get settings from UI
    source = dpg.get_value("source_combo")
    technology = dpg.get_value("tech_combo")
    quality = dpg.get_value("quality_combo")
    scale = dpg.get_value("scale_slider")
    
    # Configure and start upscaling
    engine.configure(source=source, technology=technology, 
                    quality=quality, scale_factor=scale)
    engine.start()

# Main window and UI elements
dpg.create_context()
dpg.create_viewport(title="Nu Scaler", width=800, height=600)

with dpg.window(label="Nu Scaler Control", width=800, height=600):
    # Source selection
    with dpg.group(horizontal=True):
        dpg.add_text("Capture Source:")
        dpg.add_combo(["Fullscreen", "Window", "Region"], 
                     default_value="Window", tag="source_combo")
    
    # Upscaling settings
    with dpg.collapsing_header(label="Upscaling Settings", default_open=True):
        dpg.add_combo(["FSR", "DLSS", "XeSS", "Fallback"], 
                     default_value="FSR", tag="tech_combo")
        dpg.add_combo(["Ultra", "Quality", "Balanced", "Performance"], 
                     default_value="Quality", tag="quality_combo")
        dpg.add_slider_float(label="Scale Factor", default_value=1.5, 
                           min_value=1.0, max_value=3.0, tag="scale_slider")
    
    # Control buttons
    with dpg.group(horizontal=True):
        dpg.add_button(label="Start", callback=start_upscaling)
        dpg.add_button(label="Stop", callback=stop_upscaling)

# Performance monitoring
with dpg.window(label="Performance", width=400, height=200):
    with dpg.plot(label="FPS", height=150, width=380):
        dpg.add_plot_axis(dpg.mvXAxis, label="time")
        dpg.add_plot_axis(dpg.mvYAxis, label="fps", tag="y_axis")
        dpg.add_line_series([], [], label="FPS", parent="y_axis", tag="fps_data")

dpg.setup_dearpygui()
dpg.show_viewport()
dpg.start_dearpygui()
```

## 6. Cross-Platform Considerations

### Windows:
- Utilize DXGI Desktop Duplication for most reliable capture
- Fallback to Windows Graphics Capture for applications with restrictions
- Support DirectX 11/12 where needed for compatibility

### Linux:
- X11: Use XShm + XCB for efficient capture
- Wayland: Implement PipeWire capture pipeline
- Support both desktop environments with minimal configuration

## 7. Performance Optimizations

- **Double-buffering**: Implement a producer-consumer pattern with multiple buffers
- **Zero-copy pathways**: Keep data on GPU where possible to avoid CPU-GPU transfers
- **Asynchronous processing**: Use separate threads for capture, processing, and rendering
- **Latency reduction**: Minimize frame-to-frame processing time
- **Memory management**: Reuse buffers and textures to avoid allocation overhead

## 8. Testing and Quality Assurance

- Implement automated testing for core components
- Performance benchmarking suite for various hardware configurations
- Cross-platform validation on different GPUs (NVIDIA, AMD, Intel)
- User acceptance testing with various games and applications

## 9. Packaging and Distribution

- **Windows**: Create MSI/EXE installer with bundled dependencies
- **Linux**: Build AppImage and/or Flatpak packages
- **Development**: Provide development packages for extension

## 10. Conclusion

This rework of Nu Scaler will maintain the high-performance aspects of the current Rust implementation while adding the flexibility and rapid development capabilities of Python. The hybrid approach leverages the strengths of both languages:

- **Rust**: Performance-critical capture, GPU interaction, and processing
- **Python**: User interface, configuration, and extensibility

By following this plan, Nu Scaler will become a more maintainable, extensible, and user-friendly application while maintaining its core performance advantages. 