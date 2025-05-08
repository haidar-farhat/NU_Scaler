use crate::gpu::detector::{GpuInfo, GpuVendor};
use anyhow::Result;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};
use wgpu::{Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Device, Queue};

/// VRAM usage statistics
#[derive(Debug, Clone)]
pub struct VramStats {
    /// Total GPU memory in MB
    pub total_mb: f32,
    /// Used GPU memory in MB
    pub used_mb: f32,
    /// Free GPU memory in MB
    pub free_mb: f32,
    /// Memory allocated by this application in MB
    pub app_allocated_mb: f32,
    /// Timestamp when these stats were collected
    pub timestamp: Instant,
}

impl Default for VramStats {
    fn default() -> Self {
        Self {
            total_mb: 0.0,
            used_mb: 0.0,
            free_mb: 0.0,
            app_allocated_mb: 0.0,
            timestamp: Instant::now(),
        }
    }
}

impl VramStats {
    /// Get the usage percentage
    pub fn usage_percent(&self) -> f32 {
        if self.total_mb > 0.0 {
            (self.used_mb / self.total_mb) * 100.0
        } else {
            0.0
        }
    }
}

/// GPU memory pressure level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPressure {
    /// Plenty of memory available
    Low,
    /// Getting close to limits, consider conservative usage
    Medium,
    /// Memory usage is high, aggressively reduce usage
    High,
    /// Critically low memory, emergency measures needed
    Critical,
}

/// Memory buffer allocation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationStrategy {
    /// Aggressive: Pre-allocate buffers, never shrink
    Aggressive,
    /// Balanced: Use buffer pool with occasional cleanup
    Balanced,
    /// Conservative: Only allocate when needed, clean up quickly
    Conservative,
    /// Minimal: Minimize memory usage at cost of performance
    Minimal,
}

/// GPU memory pool for buffer reuse and recycling
pub struct MemoryPool {
    /// Reference to GPU device
    device: Arc<Device>,
    /// Reference to GPU queue
    queue: Arc<Queue>,
    /// Buffer pools by size
    buffer_pools: Mutex<HashMap<usize, Vec<Buffer>>>,
    /// Memory tracking statistics
    stats: Mutex<VramStats>,
    /// Currently allocated buffers
    allocated_buffers: AtomicUsize,
    /// Total bytes allocated
    allocated_bytes: AtomicUsize,
    /// Maximum number of buffers per pool
    max_pool_size: AtomicUsize,
    /// Current allocation strategy
    strategy: Mutex<AllocationStrategy>,
    /// Last time the pool was cleaned up
    last_cleanup: Mutex<Instant>,
    /// GPU information
    _gpu_info: Option<GpuInfo>,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new(device: Arc<Device>, queue: Arc<Queue>, gpu_info: Option<GpuInfo>) -> Self {
        let strategy = match gpu_info.as_ref().map(|g| g.vendor) {
            Some(GpuVendor::Nvidia) => AllocationStrategy::Aggressive, // NVIDIA GPUs handle large allocations well
            Some(GpuVendor::Amd) => AllocationStrategy::Balanced,      // AMD is in the middle
            Some(GpuVendor::Intel) => AllocationStrategy::Conservative, // Intel integrated GPUs have limited memory
            _ => AllocationStrategy::Balanced, // Default to balanced for unknown GPUs
        };

        println!("[MemoryPool] Created with {:?} strategy", strategy);

        // Better estimate for total VRAM - properly detect high-end cards
        let estimated_vram = if let Some(info) = &gpu_info {
            if info.is_discrete {
                match info.vendor {
                    // Better estimates for high-end cards
                    GpuVendor::Nvidia => {
                        if info.name.contains("3090")
                            || info.name.contains("4090")
                            || info.name.contains("A6000")
                            || info.name.contains("A100")
                        {
                            24576.0 // 24GB for high-end NVIDIA
                        } else if info.name.contains("3080")
                            || info.name.contains("4080")
                            || info.name.contains("2080 Ti")
                        {
                            12288.0 // 12GB for mid-high end
                        } else if info.name.contains("3070") || info.name.contains("2080") {
                            8192.0 // 8GB for mid-range
                        } else {
                            6144.0 // 6GB for regular cards
                        }
                    }
                    GpuVendor::Amd => {
                        if info.name.contains("7900") || info.name.contains("6900") {
                            16384.0 // 16GB for high-end AMD
                        } else if info.name.contains("6800") {
                            12288.0 // 12GB for mid-high AMD
                        } else {
                            8192.0 // 8GB for regular AMD cards
                        }
                    }
                    _ => 4096.0, // Assume 4GB for other discrete
                }
            } else {
                match info.vendor {
                    GpuVendor::Intel => 2048.0, // Assume 2GB for Intel integrated
                    _ => 2048.0,                // Assume 2GB for other integrated
                }
            }
        } else {
            4096.0 // Default to 4GB when GPU info unavailable
        };

        let stats = VramStats {
            total_mb: estimated_vram,
            used_mb: 0.0,
            free_mb: estimated_vram,
            app_allocated_mb: 0.0,
            timestamp: Instant::now(),
        };

        let pool = Self {
            device,
            queue,
            buffer_pools: Mutex::new(HashMap::new()),
            stats: Mutex::new(stats),
            allocated_buffers: AtomicUsize::new(0),
            allocated_bytes: AtomicUsize::new(0),
            max_pool_size: AtomicUsize::new(16), // Increase default pool size for better reuse
            strategy: Mutex::new(strategy),
            last_cleanup: Mutex::new(Instant::now()),
            _gpu_info: gpu_info,
        };

        // Pre-allocate buffers for common sizes to improve initial performance
        if strategy == AllocationStrategy::Aggressive {
            pool.preallocate_common_buffers();
        }

        pool
    }

