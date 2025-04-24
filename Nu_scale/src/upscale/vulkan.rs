use anyhow::Result;
use image::RgbaImage;
use log::{debug, info, warn};

use crate::render::VulkanRenderer;
use super::{Upscaler, UpscalingQuality};
use super::common::UpscalingAlgorithm;

pub struct VulkanUpscaler {
    renderer: Option<VulkanRenderer>,
    quality: UpscalingQuality,
    algorithm: UpscalingAlgorithm,
    input_width: u32,
    input_height: u32,
    output_width: u32,
    output_height: u32,
    initialized: bool,
}

impl VulkanUpscaler {
    pub fn new(quality: UpscalingQuality, algorithm: UpscalingAlgorithm) -> Result<Self> {
        debug!("Creating new VulkanUpscaler with quality {:?} and algorithm {:?}", quality, algorithm);
        Ok(Self {
            renderer: None,
            quality,
            algorithm,
            input_width: 0,
            input_height: 0,
            output_width: 0,
            output_height: 0,
            initialized: false,
        })
    }
}

impl Upscaler for VulkanUpscaler {
    fn initialize(&mut self, input_width: u32, input_height: u32, output_width: u32, output_height: u32) -> Result<()> {
        info!("Initializing VulkanUpscaler {}x{} -> {}x{}", input_width, input_height, output_width, output_height);
        
        self.input_width = input_width;
        self.input_height = input_height;
        self.output_width = output_width;
        self.output_height = output_height;
        
        // Create renderer if it doesn't exist
        if self.renderer.is_none() {
            let renderer = VulkanRenderer::new()
                .map_err(|e| anyhow::anyhow!("Failed to create Vulkan renderer: {}", e))?;
            self.renderer = Some(renderer);
        }
        
        // Initialize renderer with our algorithm
        if let Some(renderer) = &mut self.renderer {
            renderer.init(self.algorithm)
                .map_err(|e| anyhow::anyhow!("Failed to initialize Vulkan renderer: {}", e))?;
        } else {
            return Err(anyhow::anyhow!("Renderer is unexpectedly None after creation"));
        }
        
        self.initialized = true;
        Ok(())
    }
    
    fn upscale(&self, input: &RgbaImage) -> Result<RgbaImage> {
        if !self.initialized {
            return Err(anyhow::anyhow!("VulkanUpscaler not initialized"));
        }
        
        let input_width = input.width();
        let input_height = input.height();
        
        // Ensure dimensions match what we were initialized with
        if input_width != self.input_width || input_height != self.input_height {
            warn!("Input dimensions ({}x{}) don't match initialized dimensions ({}x{})",
                  input_width, input_height, self.input_width, self.input_height);
        }
        
        // Create output buffer
        let mut output_buffer = vec![0u8; (self.output_width * self.output_height * 4) as usize];
        
        // Perform upscaling using Vulkan renderer
        if let Some(renderer) = &self.renderer {
            renderer.upscale(
                input.as_raw(), 
                input_width, 
                input_height,
                &mut output_buffer,
                self.output_width,
                self.output_height
            ).map_err(|e| anyhow::anyhow!("Vulkan upscaling failed: {}", e))?;
            
            // Convert buffer to RgbaImage
            let output = RgbaImage::from_raw(self.output_width, self.output_height, output_buffer)
                .ok_or_else(|| anyhow::anyhow!("Failed to create output image from buffer"))?;
                
            Ok(output)
        } else {
            Err(anyhow::anyhow!("Vulkan renderer is not initialized"))
        }
    }
    
    fn upscale_with_algorithm(&self, input: &RgbaImage, algorithm: UpscalingAlgorithm) -> Result<RgbaImage> {
        // If the algorithm is the same as our current one, just use the standard upscale
        if algorithm == self.algorithm {
            return self.upscale(input);
        }
        
        // Otherwise, create a new upscaler with the requested algorithm
        warn!("Changing algorithm requires reinitializing Vulkan upscaler, which may be inefficient");
        let mut new_upscaler = VulkanUpscaler::new(self.quality, algorithm)?;
        new_upscaler.initialize(self.input_width, self.input_height, self.output_width, self.output_height)?;
        let result = new_upscaler.upscale(input);
        new_upscaler.cleanup()?;
        result
    }
    
    fn is_supported() -> bool {
        VulkanRenderer::is_supported()
    }
    
    fn name(&self) -> &'static str {
        "Vulkan"
    }
    
    fn quality(&self) -> UpscalingQuality {
        self.quality
    }
    
    fn set_quality(&mut self, quality: UpscalingQuality) -> Result<()> {
        self.quality = quality;
        // No need to reinitialize - quality primarily affects algorithm selection
        // which we've already captured during initialization
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<()> {
        debug!("Cleaning up VulkanUpscaler");
        if let Some(renderer) = &mut self.renderer {
            renderer.cleanup();
        }
        self.renderer = None;
        self.initialized = false;
        Ok(())
    }
    
    fn needs_initialization(&self) -> bool {
        !self.initialized
    }
    
    fn input_width(&self) -> u32 {
        self.input_width
    }
    
    fn input_height(&self) -> u32 {
        self.input_height
    }
} 