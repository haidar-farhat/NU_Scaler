use egui::{Vec2, Pos2, Rect, Context};

/// Dialog for selecting a screen region to capture
pub struct RegionDialog {
    /// Selected region x coordinate
    x: i32,
    /// Selected region y coordinate
    y: i32,
    /// Selected region width
    width: i32,
    /// Selected region height
    height: i32,
    /// Whether the user is currently dragging to select a region
    dragging: bool,
    /// Starting position of the drag
    drag_start: Option<Pos2>,
    /// Current position of the drag
    drag_current: Option<Pos2>,
    /// Dialog result (true if OK clicked, false otherwise)
    result: Option<bool>,
}

impl Default for RegionDialog {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 640,
            height: 480,
            dragging: false,
            drag_start: None,
            drag_current: None,
            result: None,
        }
    }
}

impl RegionDialog {
    /// Create a new region selection dialog
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the initial position and size of the region
    pub fn set_region(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
    }
    
    /// Get the selected region
    pub fn get_region(&self) -> (i32, i32, i32, i32) {
        (self.x, self.y, self.width, self.height)
    }
    
    /// Show the dialog
    /// Returns true if OK was clicked, false otherwise
    pub fn show(&mut self, ctx: &Context) -> bool {
        self.result = None;
        
        egui::Window::new("Select Region")
            .collapsible(false)
            .resizable(true)
            .default_size(Vec2::new(400.0, 300.0))
            .show(ctx, |ui| {
                ui.heading("Select Screen Region");
                ui.label("Click and drag to select a region on the screen");
                
                ui.add_space(10.0);
                
                // Display form for manual input
                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.add(egui::DragValue::new(&mut self.x).speed(1.0));
                    ui.label("Y:");
                    ui.add(egui::DragValue::new(&mut self.y).speed(1.0));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    let mut width_u32 = self.width.max(1) as u32;
                    if ui.add(egui::DragValue::new(&mut width_u32).speed(1.0).clamp_range(1..=10000)).changed() {
                        self.width = width_u32 as i32;
                    }
                    
                    ui.label("Height:");
                    let mut height_u32 = self.height.max(1) as u32;
                    if ui.add(egui::DragValue::new(&mut height_u32).speed(1.0).clamp_range(1..=10000)).changed() {
                        self.height = height_u32 as i32;
                    }
                });
                
                ui.add_space(10.0);
                
                // Preview canvas
                let response = ui.allocate_rect(
                    Rect::from_min_size(
                        ui.min_rect().min,
                        Vec2::new(ui.available_width(), 200.0),
                    ),
                    egui::Sense::drag(),
                );
                
                let rect = response.rect;
                let painter = ui.painter();
                
                // Draw background
                painter.rect_filled(
                    rect,
                    0.0,
                    egui::Color32::from_rgb(30, 30, 30),
                );
                
                // Draw selection rectangle
                let scale_x = rect.width() / 1920.0; // Assuming 1920x1080 screen for preview
                let scale_y = rect.height() / 1080.0;
                
                let selection_rect = Rect::from_min_size(
                    Pos2::new(
                        rect.min.x + (self.x as f32 * scale_x),
                        rect.min.y + (self.y as f32 * scale_y),
                    ),
                    Vec2::new(
                        self.width as f32 * scale_x,
                        self.height as f32 * scale_y,
                    ),
                );
                
                painter.rect_stroke(
                    selection_rect,
                    0.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 165, 0)),
                );
                
                // Handle dragging to select region
                if response.drag_started() {
                    self.dragging = true;
                    self.drag_start = response.interact_pointer_pos();
                    self.drag_current = self.drag_start;
                } else if self.dragging && response.dragged() {
                    self.drag_current = response.interact_pointer_pos();
                    
                    if let (Some(start), Some(current)) = (self.drag_start, self.drag_current) {
                        // Convert screen coordinates to actual pixel positions
                        let screen_x1 = ((start.x - rect.min.x) / scale_x) as i32;
                        let screen_y1 = ((start.y - rect.min.y) / scale_y) as i32;
                        let screen_x2 = ((current.x - rect.min.x) / scale_x) as i32;
                        let screen_y2 = ((current.y - rect.min.y) / scale_y) as i32;
                        
                        // Update region coordinates
                        self.x = screen_x1.min(screen_x2);
                        self.y = screen_y1.min(screen_y2);
                        self.width = (screen_x1.max(screen_x2) - self.x).max(1);
                        self.height = (screen_y1.max(screen_y2) - self.y).max(1);
                    }
                } else if response.drag_released() {
                    self.dragging = false;
                    self.drag_start = None;
                    self.drag_current = None;
                }
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                        if ui.button("OK").clicked() {
                            self.result = Some(true);
                        }
                        if ui.button("Cancel").clicked() {
                            self.result = Some(false);
                        }
                    });
                });
            });
        
        // Return result
        self.result.unwrap_or(false)
    }
    
    /// Check if the dialog was cancelled
    pub fn was_cancelled(&self) -> bool {
        self.result == Some(false)
    }
} 