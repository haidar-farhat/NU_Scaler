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