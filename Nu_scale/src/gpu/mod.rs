pub mod vulkan_init;

pub use vulkan_init::{VulkanContext, is_vulkan_supported, initialize_vulkan};

/// Enum to represent different GPU providers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpuProvider {
    Vulkan,
}

impl GpuProvider {
    /// Get a string representation of the GPU provider
    pub fn as_str(&self) -> &'static str {
        match self {
            GpuProvider::Vulkan => "Vulkan",
        }
    }
    
    /// Check if the GPU provider is supported on this system
    pub fn is_supported(&self) -> bool {
        match self {
            GpuProvider::Vulkan => vulkan_init::is_vulkan_supported(),
        }
    }
}

/// Initialize the specified GPU provider
pub fn initialize_gpu(provider: GpuProvider) -> anyhow::Result<GpuState> {
    match provider {
        GpuProvider::Vulkan => {
            let context = vulkan_init::initialize_vulkan()?;
            Ok(GpuState::Vulkan(context))
        }
    }
}

/// Represents the state of an initialized GPU
#[derive(Clone)]
pub enum GpuState {
    Vulkan(VulkanContext),
} 