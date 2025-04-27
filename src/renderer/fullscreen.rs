use std::sync::{Arc, Mutex, atomic::AtomicBool, Ordering};
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use anyhow::Result;
use eframe::{self, egui};
use egui::{Vec2, TextureOptions};
use image::RgbaImage;
use std::time::{Instant, Duration};
use log::{warn, error, trace, info};
use std::panic::AssertUnwindSafe;
use rand;
use egui_wgpu::WgpuConfiguration;

use crate::capture::common::FrameBuffer;
use crate::upscale::{Upscaler, UpscalingTechnology, UpscalingQuality, UpscalingAlgorithm};
use crate::capture::CaptureTarget;
use crate::capture::ScreenCapture;
use crate::capture::frame_buffer_ext::ArcFrameBufferExt; 