    /// Pre-allocate buffers for common sizes to improve initial performance
    fn preallocate_common_buffers(&self) {
        let common_sizes = [
            1920 * 1080 * 4, // Full HD
            2560 * 1440 * 4, // 2K
            3840 * 2160 * 4, // 4K
            5120 * 2880 * 4, // 5K
        ];

        for &size in &common_sizes {
            // Pre-allocate input, output and staging buffers
            let _input_buffer = self.get_buffer(
                size,
                BufferUsages::STORAGE | BufferUsages::COPY_DST,
                Some(&format!("Preallocated Input Buffer ({})", size)),
            );

            let _output_buffer = self.get_buffer(
                size,
                BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                Some(&format!("Preallocated Output Buffer ({})", size)),
            );

            let _staging_buffer = self.get_buffer(
                size,
                BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                Some(&format!("Preallocated Staging Buffer ({})", size)),
            );
        }

        println!("[MemoryPool] Pre-allocated buffers for common resolutions");
    }

    /// Get a buffer from the pool or create a new one
    pub fn get_buffer(&self, size: usize, usage: BufferUsages, label: Option<&str>) -> Buffer {
        let strategy = *self.strategy.lock().unwrap();

        // For minimal strategy, always create a new buffer without pooling
        if strategy == AllocationStrategy::Minimal {
            self.create_new_buffer(size, usage, label)
        } else {
            // Try to get a buffer from the pool first
            let mut pools = self.buffer_pools.lock().unwrap();

            // Round up size to nearest 1MB for better pooling
            let aligned_size = ((size + (1024 * 1024 - 1)) / (1024 * 1024)) * (1024 * 1024);

            if let Some(pool) = pools.get_mut(&aligned_size) {
                if let Some(buffer) = pool.pop() {
                    // Found a buffer in the pool
                    return buffer;
                }
            }

            // No buffer in pool, create a new one
            self.create_new_buffer(aligned_size, usage, label)
        }
    }

    /// Return a buffer to the pool
    pub fn return_buffer(&self, buffer: Buffer) {
        let strategy = *self.strategy.lock().unwrap();

        // For minimal or conservative strategy, just drop the buffer
        if strategy == AllocationStrategy::Minimal || strategy == AllocationStrategy::Conservative {
            return;
        }

        let size = buffer.size() as usize;
        let max_pool_size = self.max_pool_size.load(Ordering::Relaxed);

        let mut pools = self.buffer_pools.lock().unwrap();
        let pool = pools.entry(size).or_insert_with(Vec::new);

        // Only add to pool if we're below max size
        if pool.len() < max_pool_size {
            pool.push(buffer);
        }
        // Otherwise, let it drop
    }

