pub mod detector;
pub mod memory;

use anyhow::Result;
use std::sync::Arc;
use wgpu::{Device, Queue};
use crate::gpu::memory::{MemoryPool, AllocationStrategy, MemoryPressure, VramStats};
use detector::GpuInfo;

/// Supported GPU providers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpuProvider {
    Wgpu,
    Vulkan,
}

/// Placeholder for GPU context (device, queue, etc.)
pub struct GpuContext;

/// Trait for GPU device/context management
pub trait GpuManager {
    /// Initialize the GPU context
    fn initialize(&mut self, provider: GpuProvider) -> Result<GpuContext>;
    /// Check if a provider is supported
    fn is_supported(&self, provider: GpuProvider) -> bool;
}

/// Wrapper for GPU resources including device, queue, and memory pool
pub struct GpuResources {
    /// GPU device
    pub device: Arc<Device>,
    /// GPU queue
    pub queue: Arc<Queue>,
    /// Memory pool for buffer management
    pub memory_pool: Arc<MemoryPool>,
    /// Information about the GPU
    pub gpu_info: Option<GpuInfo>,
}

impl GpuResources {
    /// Create new GPU resources with device and queue
    pub fn new(device: Arc<Device>, queue: Arc<Queue>, gpu_info: Option<GpuInfo>) -> Self {
        let memory_pool = Arc::new(MemoryPool::new(device.clone(), queue.clone(), gpu_info.clone()));
        
        Self {
            device,
            queue,
            memory_pool,
            gpu_info,
        }
    }
    
    /// Get VRAM statistics
    pub fn get_vram_stats(&self) -> VramStats {
        self.memory_pool.get_stats()
    }
    
    /// Get current memory pressure level
    pub fn get_memory_pressure(&self) -> MemoryPressure {
        self.memory_pool.get_current_memory_pressure()
    }
    
    /// Set memory allocation strategy
    pub fn set_allocation_strategy(&self, strategy: AllocationStrategy) {
        self.memory_pool.set_allocation_strategy(strategy);
    }
    
    /// Update memory strategy based on current usage
    pub fn update_memory_strategy(&self) {
        self.memory_pool.update_strategy();
    }
    
    /// Clean up memory pools to free resources
    pub fn cleanup_memory(&self) {
        self.memory_pool.cleanup_pools();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyGpuManager;
    impl GpuManager for DummyGpuManager {
        fn initialize(&mut self, _provider: GpuProvider) -> Result<GpuContext> {
            unimplemented!()
        }
        fn is_supported(&self, _provider: GpuProvider) -> bool {
            unimplemented!()
        }
    }

    #[test]
    #[should_panic]
    fn test_initialize_panics() {
        let mut mgr = DummyGpuManager;
        let _ = mgr.initialize(GpuProvider::Wgpu).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_is_supported_panics() {
        let mgr = DummyGpuManager;
        let _ = mgr.is_supported(GpuProvider::Wgpu);
    }
} 