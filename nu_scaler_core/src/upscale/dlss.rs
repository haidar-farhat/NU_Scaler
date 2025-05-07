use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::ffi::c_void;

use crate::dlss_manager::{self/*, DlssManagerError*/}; // Removed unused DlssManagerError
use dlss_sys::{self, SlDlssFeature, SlStatus, SlDLSSOptions, SlDLSSMode, SlBoolean}; // Changed crate::dlss_sys to dlss_sys
use crate::gpu::{GpuResources, GpuError/*, GpuProvider*/}; // Removed unused GpuProvider
use crate::upscale::{Upscaler, UpscalingQuality};

pub struct DlssUpscaler {
    quality: UpscalingQuality,
    gpu_resources: Option<Arc<GpuResources>>,
    dlss_feature: Option<SlDlssFeature>,
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
            input_width: 0,
            input_height: 0,
            output_width: 0,
            output_height: 0,
            initialized: false,
        }
    }

    // Method for the factory to set GpuResources
    // pub fn set_gpu_resources(&mut self, gpu_resources: Arc<GpuResources>) {
    //     self.gpu_resources = Some(gpu_resources);
    // }

    // Conforming to the UpscalerFactory pattern
    pub fn set_device_queue(&mut self, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) {
        // GpuInfo might be obtainable from the device/adapter or passed in if crucial.
        // For now, passing None. If GpuInfo is needed for GpuResources MemoryPool or other critical functions,
        // this might need to be sourced properly.
        let gpu_info = None; // Placeholder: GpuInfo might be needed from adapter.
        self.gpu_resources = Some(Arc::new(GpuResources::new(device, queue, gpu_info)));
    }

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

        // Get native device handle dynamically
        let native_device_handle = unsafe { gpu_res.get_native_device_handle()? };
        if native_device_handle.is_null() {
            return Err(anyhow!("Failed to get native GPU device handle or handle is null. Potential GpuError: {:?}", GpuError::NullHandle));
        }
        println!("[DLSS Upscaler] Got native device handle: {:?}", native_device_handle);
        
        let mut dlss_feature_handle: SlDlssFeature = std::ptr::null_mut();
        let status = unsafe {
            dlss_sys::slCreateDlssFeature(
                native_device_handle,
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
        if !self.initialized {
            return Err(anyhow!("DlssUpscaler: Not initialized."));
        }
        let dlss_feature = self.dlss_feature.ok_or_else(|| anyhow!("DlssUpscaler: DLSS feature handle is None even after initialization."))?;
        let gpu_res = self.gpu_resources.as_ref().ok_or_else(|| anyhow!("DlssUpscaler: GpuResources not set."))?;
        
        let device = &gpu_res.device;
        let queue = &gpu_res.queue;

        let bytes_per_pixel = 4u32;

        // 1. Input Texture
        let input_texture_format = wgpu::TextureFormat::Rgba8Unorm;
        let input_texture_desc = wgpu::TextureDescriptor {
            label: Some("dlss_input_texture"),
            size: wgpu::Extent3d {
                width: self.input_width,
                height: self.input_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: input_texture_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };
        let input_texture = device.create_texture(&input_texture_desc);

        // Assuming input_bytes is tightly packed (no row padding)
        let source_input_bytes_per_row = bytes_per_pixel * self.input_width;
        if (source_input_bytes_per_row * self.input_height) as usize != input_bytes.len() {
            return Err(anyhow!(
                "Input byte length {} does not match expected {}x{}x{} = {}", 
                input_bytes.len(), self.input_width, self.input_height, bytes_per_pixel, (source_input_bytes_per_row * self.input_height)
            ));
        }

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &input_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            input_bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(source_input_bytes_per_row),
                rows_per_image: Some(self.input_height),
            },
            input_texture_desc.size,
        );
        let native_input_color_handle = unsafe { gpu_res.get_native_texture_handle(&input_texture)? };

        // 2. Output Texture
        let output_texture_format = wgpu::TextureFormat::Rgba8Unorm; 
        let output_texture_desc = wgpu::TextureDescriptor {
            label: Some("dlss_output_texture"),
            size: wgpu::Extent3d {
                width: self.output_width,
                height: self.output_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: output_texture_format,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC, 
            view_formats: &[],
        };
        let output_texture = device.create_texture(&output_texture_desc);
        let native_output_color_handle = unsafe { gpu_res.get_native_texture_handle(&output_texture)? };

        // 3. Depth Texture (passing null for now)
        let native_input_depth_handle: *const c_void = std::ptr::null();

        // 4. Jitter (0,0 for now)
        let jitter_x = 0.0f32;
        let jitter_y = 0.0f32;

        // 5. Call slEvaluateDlssFeature
        let eval_status = unsafe {
            dlss_sys::slEvaluateDlssFeature(
                dlss_feature,
                native_input_color_handle,
                native_input_depth_handle,
                jitter_x,
                jitter_y,
                native_output_color_handle, 
            )
        };

        if eval_status != SlStatus::Success {
            return Err(anyhow!("slEvaluateDlssFeature failed with status: {:?}", eval_status));
        }

        // 6. Retrieve result from Output Texture
        let tightly_packed_output_bytes_per_row = bytes_per_pixel * self.output_width;
        let aligned_output_bytes_per_row = {
            let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            (tightly_packed_output_bytes_per_row + alignment - 1) / alignment * alignment
        };
        let output_buffer_size = (aligned_output_bytes_per_row * self.output_height) as wgpu::BufferAddress;

        let output_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dlss_output_staging_buffer"),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("dlss_result_copy_encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_staging_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(aligned_output_bytes_per_row),
                    rows_per_image: Some(self.output_height),
                },
            },
            output_texture_desc.size,
        );

        queue.submit(Some(encoder.finish()));

        // Map buffer and get data
        let (sender, receiver) = std::sync::mpsc::channel();
        output_staging_buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
            // It's good practice to check if send fails, though in this single-threaded map_async context, it rarely does.
            if sender.send(result).is_err() {
                 // eprintln! or log an error if the receiver was dropped, though unlikely here.
            }
        });

        device.poll(wgpu::Maintain::Wait); 

        match receiver.recv() {
            Ok(Ok(())) => { // Mapping successful
                let mapped_data_range = output_staging_buffer.slice(..).get_mapped_range();
                let mut final_output_bytes = Vec::with_capacity((bytes_per_pixel * self.output_width * self.output_height) as usize);
                
                for r in 0..self.output_height {
                    let row_start_in_padded_buffer = (r * aligned_output_bytes_per_row) as usize;
                    let row_end_in_padded_buffer = row_start_in_padded_buffer + tightly_packed_output_bytes_per_row as usize;
                    final_output_bytes.extend_from_slice(&mapped_data_range[row_start_in_padded_buffer..row_end_in_padded_buffer]);
                }
                drop(mapped_data_range); // Explicitly drop before unmap, as per wgpu best practices
                output_staging_buffer.unmap();
                Ok(final_output_bytes)
            }
            Ok(Err(e)) => Err(anyhow!("Failed to map DLSS output buffer: {:?}", e)),
            Err(e) => Err(anyhow!("Failed to receive DLSS output buffer map result: {:?}", e)),
        }
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