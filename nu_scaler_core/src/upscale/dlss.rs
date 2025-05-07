use anyhow::{anyhow, Result};
use std::ffi::c_void;
use std::sync::Arc;

use crate::dlss_manager::{self /*, DlssManagerError*/}; // Removed unused DlssManagerError
use crate::gpu::GpuResources; // Removed unused GpuProvider
use crate::upscale::{Upscaler, UpscalingQuality};
use dlss_sys::{self, SlBoolean, SlDLSSMode, SlDLSSOptions, SlDlssFeature, SlStatus}; // Changed crate::dlss_sys to dlss_sys
 // For create_buffer_init

pub struct DlssUpscaler {
    quality: UpscalingQuality,
    gpu_resources: Option<Arc<GpuResources>>,
    dlss_feature: Option<SlDlssFeature>,
    input_width: u32, // Render resolution (input to DLSS)
    input_height: u32, // Render resolution (input to DLSS)
    output_width: u32, // Target output resolution
    output_height: u32, // Target output resolution
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

    // Determines the actual rendering (input) resolution for DLSS based on output and quality
    fn get_dlss_render_resolutions(
        quality: UpscalingQuality,
        target_output_width: u32,
        target_output_height: u32,
    ) -> (SlDLSSMode, u32, u32) {
        let mode = match quality {
            UpscalingQuality::Ultra => SlDLSSMode::UltraQuality, // Assuming UltraQuality is preferred over DLAA for upscaling
            UpscalingQuality::Quality => SlDLSSMode::MaxQuality,
            UpscalingQuality::Balanced => SlDLSSMode::Balanced,
            UpscalingQuality::Performance => SlDLSSMode::MaxPerformance,
        };

        // These are example ratios, refer to NVIDIA's guidelines for precise values per mode
        let (render_width_ratio, render_height_ratio) = match mode {
            SlDLSSMode::UltraQuality => (2.0/3.0, 2.0/3.0), // Placeholder, typically fixed ratios like 66.7%
            SlDLSSMode::MaxQuality => (2.0/3.0, 2.0/3.0),      // e.g., 66.7%
            SlDLSSMode::Balanced => (0.58, 0.58),        // e.g., 58%
            SlDLSSMode::MaxPerformance => (0.50, 0.50),    // e.g., 50%
            SlDLSSMode::UltraPerformance => (1.0/3.0, 1.0/3.0), // e.g., 33.3%
            SlDLSSMode::DLAA => (1.0, 1.0), // DLAA renders at native resolution
            _ => (1.0, 1.0), // Default or Off, render at native
        };

        let render_width = (target_output_width as f32 * render_width_ratio).round() as u32;
        let render_height = (target_output_height as f32 * render_height_ratio).round() as u32;

        (mode, render_width.max(1), render_height.max(1)) // Ensure non-zero
    }
}

