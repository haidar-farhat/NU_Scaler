//! NuScaler Core Library - SOLID API scaffolding

use crate::benchmark::{py_benchmark_upscaler, py_run_comparison_benchmark, PyBenchmarkResult};
use crate::capture::realtime::RealTimeCapture;
use crate::gpu::detector::GpuDetector;
#[cfg(feature = "python")]
use crate::gpu::memory::PyVramStats;
use crate::gpu::memory::{/*VramStats,*/ AllocationStrategy, MemoryPressure};
use crate::gpu::GpuResources;
use crate::upscale::{Upscaler, UpscalerFactory, UpscalingQuality, UpscalingTechnology};
use anyhow::{anyhow, Result};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::sync::Arc;
use wgpu::BufferUsages;

pub mod benchmark;
pub mod capture;
pub mod dlss_manager;
pub mod gpu;
pub mod renderer;
pub mod upscale;

use capture::realtime::{CaptureTarget, ScreenCapture};
use gpu::detector::{GpuInfo, GpuVendor};
use upscale::{UpscaleAlgorithm, WgpuUpscaler};

// Import DlssUpscaler from the correct module
use crate::upscale::dlss::DlssUpscaler as InnerDlssUpscaler;

/// Public API for initializing the core library (placeholder)
pub fn initialize() {
    // TODO: Initialize logging, config, etc.
}

#[pyclass]
pub struct PyWgpuUpscaler {
    inner: WgpuUpscaler,
    upscale_scale: f32,
}

#[pymethods]
impl PyWgpuUpscaler {
    #[new]
    #[pyo3(signature = (quality = "quality", algorithm = "nearest"))]
    /// Create a new WgpuUpscaler. quality: "ultra"|"quality"|"balanced"|"performance". algorithm: "nearest"|"bilinear".
    pub fn new(quality: &str, algorithm: &str) -> PyResult<Self> {
        let q = match quality.to_lowercase().as_str() {
            "ultra" => UpscalingQuality::Ultra,
            "quality" => UpscalingQuality::Quality,
            "balanced" => UpscalingQuality::Balanced,
            "performance" => UpscalingQuality::Performance,
            _ => UpscalingQuality::Quality,
        };
        let alg = match algorithm.to_lowercase().as_str() {
            "nearest" => UpscaleAlgorithm::Nearest,
            "bilinear" => UpscaleAlgorithm::Bilinear,
            _ => UpscaleAlgorithm::Nearest,
        };
        Ok(Self {
            inner: WgpuUpscaler::new(q, alg),
            upscale_scale: 2.0,
        })
    }

    /// Initialize the upscaler with input/output dimensions
    pub fn initialize(
        &mut self,
        input_width: u32,
        input_height: u32,
        output_width: u32,
        output_height: u32,
    ) -> PyResult<()> {
        if input_width > 0 && input_height > 0 {
            let width_scale = output_width as f32 / input_width as f32;
            let height_scale = output_height as f32 / input_height as f32;
            self.upscale_scale = (width_scale + height_scale) / 2.0;
        }

        self.inner
            .initialize(input_width, input_height, output_width, output_height)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    #[getter]
    pub fn get_upscale_scale(&self) -> PyResult<f32> {
        Ok(self.upscale_scale)
    }

    #[setter]
    pub fn set_upscale_scale(&mut self, scale: f32) -> PyResult<()> {
        if scale < 1.0 || scale > 4.0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Scale factor must be between 1.0 and 4.0",
            ));
        }
        self.upscale_scale = scale;
        Ok(())
    }

    /// Upscale a frame (input: bytes, returns: bytes)
    pub fn upscale<'py>(&self, py: Python<'py>, input: &PyBytes) -> PyResult<&'py PyBytes> {
        let input_bytes = input.as_bytes();
        let out = self
            .inner
            .upscale(input_bytes)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyBytes::new(py, &out))
    }

    /// Reload the WGSL shader from a file path
    pub fn reload_shader(&mut self, path: &str) -> PyResult<()> {
        self.inner
            .reload_shader(path)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    /// Set the thread count for upscaling/concurrency
    pub fn set_thread_count(&mut self, n: u32) -> PyResult<()> {
        self.inner.set_thread_count(n);
        Ok(())
    }

    /// Set the buffer pool size
    pub fn set_buffer_pool_size(&mut self, n: u32) -> PyResult<()> {
        self.inner.set_buffer_pool_size(n);
        Ok(())
    }

    /// Set the GPU allocator preset
    pub fn set_gpu_allocator(&mut self, preset: &str) -> PyResult<()> {
        self.inner.set_gpu_allocator(preset);
        Ok(())
    }

    /// Batch upscale: takes a list of bytes objects, returns a list of bytes objects
    pub fn upscale_batch<'py>(
        &self,
        py: Python<'py>,
        frames: &PyAny,
    ) -> PyResult<Vec<&'py PyBytes>> {
        let frames_vec: Vec<&[u8]> = frames
            .iter()?
            .map(|item| item?.extract::<&PyBytes>().map(|b| b.as_bytes()))
            .collect::<Result<_, _>>()?;
        let outs = self
            .inner
            .upscale_batch(&frames_vec)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(outs.into_iter().map(|out| PyBytes::new(py, &out)).collect())
    }
}

