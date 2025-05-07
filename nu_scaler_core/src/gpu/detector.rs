use anyhow::Result;
use wgpu::{Adapter, AdapterInfo, Backends, Instance, DeviceType, Backend};
use std::sync::Arc;
use pyo3::prelude::*;

use crate::upscale::UpscalingTechnology;

/// GPU vendor identification
#[derive(Debug, Clone, Copy, PartialEq)]
#[pyclass]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Other,
    Unknown,
}

/// GPU information
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: GpuVendor,
    pub device_type: DeviceType,
    pub backend: wgpu::Backend,
    pub vendor_id: u32,
    pub device_id: u32,
    pub driver_info: String,
    pub is_discrete: bool,
}

impl Default for GpuInfo {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            vendor: GpuVendor::Unknown,
            device_type: DeviceType::Other,
            backend: wgpu::Backend::Empty,
            vendor_id: 0,
            device_id: 0,
            driver_info: String::new(),
            is_discrete: false,
        }
    }
}

impl From<AdapterInfo> for GpuInfo {
    fn from(info: AdapterInfo) -> Self {
        let vendor = match info.vendor {
            0x10DE => GpuVendor::Nvidia, // NVIDIA
            0x1002 => GpuVendor::Amd,    // AMD
            0x8086 => GpuVendor::Intel,  // Intel
            _ => GpuVendor::Other,
        };
        
        let is_discrete = matches!(info.device_type, DeviceType::DiscreteGpu);

        // Print GPU info during detection
        println!("[GPU Detector] Found GPU: {} (Vendor ID: 0x{:X}, Device ID: 0x{:X})", 
            info.name, info.vendor, info.device);
        println!("[GPU Detector] Device type: {:?}, Backend: {:?}", info.device_type, info.backend);
        println!("[GPU Detector] Driver info: {}", info.driver_info);
        
        Self {
            name: info.name,
            vendor,
            device_type: info.device_type,
            backend: info.backend,
            vendor_id: info.vendor,
            device_id: info.device,
            driver_info: info.driver_info,
            is_discrete,
        }
    }
}

/// GPU detector to identify available hardware
pub struct GpuDetector {
    instance: Instance,
    primary_gpu: Option<GpuInfo>,
    all_gpus: Vec<GpuInfo>,
}

