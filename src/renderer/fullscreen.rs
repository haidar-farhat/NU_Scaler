use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use eframe::egui::{self, TextureHandle, RgbaImage, Rect, Pos2};
use egui::TextureOptions;
use egui_plot::*;

use crate::capture::{CaptureTarget, FrameBuffer};
use crate::upscale::{Upscaler, UpscalingAlgorithm, UpscalingQuality, UpscalingTechnology};
use crate::config::SETTINGS;
use crate::profiling::FrameTimer;
use std::fs::{self, File, OpenOptions};

impl FullscreenUpscalerUi {
    fn start_processing_thread(&mut self) {
        let metrics_tx_clone = metrics_tx.clone();

        let processing_thread = std::thread::spawn(move || {
        });

        self.processing_thread = Some(processing_thread);
    }

    fn update_source_window_position(&mut self, _ctx: &egui::Context) {
    }

    fn update_texture(&mut self) -> Result<bool> {
        self.texture = Some(Arc::new(Mutex::new(ctx.load_texture(
            "fullscreen_frame",
            egui::ColorImage::from_rgba_unmultiplied(
                [width as usize, height as usize],
                &pixels,
            ),
            TextureOptions::default()
        ))));
        *texture_handle = ctx.load_texture(
            "fullscreen_frame_update",
            egui::ColorImage::from_rgba_unmultiplied(
                [width as usize, height as usize],
                &pixels,
            ),
            TextureOptions::default()
        );
        Ok(true)
    }
}

impl eframe::App for FullscreenUpscalerUi {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.check_and_reinitialize_upscaler();
    }
}

impl FullscreenUpscalerUi {
    pub fn render_upscaled_content(&self, ui: &mut egui::Ui) -> bool {
        texture_lock.is_some()
    }
} 