impl Drop for PyWgpuUpscaler {
    fn drop(&mut self) {
        println!("[Rust] Dropping PyWgpuUpscaler at {:p}", self);
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyWindowByTitle {
    #[pyo3(get, set)]
    pub title: String,
}

#[pymethods]
impl PyWindowByTitle {
    #[new]
    pub fn new(title: String) -> Self {
        Self { title }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyRegion {
    #[pyo3(get, set)]
    pub x: i32,
    #[pyo3(get, set)]
    pub y: i32,
    #[pyo3(get, set)]
    pub width: u32,
    #[pyo3(get, set)]
    pub height: u32,
}

#[pymethods]
impl PyRegion {
    #[new]
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[pyclass(unsendable)]
pub struct PyScreenCapture {
    inner: ScreenCapture,
}

#[pymethods]
impl PyScreenCapture {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: ScreenCapture::new(),
        }
    }
    #[staticmethod]
    pub fn list_windows() -> Vec<String> {
        ScreenCapture::list_windows()
    }
    pub fn start(
        &mut self,
        target: PyCaptureTarget,
        window: Option<PyWindowByTitle>,
        region: Option<PyRegion>,
    ) -> PyResult<()> {
        let tgt = target.to_internal(window, region);
        self.inner
            .start(tgt)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }
    pub fn stop(&mut self) {
        self.inner.stop();
    }
    pub fn get_frame<'py>(
        &mut self,
        py: Python<'py>,
    ) -> PyResult<Option<(PyObject, usize, usize)>> {
        match self.inner.get_frame() {
            Some((frame_data, width, height)) => {
                // Check the target to see if conversion is needed
                let rgba_data = match self.inner.target {
                    // FullScreen capture (scrap) already returns RGBA (based on our conversion in realtime.rs)
                    Some(CaptureTarget::FullScreen) => frame_data,
                    // Window capture (GDI) returns BGRA, needs conversion
                    Some(CaptureTarget::WindowByTitle(_)) | Some(CaptureTarget::Region { .. }) => {
                        // Assume Region uses GDI too if implemented
                        if frame_data.len() == width * height * 4 {
                            let mut rgba = Vec::with_capacity(width * height * 4);
                            for chunk in frame_data.chunks_exact(4) {
                                // BGRA -> RGBA
                                rgba.push(chunk[2]); // R
                                rgba.push(chunk[1]); // G
                                rgba.push(chunk[0]); // B
                                rgba.push(chunk[3]); // A
                            }
                            rgba // Return the converted RGBA data
                        } else {
                            // Return original data if size is wrong (shouldn't happen often)
                            println!(
                                "[FFI] Warning: GDI frame size mismatch, skipping conversion."
                            );
                            frame_data
                        }
                    }
                    None => frame_data, // Should not happen if capture started
                };

                // Return the (potentially converted) RGBA data as PyBytes
                let py_bytes = PyBytes::new(py, &rgba_data);
                Ok(Some((py_bytes.into(), width, height)))
            }
            None => Ok(None),
        }
    }
}

impl Drop for PyScreenCapture {
    fn drop(&mut self) {
        println!("[Rust] Dropping PyScreenCapture at {:p}", self);
    }
}

#[pyclass]
#[derive(Clone)]
pub enum PyCaptureTarget {
    FullScreen,
    WindowByTitle,
    Region,
}

impl PyCaptureTarget {
    pub fn to_internal(
        &self,
        window: Option<PyWindowByTitle>,
        region: Option<PyRegion>,
    ) -> CaptureTarget {
        match self {
            PyCaptureTarget::FullScreen => CaptureTarget::FullScreen,
            PyCaptureTarget::WindowByTitle => {
                let title = window.map(|w| w.title).unwrap_or_default();
                CaptureTarget::WindowByTitle(title)
            }
            PyCaptureTarget::Region => {
                let r = region.unwrap_or(PyRegion {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                });
                CaptureTarget::Region {
                    x: r.x,
                    y: r.y,
                    width: r.width,
                    height: r.height,
                }
            }
        }
    }
}

/// WGPU Upscaler with added features
#[pyclass]
pub struct PyAdvancedWgpuUpscaler {
    inner: upscale::WgpuUpscaler,
    gpu_resources: Option<Arc<GpuResources>>,
    upscale_scale: f32,
}

#[pymethods]
impl PyAdvancedWgpuUpscaler {
    #[new]
    #[pyo3(signature = (quality = "quality", algorithm = "nearest", adaptive_quality = true))]
    /// Create a new advanced WgpuUpscaler with memory management features
    pub fn new(quality: &str, algorithm: &str, adaptive_quality: bool) -> PyResult<Self> {
        let q = match quality.to_lowercase().as_str() {
            "ultra" => upscale::UpscalingQuality::Ultra,
            "quality" => upscale::UpscalingQuality::Quality,
            "balanced" => upscale::UpscalingQuality::Balanced,
            "performance" => upscale::UpscalingQuality::Performance,
            _ => upscale::UpscalingQuality::Quality,
        };

        let alg = match algorithm.to_lowercase().as_str() {
            "bilinear" => upscale::UpscaleAlgorithm::Bilinear,
            _ => upscale::UpscaleAlgorithm::Nearest,
        };

        let mut upscaler = upscale::WgpuUpscaler::new(q, alg);
        upscaler.set_adaptive_quality(adaptive_quality);

        // Create a GPU detector and get resources
        let mut detector = GpuDetector::new();
        if let Err(e) = detector.detect_gpus() {
            println!("Warning: GPU detection failed: {}", e);
        }

        let primary_gpu = detector.get_primary_gpu().cloned();

        // Create GPU resources with memory management
        let gpu_resources = match pollster::block_on(detector.create_device_queue()) {
            Ok((device, queue)) => {
                let resources = Arc::new(GpuResources::new(device, queue, primary_gpu));

                // Force GPU activation on Windows to improve performance
                #[cfg(target_os = "windows")]
                if let Err(e) = resources.memory_pool.force_gpu_usage() {
                    println!("Warning: Failed to force GPU activation: {}", e);
                }

                upscaler.set_gpu_resources(resources.clone());
                Some(resources)
            }
            Err(e) => {
                println!("Warning: Failed to create GPU device and queue: {}", e);
                None
            }
        };

        Ok(Self {
            inner: upscaler,
            gpu_resources,
            upscale_scale: 2.0,
        })
    }

    /// Initialize the upscaler with the given dimensions
    pub fn initialize(
        &mut self,
        input_width: u32,
        input_height: u32,
        output_width: u32,
        output_height: u32,
    ) -> PyResult<()> {
        // Prime the GPU by pre-allocating a few buffers
        if let Some(resources) = &self.gpu_resources {
            // Force buffer allocation for these dimensions to ensure GPU is properly initialized
            let input_size = (input_width * input_height * 4) as usize;
            let output_size = (output_width * output_height * 4) as usize;

            // Use the memory pool to allocate buffers
            let _input_buffer = resources.memory_pool.get_buffer(
                input_size,
                BufferUsages::STORAGE | BufferUsages::COPY_DST,
                Some("Input Buffer Priming"),
            );

            let _output_buffer = resources.memory_pool.get_buffer(
                output_size,
                BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                Some("Output Buffer Priming"),
            );

            // Update VRAM usage stats
            let _ = resources.memory_pool.update_vram_usage();
        }

        if let Err(e) =
            self.inner
                .initialize(input_width, input_height, output_width, output_height)
        {
            Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to initialize upscaler: {}",
                e
            )))
        } else {
            self.upscale_scale = output_width as f32 / input_width as f32;

            // After initialization, update the memory strategy based on image size
            if let Some(resources) = &self.gpu_resources {
                let total_pixels = input_width as usize * input_height as usize;
                if total_pixels > 4 * 1920 * 1080 {
                    // For very large images, be more conservative
                    resources
                        .memory_pool
                        .set_allocation_strategy(AllocationStrategy::Conservative);
                } else if total_pixels > 1920 * 1080 {
                    // For medium images, be balanced
                    resources
                        .memory_pool
                        .set_allocation_strategy(AllocationStrategy::Balanced);
                } else {
                    // For small images, be aggressive
                    resources
                        .memory_pool
                        .set_allocation_strategy(AllocationStrategy::Aggressive);
                }

                // Update VRAM usage stats after initialization
                let _ = resources.memory_pool.update_vram_usage();
            }

            Ok(())
        }
    }

    /// Force GPU activation to maximize performance
    pub fn force_gpu_activation(&self) -> PyResult<()> {
        if let Some(resources) = &self.gpu_resources {
            #[cfg(target_os = "windows")]
            if let Err(e) = resources.memory_pool.force_gpu_usage() {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to force GPU activation: {}",
                    e
                )));
            }

            // Update VRAM usage stats
            if let Err(e) = resources.memory_pool.update_vram_usage() {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to update VRAM stats: {}",
                    e
                )));
            }

            Ok(())
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                "No GPU resources available",
            ))
        }
    }

    /// Upscale an image
    pub fn upscale(&self, input: &[u8]) -> PyResult<PyObject> {
        // Pre-upscale update if needed
        if let Some(resources) = &self.gpu_resources {
            // Check memory pressure and adjust if needed
            let memory_level = resources.memory_pool.get_current_memory_pressure();
            if memory_level == MemoryPressure::Critical || memory_level == MemoryPressure::High {
                // For high memory pressure, force cleanup
                resources.cleanup_memory();
            }
        }

        // Perform upscaling
        match self.inner.upscale(input) {
            Ok(output) => Python::with_gil(|py| {
                // Post-upscale update
                if let Some(resources) = &self.gpu_resources {
                    // Update stats occasionally
                    let _ = resources.memory_pool.update_vram_usage();
                }

                Ok(PyBytes::new(py, &output).into())
            }),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to upscale: {}",
                e
            ))),
        }
    }

    /// Get the upscale scale factor
    #[getter]
    pub fn get_upscale_scale(&self) -> PyResult<f32> {
        Ok(self.upscale_scale)
    }

    /// Set the upscale scale factor
    #[setter]
    pub fn set_upscale_scale(&mut self, scale: f32) -> PyResult<()> {
        if scale < 1.0 || scale > 4.0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Scale factor must be between 1.0 and 4.0",
            ));
        }
        self.upscale_scale = scale;
        Ok(())
    }

    /// Get VRAM stats (total, used, free)
    #[cfg(feature = "python")]
    pub fn get_vram_stats(&self) -> PyResult<PyVramStats> {
        match &self.gpu_resources {
            Some(res) => {
                let stats = res.get_vram_stats();
                Ok(PyVramStats::from(stats))
            }
            None => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "No GPU resources available",
            )),
        }
    }

    /// Set the memory allocation strategy
    pub fn set_memory_strategy(&self, strategy: &str) -> PyResult<()> {
        if let Some(resources) = &self.gpu_resources {
            let strategy = match strategy.to_lowercase().as_str() {
                "aggressive" => AllocationStrategy::Aggressive,
                "balanced" => AllocationStrategy::Balanced,
                "conservative" => AllocationStrategy::Conservative,
                "minimal" => AllocationStrategy::Minimal,
                _ => AllocationStrategy::Balanced,
            };
            resources.set_allocation_strategy(strategy);
            Ok(())
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                "GPU resources not initialized",
            ))
        }
    }

    /// Get current VRAM usage as a percentage
    pub fn get_vram_usage_percent(&self) -> PyResult<f32> {
        if let Some(resources) = &self.gpu_resources {
            let stats = resources.get_vram_stats();
            if stats.total_mb > 0.0 {
                Ok((stats.used_mb / stats.total_mb) * 100.0)
            } else {
                Ok(0.0)
            }
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                "GPU resources not initialized",
            ))
        }
    }

    /// Check if adaptive quality is enabled
    #[getter]
    pub fn get_adaptive_quality(&self) -> PyResult<bool> {
        // Use inner's public method to get the value
        Ok(self.inner.is_adaptive_quality_enabled())
    }

    /// Enable or disable adaptive quality
    #[setter]
    pub fn set_adaptive_quality(&mut self, enabled: bool) -> PyResult<()> {
        self.inner.set_adaptive_quality(enabled);
        Ok(())
    }

    /// Clean up GPU memory
    pub fn cleanup_memory(&self) -> PyResult<()> {
        if let Some(resources) = &self.gpu_resources {
            resources.cleanup_memory();
            Ok(())
        } else {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                "GPU resources not initialized",
            ))
        }
    }

    /// Get name of the upscaler
    #[getter]
    pub fn name(&self) -> PyResult<&'static str> {
        Ok(self.inner.name())
    }

    /// Get quality level
    #[getter]
    pub fn quality(&self) -> PyResult<String> {
        let quality = self.inner.quality();
        Ok(format!("{:?}", quality))
    }

    /// Set quality level
    #[setter]
    pub fn set_quality(&mut self, quality: &str) -> PyResult<()> {
        let q = match quality.to_lowercase().as_str() {
            "ultra" => upscale::UpscalingQuality::Ultra,
            "quality" => upscale::UpscalingQuality::Quality,
            "balanced" => upscale::UpscalingQuality::Balanced,
            "performance" => upscale::UpscalingQuality::Performance,
            _ => upscale::UpscalingQuality::Quality,
        };

        match self.inner.set_quality(q) {
            Ok(_) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to set quality: {}",
                e
            ))),
        }
    }

    /// Force update GPU memory usage
    pub fn update_gpu_stats(&self) -> PyResult<()> {
        match &self.gpu_resources {
            Some(res) => match res.memory_pool.update_vram_usage() {
                Ok(_) => Ok(()),
                Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to update GPU stats: {}",
                    e
                ))),
            },
            None => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "No GPU resources available",
            )),
        }
    }

    /// Get detailed GPU information
    pub fn get_gpu_info(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let info = pyo3::types::PyDict::new(py);

            if let Some(res) = &self.gpu_resources {
                if let Some(gpu_info) = &res.gpu_info {
                    info.set_item("name", gpu_info.name.clone())?;
                    info.set_item("vendor", format!("{:?}", gpu_info.vendor))?;
                    info.set_item("device_type", format!("{:?}", gpu_info.device_type))?;
                    info.set_item("backend", format!("{:?}", gpu_info.backend))?;
                    info.set_item("vendor_id", format!("0x{:X}", gpu_info.vendor_id))?;
                    info.set_item("device_id", format!("0x{:X}", gpu_info.device_id))?;
                    info.set_item("driver_info", gpu_info.driver_info.clone())?;
                    info.set_item("is_discrete", gpu_info.is_discrete)?;

                    // Add buffer allocation stats
                    info.set_item(
                        "allocated_buffers",
                        res.memory_pool.get_allocated_buffers_count(),
                    )?;
                    info.set_item("allocated_bytes", res.memory_pool.get_allocated_bytes())?;

                    // Get VRAM stats
                    let stats = res.get_vram_stats();
                    info.set_item("total_vram_mb", stats.total_mb)?;
                    info.set_item("used_vram_mb", stats.used_mb)?;
                    info.set_item("free_vram_mb", stats.free_mb)?;

                    return Ok(info.into());
                }
            }

            info.set_item("name", "No GPU detected")?;
            info.set_item("error", "GPU info not available")?;

            Ok(info.into())
        })
    }

    /// Force a manual cleanup of GPU resources
    pub fn force_cleanup(&self) -> PyResult<()> {
        match &self.gpu_resources {
            Some(res) => {
                res.cleanup_memory();

                // Force GPU activation after cleanup to ensure GPU stays active
                #[cfg(target_os = "windows")]
                let _ = res.memory_pool.force_gpu_usage();

                match res.memory_pool.update_vram_usage() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to update GPU stats after cleanup: {}",
                        e
                    ))),
                }
            }
            None => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "No GPU resources available",
            )),
        }
    }
}

