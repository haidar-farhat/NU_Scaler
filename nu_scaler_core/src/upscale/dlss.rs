use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::ffi::c_void;

use crate::dlss_manager::{self, DlssManagerError};
use crate::dlss_sys::{self, SlDlssFeature, SlStatus, SlDLSSOptions, SlDLSSMode, SlBoolean};
use crate::gpu::{GpuResources, GpuError, GpuProvider}; // Added GpuProvider if needed for GpuResources construction
use crate::upscale::{Upscaler, UpscalingQuality};

pub struct DlssUpscaler {
    quality: UpscalingQuality,
    gpu_resources: Option<Arc<GpuResources>>,
    dlss_feature: Option<SlDlssFeature>,
    native_device_handle: *mut c_void, 
    input_width: u32,
    input_height: u32,
    output_width: u32,
    output_height: u32,
    initialized: bool,
}

impl DlssUpscaler {
    pub fn new(quality: UpscalingQuality) -> Self {
        Self {
            quality,
            gpu_resources: None, // Initialize as None
            dlss_feature: None,
            native_device_handle: std::ptr::null_mut(),
            input_width: 0,
            input_height: 0,
            output_width: 0,
            output_height: 0,
            initialized: false,
        }
    }

    // Method for the factory to set GpuResources
    // Conforming to the set_device_queue structure seen in the factory, but taking GpuResources
    pub fn set_gpu_resources(&mut self, gpu_resources: Arc<GpuResources>) {
        self.gpu_resources = Some(gpu_resources);
    }

    // Alternative, if GpuResources needs to be constructed here
    // pub fn set_device_queue(&mut self, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, /* gpu_info: Option<GpuInfo> */) {
    //     // Potentially create GpuInfo or pass as None
    //     let gpu_info = None; // Placeholder
    //     self.gpu_resources = Some(Arc::new(GpuResources::new(device, queue, gpu_info)));
    // }

    fn map_quality_to_dlss_mode(quality: UpscalingQuality, output_width: u32, output_height: u32) -> (SlDLSSMode, u32, u32) {
        let mode = match quality {
            UpscalingQuality::Ultra => SlDLSSMode::UltraQuality,
            UpscalingQuality::Quality => SlDLSSMode::MaxQuality,
            UpscalingQuality::Balanced => SlDLSSMode::Balanced,
            UpscalingQuality::Performance => SlDLSSMode::MaxPerformance,
        };
        let render_width = match mode {
            SlDLSSMode::UltraQuality | SlDLSSMode::DLAA => output_width * 2 / 3, 
            SlDLSSMode::MaxQuality => output_width * 2 / 3, 
            SlDLSSMode::Balanced => output_width * 58 / 100, 
            SlDLSSMode::MaxPerformance => output_width / 2, 
            SlDLSSMode::UltraPerformance => output_width / 3, 
            _ => output_width,
        };
        let render_height = match mode {
            SlDLSSMode::UltraQuality | SlDLSSMode::DLAA => output_height * 2 / 3,
            SlDLSSMode::MaxQuality => output_height * 2 / 3,
            SlDLSSMode::Balanced => output_height * 58 / 100,
            SlDLSSMode::MaxPerformance => output_height / 2,
            SlDLSSMode::UltraPerformance => output_height / 3,
            _ => output_height,
        };
        (mode, render_width, render_height)
    }
}

