# Nu Scaler Rust Implementation Plan

This document outlines the Rust-side implementation plan for the Nu_Scaler rework, focusing on high-performance, cross-platform, and Python-integrable architecture.

---

## 1. Project Structure

```
Nu_scale/
├── src/
│   ├── capture/         # Platform-specific and cross-platform screen capture
│   ├── gpu/             # GPU initialization and utilities (WGPU, Vulkan)
│   ├── upscale/         # Upscaling algorithms (FSR, DLSS, fallback, etc.)
│   ├── renderer/        # Rendering and overlay display
│   ├── ui/              # (Optional) Rust-native GUI for debugging
│   ├── python.rs        # PyO3 Python bindings (if FFI used)
│   ├── lib.rs           # Library entry point
│   └── main.rs          # CLI entry point
├── Cargo.toml
```

---

## 2. Core Modules & Responsibilities

### 2.1 Capture Module (`src/capture/`)
- **Goal:** Efficient, cross-platform screen/window/region capture.
- **Windows:** Use DXGI Desktop Duplication API and Windows.Graphics.Capture (via `windows` crate or `scap`)
- **Linux/X11:** Use XCB + XShm (or `scrap`/`scap`)
- **Linux/Wayland:** Use PipeWire (via `scap`)
- **API:**
  - `trait ScreenCapture { fn capture_frame(&mut self, target: &CaptureTarget) -> Result<RgbaImage>; ... }`
  - `fn create_capturer() -> Result<impl ScreenCapture>`
- **Best Practices:**
  - Use zero-copy or GPU-to-GPU paths where possible
  - Abstract platform differences behind a common trait

### 2.2 GPU Module (`src/gpu/`)
- **Goal:** Unified GPU initialization and resource management
- **API:**
  - `struct GpuContext { device: wgpu::Device, queue: wgpu::Queue, ... }`
  - Helper functions for buffer/texture creation, shader loading
- **Crates:** `wgpu`, `bytemuck`, `pollster`
- **Best Practices:**
  - Prefer WGPU for cross-platform abstraction
  - Use async device initialization

### 2.3 Upscale Module (`src/upscale/`)
- **Goal:** Provide multiple upscaling algorithms (FSR, DLSS, XeSS, fallback)
- **API:**
  - `trait Upscaler { fn upscale(&self, input: &RgbaImage) -> Result<RgbaImage>; ... }`
  - Factory: `fn create_upscaler(tech, quality, alg) -> Result<Box<dyn Upscaler>>`
- **Crates:** `image`, `wgpu`, vendor SDKs (optional)
- **Best Practices:**
  - Use compute shaders for GPU upscaling
  - Fallback to CPU for unsupported hardware

### 2.4 Renderer Module (`src/renderer/`)
- **Goal:** Display upscaled frames in a borderless overlay or window
- **API:**
  - `fn run_fullscreen_upscaler(...) -> Result<()>`
- **Crates:** `winit`, `wgpu`, `raw-window-handle`
- **Best Practices:**
  - Use double/triple buffering
  - Synchronize with VSync or target FPS

### 2.5 Python FFI/IPC Integration (`src/python.rs`)
- **Goal:** Expose Rust core to Python (via PyO3) or provide IPC hooks
- **API:**
  - PyO3: `#[pymodule]`, `#[pyclass]`, `#[pymethods]`
  - IPC: TCP/Unix socket server, JSON protocol
- **Crates:** `pyo3`, `maturin`, `serde`, `serde_json`, `interprocess`
- **Best Practices:**
  - Release GIL for long-running Rust tasks
  - Minimize data transfer across FFI/IPC boundary

---

## 3. Threading & Performance
- Use `std::thread` or `tokio` for capture, upscaling, and rendering threads
- Use `Arc<AtomicBool>` for stop signals
- Use `crossbeam-channel` or `tokio::sync` for frame queues
- Profile with `tracing` or `log` crates
- Implement double-buffering for frame handoff

---

## 4. Cross-Platform & Build
- Use `cfg(windows)`/`cfg(unix)` for platform-specific code
- Feature flags for optional tech (e.g., `fsr`, `dlss`, `python`)
- Use `maturin` for building Python wheels
- Use `cargo` for CLI and library builds

---

## 5. Example: WGPU Pipeline Skeleton

```rust
pub struct WgpuRenderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    // ...
}

impl WgpuRenderer {
    pub async fn new(window: &winit::window::Window) -> Result<Self> {
        // Initialize WGPU device, queue, surface, pipeline
        // ...
        Ok(Self { /* ... */ })
    }
    pub fn upscale_frame(&self, input: &wgpu::Texture, output: &wgpu::Texture) {
        // Dispatch compute shader
    }
}
```

---

## 6. Best Practices & Recommendations
- **Error Handling:** Use `anyhow` and `thiserror` for robust error reporting
- **Logging:** Use `log` and `env_logger` for diagnostics
- **Testing:** Unit test each module; integration test end-to-end pipeline
- **Documentation:** Document all public APIs and provide usage examples
- **CI:** Use GitHub Actions for cross-platform build/test

---

## 7. Next Steps
1. Refactor capture and GPU modules to use WGPU and unified traits
2. Implement or integrate upscaling compute shaders
3. Build FFI layer with PyO3 (or IPC hooks)
4. Provide CLI and Python test harnesses
5. Profile and optimize for latency and throughput

---

By following this plan, the Rust core of Nu_Scaler will be robust, high-performance, and ready for seamless integration with a Python GUI or automation layer. 