impl Drop for PyAdvancedWgpuUpscaler {
    fn drop(&mut self) {
        println!("[Rust] Dropping PyAdvancedWgpuUpscaler at {:p}", self);
    }
}

/// Create an advanced GPU-managed upscaler with automatic memory management
#[pyfunction]
pub fn create_advanced_upscaler(quality: &str) -> PyResult<PyAdvancedWgpuUpscaler> {
    PyAdvancedWgpuUpscaler::new(quality, "bilinear", true)
}

#[pymodule]
fn nu_scaler_core(_py: Python, m: &PyModule) -> PyResult<()> {
    // Upscaling quality levels
    m.add("QUALITY_ULTRA", UpscalingQuality::Ultra)?;
    m.add("QUALITY_QUALITY", UpscalingQuality::Quality)?;
    m.add("QUALITY_BALANCED", UpscalingQuality::Balanced)?;
    m.add("QUALITY_PERFORMANCE", UpscalingQuality::Performance)?;

    // Upscaling technologies
    m.add("TECH_FSR", UpscalingTechnology::FSR)?;
    m.add("TECH_DLSS", UpscalingTechnology::DLSS)?;
    m.add("TECH_WGPU", UpscalingTechnology::Wgpu)?;
    m.add("TECH_FALLBACK", UpscalingTechnology::Fallback)?;

    // GPU vendors
    m.add("VENDOR_NVIDIA", GpuVendor::Nvidia)?;
    m.add("VENDOR_AMD", GpuVendor::Amd)?;
    m.add("VENDOR_INTEL", GpuVendor::Intel)?;
    m.add("VENDOR_OTHER", GpuVendor::Other)?;

    // Add Python wrapper classes
    m.add_class::<PyWgpuUpscaler>()?;
    m.add_class::<PyScreenCapture>()?;
    m.add_class::<PyCaptureTarget>()?;
    m.add_class::<PyWindowByTitle>()?;
    m.add_class::<PyRegion>()?;

    // Add benchmark classes and functions
    m.add_class::<PyBenchmarkResult>()?;
    m.add_function(wrap_pyfunction!(py_benchmark_upscaler, m)?)?;
    m.add_function(wrap_pyfunction!(py_run_comparison_benchmark, m)?)?;

    // Register PyVramStats with proper feature gate
    #[cfg(feature = "python")]
    m.add_class::<PyVramStats>()?;

    // Register advanced upscaler
    m.add_class::<PyAdvancedWgpuUpscaler>()?;

    // Add memory-managed upscaler factory function
    m.add_function(wrap_pyfunction!(create_advanced_upscaler, m)?)?;

    // Add factory functions for creating upscalers
    #[pyfn(m)]
    #[pyo3(name = "create_fsr_upscaler")]
    fn create_fsr_upscaler_pyfn(_quality: &str) -> PyResult<()> {
        #[cfg(feature = "fsr3")]
        {
            println!("[PyO3] Creating FSR-optimized upscaler (fsr3 feature enabled, but not implemented)");
            Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "PyFsrUpscaler not yet implemented.",
            ))
        }
        #[cfg(not(feature = "fsr3"))]
        {
            println!("[PyO3] Warning: create_fsr_upscaler called, but 'fsr3' feature is not enabled in nu_scaler_core.");
            Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "FSR3 support is not enabled in this build.",
            ))
        }
    }

    #[pyfn(m)]
    #[pyo3(name = "create_dlss_upscaler")]
    fn create_dlss_upscaler_pyfn(quality: &str) -> PyResult<PyDlssUpscaler> {
        PyDlssUpscaler::new(quality)
    }

    Ok(())
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_nuscaler() {
        let result = NuScaler::new();
        assert!(
            result.is_ok(),
            "Failed to create NuScaler: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_create_with_technology() {
        let result =
            NuScaler::with_technology(UpscalingTechnology::Wgpu, UpscalingQuality::Balanced);
        assert!(
            result.is_ok(),
            "Failed to create NuScaler with technology: {:?}",
            result.err()
        );
    }
}