impl Upscaler for DlssUpscaler {
    fn initialize(&mut self, input_width: u32, input_height: u32, output_width: u32, output_height: u32) -> Result<()> {
        if self.initialized {
            if self.input_width == input_width && self.input_height == input_height && 
               self.output_width == output_width && self.output_height == output_height {
                return Ok(());
            }
            if let Some(feature) = self.dlss_feature.take() {
                unsafe { dlss_sys::slDestroyDlssFeature(feature) };
                println!("[DLSS Upscaler] Destroyed existing DLSS feature due to dimension change.");
            }
            self.initialized = false;
        }

        let gpu_res = self.gpu_resources.as_ref().ok_or_else(|| anyhow!("GpuResources not set before initialize"))?;

        self.input_width = input_width;
        self.input_height = input_height;
        self.output_width = output_width;
        self.output_height = output_height;
        
        println!("[DLSS Upscaler] Initializing with Input: {}x{}, Output: {}x{}", 
            input_width, input_height, output_width, output_height);

        dlss_manager::ensure_sdk_initialized().map_err(|e| anyhow!("DLSS SDK init failed: {:?}", e))?;
        println!("[DLSS Upscaler] DLSS SDK ensured to be initialized.");

        self.native_device_handle = unsafe { gpu_res.get_native_device_handle()? };
        if self.native_device_handle.is_null() {
            return Err(anyhow!("Failed to get native GPU device handle or handle is null. Potential GpuError: {:?}", GpuError::NullHandle));
        }
        println!("[DLSS Upscaler] Got native device handle: {:?}", self.native_device_handle);
        
        let mut dlss_feature_handle: SlDlssFeature = std::ptr::null_mut();
        let status = unsafe {
            dlss_sys::slCreateDlssFeature(
                self.native_device_handle,
                input_width, 
                input_height, 
                0, 
                &mut dlss_feature_handle,
            )
        };

        if status != SlStatus::Success || dlss_feature_handle.is_null() {
            return Err(anyhow!("slCreateDlssFeature failed with status {:?} or returned null handle.", status));
        }
        self.dlss_feature = Some(dlss_feature_handle);
        println!("[DLSS Upscaler] slCreateDlssFeature successful. Handle: {:?}", dlss_feature_handle);

        let (dlss_mode, _render_w, _render_h) = Self::map_quality_to_dlss_mode(self.quality, output_width, output_height);
        
        let options = SlDLSSOptions {
            mode: dlss_mode,
            output_width: output_width,
            output_height: output_height,
            color_buffers_hdr: SlBoolean::False, 
            ..SlDLSSOptions::default()
        };
        
        println!("[DLSS Upscaler] DLSS Options prepared: mode={:?}, output={}x{}", options.mode, options.output_width, options.output_height);
        // Actual setting of options via slDLSSSetOptions or slSetFeatureSpecifics would happen here if API allows/requires.

        self.initialized = true;
        Ok(())
    }

    fn upscale(&self, input_bytes: &[u8]) -> Result<Vec<u8>> {
        if !self.initialized || self.dlss_feature.is_none() || self.gpu_resources.is_none() {
            return Err(anyhow!("DlssUpscaler not initialized, feature not created, or GpuResources not set."));
        }
        // let _dlss_feature = self.dlss_feature.unwrap();
        // let _gpu_res = self.gpu_resources.as_ref().unwrap();

        // Actual upscale logic with WGPU texture creation & FFI call is still TODO.
        println!("[DLSS Upscaler] Upscale called for input size: {} bytes. (Output target: {}x{})", 
            input_bytes.len(), self.output_width, self.output_height);
        Err(anyhow!("DLSS upscale logic not yet fully implemented."))
    }

    fn name(&self) -> &'static str {
        "DLSSUpscaler"
    }

    fn quality(&self) -> UpscalingQuality {
        self.quality
    }

    fn set_quality(&mut self, quality: UpscalingQuality) -> Result<()> {
        if self.quality == quality {
            return Ok(());
        }
        self.quality = quality;
        if self.initialized {
            println!("[DLSS Upscaler] Quality changed to {:?}. Re-initialization might be needed, or options updated on feature.", quality);
            // Logic to call slDLSSSetOptions or mark for re-init would go here.
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl Drop for DlssUpscaler {
    fn drop(&mut self) {
        if let Some(feature_handle) = self.dlss_feature.take() {
            println!("[DLSS Upscaler] Dropping DlssUpscaler, destroying DLSS feature: {:?}", feature_handle);
            let status = unsafe { dlss_sys::slDestroyDlssFeature(feature_handle) };
            if status != SlStatus::Success {
                eprintln!("[DLSS Upscaler] Error destroying DLSS feature {:?}: {:?}", feature_handle, status);
            }
        }
    }
} 