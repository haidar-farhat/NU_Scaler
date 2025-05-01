//! NuScaler Core Library - SOLID API scaffolding

use pyo3::prelude::*;
use pyo3::types::PyBytes;
use crate::upscale::Upscaler;
use crate::capture::realtime::RealTimeCapture;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use once_cell::sync::Lazy;

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
    pub fn upscale_batch<'py>(&mut self, py: Python<'py>, frames: &PyAny) -> PyResult<Vec<&'py PyBytes>> {
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
        self.inner.start(tgt).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))
    }
    pub fn stop(&mut self) {
        self.inner.stop();
    }
    pub fn get_frame<'py>(&mut self, py: Python<'py>) -> PyResult<Option<(PyObject, usize, usize)>> {
        match self.inner.get_frame() {
            Some((frame_data, width, height)) => {
                let py_bytes = PyBytes::new(py, &frame_data);
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
