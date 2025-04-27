use std::path::Path;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use std::thread;
use crate::capture::CaptureTarget;
use crate::upscale::{UpscalingTechnology, UpscalingQuality};
use image::imageops::{self, FilterType};

Ok(imageops::resize(input, width, height, FilterType::Lanczos3)) 