/// A struct to hold all application state
pub struct NuScaler {
    capture: ScreenCapture,
    upscaler: Box<dyn Upscaler>,
    gpu_info: Option<GpuInfo>,
    _device: Option<Arc<wgpu::Device>>,
    _queue: Option<Arc<wgpu::Queue>>,
}

// Add WindowInfo struct
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub title: String,
}

impl NuScaler {
    /// Create a new NuScaler instance
    pub fn new() -> Result<Self> {
        // Initialize GPU detector
        let mut gpu_detector = GpuDetector::new();
        gpu_detector.detect_gpus()?;

        // Determine best upscaling technology based on GPU
        let upscaling_tech = gpu_detector.determine_best_upscaling_technology();
        let gpu_info = gpu_detector.get_primary_gpu().cloned();

        // Create appropriate upscaler based on detected GPU
        let mut upscaler =
            UpscalerFactory::create_upscaler(upscaling_tech, UpscalingQuality::Balanced);

        // Initialize shared GPU resources
        let (device, queue) = pollster::block_on(gpu_detector.create_device_queue())?;
        UpscalerFactory::set_shared_resources(&mut upscaler, device.clone(), queue.clone())?;

        let description = gpu_detector.get_gpu_description();
        println!("[NuScaler] Initialized with: {}", description);
        println!(
            "[NuScaler] Using upscaler: {} (Technology: {:?})",
            upscaler.name(),
            upscaling_tech
        );

        Ok(Self {
            capture: ScreenCapture::new(),
            upscaler,
            gpu_info,
            _device: Some(device),
            _queue: Some(queue),
        })
    }

