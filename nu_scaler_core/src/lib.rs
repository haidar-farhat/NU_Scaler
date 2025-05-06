//! NuScaler Core Library - SOLID API scaffolding

use pyo3::prelude::*;
use pyo3::types::PyBytes;
use crate::upscale::Upscaler;
use crate::capture::realtime::RealTimeCapture;
use anyhow::Result;
use std::sync::Arc;

pub mod capture;
pub mod gpu;
pub mod upscale;
pub mod renderer;
pub mod benchmark;

use upscale::{WgpuUpscaler, UpscalingQuality, UpscaleAlgorithm};
use capture::realtime::{ScreenCapture, CaptureTarget};
use gpu::detector::{GpuDetector, GpuInfo, GpuVendor};

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
    pub fn initialize(&mut self, input_width: u32, input_height: u32, output_width: u32, output_height: u32) -> PyResult<()> {
        if input_width > 0 && input_height > 0 {
            let width_scale = output_width as f32 / input_width as f32;
            let height_scale = output_height as f32 / input_height as f32;
            self.upscale_scale = (width_scale + height_scale) / 2.0;
        }
        
        self.inner.initialize(input_width, input_height, output_width, output_height)
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
                "Scale factor must be between 1.0 and 4.0"
            ));
        }
        self.upscale_scale = scale;
        Ok(())
    }

    /// Upscale a frame (input: bytes, returns: bytes)
    pub fn upscale<'py>(&self, py: Python<'py>, input: &PyBytes) -> PyResult<&'py PyBytes> {
        let input_bytes = input.as_bytes();
        let out = self.inner.upscale(input_bytes)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyBytes::new(py, &out))
    }

    /// Reload the WGSL shader from a file path
    pub fn reload_shader(&mut self, path: &str) -> PyResult<()> {
        self.inner.reload_shader(path)
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
    pub fn upscale_batch<'py>(&self, py: Python<'py>, frames: &PyAny) -> PyResult<Vec<&'py PyBytes>> {
        let frames_vec: Vec<&[u8]> = frames
            .iter()?
            .map(|item| item?.extract::<&PyBytes>().map(|b| b.as_bytes()))
            .collect::<Result<_, _>>()?;
        let outs = self.inner.upscale_batch(&frames_vec)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(outs.into_iter().map(|out| PyBytes::new(py, &out)).collect())
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
        Self { x, y, width, height }
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
        Self { inner: ScreenCapture::new() }
    }
    #[staticmethod]
    pub fn list_windows() -> Vec<String> {
        ScreenCapture::list_windows()
    }
    pub fn start(&mut self, target: PyCaptureTarget, window: Option<PyWindowByTitle>, region: Option<PyRegion>) -> PyResult<()> {
        let tgt = target.to_internal(window, region);
        self.inner.start(tgt).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }
    pub fn stop(&mut self) {
        self.inner.stop();
    }
    pub fn get_frame<'py>(&mut self, py: Python<'py>) -> PyResult<Option<(PyObject, usize, usize)>> {
        match self.inner.get_frame() {
            Some((frame_data, width, height)) => {
                // Check the target to see if conversion is needed
                let rgba_data = match self.inner.target {
                    // FullScreen capture (scrap) already returns RGBA (based on our conversion in realtime.rs)
                    Some(CaptureTarget::FullScreen) => frame_data,
                    // Window capture (GDI) returns BGRA, needs conversion
                    Some(CaptureTarget::WindowByTitle(_)) |
                    Some(CaptureTarget::Region { .. }) => { // Assume Region uses GDI too if implemented
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
                             println!("[FFI] Warning: GDI frame size mismatch, skipping conversion.");
                             frame_data
                         }
                    }
                    None => frame_data, // Should not happen if capture started
                };

                // Return the (potentially converted) RGBA data as PyBytes
                let py_bytes = PyBytes::new(py, &rgba_data);
                Ok(Some((py_bytes.into(), width, height)))
            },
            None => Ok(None),
        }
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
    pub fn to_internal(&self, window: Option<PyWindowByTitle>, region: Option<PyRegion>) -> CaptureTarget {
        match self {
            PyCaptureTarget::FullScreen => CaptureTarget::FullScreen,
            PyCaptureTarget::WindowByTitle => {
                let title = window.map(|w| w.title).unwrap_or_default();
                CaptureTarget::WindowByTitle(title)
            },
            PyCaptureTarget::Region => {
                let r = region.unwrap_or(PyRegion { x: 0, y: 0, width: 0, height: 0 });
                CaptureTarget::Region { x: r.x, y: r.y, width: r.width, height: r.height }
            },
        }
    }
}

