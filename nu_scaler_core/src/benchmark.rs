use pyo3::prelude::*;
// use pyo3::types::PyBytes; // Remove unused
use crate::gpu::detector::GpuDetector;
use crate::upscale::{UpscalerFactory, /*Upscaler,*/ UpscalingQuality, UpscalingTechnology};
use anyhow::{anyhow, Result};
use std::time::Instant;

/// Performance metrics collected during benchmark
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub upscaler_name: String,
    pub technology: UpscalingTechnology,
    pub quality: UpscalingQuality,
    pub input_resolution: (u32, u32),
    pub output_resolution: (u32, u32),
    pub scale_factor: f32,
    pub avg_frame_time_ms: f32,
    pub fps: f32,
    pub frames_processed: u32,
    pub total_duration_ms: f32,
}

/// Python-friendly version of BenchmarkResult
#[pyclass]
pub struct PyBenchmarkResult {
    #[pyo3(get)]
    pub upscaler_name: String,
    #[pyo3(get)]
    pub technology: String,
    #[pyo3(get)]
    pub quality: String,
    #[pyo3(get)]
    pub input_width: u32,
    #[pyo3(get)]
    pub input_height: u32,
    #[pyo3(get)]
    pub output_width: u32,
    #[pyo3(get)]
    pub output_height: u32,
    #[pyo3(get)]
    pub scale_factor: f32,
    #[pyo3(get)]
    pub avg_frame_time_ms: f32,
    #[pyo3(get)]
    pub fps: f32,
    #[pyo3(get)]
    pub frames_processed: u32,
    #[pyo3(get)]
    pub total_duration_ms: f32,
}

impl From<BenchmarkResult> for PyBenchmarkResult {
    fn from(res: BenchmarkResult) -> Self {
        Self {
            upscaler_name: res.upscaler_name,
            technology: format!("{:?}", res.technology),
            quality: format!("{:?}", res.quality),
            input_width: res.input_resolution.0,
            input_height: res.input_resolution.1,
            output_width: res.output_resolution.0,
            output_height: res.output_resolution.1,
            scale_factor: res.scale_factor,
            avg_frame_time_ms: res.avg_frame_time_ms,
            fps: res.fps,
            frames_processed: res.frames_processed,
            total_duration_ms: res.total_duration_ms,
        }
    }
}

/// Benchmark a specific upscaler technology
pub fn benchmark_upscaler(
    technology: UpscalingTechnology,
    quality: UpscalingQuality,
    input_width: u32,
    input_height: u32,
    scale_factor: f32,
    frame_count: u32,
    test_data: &[u8],
) -> Result<BenchmarkResult> {
    if test_data.len() < input_width as usize * input_height as usize * 4 {
        return Err(anyhow!(
            "Test data too small for the specified input resolution"
        ));
    }

    // Create GPU detector to get shared resources
    let mut gpu_detector = GpuDetector::new();
    gpu_detector.detect_gpus()?;

    // Create the upscaler using factory
    let mut upscaler = UpscalerFactory::create_upscaler(technology, quality);

    // Initialize shared GPU resources
    let (device, queue) = pollster::block_on(gpu_detector.create_device_queue())?;
    UpscalerFactory::set_shared_resources(&mut upscaler, device.clone(), queue.clone())?;

    // Calculate output dimensions
    let output_width = (input_width as f32 * scale_factor).round() as u32;
    let output_height = (input_height as f32 * scale_factor).round() as u32;

    // Initialize upscaler
    upscaler.initialize(input_width, input_height, output_width, output_height)?;

    let upscaler_name = upscaler.name().to_string();

    // Run the benchmark
    let mut frame_times = Vec::with_capacity(frame_count as usize);
    let start_time = Instant::now();

    for _ in 0..frame_count {
        let frame_start = Instant::now();
        let _result = upscaler.upscale(test_data)?;
        frame_times.push(frame_start.elapsed().as_secs_f32() * 1000.0);
    }

    let total_duration = start_time.elapsed();
    let total_duration_ms = total_duration.as_secs_f32() * 1000.0;

    // Calculate statistics
    let avg_frame_time_ms = frame_times.iter().sum::<f32>() / frame_times.len() as f32;
    let fps = 1000.0 / avg_frame_time_ms;

    Ok(BenchmarkResult {
        upscaler_name,
        technology,
        quality,
        input_resolution: (input_width, input_height),
        output_resolution: (output_width, output_height),
        scale_factor,
        avg_frame_time_ms,
        fps,
        frames_processed: frame_count,
        total_duration_ms,
    })
}

/// Run a multi-technology benchmark comparing all available upscalers
pub fn run_upscaler_comparison(
    input_width: u32,
    input_height: u32,
    scale_factor: f32,
    frame_count: u32,
) -> Result<Vec<BenchmarkResult>> {
    // Generate test pattern
    let test_data = generate_test_pattern(input_width, input_height);

    // List of technologies to benchmark
    let technologies = [
        UpscalingTechnology::FSR,
        UpscalingTechnology::DLSS,
        UpscalingTechnology::Wgpu,
        UpscalingTechnology::Fallback,
    ];

    // List of qualities to benchmark
    let qualities = [
        UpscalingQuality::Ultra,
        UpscalingQuality::Quality,
        UpscalingQuality::Balanced,
        UpscalingQuality::Performance,
    ];

    let mut results = Vec::new();

    // Run benchmark for each technology and quality combination
    for tech in &technologies {
        for quality in &qualities {
            match benchmark_upscaler(
                *tech,
                *quality,
                input_width,
                input_height,
                scale_factor,
                frame_count,
                &test_data,
            ) {
                Ok(result) => results.push(result),
                Err(e) => println!("Error benchmarking {:?}/{:?}: {}", tech, quality, e),
            }
        }
    }

    Ok(results)
}

/// Generate a test pattern for benchmarking
fn generate_test_pattern(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            // Create a colorful gradient test pattern
            let r = (x * 255 / width) as u8;
            let g = (y * 255 / height) as u8;
            let b = ((x + y) * 255 / (width + height)) as u8;
            let a = 255u8; // Full alpha

            data.push(r);
            data.push(g);
            data.push(b);
            data.push(a);
        }
    }

    data
}

/// Python-friendly benchmark function
#[pyfunction]
pub fn py_benchmark_upscaler(
    technology: &str,
    quality: &str,
    input_width: u32,
    input_height: u32,
    scale_factor: f32,
    frame_count: u32,
) -> PyResult<PyBenchmarkResult> {
    // Convert parameters to internal types
    let tech = match technology.to_lowercase().as_str() {
        "fsr" => UpscalingTechnology::FSR,
        "dlss" => UpscalingTechnology::DLSS,
        "wgpu" => UpscalingTechnology::Wgpu,
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

    // Generate test pattern
    let test_data = generate_test_pattern(input_width, input_height);

    // Run benchmark
    match benchmark_upscaler(
        tech,
        qual,
        input_width,
        input_height,
        scale_factor,
        frame_count,
        &test_data,
    ) {
        Ok(result) => Ok(result.into()),
        Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
            "Benchmark error: {}",
            e
        ))),
    }
}

/// Python-friendly comparison benchmark
#[pyfunction]
pub fn py_run_comparison_benchmark(
    input_width: u32,
    input_height: u32,
    scale_factor: f32,
    frame_count: u32,
) -> PyResult<Vec<PyBenchmarkResult>> {
    match run_upscaler_comparison(input_width, input_height, scale_factor, frame_count) {
        Ok(results) => Ok(results.into_iter().map(Into::into).collect()),
        Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
            "Benchmark error: {}",
            e
        ))),
    }
}