    /// Create a new NuScaler instance with specific upscaling technology
    pub fn with_technology(
        technology: UpscalingTechnology,
        quality: UpscalingQuality,
    ) -> Result<Self> {
        let mut gpu_detector = GpuDetector::new();
        gpu_detector.detect_gpus()?;
        let gpu_info = gpu_detector.get_primary_gpu().cloned();

        // Create upscaler with requested technology
        let mut upscaler = UpscalerFactory::create_upscaler(technology, quality);

        // Initialize shared GPU resources
        let (device, queue) = pollster::block_on(gpu_detector.create_device_queue())?;
        UpscalerFactory::set_shared_resources(&mut upscaler, device.clone(), queue.clone())?;

        println!(
            "[NuScaler] Initialized with technology: {:?}, quality: {:?}",
            technology, quality
        );

        Ok(Self {
            capture: ScreenCapture::new(),
            upscaler,
            gpu_info,
            _device: Some(device),
            _queue: Some(queue),
        })
    }

    /// Get the list of available windows for capture
    pub fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        // Use the static method instead of instance method
        let window_titles = ScreenCapture::list_windows();
        Ok(window_titles
            .into_iter()
            .map(|title| WindowInfo { title })
            .collect())
    }

    /// Set the capture target
    pub fn set_capture_target(&mut self, target: CaptureTarget) -> Result<()> {
        // Use start instead of set_target
        self.capture.start(target).map_err(|e| anyhow!(e))
    }

    /// Set the upscaling quality
    pub fn set_quality(&mut self, quality: UpscalingQuality) -> Result<()> {
        self.upscaler.set_quality(quality)
    }

    /// Capture and upscale a single frame
    pub fn capture_and_upscale(
        &mut self,
        input_width: u32,
        input_height: u32,
        output_width: u32,
        output_height: u32,
    ) -> Result<Vec<u8>> {
        // Initialize upscaler with dimensions if needed
        self.upscaler
            .initialize(input_width, input_height, output_width, output_height)?;

        // Capture frame - use get_frame instead of capture_frame
        let frame_data = self
            .capture
            .get_frame()
            .ok_or_else(|| anyhow!("No frame captured"))?;

        // Upscale the frame
        let timer = std::time::Instant::now();
        let result = self.upscaler.upscale(&frame_data.0)?;
        let elapsed = timer.elapsed();

        println!(
            "[NuScaler] Upscaled {}x{} to {}x{} in {:.2}ms",
            input_width,
            input_height,
            output_width,
            output_height,
            elapsed.as_secs_f32() * 1000.0
        );

        Ok(result)
    }

    /// Get information about the detected GPU
    pub fn get_gpu_info(&self) -> Option<&GpuInfo> {
        self.gpu_info.as_ref()
    }

    /// Get the name of the active upscaler
    pub fn get_upscaler_name(&self) -> &'static str {
        self.upscaler.name()
    }
}

