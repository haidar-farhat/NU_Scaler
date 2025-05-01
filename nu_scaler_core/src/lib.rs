//! NuScaler Core Library - SOLID API scaffolding

use pyo3::prelude::*;

pub mod capture;
pub mod gpu;
pub mod upscale;
pub mod renderer;

/// Public API for initializing the core library (placeholder)
pub fn initialize() {
    // TODO: Initialize logging, config, etc.
}

#[pyclass]
pub struct Engine;

#[pymethods]
impl Engine {
    #[new]
    pub fn new() -> Self {
        Engine
    }
    pub fn hello(&self) -> &'static str {
        "Hello from Rust Engine!"
    }
}

#[pymodule]
fn nu_scaler_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Engine>()?;
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
