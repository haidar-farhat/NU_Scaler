# Nu Scale Python Integration Guide

This guide details how to implement the Python integration with the existing Rust Nu_scale codebase, focusing on creating a powerful hybrid application that leverages both languages' strengths.

## 1. Setting Up PyO3 for Rust-Python Binding

### 1.1 Update Cargo.toml

First, add PyO3 dependencies to the Cargo.toml file:

```toml
[dependencies]
# Existing dependencies...

# Python integration
pyo3 = { version = "0.18", features = ["extension-module"] }
maturin = "1.1"

[lib]
name = "nu_scaler"
# This is required for Python to find the library
crate-type = ["cdylib", "rlib"]
```

### 1.2 Create Python Package Structure

```
nu_scaler/
├── Cargo.toml
├── pyproject.toml
├── setup.py
├── src/
│   └── ... (Rust source files)
└── python/
    ├── nu_scaler/
    │   ├── __init__.py
    │   ├── gui/
    │   │   ├── __init__.py
    │   │   ├── main_window.py
    │   │   └── settings_panel.py
    │   └── utils/
    │       ├── __init__.py
    │       └── config.py
    ├── README.md
    └── examples/
        └── simple_upscaler.py
```

### 1.3 Create pyproject.toml

```toml
[build-system]
requires = ["maturin>=1.1,<2.0"]
build-backend = "maturin"

[project]
name = "nu_scaler"
version = "0.1.0"
description = "Real-time upscaling application for gaming and video"
readme = "python/README.md"
requires-python = ">=3.8"
classifiers = [
    "Programming Language :: Python :: 3",
    "Programming Language :: Rust",
    "License :: OSI Approved :: MIT License",
    "Operating System :: Microsoft :: Windows",
    "Operating System :: POSIX :: Linux",
]

[project.dependencies]
dearpygui = "^1.9.0"
numpy = "^1.24.0"
pillow = "^10.0.0"

[tool.maturin]
python-source = "python"
module-name = "nu_scaler._rust"
```

## 2. Exposing Rust Functions to Python

### 2.1 Create PyO3 Module

Create a new file `src/python.rs` to define Python bindings:

```rust
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use anyhow::Result;

// Import necessary modules
use crate::capture::{CaptureTarget, ScreenCapture};
use crate::upscale::{UpscalingQuality, UpscalingTechnology, Upscaler};
use crate::renderer;

/// Python module for Nu Scaler
#[pymodule]
fn _rust(_py: Python, m: &PyModule) -> PyResult<()> {
    // Add classes
    m.add_class::<PyEngine>()?;
    m.add_class::<PyCapture>()?;
    m.add_class::<PyUpscaler>()?;

    // Add enums
    m.add("QUALITY_ULTRA", UpscalingQuality::Ultra)?;
    m.add("QUALITY_QUALITY", UpscalingQuality::Quality)?;
    m.add("QUALITY_BALANCED", UpscalingQuality::Balanced)?;
    m.add("QUALITY_PERFORMANCE", UpscalingQuality::Performance)?;

    m.add("TECH_FSR", UpscalingTechnology::FSR)?;
    m.add("TECH_DLSS", UpscalingTechnology::DLSS)?;
    m.add("TECH_FSR3", UpscalingTechnology::FSR3)?;
    m.add("TECH_FALLBACK", UpscalingTechnology::Fallback)?;

    // Add functions
    m.add_function(wrap_pyfunction!(upscale_image, m)?)?;
    m.add_function(wrap_pyfunction!(get_version, m)?)?;

    Ok(())
}

/// Python wrapper for the Engine
#[pyclass(name = "Engine")]
struct PyEngine {
    capture_thread: Option<std::thread::JoinHandle<()>>,
    stop_signal: std::sync::Arc<std::sync::atomic::AtomicBool>,
    frame_buffer: std::sync::Arc<crate::capture::common::FrameBuffer>,
    engine_running: bool,
}

#[pymethods]
impl PyEngine {
    #[new]
    fn new() -> Self {
        let stop_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let frame_buffer = std::sync::Arc::new(crate::capture::common::FrameBuffer::new(5));
        
        PyEngine {
            capture_thread: None,
            stop_signal,
            frame_buffer,
            engine_running: false,
        }
    }

    /// Start the upscaling engine
    fn start(&mut self, py_capture: &PyCapture, py_upscaler: &PyUpscaler) -> PyResult<()> {
        // Implementation details here
        Python::allow_threads(py.gil(), || {
            // Start the engine in a background thread
            self.engine_running = true;
            // Actual implementation would initialize the capture and upscaling pipeline
        });
        Ok(())
    }

    /// Stop the upscaling engine
    fn stop(&mut self) -> PyResult<()> {
        // Implementation details here
        self.stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
        if let Some(thread) = self.capture_thread.take() {
            thread.join().map_err(|_| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to join capture thread"))?;
        }
        self.engine_running = false;
        Ok(())
    }

    /// Get engine statistics
    fn get_stats(&self) -> PyResult<EngineStats> {
        // Implementation details here
        Ok(EngineStats {
            fps: 60.0,
            frame_time: 16.67,
            // Other stats
        })
    }
}

/// Python wrapper for Capture
#[pyclass(name = "Capture")]
struct PyCapture {
    target: CaptureTarget,
    // Additional fields
}

#[pymethods]
impl PyCapture {
    #[new]
    fn new() -> Self {
        PyCapture {
            target: CaptureTarget::FullScreen,
            // Initialize other fields
        }
    }

    /// Set the capture source
    fn set_source(&mut self, source_type: &str, source_id: Option<&str>) -> PyResult<()> {
        self.target = match source_type {
            "fullscreen" => CaptureTarget::FullScreen,
            "window" => {
                if let Some(title) = source_id {
                    CaptureTarget::WindowByTitle(title.to_string())
                } else {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Window title is required for window capture"));
                }
            },
            "region" => {
                // Parse region coordinates from source_id
                // For simplicity, using default values here
                CaptureTarget::Region { x: 0, y: 0, width: 1920, height: 1080 }
            },
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Unknown source type: {}", source_type)
                ));
            }
        };
        Ok(())
    }

    /// List available windows for capture
    fn list_windows(&self) -> PyResult<Vec<(String, String)>> {
        // Implementation details here
        Python::allow_threads(py.gil(), || {
            let capture = crate::capture::create_capturer()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create capturer: {}", e)))?;
            
            let windows = capture.list_windows()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to list windows: {}", e)))?;
            
            Ok(windows.into_iter().map(|w| (w.id.to_string(), w.title)).collect())
        })
    }
}

/// Python wrapper for Upscaler
#[pyclass(name = "Upscaler")]
struct PyUpscaler {
    technology: UpscalingTechnology,
    quality: UpscalingQuality,
    scale_factor: f32,
    algorithm: Option<crate::UpscalingAlgorithm>,
}

#[pymethods]
impl PyUpscaler {
    #[new]
    fn new() -> Self {
        PyUpscaler {
            technology: UpscalingTechnology::Fallback,
            quality: UpscalingQuality::Quality,
            scale_factor: 1.5,
            algorithm: None,
        }
    }

    /// Set the upscaling technology
    fn set_technology(&mut self, tech: &str) -> PyResult<()> {
        self.technology = match tech.to_lowercase().as_str() {
            "fsr" => UpscalingTechnology::FSR,
            "dlss" => UpscalingTechnology::DLSS,
            "fsr3" => UpscalingTechnology::FSR3,
            "xess" => UpscalingTechnology::XeSS,
            "fallback" => UpscalingTechnology::Fallback,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Unknown technology: {}", tech)
                ));
            }
        };
        Ok(())
    }

    /// Set the upscaling quality
    fn set_quality(&mut self, quality: &str) -> PyResult<()> {
        self.quality = match quality.to_lowercase().as_str() {
            "ultra" => UpscalingQuality::Ultra,
            "quality" => UpscalingQuality::Quality,
            "balanced" => UpscalingQuality::Balanced,
            "performance" => UpscalingQuality::Performance,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Unknown quality: {}", quality)
                ));
            }
        };
        Ok(())
    }

    /// Set the scale factor
    fn set_scale_factor(&mut self, scale: f32) -> PyResult<()> {
        if scale < 1.0 || scale > 4.0 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Scale factor must be between 1.0 and 4.0"
            ));
        }
        self.scale_factor = scale;
        Ok(())
    }
}

/// Engine statistics
#[pyclass(name = "EngineStats")]
struct EngineStats {
    #[pyo3(get)]
    fps: f32,
    #[pyo3(get)]
    frame_time: f32,
    // Additional stats
}

/// Upscale a single image file
#[pyfunction]
fn upscale_image(
    input_path: &str,
    output_path: &str,
    technology: &str,
    quality: &str,
    scale_factor: f32,
) -> PyResult<()> {
    // Convert string parameters to enums
    let tech = match technology.to_lowercase().as_str() {
        "fsr" => UpscalingTechnology::FSR,
        "dlss" => UpscalingTechnology::DLSS,
        "fsr3" => UpscalingTechnology::FSR3,
        "fallback" => UpscalingTechnology::Fallback,
        _ => UpscalingTechnology::Fallback,
    };
    
    let qual = match quality.to_lowercase().as_str() {
        "ultra" => UpscalingQuality::Ultra,
        "quality" => UpscalingQuality::Quality,
        "balanced" => UpscalingQuality::Balanced,
        "performance" => UpscalingQuality::Performance,
        _ => UpscalingQuality::Quality,
    };
    
    // Call the Rust function
    crate::upscale_image(input_path, output_path, tech, qual, scale_factor)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to upscale image: {}", e)))
}

/// Get the application version
#[pyfunction]
fn get_version() -> &'static str {
    crate::VERSION
}
```