// Helper async function to initialize WGPU resources for standalone upscalers
// To avoid duplication, this could be refactored if NuScaler::initialize_wgpu is made more generic/static
async fn init_wgpu_for_standalone_upscaler(
) -> Result<(Arc<wgpu::Device>, Arc<wgpu::Queue>, Option<GpuInfo>)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY, // Consider making this configurable
        dx12_shader_compiler: wgpu::Dx12Compiler::default(), // Or Fxc, Dxc based on needs
        gles_minor_version: wgpu::Gles3MinorVersion::default(), // if GLES backend is used
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None, // No surface needed for compute-only upscalers
            force_fallback_adapter: false,
        })
        .await
        .ok_or_else(|| anyhow!("Failed to find a suitable GPU adapter for standalone upscaler."))?;

    // Get adapter info and convert to GpuInfo
    let adapter_info = adapter.get_info();
    let gpu_info = Some(GpuInfo::from(adapter_info));

    // Define features needed. For DLSS, raw resource handles are key.
    // wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES might be relevant for some advanced scenarios
    // but for just getting handles, no specific wgpu features beyond basic might be needed unless
    // the HAL interop itself implies them.
    let features = wgpu::Features::empty();
    // Check adapter.features() for what's supported. DLSS might need specific texture formats/usages.

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("standalone_upscaler_device"),
                required_features: features,
                required_limits: wgpu::Limits::default(),
            },
            None, // Trace path
        )
        .await
        .map_err(|e| anyhow!("Failed to request device for standalone upscaler: {}", e))?;

    Ok((Arc::new(device), Arc::new(queue), gpu_info))
}

