pub mod detector;
pub mod memory;

use anyhow::Result;
use std::sync::Arc;
use wgpu::{Device, Queue};
use crate::gpu::memory::{MemoryPool, AllocationStrategy, MemoryPressure, VramStats};
use detector::GpuInfo;
// use thiserror::Error; // Unused

/// Supported GPU providers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpuProvider {
    Wgpu,
    Vulkan,
}

#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    #[error("Native handle is null")]
    NullHandle,
    #[error("Unsupported HAL backend for native handle retrieval")]
    UnsupportedBackend,
    #[error("Failed to get native adapter handle")]
    FailedToGetNativeAdapterHandle,
    #[error("Failed to get native device handle")]
    FailedToGetNativeDeviceHandle,
    #[error("Failed to get native texture handle")]
    FailedToGetNativeTextureHandle,
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

    /// # Safety
    ///
    /// The returned pointer is a raw, non-null, native device handle (e.g., ID3D12Device* or VkDevice).
    /// The caller is responsible for ensuring that the handle is used correctly
    /// and within the lifetime of the WGPU device.
    /// The underlying WGPU instance and device must remain alive while this handle is in use.
    pub unsafe fn get_native_device_handle(&self) -> Result<*mut std::ffi::c_void, GpuError> {
        // Import HAL APIs. These might need to be gated by cfg attributes
        // if you only want to compile support for specific backends.
        // use wgpu::hal::Device as HalDevice; // Removed unused import
        // Assuming Dx12 and Vulkan are the primary targets. Add others as needed.
        #[cfg(feature = "dx12")]
        {
            if let Some(raw_device_handle) = self.device.as_hal::<Dx12Api>().raw_device() {
                // For ID3D12Device, this should directly be the pointer.
                // Ensure the type conversion is correct for your specific HAL version/needs.
                return Ok(raw_device_handle as *mut std::ffi::c_void);
            }
        }

        #[cfg(feature = "vulkan")]
        {
             // For Vulkan, `raw_device()` returns `ash::vk::Device` which is a newtype around `vk::VkDevice_T*`
             // We need to get the actual pointer.
            if let Some(vk_device) = self.device.as_hal::<VulkanApi>().raw_device() {
                 return Ok(vk_device.handle() as *mut std::ffi::c_void);
            }
        }
        
        // Fallback or if no specific backend feature is enabled/matched
        Err(GpuError::UnsupportedBackend)
    }

    /// # Safety
    ///
    /// The returned pointer is a raw, non-null, native texture handle (e.g., ID3D12Resource* or VkImage).
    /// The caller is responsible for ensuring that the handle is used correctly
    /// and within the lifetime of the WGPU texture and device.
    /// The underlying WGPU instance, device, and texture must remain alive while this handle is in use.
    pub unsafe fn get_native_texture_handle(&self, texture: &wgpu::Texture) -> Result<*mut std::ffi::c_void, GpuError> {
        // Use as_hal directly on the texture object, no separate HAL Texture trait import needed.
        #[cfg(feature = "dx12")] // WARNING: This cfg flag might not work as intended
        use wgpu::hal::dx12::Api as Dx12Api;
        #[cfg(feature = "vulkan")] // WARNING: This cfg flag might not work as intended
        use wgpu::hal::vulkan::Api as VulkanApi;

        #[cfg(feature = "dx12")] // WARNING: This cfg flag might not work as intended
        {
            if let Some(raw_texture_handle) = texture.as_hal::<Dx12Api>().raw_texture() {
                 return Ok(raw_texture_handle as *mut std::ffi::c_void);
            }
        }

        #[cfg(feature = "vulkan")] // WARNING: This cfg flag might not work as intended
        {
            if let Some(vk_image) = texture.as_hal::<VulkanApi>().raw_texture() {
                return Ok(vk_image as *mut std::ffi::c_void); 
            }
        }
        
        Err(GpuError::UnsupportedBackend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyGpuManager;
    impl GpuManager for DummyGpuManager {
        fn initialize(&mut self, _provider: GpuProvider) -> Result<GpuContext> { // Prefix unused param
            unimplemented!()
        }
        fn is_supported(&self, _provider: GpuProvider) -> bool { // Prefix unused param
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