#[pymodule]
fn nu_scaler(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyWgpuUpscaler>()?;
    m.add_class::<PyScreenCapture>()?;
    m.add_class::<PyCaptureTarget>()?;
    m.add_class::<PyWindowByTitle>()?;
    m.add_class::<PyRegion>()?;
    Ok(())
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

/// A struct to hold all application state
pub struct NuScaler {
    capture: Box<dyn ScreenCapture>,
    upscaler: Box<dyn Upscaler>,
    gpu_info: Option<GpuInfo>,
    device: Option<Arc<wgpu::Device>>,
    queue: Option<Arc<wgpu::Queue>>,
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
        let mut upscaler = UpscalerFactory::create_upscaler(
            upscaling_tech, 
            UpscalingQuality::Balanced
        );
        
        // Initialize shared GPU resources
        let (device, queue) = pollster::block_on(gpu_detector.create_device_queue())?;
        UpscalerFactory::set_shared_resources(&mut upscaler, device.clone(), queue.clone())?;
        
        let description = gpu_detector.get_gpu_description();
        println!("[NuScaler] Initialized with: {}", description);
        println!("[NuScaler] Using upscaler: {} (Technology: {:?})", upscaler.name(), upscaling_tech);
        
        Ok(Self {
            capture: crate::capture::create_capturer()?,
            upscaler,
            gpu_info,
            device: Some(device),
            queue: Some(queue),
        })
    }
    
    /// Create a new NuScaler instance with specific upscaling technology
    pub fn with_technology(technology: UpscalingTechnology, quality: UpscalingQuality) -> Result<Self> {
        let mut gpu_detector = GpuDetector::new();
        gpu_detector.detect_gpus()?;
        let gpu_info = gpu_detector.get_primary_gpu().cloned();
        
        // Create upscaler with requested technology
        let mut upscaler = UpscalerFactory::create_upscaler(technology, quality);
        
        // Initialize shared GPU resources
        let (device, queue) = pollster::block_on(gpu_detector.create_device_queue())?;
        UpscalerFactory::set_shared_resources(&mut upscaler, device.clone(), queue.clone())?;
        
        println!("[NuScaler] Initialized with technology: {:?}, quality: {:?}", technology, quality);
        
        Ok(Self {
            capture: crate::capture::create_capturer()?,
            upscaler,
            gpu_info,
            device: Some(device),
            queue: Some(queue),
        })
    }
    
    /// Get the list of available windows for capture
    pub fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        self.capture.list_windows()
    }
    
    /// Set the capture target
    pub fn set_capture_target(&mut self, target: CaptureTarget) -> Result<()> {
        self.capture.set_target(target)
    }
    
    /// Set the upscaling quality
    pub fn set_quality(&mut self, quality: UpscalingQuality) -> Result<()> {
        self.upscaler.set_quality(quality)
    }
    
    /// Capture and upscale a single frame
    pub fn capture_and_upscale(&mut self, input_width: u32, input_height: u32, output_width: u32, output_height: u32) -> Result<Vec<u8>> {
        // Initialize upscaler with dimensions if needed
        self.upscaler.initialize(input_width, input_height, output_width, output_height)?;
        
        // Capture frame
        let frame = self.capture.capture_frame()?;
        
        // Upscale the frame
        let timer = std::time::Instant::now();
        let result = self.upscaler.upscale(&frame.data)?;
        let elapsed = timer.elapsed();
        
        println!("[NuScaler] Upscaled {}x{} to {}x{} in {:.2}ms", 
            input_width, input_height, output_width, output_height, elapsed.as_secs_f32() * 1000.0);
        
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

// Python module for exported functions
#[pymodule]
fn nu_scaler_core(py: Python, m: &PyModule) -> PyResult<()> {
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
    
    // Add factory functions for creating upscalers
    #[pyfn(m)]
    fn create_best_upscaler(quality: &str) -> PyResult<PyWgpuUpscaler> {
        // Initialize GPU detector
        let mut gpu_detector = GpuDetector::new();
        match gpu_detector.detect_gpus() {
            Ok(_) => {},
            Err(e) => return Err(pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to detect GPUs: {}", e))),
        }
        
        // Determine best upscaling technology
        let tech = gpu_detector.determine_best_upscaling_technology();
        
        // Convert quality string to enum
        let q = match quality.to_lowercase().as_str() {
            "ultra" => UpscalingQuality::Ultra,
            "quality" => UpscalingQuality::Quality,
            "balanced" => UpscalingQuality::Balanced,
            "performance" => UpscalingQuality::Performance,
            _ => UpscalingQuality::Quality,
        };
        
        // For now, we can only create WgpuUpscaler directly from Python
        // So we determine the algorithm based on the best tech
        let algorithm = match tech {
            UpscalingTechnology::FSR => "bilinear",   // FSR works best with bilinear base
            UpscalingTechnology::DLSS => "bilinear",  // DLSS works best with bilinear base
            _ => "nearest",                           // Default to nearest for other tech
        };
        
        // Log the detected GPU and selected technology
        let gpu_info = gpu_detector.get_primary_gpu().cloned();
        if let Some(gpu) = gpu_info {
            println!("[PyO3] Detected GPU: {} (Vendor: {:?})", gpu.name, gpu.vendor);
        }
        println!("[PyO3] Selected upscaling technology: {:?}", tech);
        
        // Create the upscaler
        PyWgpuUpscaler::new(quality, algorithm)
    }
    
    #[pyfn(m)]
    fn create_fsr_upscaler(quality: &str) -> PyResult<PyWgpuUpscaler> {
        // For now, we create a WgpuUpscaler configured for FSR-like operation
        // In a real implementation, this would create an actual FSR upscaler
        println!("[PyO3] Creating FSR-optimized upscaler");
        PyWgpuUpscaler::new(quality, "bilinear")
    }
    
    #[pyfn(m)]
    fn create_dlss_upscaler(quality: &str) -> PyResult<PyWgpuUpscaler> {
        // For now, we create a WgpuUpscaler configured for DLSS-like operation
        // In a real implementation, this would create an actual DLSS upscaler
        println!("[PyO3] Creating DLSS-optimized upscaler");
        PyWgpuUpscaler::new(quality, "bilinear")
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
        assert!(result.is_ok(), "Failed to create NuScaler: {:?}", result.err());
    }
    
    #[test]
    fn test_create_with_technology() {
        let result = NuScaler::with_technology(UpscalingTechnology::Wgpu, UpscalingQuality::Balanced);
        assert!(result.is_ok(), "Failed to create NuScaler with technology: {:?}", result.err());
    }
}