    /// Create a new buffer
    fn create_new_buffer(&self, size: usize, usage: BufferUsages, label: Option<&str>) -> Buffer {
        let buffer = self.device.create_buffer(&BufferDescriptor {
            label,
            size: size as u64,
            usage,
            mapped_at_creation: false,
        });

        // Update stats
        self.allocated_buffers.fetch_add(1, Ordering::Relaxed);
        self.allocated_bytes.fetch_add(size, Ordering::Relaxed);

        let mut stats = self.stats.lock().unwrap();
        stats.app_allocated_mb =
            self.allocated_bytes.load(Ordering::Relaxed) as f32 / (1024.0 * 1024.0);
        stats.used_mb += size as f32 / (1024.0 * 1024.0);
        stats.free_mb = stats.total_mb - stats.used_mb;

        buffer
    }

    /// Update memory strategy based on current usage
    pub fn update_strategy(&self) {
        let stats = self.stats.lock().unwrap();
        let pressure = self.get_memory_pressure(&stats);

        let new_strategy = match pressure {
            MemoryPressure::Low => AllocationStrategy::Aggressive,
            MemoryPressure::Medium => AllocationStrategy::Balanced,
            MemoryPressure::High => AllocationStrategy::Conservative,
            MemoryPressure::Critical => AllocationStrategy::Minimal,
        };

        let mut strategy = self.strategy.lock().unwrap();
        if *strategy != new_strategy {
            println!(
                "[MemoryPool] Changing allocation strategy from {:?} to {:?} (pressure: {:?})",
                *strategy, new_strategy, pressure
            );
            *strategy = new_strategy;

            // If becoming more conservative, clean up pools
            if new_strategy == AllocationStrategy::Conservative
                || new_strategy == AllocationStrategy::Minimal
            {
                drop(strategy); // Release lock before cleanup
                self.cleanup_pools();
            }
        }
    }

    /// Get current memory pressure level
    pub fn get_memory_pressure(&self, stats: &VramStats) -> MemoryPressure {
        let usage_percent = if stats.total_mb > 0.0 {
            stats.used_mb / stats.total_mb * 100.0
        } else {
            50.0 // Default to medium if we don't know total
        };

        match usage_percent {
            x if x < 50.0 => MemoryPressure::Low,
            x if x < 75.0 => MemoryPressure::Medium,
            x if x < 90.0 => MemoryPressure::High,
            _ => MemoryPressure::Critical,
        }
    }

    /// Clean up buffer pools to free memory
    pub fn cleanup_pools(&self) {
        let mut last_cleanup = self.last_cleanup.lock().unwrap();
        if last_cleanup.elapsed() < Duration::from_secs(5) {
            // Don't clean up too often
            return;
        }

        *last_cleanup = Instant::now();

        let strategy = *self.strategy.lock().unwrap();
        let mut pools = self.buffer_pools.lock().unwrap();

        // For conservative strategies, be more aggressive with cleanup
        let pool_size_limit = match strategy {
            AllocationStrategy::Aggressive => 8,
            AllocationStrategy::Balanced => 4,
            AllocationStrategy::Conservative => 2,
            AllocationStrategy::Minimal => 0, // No pooling for minimal
        };

        self.max_pool_size.store(pool_size_limit, Ordering::Relaxed);

        // Trim pools to size
        for pool in pools.values_mut() {
            while pool.len() > pool_size_limit {
                pool.pop();
            }
        }

        // For minimal, clear all pools
        if strategy == AllocationStrategy::Minimal {
            pools.clear();
        }

        // Update stats
        let mut stats = self.stats.lock().unwrap();
        stats.timestamp = Instant::now();

        // Try to query actual VRAM usage from system
        if let Some(updated_stats) = self.query_vram_stats() {
            *stats = updated_stats;
        }
    }

    /// Try to query VRAM stats from the system (platform-specific)
    fn query_vram_stats(&self) -> Option<VramStats> {
        #[cfg(target_os = "windows")]
        {
            self.query_vram_windows()
        }

        #[cfg(target_os = "linux")]
        {
            self.query_vram_linux()
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            None
        }
    }