impl GpuDetector {
    /// Create a new GPU detector instance
    pub fn new() -> Self {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });
        
        Self {
            instance,
            primary_gpu: None,
            all_gpus: Vec::new(),
        }
    }
    
    /// Detect all available GPUs in the system
    pub fn detect_gpus(&mut self) -> Result<()> {
        let adapters = pollster::block_on(self.enumerate_adapters())?;
        
        // Convert to GpuInfo and store
        self.all_gpus = adapters.iter()
            .map(|adapter| {
                let info = adapter.get_info();
                GpuInfo::from(info)
            })
            .collect();
        
        // Find best GPU for primary
        self.primary_gpu = self.determine_primary_gpu();
        
        Ok(())
    }
    
    /// Enumerate all WGPU adapters
    async fn enumerate_adapters(&self) -> Result<Vec<Adapter>> {
        // Convert enumerated adapters to Vec directly
        let adapters = self.instance.enumerate_adapters(Backends::PRIMARY);
        // Manually collect into a vector since adapters is already an iterator
        let mut adapter_vec = Vec::new();
        for adapter in adapters {
            adapter_vec.push(adapter);
        }
        Ok(adapter_vec)
    }
    
    /// Determine the primary GPU to use
    fn determine_primary_gpu(&self) -> Option<GpuInfo> {
        if self.all_gpus.is_empty() {
            return None;
        }
        
        // Prefer discrete GPUs
        let discrete_gpus: Vec<_> = self.all_gpus.iter()
            .filter(|gpu| gpu.is_discrete)
            .collect();
        
        if !discrete_gpus.is_empty() {
            // Prefer NVIDIA > AMD > Intel > Others for discrete
            for vendor in [GpuVendor::Nvidia, GpuVendor::Amd, GpuVendor::Intel] {
                let gpu = discrete_gpus.iter()
                    .find(|g| g.vendor == vendor)
                    .cloned()
                    .cloned();
                
                if gpu.is_some() {
                    return gpu;
                }
            }
            
            // If no preferred vendor, return first discrete
            return Some(discrete_gpus[0].clone());
        }
        
        // Fall back to any GPU
        Some(self.all_gpus[0].clone())
    }
    
    /// Get information about the primary GPU
    pub fn get_primary_gpu(&self) -> Option<&GpuInfo> {
        self.primary_gpu.as_ref()
    }
    
    /// Get all detected GPUs
    pub fn get_all_gpus(&self) -> &[GpuInfo] {
        &self.all_gpus
    }
    
    /// Determine the best upscaling technology for the detected GPU
    pub fn determine_best_upscaling_technology(&self) -> UpscalingTechnology {
        match self.primary_gpu {
            Some(ref gpu) => match gpu.vendor {
                GpuVendor::Nvidia => {
                    UpscalingTechnology::DLSS
                },
                GpuVendor::Amd => {
                    #[cfg(feature = "fsr3")]
                    {
                        UpscalingTechnology::FSR
                    }
                    #[cfg(not(feature = "fsr3"))]
                    {
                        println!("[GpuDetector] AMD GPU detected, but 'fsr3' feature not enabled. Falling back to Wgpu.");
                        UpscalingTechnology::Wgpu // Fallback if fsr3 feature is off
                    }
                },
                GpuVendor::Intel => {
                    #[cfg(feature = "fsr3")]
                    {
                        UpscalingTechnology::FSR // Intel GPUs can use FSR as well
                    }
                    #[cfg(not(feature = "fsr3"))]
                    {
                        println!("[GpuDetector] Intel GPU detected, but 'fsr3' feature not enabled. Falling back to Wgpu.");
                        UpscalingTechnology::Wgpu // Fallback if fsr3 feature is off
                    }
                },
                _ => {
                    UpscalingTechnology::Wgpu
                }
            },
            None => {
                UpscalingTechnology::Fallback
            }
        }
    }
    
    /// Get a human-readable description of the primary GPU
    pub fn get_gpu_description(&self) -> String {
        match &self.primary_gpu {
            Some(gpu) => {
                let gpu_type = match gpu.device_type {
                    DeviceType::DiscreteGpu => "Discrete",
                    DeviceType::IntegratedGpu => "Integrated",
                    DeviceType::Cpu => "CPU",
                    DeviceType::VirtualGpu => "Virtual",
                    DeviceType::Other => "Other",
                };
                
                let vendor = match gpu.vendor {
                    GpuVendor::Nvidia => "NVIDIA",
                    GpuVendor::Amd => "AMD",
                    GpuVendor::Intel => "Intel",
                    GpuVendor::Other => "Other",
                    GpuVendor::Unknown => "Unknown",
                };
                
                format!("{} {} GPU: {} ({:?})", gpu_type, vendor, gpu.name, gpu.backend)
            },
            None => "No GPU detected".to_string()
        }
    }
    
    /// Create a device and queue from the primary GPU
    pub async fn create_device_queue(&self) -> Result<(Arc<wgpu::Device>, Arc<wgpu::Queue>)> {
        let adapter = self.instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            }
        ).await.ok_or_else(|| anyhow::anyhow!("Failed to get adapter"))?;
        
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Primary Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ).await?;
        
        Ok((Arc::new(device), Arc::new(queue)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu::{AdapterInfo, DeviceType, Backend};

    // Helper to create a basic AdapterInfo for testing
    fn mock_adapter_info(vendor_id: u32) -> AdapterInfo {
        AdapterInfo {
            name: String::new(),
            vendor: vendor_id,
            device: 0,
            device_type: DeviceType::Other,
            driver: String::new(),
            driver_info: String::new(),
            backend: Backend::Empty,
        }
    }

    #[test]
    fn test_gpu_vendor_from_id() {
        assert_eq!(GpuVendor::Nvidia, GpuInfo::from(mock_adapter_info(0x10DE)).vendor);
        assert_eq!(GpuVendor::Amd, GpuInfo::from(mock_adapter_info(0x1002)).vendor);
        assert_eq!(GpuVendor::Intel, GpuInfo::from(mock_adapter_info(0x8086)).vendor);
        assert_eq!(GpuVendor::Other, GpuInfo::from(mock_adapter_info(0xABCD)).vendor);
    }
} 