### 2.2 Update lib.rs

Add the Python module to the `lib.rs` file:

```rust
// Existing imports and modules...

// Python integration
#[cfg(feature = "python")]
pub mod python;
```

## 3. Creating a Python Package

### 3.1 Define Python Module Structure

In `python/nu_scaler/__init__.py`:

```python
"""
Nu Scaler - Real-time upscaling application for gaming and video
"""

# Import the Rust implementation
from ._rust import Engine, Capture, Upscaler
from ._rust import upscale_image, get_version
from ._rust import (
    QUALITY_ULTRA, QUALITY_QUALITY, QUALITY_BALANCED, QUALITY_PERFORMANCE,
    TECH_FSR, TECH_DLSS, TECH_FSR3, TECH_FALLBACK
)

# Version information
__version__ = get_version()

# Import Python modules
from .gui import MainWindow, SettingsPanel

def create_app():
    """Create and return the main application instance"""
    return MainWindow()
```

### 3.2 Create Python GUI with DearPyGui

In `python/nu_scaler/gui/main_window.py`:

```python
import dearpygui.dearpygui as dpg
from .._rust import Engine, Capture, Upscaler
import threading
import time
import numpy as np

class MainWindow:
    def __init__(self):
        self.engine = Engine()
        self.capture = Capture()
        self.upscaler = Upscaler()
        
        self.engine_thread = None
        self.monitor_thread = None
        self.running = False
        
        self.fps_history = []
        self.time_history = []
        
        # Initialize DearPyGui
        dpg.create_context()
        dpg.create_viewport(title="Nu Scaler", width=800, height=600)
        dpg.setup_dearpygui()
        
        self._setup_ui()
    
    def _setup_ui(self):
        # Main window
        with dpg.window(label="Nu Scaler Control", width=800, height=600, tag="main_window"):
            # Source selection
            with dpg.collapsing_header(label="Capture Settings", default_open=True):
                with dpg.group(horizontal=True):
                    dpg.add_text("Capture Source:")
                    dpg.add_combo(
                        ["Fullscreen", "Window", "Region"], 
                        default_value="Fullscreen", 
                        callback=self._on_source_change,
                        tag="source_combo"
                    )
                
                # Window selection (hidden by default)
                with dpg.group(tag="window_group", show=False):
                    dpg.add_button(label="Refresh Window List", callback=self._refresh_windows)
                    dpg.add_combo(
                        [], 
                        label="Window",
                        tag="window_combo",
                        width=300
                    )
                
                # Region selection (hidden by default)
                with dpg.group(tag="region_group", show=False):
                    with dpg.group(horizontal=True):
                        dpg.add_input_int(label="X", default_value=0, tag="region_x")
                        dpg.add_input_int(label="Y", default_value=0, tag="region_y")
                    with dpg.group(horizontal=True):
                        dpg.add_input_int(label="Width", default_value=1920, tag="region_width")
                        dpg.add_input_int(label="Height", default_value=1080, tag="region_height")
            
            # Upscaling settings
            with dpg.collapsing_header(label="Upscaling Settings", default_open=True):
                dpg.add_combo(
                    ["FSR", "DLSS", "FSR3", "Fallback"],
                    label="Technology",
                    default_value="FSR",
                    callback=self._on_tech_change,
                    tag="tech_combo"
                )
                
                dpg.add_combo(
                    ["Ultra", "Quality", "Balanced", "Performance"],
                    label="Quality",
                    default_value="Quality",
                    tag="quality_combo"
                )
                
                dpg.add_slider_float(
                    label="Scale Factor",
                    default_value=1.5,
                    min_value=1.0,
                    max_value=3.0,
                    tag="scale_slider"
                )
            
            # Control buttons
            with dpg.group(horizontal=True):
                dpg.add_button(label="Start", callback=self._start_upscaling, tag="start_button")
                dpg.add_button(label="Stop", callback=self._stop_upscaling, tag="stop_button", enabled=False)
        
        # Performance window
        with dpg.window(label="Performance", width=400, height=200, pos=(50, 400), tag="perf_window"):
            with dpg.plot(label="FPS", height=150, width=380):
                dpg.add_plot_axis(dpg.mvXAxis, label="time", tag="x_axis")
                dpg.add_plot_axis(dpg.mvYAxis, label="fps", tag="y_axis")
                dpg.add_line_series([], [], label="FPS", parent="y_axis", tag="fps_data")
    
    def _on_source_change(self, sender, app_data):
        # Show/hide appropriate UI elements based on selected source
        if app_data == "Window":
            dpg.configure_item("window_group", show=True)
            dpg.configure_item("region_group", show=False)
            self._refresh_windows()
        elif app_data == "Region":
            dpg.configure_item("window_group", show=False)
            dpg.configure_item("region_group", show=True)
        else:
            dpg.configure_item("window_group", show=False)
            dpg.configure_item("region_group", show=False)
    
    def _on_tech_change(self, sender, app_data):
        # TODO: Modify UI based on selected technology
        # For example, disable quality settings for technologies that don't support them
        pass
    
    def _refresh_windows(self):
        # Get list of windows from the capture module
        try:
            windows = self.capture.list_windows()
            window_titles = [title for _, title in windows]
            dpg.configure_item("window_combo", items=window_titles)
        except Exception as e:
            print(f"Error listing windows: {e}")
    
    def _start_upscaling(self):
        # Get settings from UI
        source_type = dpg.get_value("source_combo").lower()
        technology = dpg.get_value("tech_combo")
        quality = dpg.get_value("quality_combo")
        scale_factor = dpg.get_value("scale_slider")
        
        # Configure capture source
        try:
            if source_type == "window":
                window_title = dpg.get_value("window_combo")
                self.capture.set_source("window", window_title)
            elif source_type == "region":
                x = dpg.get_value("region_x")
                y = dpg.get_value("region_y")
                width = dpg.get_value("region_width")
                height = dpg.get_value("region_height")
                region_str = f"{x},{y},{width},{height}"
                self.capture.set_source("region", region_str)
            else:
                self.capture.set_source("fullscreen", None)
            
            # Configure upscaler
            self.upscaler.set_technology(technology)
            self.upscaler.set_quality(quality)
            self.upscaler.set_scale_factor(scale_factor)
            
            # Start the engine
            self.engine.start(self.capture, self.upscaler)
            
            # Update UI
            dpg.configure_item("start_button", enabled=False)
            dpg.configure_item("stop_button", enabled=True)
            
            # Start monitoring thread
            self.running = True
            self.fps_history = []
            self.time_history = []
            self.monitor_thread = threading.Thread(target=self._monitor_performance)
            self.monitor_thread.daemon = True
            self.monitor_thread.start()
            
        except Exception as e:
            print(f"Error starting upscaling: {e}")
    
    def _stop_upscaling(self):
        # Stop the engine
        try:
            self.engine.stop()
            self.running = False
            
            # Update UI
            dpg.configure_item("start_button", enabled=True)
            dpg.configure_item("stop_button", enabled=False)
            
            # Wait for monitoring thread to finish
            if self.monitor_thread:
                self.monitor_thread.join(timeout=1.0)
                self.monitor_thread = None
        except Exception as e:
            print(f"Error stopping upscaling: {e}")
    
    def _monitor_performance(self):
        # Monitor performance in a separate thread
        start_time = time.time()
        while self.running:
            try:
                stats = self.engine.get_stats()
                
                # Update FPS history
                current_time = time.time() - start_time
                self.fps_history.append(stats.fps)
                self.time_history.append(current_time)
                
                # Keep history limited to 100 points
                if len(self.fps_history) > 100:
                    self.fps_history = self.fps_history[-100:]
                    self.time_history = self.time_history[-100:]
                
                # Update plot
                dpg.set_value("fps_data", [self.time_history, self.fps_history])
                
                # Sleep a bit
                time.sleep(0.1)
            except Exception as e:
                print(f"Error in performance monitor: {e}")
                break
    
    def run(self):
        dpg.show_viewport()
        dpg.start_dearpygui()
        dpg.destroy_context()
```

