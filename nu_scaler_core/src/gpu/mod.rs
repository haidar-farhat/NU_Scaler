pub mod detector;
pub mod memory;

use crate::gpu::memory::{AllocationStrategy, MemoryPool, MemoryPressure, VramStats};
use anyhow::Result;
use detector::GpuInfo;
use std::sync::Arc;
use wgpu::{Device, Queue};
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
        let memory_pool = Arc::new(MemoryPool::new(
            device.clone(),
            queue.clone(),
            gpu_info.clone(),
        ));

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
        #[cfg(target_os = "windows")]
        {
            use wgpu::hal::dx12::Api as Dx12Api;
            let native_handle_opt: Option<*mut std::ffi::c_void> = self
                .device
                .as_hal::<Dx12Api, _, _>(|hal_device_opt| {
                    hal_device_opt.map(|d| d.raw_device().as_ptr() as *mut std::ffi::c_void)
                })
                .flatten();

            if let Some(handle) = native_handle_opt {
                if !handle.is_null() {
                    return Ok(handle);
                }
            }
            // If native_handle_opt was None or handle was null, fall through to error or other backends
        }

        #[cfg(target_os = "linux")]
        {
            use wgpu::hal::vulkan::Api as VulkanApi;
            let native_handle_opt: Option<*mut std::ffi::c_void> =
                self.device.as_hal::<VulkanApi, _, _>(|hal_device_opt| {
                    hal_device_opt.map(|d| d.raw_device().handle() as *mut std::ffi::c_void)
                });
            if let Some(handle) = native_handle_opt {
                if !handle.is_null() {
                    return Ok(handle);
                }
            }
        }

        Err(GpuError::UnsupportedBackend)
    }

    /// # Safety
    ///
    /// The returned pointer is a raw, non-null, native texture handle (e.g., ID3D12Resource* or VkImage).
    /// The caller is responsible for ensuring that the handle is used correctly
    /// and within the lifetime of the WGPU texture and device.
    /// The underlying WGPU instance, device, and texture must remain alive while this handle is in use.
    pub unsafe fn get_native_texture_handle(
        &self,
        _texture: &wgpu::Texture,
    ) -> Result<*mut std::ffi::c_void, GpuError> {
        #[cfg(target_os = "windows")]
        {
            // TODO: Find the correct way to get ID3D12Resource* from wgpu_hal::dx12::Texture in wgpu-hal 0.19
            // The `resource` field is pub(super) and no obvious public method seems available.
            // Temporarily returning error.
            eprintln!("[get_native_texture_handle] DX12: Texture resource access not yet implemented correctly for wgpu-hal 0.19.");
            return Err(GpuError::UnsupportedBackend);
            /*
            use wgpu::hal::dx12::Api as Dx12Api;
            let mut native_handle_opt: Option<*mut std::ffi::c_void> = None;
            _texture.as_hal::<Dx12Api, _>(|hal_texture_opt| {
                if let Some(ht) = hal_texture_opt {
                    // native_handle_opt = Some(ht.resource.as_ptr() as *mut std::ffi::c_void); // ht.resource is private
                    // native_handle_opt = Some(ht.raw_resource().as_ptr() as *mut std::ffi::c_void); // ht.raw_resource() not found
                }
            });
            if let Some(handle) = native_handle_opt {
                if !handle.is_null() {
                    return Ok(handle);
                }
            }
            */
        }

        #[cfg(target_os = "linux")]
        {
            use wgpu::hal::vulkan::Api as VulkanApi;
            let mut native_handle_opt: Option<*mut std::ffi::c_void> = None;
            _texture.as_hal::<VulkanApi, _>(|hal_texture_opt| {
                if let Some(ht) = hal_texture_opt {
                    native_handle_opt = Some(ht.raw_texture() as *mut std::ffi::c_void);
                    // vk::Image is u64
                }
            });
            if let Some(handle) = native_handle_opt {
                if !handle.is_null() {
                    return Ok(handle);
                }
            }
        }

        Err(GpuError::UnsupportedBackend)
    }

    /// # Safety
    ///
    /// The returned pointer is a raw, non-null, native buffer handle (e.g., ID3D12Resource* or VkBuffer).
    /// The caller is responsible for ensuring that the handle is used correctly
    /// and within the lifetime of the WGPU buffer and device.
    /// The underlying WGPU instance, device, and buffer must remain alive while this handle is in use.
    pub unsafe fn get_native_buffer_handle(&self, _buffer: &wgpu::Buffer) -> Result<*mut std::ffi::c_void, GpuError> {
        // Determine if running on Windows (DX12) or Linux (Vulkan)
        // For now, mirroring the structure of get_native_texture_handle

        #[cfg(target_os = "windows")]
        {
            // TODO: Find the correct way to get ID3D12Resource* from wgpu_hal::dx12::Buffer in wgpu-hal 0.19
            // The `resource` field in wgpu_hal::dx12::Buffer is pub(super).
            eprintln!("[get_native_buffer_handle] DX12: Buffer resource access not yet implemented correctly for wgpu-hal 0.19.");
            return Err(GpuError::UnsupportedBackend);
            /*
            use wgpu::hal::dx12::Api as Dx12Api;
            let native_handle_opt: Option<*mut std::ffi::c_void> =
                _buffer.as_hal::<Dx12Api, _, _>(|hal_buffer_opt| {
                    hal_buffer_opt.map(|b| b.resource.as_ptr() as *mut std::ffi::c_void) // b.resource is pub(super)
                }).flatten();

            if let Some(handle) = native_handle_opt {
                if !handle.is_null() {
                    return Ok(handle);
                }
            }
            */
        }

        #[cfg(target_os = "linux")]
        {
            use wgpu::hal::vulkan::Api as VulkanApi;
            let native_handle_opt: Option<*mut std::ffi::c_void> =
                _buffer.as_hal::<VulkanApi, _, _>(|hal_buffer_opt| {
                    hal_buffer_opt.map(|b| b.raw_handle().as_raw() as *mut std::ffi::c_void)
                }).flatten(); // VKBuffer is u64, as_raw() converts to pointer
            
            if let Some(handle) = native_handle_opt {
                if !handle.is_null() { 
                    return Ok(handle);
                }
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
        fn initialize(&mut self, _provider: GpuProvider) -> Result<GpuContext> {
            // Prefix unused param
            unimplemented!()
        }
        fn is_supported(&self, _provider: GpuProvider) -> bool {
            // Prefix unused param
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