#[pyclass(name = "DlssUpscaler", unsendable)]
pub struct PyDlssUpscaler {
    inner: InnerDlssUpscaler,
    // Keep Arcs to ensure device and queue live as long as this Python object
    _device: Arc<wgpu::Device>,
    _queue: Arc<wgpu::Queue>,
    _gpu_info: Option<GpuInfo>, // Store for potential future use
}

#[pymethods]
impl PyDlssUpscaler {
    #[new]
    pub fn new(quality_str: &str) -> PyResult<Self> {
        let quality = match quality_str.to_lowercase().as_str() {
            "ultra" => UpscalingQuality::Ultra,
            "quality" => UpscalingQuality::Quality,
            "balanced" => UpscalingQuality::Balanced,
            "performance" => UpscalingQuality::Performance,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid quality: {}",
                    quality_str
                )))
            }
        };

        // Initialize WGPU resources. This blocks.
        let (device_arc, queue_arc, gpu_info_opt) =
            pollster::block_on(init_wgpu_for_standalone_upscaler()).map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "WGPU initialization failed for DlssUpscaler: {}",
                    e
                ))
            })?;

        let mut upscaler = InnerDlssUpscaler::new(quality);

        // Construct GpuResources and set it on the inner upscaler
        // The DlssUpscaler::set_device_queue expects device and queue, then creates GpuResources internally.
        upscaler.set_device_queue(device_arc.clone(), queue_arc.clone());
        // If GpuInfo is critical for DlssUpscaler's GpuResources, ensure it's passed or handled.
        // Currently, DlssUpscaler->set_device_queue creates GpuResources with GpuInfo as None.

        Ok(Self {
            inner: upscaler,
            _device: device_arc,
            _queue: queue_arc,
            _gpu_info: gpu_info_opt,
        })
    }

    pub fn initialize(
        &mut self,
        input_width: u32,
        input_height: u32,
        output_width: u32,
        output_height: u32,
    ) -> PyResult<()> {
        self.inner
            .initialize(input_width, input_height, output_width, output_height)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    pub fn upscale<'py>(&self, py: Python<'py>, input: &'py PyBytes) -> PyResult<&'py PyBytes> {
        let input_bytes = input.as_bytes();
        let out_bytes = self
            .inner
            .upscale(input_bytes)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyBytes::new(py, &out_bytes))
    }

    #[getter]
    pub fn name(&self) -> PyResult<String> {
        Ok(self.inner.name().to_string())
    }

    #[getter]
    pub fn quality(&self) -> PyResult<String> {
        match self.inner.quality() {
            UpscalingQuality::Ultra => Ok("ultra".to_string()),
            UpscalingQuality::Quality => Ok("quality".to_string()),
            UpscalingQuality::Balanced => Ok("balanced".to_string()),
            UpscalingQuality::Performance => Ok("performance".to_string()),
        }
    }

    pub fn set_quality(&mut self, quality_str: &str) -> PyResult<()> {
        let quality = match quality_str.to_lowercase().as_str() {
            "ultra" => UpscalingQuality::Ultra,
            "quality" => UpscalingQuality::Quality,
            "balanced" => UpscalingQuality::Balanced,
            "performance" => UpscalingQuality::Performance,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid quality: {}",
                    quality_str
                )))
            }
        };
        self.inner
            .set_quality(quality)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }
}