impl Upscaler for DlssUpscaler {
    fn initialize(
        &mut self,
        requested_input_width: u32,  // This is the application's desired render width (pre-DLSS)
        requested_input_height: u32, // This is the application's desired render height (pre-DLSS)
        target_output_width: u32,    // This is the final display/output width
        target_output_height: u32,   // This is the final display/output height
    ) -> Result<()> {
        // Determine the true DLSS mode and actual input (render) dimensions based on quality and target output
        let (actual_dlss_mode, actual_input_width, actual_input_height) = 
            Self::get_dlss_render_resolutions(self.quality, target_output_width, target_output_height);

        if self.initialized {
            // If already initialized, check if a re-initialization is truly needed
            if self.input_width == actual_input_width
                && self.input_height == actual_input_height
                && self.output_width == target_output_width
                && self.output_height == target_output_height
            {
                println!("[DLSS Upscaler] Already initialized with correct dimensions and quality. Skipping.");
                return Ok(());
            }
            // If dimensions or quality implied different settings, destroy old feature
            if let Some(feature) = self.dlss_feature.take() {
                unsafe { dlss_sys::slDestroyDlssFeature(feature) };
                println!(
                    "[DLSS Upscaler] Destroyed existing DLSS feature due to settings change."
                );
            }
            self.initialized = false;
        }

        let gpu_res = self
            .gpu_resources
            .as_ref()
            .ok_or_else(|| anyhow!("GpuResources not set before initialize"))?;

        // Store the determined resolutions
        self.input_width = actual_input_width; 
        self.input_height = actual_input_height;
        self.output_width = target_output_width;
        self.output_height = target_output_height;

        println!(
            "[DLSS Upscaler] Initializing for DLSS Mode: {:?}, Input (Render): {}x{}, Output (Display): {}x{}",
            actual_dlss_mode, self.input_width, self.input_height, self.output_width, self.output_height
        );

        dlss_manager::ensure_sdk_initialized()
            .map_err(|e| anyhow!("DLSS SDK init failed: {:?}", e))?;
        println!("[DLSS Upscaler] DLSS SDK ensured to be initialized.");

        let native_device_handle = unsafe { gpu_res.get_native_device_handle()? };
        if native_device_handle.is_null() {
            return Err(anyhow!("Failed to get native GPU device handle or handle is null."));
        }
        println!(
            "[DLSS Upscaler] Got native device handle: {:?}",
            native_device_handle
        );

        let mut dlss_feature_handle: SlDlssFeature = std::ptr::null_mut();
        // Create DLSS feature with TARGET OUTPUT dimensions
        let status_create = unsafe {
            dlss_sys::slCreateDlssFeature(
                native_device_handle,
                self.output_width,  // Use target output width
                self.output_height, // Use target output height
                0, // flags (dlss_sys::SL_DLSS_FLAG_NONE or similar if defined, else 0)
                &mut dlss_feature_handle,
            )
        };

        if status_create != SlStatus::Success || dlss_feature_handle.is_null() {
            return Err(anyhow!(
                "slCreateDlssFeature failed with status {:?} or returned null handle. Target output: {}x{}",
                status_create, self.output_width, self.output_height
            ));
        }
        self.dlss_feature = Some(dlss_feature_handle);
        println!(
            "[DLSS Upscaler] slCreateDlssFeature successful for output {}x{}. Handle: {:?}",
            self.output_width, self.output_height, dlss_feature_handle
        );

        // Set DLSS options using slDLSSSetOptions (if available and needed after feature creation)
        // The `dlss-sys` crate has `slDLSSSetOptions` which takes a viewport handle.
        // This seems to imply that options are set per viewport AFTER a generic feature might be created,
        // or the SlDlssFeature handle implicitly is the viewport for these calls.
        // For now, we assume the feature handle is sufficient if slDLSSSetOptions is not used or integrated differently.
        // However, the SlDLSSOptions struct is available and should be used.
        // The NVIDIA Streamline examples often show slSetFeatureSpecifics or equivalent
        // to pass DLSS options *after* creating the generic Streamline feature.
        // The current `dlss-sys` seems to have `slDLSSSetOptions` which takes `SlViewportHandle`.
        // Let's assume `slDLSSSetOptions` should be called. What is the viewport handle?
        // The provided `dlss-sys` also defines `slDLSSOptions`.
        // The feature handle we got IS the SlDlssFeature, not a generic viewport.
        // Let's check if there's an `slDLSSSetFeatureOptions` or similar, or if `slDLSSSetOptions` uses the feature handle.

        // The `dlss-sys` has `pub fn slDLSSSetOptions(viewport: SlViewportHandle, options: *const SlDLSSOptions) -> SlStatus;`
        // This is problematic as `SlDlssFeature` is not `SlViewportHandle`.
        // This part of `dlss-sys` might be mismatched with how DLSS is actually configured.
        // For now, we will prepare options but cannot directly call a `set_options` on the feature handle with current `dlss-sys`.
        // This is a limitation of the current `dlss-sys` FFI bindings if they don't align with feature-specific option setting.

        let options = SlDLSSOptions {
            mode: actual_dlss_mode, // Use the mode determined by quality
            output_width: self.output_width,
            output_height: self.output_height,
            // input_width: self.input_width, // SlDLSSOptions doesn't have input_width/height fields in typical SDKs
            // input_height: self.input_height, // these are usually implicit or part of constants
            color_buffers_hdr: SlBoolean::False, // Default, assuming LDR for now
            // sharpness: 0.0, // Default or configurable
            // ... other fields like pre_exposure, exposure_scale, presets from SlDLSSOptions::default()
            ..SlDLSSOptions::default() // Use defaults for other fields for now
        };
        println!(
            "[DLSS Upscaler] DLSS Options prepared: mode={:?}, output={}x{}, (render_input_for_app: {}x{})",
            options.mode, options.output_width, options.output_height, self.input_width, self.input_height
        );
        // TODO: If `slDLSSSetOptions` or a similar function is to be called,
        // it needs to be identified how to do this with the feature handle or if `dlss-sys` needs updates.
        // For now, proceeding without explicitly setting options after creation, 
        // assuming `slCreateDlssFeature` and parameters to `slEvaluateDlssFeature` are primary drivers.

        self.initialized = true;
        Ok(())
    }

