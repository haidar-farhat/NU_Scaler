use std::path::Path;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use std::thread;
use crate::capture::CaptureTarget;
use crate::upscale::{UpscalingTechnology, UpscalingQuality};
use image::imageops;
use image::imageops::FilterType;

Ok(imageops::resize(input, width, height, imageops::FilterType::Lanczos3)) 