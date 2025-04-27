use anyhow::{anyhow, Result};
use std::sync::Arc;
use vulkano::{
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo, QueueFlags,
    },
    instance::{Instance, InstanceCreateInfo},
    VulkanLibrary,
};

/// Holds references to initialized Vulkan objects
#[derive(Clone)]
pub struct VulkanContext {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub compute_queue: Arc<vulkano::device::Queue>,
}

/// Check if Vulkan is supported on this system
pub fn is_vulkan_supported() -> bool {
    match VulkanLibrary::new() {
        Ok(_) => true,
        Err(e) => {
            log::warn!("Vulkan not supported: {}", e);
            false
        }
    }
}

/// Initialize Vulkan and select a suitable compute-capable device
pub fn initialize_vulkan() -> Result<VulkanContext> {
    // Load Vulkan library
    let library = VulkanLibrary::new().map_err(|e| anyhow!("Failed to load Vulkan library: {}", e))?;
    
    // Create instance with app info
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            application_name: Some("NU_Scaler".into()),
            application_version: vulkano::Version { major: 1, minor: 0, patch: 0 },
            engine_name: Some("NU_Scaler".into()),
            engine_version: vulkano::Version { major: 1, minor: 0, patch: 0 },
            ..Default::default()
        },
    )
    .map_err(|e| anyhow!("Failed to create Vulkan instance: {}", e))?;

    // Get physical devices
    log::info!("Looking for compute-capable physical devices");
    let devices = instance
        .enumerate_physical_devices()
        .map_err(|e| anyhow!("Failed to enumerate physical devices: {}", e))?;

    // Find suitable physical device
    let (physical_device, queue_family_index) = find_suitable_device(devices)?;
    
    // Get device properties for logging
    let device_properties = physical_device.properties();
    log::info!(
        "Selected device: {} (type: {:?})",
        device_properties.device_name,
        device_properties.device_type
    );

    // Compute queue info
    let queue_create_info = QueueCreateInfo {
        queue_family_index,
        ..Default::default()
    };

    // Create logical device
    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![queue_create_info],
            enabled_extensions: DeviceExtensions {
                khr_storage_buffer_storage_class: true, // Required for compute shaders
                ..DeviceExtensions::empty()
            },
            ..Default::default()
        },
    )
    .map_err(|e| anyhow!("Failed to create logical device: {}", e))?;

    // Get compute queue
    let compute_queue = queues.next().ok_or_else(|| anyhow!("Failed to get compute queue"))?;
    
    log::info!("Successfully initialized Vulkan for compute operations");
    
    Ok(VulkanContext {
        instance,
        device,
        compute_queue,
    })
}

/// Find a suitable physical device and queue family
fn find_suitable_device(
    devices: impl ExactSizeIterator<Item = Arc<PhysicalDevice>>,
) -> Result<(Arc<PhysicalDevice>, u32)> {
    // Preferred device types in order
    let preferred_types = [
        PhysicalDeviceType::DiscreteGpu,
        PhysicalDeviceType::IntegratedGpu,
        PhysicalDeviceType::VirtualGpu,
        PhysicalDeviceType::Cpu,
        PhysicalDeviceType::Other,
    ];

    // First, sort devices by preferred type
    let mut device_candidates = Vec::new();
    
    for device in devices {
        // Find a queue family that supports compute operations
        for (queue_family_index, queue_family) in device.queue_family_properties().iter().enumerate() {
            if queue_family.queue_flags.contains(QueueFlags::COMPUTE) {
                // Get device properties for scoring
                let properties = device.properties();
                
                // Create candidate
                device_candidates.push((device.clone(), queue_family_index as u32, properties.device_type));
                
                // Break once we find a suitable queue family for this device
                break;
            }
        }
    }

    // Sort by device type preference
    device_candidates.sort_by_key(|(_, _, device_type)| {
        preferred_types
            .iter()
            .position(|&preferred_type| preferred_type == *device_type)
            .unwrap_or(usize::MAX)
    });

    // Select the first candidate or return error
    device_candidates.first()
        .map(|(device, queue_family_index, _)| (device.clone(), *queue_family_index))
        .ok_or_else(|| anyhow!("No suitable compute-capable Vulkan device found"))
} 