    fn upscale(&self, input_bytes: &[u8]) -> Result<Vec<u8>> {
        if !self.initialized {
            return Err(anyhow!("DlssUpscaler: Not initialized."));
        }
        let dlss_feature = self.dlss_feature.ok_or_else(|| {
            anyhow!("DlssUpscaler: DLSS feature handle is None even after initialization.")
        })?;
        let gpu_res = self
            .gpu_resources
            .as_ref()
            .ok_or_else(|| anyhow!("DlssUpscaler: GpuResources not set."))?;

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
                input_bytes.len(),
                self.input_width,
                self.input_height,
                bytes_per_pixel,
                (source_input_bytes_per_row * self.input_height)
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
        let native_input_color_handle =
            unsafe { gpu_res.get_native_texture_handle(&input_texture)? };

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
        let native_output_color_handle =
            unsafe { gpu_res.get_native_texture_handle(&output_texture)? };

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
            return Err(anyhow!(
                "slEvaluateDlssFeature failed with status: {:?}",
                eval_status
            ));
        }

        // 6. Retrieve result from Output Texture
        let tightly_packed_output_bytes_per_row = bytes_per_pixel * self.output_width;
        let aligned_output_bytes_per_row = {
            let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            (tightly_packed_output_bytes_per_row + alignment - 1) / alignment * alignment
        };
        let output_buffer_size =
            (aligned_output_bytes_per_row * self.output_height) as wgpu::BufferAddress;

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
        output_staging_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                // It's good practice to check if send fails, though in this single-threaded map_async context, it rarely does.
                if sender.send(result).is_err() {
                    // eprintln! or log an error if the receiver was dropped, though unlikely here.
                }
            });

        device.poll(wgpu::Maintain::Wait);

        match receiver.recv() {
            Ok(Ok(())) => {
                // Mapping successful
                let mapped_data_range = output_staging_buffer.slice(..).get_mapped_range();
                let mut final_output_bytes = Vec::with_capacity(
                    (bytes_per_pixel * self.output_width * self.output_height) as usize,
                );

                for r in 0..self.output_height {
                    let row_start_in_padded_buffer = (r * aligned_output_bytes_per_row) as usize;
                    let row_end_in_padded_buffer =
                        row_start_in_padded_buffer + tightly_packed_output_bytes_per_row as usize;
                    final_output_bytes.extend_from_slice(
                        &mapped_data_range[row_start_in_padded_buffer..row_end_in_padded_buffer],
                    );
                }
                drop(mapped_data_range); // Explicitly drop before unmap, as per wgpu best practices
                output_staging_buffer.unmap();
                Ok(final_output_bytes)
            }
            Ok(Err(e)) => Err(anyhow!("Failed to map DLSS output buffer: {:?}", e)),
            Err(e) => Err(anyhow!(
                "Failed to receive DLSS output buffer map result: {:?}",
                e
            )),
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Drop for DlssUpscaler {
    fn drop(&mut self) {
        if let Some(feature_handle) = self.dlss_feature.take() {
            println!(
                "[DLSS Upscaler] Dropping DlssUpscaler, destroying DLSS feature: {:?}",
                feature_handle
            );
            let status = unsafe { dlss_sys::slDestroyDlssFeature(feature_handle) };
            if status != SlStatus::Success {
                eprintln!(
                    "[DLSS Upscaler] Error destroying DLSS feature {:?}: {:?}",
                    feature_handle, status
                );
            }
        }
    }
}