## 4. Building and Packaging

### 4.1 Development Build

For development, use Maturin to build the package:

```bash
# Install maturin
pip install maturin

# Build and install in development mode
maturin develop
```

### 4.2 Production Build

For production builds, create Python wheels:

```bash
# Build the wheel
maturin build --release

# The wheel will be in target/wheels/
pip install target/wheels/nu_scaler-0.1.0-*.whl
```

### 4.3 Creating Windows Installer

Use PyInstaller with a script to create a Windows installer:

```python
# build_installer.py
import PyInstaller.__main__
import shutil
import os

# Create dist directory if it doesn't exist
if not os.path.exists('dist'):
    os.makedirs('dist')

# Run PyInstaller
PyInstaller.__main__.run([
    'nu_scaler_app.py',
    '--name=NuScaler',
    '--onefile',
    '--windowed',
    '--icon=assets/icon.ico',
    '--add-data=assets;assets'
])

# Copy additional files
shutil.copy('LICENSE', 'dist/LICENSE')
shutil.copy('README.md', 'dist/README.md')
```

## 5. Example Usage

### 5.1 Basic Python Script

```python
# simple_upscaler.py
import nu_scaler

def main():
    # Create components
    engine = nu_scaler.Engine()
    capture = nu_scaler.Capture()
    upscaler = nu_scaler.Upscaler()
    
    # Configure
    capture.set_source("window", "Notepad")
    upscaler.set_technology("FSR")
    upscaler.set_quality("Quality")
    upscaler.set_scale_factor(1.5)
    
    # Start upscaling
    print("Starting upscaling engine...")
    engine.start(capture, upscaler)
    
    # Monitor performance
    try:
        while True:
            stats = engine.get_stats()
            print(f"FPS: {stats.fps:.2f}, Frame time: {stats.frame_time:.2f}ms")
            time.sleep(1)
    except KeyboardInterrupt:
        print("Stopping...")
    finally:
        engine.stop()
    
    print("Done!")

if __name__ == "__main__":
    main()
```

