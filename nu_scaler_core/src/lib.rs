//! NuScaler Core Library - SOLID API scaffolding

use pyo3::prelude::*;
use pyo3::types::PyBytes;

pub mod capture;
pub mod gpu;
pub mod upscale;
pub mod renderer;

use upscale::{WgpuUpscaler, UpscalingQuality, UpscaleAlgorithm};
use capture::realtime::{ScreenCapture, CaptureTarget};

/// Public API for initializing the core library (placeholder)
pub fn initialize() {
    // TODO: Initialize logging, config, etc.
}

#[pyclass]
pub struct PyWgpuUpscaler {
    inner: WgpuUpscaler,
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
        Ok(Self { inner: WgpuUpscaler::new(q, alg) })
    }

    /// Initialize the upscaler with input/output dimensions
    pub fn initialize(&mut self, input_width: u32, input_height: u32, output_width: u32, output_height: u32) -> PyResult<()> {
        self.inner.initialize(input_width, input_height, output_width, output_height)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    /// Upscale a frame (input: bytes, returns: bytes)
    pub fn upscale<'py>(&self, py: Python<'py>, input: &PyBytes) -> PyResult<&'py PyBytes> {
        let input_bytes = input.as_bytes();
        let out = self.inner.upscale(input_bytes)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyBytes::new(py, &out))
    }
}

#[pyclass]
pub struct PyWindowByTitle {
    #[pyo3(get, set)]
    pub title: String,
}

#[pyclass]
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

#[pyclass]
pub struct PyScreenCapture {
    inner: std::sync::Arc<std::sync::Mutex<ScreenCapture>>,
}

#[pymethods]
impl PyScreenCapture {
    #[new]
    pub fn new() -> Self {
        Self { inner: std::sync::Arc::new(std::sync::Mutex::new(ScreenCapture::new())) }
    }
    #[staticmethod]
    pub fn list_windows() -> Vec<String> {
        ScreenCapture::list_windows()
    }
    pub fn start(&self, target: PyCaptureTarget, window: Option<PyWindowByTitle>, region: Option<PyRegion>) -> PyResult<()> {
        let tgt = target.to_internal(window, region);
        self.inner.lock().unwrap().start(tgt).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))
    }
    pub fn stop(&self) {
        self.inner.lock().unwrap().stop();
    }
    pub fn get_frame<'py>(&self, py: Python<'py>) -> PyResult<Option<&'py PyBytes>> {
        match self.inner.lock().unwrap().get_frame() {
            Some(frame) => Ok(Some(PyBytes::new(py, &frame))),
            None => Ok(None),
        }
    }
}

#[pymodule]
fn nu_scaler_core(_py: Python, m: &PyModule) -> PyResult<()> {
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
