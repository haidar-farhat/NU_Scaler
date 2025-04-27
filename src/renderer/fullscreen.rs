use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use eframe::egui::{self, TextureHandle, RgbaImage, Rect, Pos2, ColorImage, TextureOptions, TextureId, Vec2, Ui};
use egui::TextureOptions;
use egui_plot::*;

use crate::capture::{CaptureTarget, FrameBuffer};
use crate::upscale::{Upscaler, UpscalingAlgorithm, UpscalingQuality, UpscalingTechnology};
use crate::config::SETTINGS;
use crate::profiling::FrameTimer;
use std::fs::{self, File, OpenOptions};
use std::sync::{Arc, Mutex};

impl FullscreenUpscalerUi {
    fn start_processing_thread(&mut self) {
        let (metrics_tx, _metrics_rx) = std::sync::mpsc::channel();
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
        let texture_lock = self.texture.lock().unwrap();
        if let Some(texture_handle) = texture_lock.as_ref() {
            if texture_handle.is_allocated() {
                let available_size = ui.available_size();
                let texture_id: TextureId = texture_handle.id();
                let size: Vec2 = texture_handle.size_vec2();
                let aspect_ratio = size.x / size.y;

                // Calculate display size
                let mut display_size = available_size;
                if available_size.x / aspect_ratio <= available_size.y {
                    display_size.y = available_size.x / aspect_ratio;
                } else {
                    display_size.x = available_size.y * aspect_ratio;
                }

                // Center the image
                let (response, _painter) = ui.allocate_painter(available_size, egui::Sense::hover());
                let rect = Rect::from_center_size(response.rect.center(), display_size);

                // Draw the image - FIX HERE
                let image_widget = egui::Image::new((texture_id, size))
                    .fit_to_exact_size(rect.size());
                image_widget.paint_at(ui, rect);
                return true;
            } else {
                ui.label("Texture not allocated.");
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.spinner();
                ui.label(" Waiting for frame...");
            });
        }
        false
    }
} 