    /// Query VRAM stats on Windows using DXGI
    #[cfg(target_os = "windows")]
    fn query_vram_windows(&self) -> Option<VramStats> {
        use std::mem::zeroed;
        use std::time::Instant;
        use windows::core::Interface;
        use windows::Win32::Graphics::Dxgi;

        unsafe {
            // Create DXGI factory
            let factory_result = Dxgi::CreateDXGIFactory1::<Dxgi::IDXGIFactory1>();
            if let Ok(factory) = factory_result {
                // Get first adapter
                if let Ok(adapter) = factory.EnumAdapters1(0) {
                    // Try to get IDXGIAdapter3 for VRAM info
                    let adapter3: Result<Dxgi::IDXGIAdapter3, _> = adapter.cast();
                    if let Ok(adapter3) = adapter3 {
                        let mut dedicated_vram: u64 = 0;
                        let mut usage_vram: u64 = 0;

                        // Get adapter description with properly created descriptor
                        let mut desc = zeroed::<Dxgi::DXGI_ADAPTER_DESC1>();
                        if adapter.GetDesc1(&mut desc).is_ok() {
                            dedicated_vram = desc.DedicatedVideoMemory as u64;
                        }

                        // Get current usage with properly created memory info struct
                        let mut budget = zeroed::<Dxgi::DXGI_QUERY_VIDEO_MEMORY_INFO>();
                        if adapter3
                            .QueryVideoMemoryInfo(
                                0,
                                Dxgi::DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
                                &mut budget,
                            )
                            .is_ok()
                        {
                            usage_vram = budget.CurrentUsage;
                        }

                        // Convert to MB
                        let total_mb = dedicated_vram as f32 / (1024.0 * 1024.0);
                        let used_mb = usage_vram as f32 / (1024.0 * 1024.0);
                        let free_mb = total_mb - used_mb;

                        let mut stats = self.stats.lock().unwrap();
                        stats.total_mb = total_mb;
                        stats.used_mb = used_mb;
                        stats.free_mb = free_mb;
                        stats.app_allocated_mb =
                            self.allocated_bytes.load(Ordering::Relaxed) as f32 / (1024.0 * 1024.0);
                        stats.timestamp = Instant::now();

                        return Some(stats.clone());
                    }
                }
            }
        }

        None
    }