### 5.2 GUI Application

```python
# nu_scaler_app.py
import nu_scaler

def main():
    app = nu_scaler.create_app()
    app.run()

if __name__ == "__main__":
    main()
```

## 6. Testing and Validation

### 6.1 Test Framework

Create a test script to validate the Python bindings:

```python
# test_bindings.py
import unittest
import nu_scaler

class TestNuScalerBindings(unittest.TestCase):
    def test_version(self):
        version = nu_scaler.get_version()
        self.assertIsInstance(version, str)
        self.assertTrue(version.count('.') >= 2)
    
    def test_engine_creation(self):
        engine = nu_scaler.Engine()
        self.assertIsNotNone(engine)
    
    def test_capture_creation(self):
        capture = nu_scaler.Capture()
        self.assertIsNotNone(capture)
    
    def test_upscaler_creation(self):
        upscaler = nu_scaler.Upscaler()
        self.assertIsNotNone(upscaler)
        
        # Test configuration methods
        upscaler.set_technology("FSR")
        upscaler.set_quality("Quality")
        upscaler.set_scale_factor(1.5)

if __name__ == "__main__":
    unittest.main()
```

### 6.2 Performance Benchmarking

Create a benchmark script to measure performance:

```python
# benchmark.py
import nu_scaler
import time
import statistics

def benchmark_upscale_image(iterations=10):
    # Benchmark image upscaling
    times = []
    
    for i in range(iterations):
        start_time = time.time()
        nu_scaler.upscale_image(
            "test_images/input.png",
            f"test_images/output_{i}.png",
            "FSR",
            "Quality",
            1.5
        )
        elapsed = time.time() - start_time
        times.append(elapsed)
        print(f"Iteration {i+1}/{iterations}: {elapsed:.4f} seconds")
    
    avg_time = statistics.mean(times)
    std_dev = statistics.stdev(times) if len(times) > 1 else 0
    
    print(f"\nResults:")
    print(f"Average time: {avg_time:.4f} seconds")
    print(f"Standard deviation: {std_dev:.4f} seconds")
    print(f"Min time: {min(times):.4f} seconds")
    print(f"Max time: {max(times):.4f} seconds")

if __name__ == "__main__":
    benchmark_upscale_image()
```

## 7. Conclusion

This integration guide provides a comprehensive roadmap for merging the existing Rust Nu_scale codebase with Python to create a hybrid application that leverages the strengths of both languages. By following this approach, the application will maintain the high-performance characteristics of the Rust core while gaining the flexibility and rapid development capabilities of Python for the UI and configuration layers.

Key benefits of this approach include:
- Maintaining performance-critical operations in Rust
- Providing an intuitive Python API for application control
- Enabling rapid UI development with modern Python frameworks
- Supporting cross-platform deployment on Windows and Linux

This approach aligns with the architectural vision outlined in the requirements and provides a clear path forward for the Nu Scaler application. 