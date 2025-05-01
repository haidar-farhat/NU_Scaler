use anyhow::Result;

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