    /// Query VRAM stats on Linux
    #[cfg(target_os = "linux")]
    fn query_vram_linux(&self) -> Option<VramStats> {
        // Try to read from sysfs
        // Different paths for different GPU vendors
        use std::fs::File;
        use std::io::Read;

        if let Some(info) = &self._gpu_info {
            match info.vendor {
                GpuVendor::Nvidia => {
                    // Try NVIDIA sysfs path
                    let path = "/proc/driver/nvidia/gpus/0/information";
                    if let Ok(mut file) = File::open(path) {
                        let mut content = String::new();
                        if file.read_to_string(&mut content).is_ok() {
                            // Parse memory info
                            if let Some(mem_line) = content.lines().find(|l| l.contains("Memory")) {
                                if let Some(mem_str) = mem_line.split(':').nth(1) {
                                    if let Ok(total_mb) = mem_str
                                        .trim()
                                        .split_whitespace()
                                        .next()
                                        .unwrap_or("0")
                                        .parse::<f32>()
                                    {
                                        // Try to get used memory
                                        let used_path = "/proc/driver/nvidia/gpus/0/vram_usage";
                                        if let Ok(mut used_file) = File::open(used_path) {
                                            let mut used_content = String::new();
                                            if used_file.read_to_string(&mut used_content).is_ok() {
                                                if let Ok(used_mb) =
                                                    used_content.trim().parse::<f32>()
                                                {
                                                    let mut stats = self.stats.lock().unwrap();
                                                    stats.total_mb = total_mb;
                                                    stats.used_mb = used_mb;
                                                    stats.free_mb = total_mb - used_mb;
                                                    stats.app_allocated_mb = self
                                                        .allocated_bytes
                                                        .load(Ordering::Relaxed)
                                                        as f32
                                                        / (1024.0 * 1024.0);
                                                    stats.timestamp = Instant::now();
                                                    return Some(stats.clone());
                                                }
                                            }
                                        }

                                        // Fallback to just total
                                        let mut stats = self.stats.lock().unwrap();
                                        stats.total_mb = total_mb;
                                        stats.used_mb = self.stats.lock().unwrap().used_mb;
                                        stats.free_mb =
                                            total_mb - self.stats.lock().unwrap().used_mb;
                                        stats.app_allocated_mb =
                                            self.allocated_bytes.load(Ordering::Relaxed) as f32
                                                / (1024.0 * 1024.0);
                                        stats.timestamp = Instant::now();
                                        return Some(stats.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                GpuVendor::Amd => {
                    // Try AMD sysfs path
                    let path = "/sys/class/drm/card0/device/mem_info_vram_total";
                    if let Ok(mut file) = File::open(path) {
                        let mut content = String::new();
                        if file.read_to_string(&mut content).is_ok() {
                            if let Ok(total_bytes) = content.trim().parse::<u64>() {
                                let total_mb = total_bytes as f32 / (1024.0 * 1024.0);

                                // Try to get used memory
                                let used_path = "/sys/class/drm/card0/device/mem_info_vram_used";
                                if let Ok(mut used_file) = File::open(used_path) {
                                    let mut used_content = String::new();
                                    if used_file.read_to_string(&mut used_content).is_ok() {
                                        if let Ok(used_bytes) = used_content.trim().parse::<u64>() {
                                            let used_mb = used_bytes as f32 / (1024.0 * 1024.0);
                                            let mut stats = self.stats.lock().unwrap();
                                            stats.total_mb = total_mb;
                                            stats.used_mb = used_mb;
                                            stats.free_mb = total_mb - used_mb;
                                            stats.app_allocated_mb =
                                                self.allocated_bytes.load(Ordering::Relaxed) as f32
                                                    / (1024.0 * 1024.0);
                                            stats.timestamp = Instant::now();
                                            return Some(stats.clone());
                                        }
                                    }
                                }

                                // Fallback to just total
                                let mut stats = self.stats.lock().unwrap();
                                stats.total_mb = total_mb;
                                stats.used_mb = self.stats.lock().unwrap().used_mb;
                                stats.free_mb = total_mb - self.stats.lock().unwrap().used_mb;
                                stats.app_allocated_mb =
                                    self.allocated_bytes.load(Ordering::Relaxed) as f32
                                        / (1024.0 * 1024.0);
                                stats.timestamp = Instant::now();
                                return Some(stats.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Get current memory stats
    pub fn get_stats(&self) -> VramStats {
        let mut stats = self.stats.lock().unwrap();

        // If stats are older than 5 seconds, try to update
        if stats.timestamp.elapsed() > Duration::from_secs(5) {
            if let Some(updated_stats) = self.query_vram_stats() {
                *stats = updated_stats;
            } else {
                // Update timestamp to avoid frequent queries
                stats.timestamp = Instant::now();
            }
        }

        stats.clone()
    }

    /// Set the maximum size of each buffer pool
    pub fn set_max_pool_size(&self, size: usize) {
        self.max_pool_size.store(size, Ordering::Relaxed);
        self.cleanup_pools();
    }

    /// Set memory allocation strategy
    pub fn set_allocation_strategy(&self, strategy: AllocationStrategy) {
        let mut current = self.strategy.lock().unwrap();
        *current = strategy;
        drop(current);

        self.cleanup_pools();
    }

    /// Get the current memory allocation strategy
    pub fn get_allocation_strategy(&self) -> AllocationStrategy {
        *self.strategy.lock().unwrap()
    }

    /// Get the current memory pressure level
    pub fn get_current_memory_pressure(&self) -> MemoryPressure {
        let stats = self.stats.lock().unwrap();
        self.get_memory_pressure(&stats)
    }

    /// Update VRAM usage periodically
    pub fn update_vram_usage(&self) -> Result<(), anyhow::Error> {
        // Try to get actual VRAM stats
        if let Some(stats) = self.query_vram_stats() {
            let mut current_stats = self.stats.lock().unwrap();
            *current_stats = stats;

            // Log VRAM usage with better formatting
            println!(
                "[MemoryPool] VRAM: {}MB used / {}MB total ({}%)",
                current_stats.used_mb,
                current_stats.total_mb,
                current_stats.used_mb / current_stats.total_mb * 100.0
            );
        }

        Ok(())
    }

    /// Get allocated buffers count
    pub fn get_allocated_buffers_count(&self) -> usize {
        self.allocated_buffers.load(Ordering::Relaxed)
    }

    /// Get allocated bytes
    pub fn get_allocated_bytes(&self) -> usize {
        self.allocated_bytes.load(Ordering::Relaxed)
    }

    /// Force update VRAM usage on Windows using DXGI
    #[cfg(target_os = "windows")]
    pub fn force_update_vram_usage_windows(&self) -> Result<VramStats, anyhow::Error> {
        match self.query_vram_windows() {
            Some(stats) => {
                let mut current_stats = self.stats.lock().unwrap();
                *current_stats = stats.clone();
                Ok(stats)
            }
            None => Err(anyhow::anyhow!("Failed to query VRAM stats")),
        }
    }

    /// Force VRAM usage on Windows (useful for ensuring GPU is active)
    #[cfg(target_os = "windows")]
    pub fn force_gpu_usage(&self) -> Result<(), anyhow::Error> {
        // Create a large buffer to force the GPU to activate
        let size = 256 * 1024 * 1024; // 256MB
        let buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("GPU Wake Buffer"),
            size: size as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        // Create a small staging buffer
        let staging = self.device.create_buffer(&BufferDescriptor {
            label: Some("Wake Staging Buffer"),
            size: 4096,
            usage: BufferUsages::COPY_SRC | BufferUsages::MAP_WRITE,
            mapped_at_creation: true,
        });

        // Initialize staging buffer
        {
            let mut view = staging.slice(..).get_mapped_range_mut();
            for i in 0..1024 {
                let idx = i * 4;
                if idx + 3 < view.len() {
                    view[idx] = (i % 256) as u8;
                    view[idx + 1] = ((i + 85) % 256) as u8;
                    view[idx + 2] = ((i + 170) % 256) as u8;
                    view[idx + 3] = 255;
                }
            }
        }

        // Unmap staging buffer
        staging.unmap();

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Wake GPU Encoder"),
            });

        // Copy staging buffer to large buffer multiple times with different offsets
        for i in 0..64 {
            let offset = i * 4096;
            encoder.copy_buffer_to_buffer(&staging, 0, &buffer, offset as u64, 4096);
        }

        // Submit command
        self.queue.submit(Some(encoder.finish()));

        // Poll device
        self.device.poll(wgpu::Maintain::Wait);

        println!(
            "[MemoryPool] Forced GPU wake-up with {}MB allocation",
            size / (1024 * 1024)
        );

        // Update VRAM stats
        self.update_vram_usage()?;

        Ok(())
    }
}

/// Implementation of `Drop` for proper cleanup
impl Drop for MemoryPool {
    fn drop(&mut self) {
        // Clear all pools
        if let Ok(mut pools) = self.buffer_pools.lock() {
            pools.clear();
        }

        // Log memory stats at end
        if let Ok(stats) = self.stats.lock() {
            println!(
                "[MemoryPool] Final stats: {}MB allocated, {}MB total, {}MB used, {}MB free",
                stats.app_allocated_mb, stats.total_mb, stats.used_mb, stats.free_mb
            );
        }
    }
}

/// Python-friendly memory stats
#[cfg(feature = "python")]
#[pyclass]
pub struct PyVramStats {
    #[pyo3(get)]
    pub total_mb: f32,
    #[pyo3(get)]
    pub used_mb: f32,
    #[pyo3(get)]
    pub free_mb: f32,
    #[pyo3(get)]
    pub app_allocated_mb: f32,
    #[pyo3(get)]
    pub usage_percent: f32,
}

#[cfg(feature = "python")]
impl From<VramStats> for PyVramStats {
    fn from(stats: VramStats) -> Self {
        let usage_percent = if stats.total_mb > 0.0 {
            (stats.used_mb / stats.total_mb) * 100.0
        } else {
            0.0
        };

        Self {
            total_mb: stats.total_mb,
            used_mb: stats.used_mb,
            free_mb: stats.free_mb,
            app_allocated_mb: stats.app_allocated_mb,
            usage_percent,
        }
    }
}
