use anyhow::{Result, anyhow};
use image::{DynamicImage, RgbaImage};
use image::imageops::{self, FilterType};
use std::sync::atomic::{AtomicBool, Ordering};
use log;
use std::time::Instant;
use crate::capture::ScreenCapture;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::capture::CaptureTarget;
use crate::upscale::{UpscalingTechnology, UpscalingQuality};

Ok(imageops::resize(input, width, height, FilterType::Lanczos3))

let _elapsed = frame_start_time.elapsed(); 