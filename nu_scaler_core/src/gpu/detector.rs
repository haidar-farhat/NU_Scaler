use anyhow::Result;
use wgpu::{Adapter, AdapterInfo, Backends, Instance, DeviceType};
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
        let adapters = self.instance.enumerate_adapters(Backends::PRIMARY);
        let adapter_vec: Vec<_> = adapters.collect();
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
                    // TODO: Check for DLSS support based on GPU model
                    // For now, return DLSS for all NVIDIA GPUs
                    UpscalingTechnology::DLSS
                },
                GpuVendor::Amd => {
                    // For AMD GPUs, use FSR
                    UpscalingTechnology::FSR
                },
                GpuVendor::Intel => {
                    // Intel GPUs can use FSR as well
                    UpscalingTechnology::FSR
                },
                _ => {
                    // Default to Wgpu for unknown GPUs
                    UpscalingTechnology::Wgpu
                }
            },
            None => {
                // If no GPU detected, use the fallback
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
    
    #[test]
    fn test_gpu_vendor_from_id() {
        assert_eq!(GpuVendor::Nvidia, GpuInfo::from(AdapterInfo {
            vendor: 0x10DE,
            ..Default::default()
        }).vendor);
        
        assert_eq!(GpuVendor::Amd, GpuInfo::from(AdapterInfo {
            vendor: 0x1002,
            ..Default::default()
        }).vendor);
        
        assert_eq!(GpuVendor::Intel, GpuInfo::from(AdapterInfo {
            vendor: 0x8086,
            ..Default::default()
        }).vendor);
        
        assert_eq!(GpuVendor::Other, GpuInfo::from(AdapterInfo {
            vendor: 0xABCD,
            ..Default::default()
        }).vendor);
    }
} 