use crate::capture;
use crate::upscale;
use crate::upscale::Upscaler;
use crate::upscale::common::UpscalingAlgorithm;
use crate::UpscalingAlgorithm as LibUpscalingAlgorithm;
use crate::upscale::{UpscalingTechnology, UpscalingQuality};

// Test function to verify imports
pub fn test_imports() {
    println!("Testing imports");
    let tech = UpscalingTechnology::FSR;
    let quality = UpscalingQuality::Ultra;
    
    // Verify I can call a function
    if let Some(alg) = crate::string_to_algorithm("lanczos") {
        println!("Got algorithm: {:?}", alg);
    }
} 