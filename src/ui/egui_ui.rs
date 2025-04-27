use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
// use std::collections::HashMap;
use std::time::{Duration, Instant};

let size = texture_id.size_vec2();
// let img = egui::Image::new(texture_id);
let img = egui::Image::new((texture_id, size));

fn update_source_window_position(
    &mut self,
    ctx: &egui::Context,
    rect: egui::Rect,
) -> Option<()> {
    // ... existing code ...
    egui::Image::new((
        texture.id(),
        eframe::egui::vec2(width, height),
    ))
    // ... existing code ...
    egui::Image::new((
        texture.id(),
        eframe::egui::vec2(width, height),
    ))
    // ... existing code ...
    if response.clicked() {
        log::info!("Set Capture Window button clicked");
        self.set_capture_target_window_title(ctx, &name);
    }
    // ... existing code ...
    if response.clicked() {
        log::info!("Reset Window button clicked");
        self.set_capture_target_window_title(ctx, "");
    }
    // ... existing code ...
    // frame.set_maximized(true);
    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
    // ... existing code ...
}

fn set_capture_target_window_title(&mut self, ctx: &egui::Context, title: &str) {
    // ... existing code ...
    .with_inner_size([1280.0, 720.0])
    // ... existing code ...
} 