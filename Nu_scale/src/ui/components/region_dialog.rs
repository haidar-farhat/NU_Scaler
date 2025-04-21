use egui::{Ui, Context};

/// Region selection dialog component
pub struct RegionDialog {
    /// X coordinate
    x: i32,
    /// Y coordinate
    y: i32,
    /// Width
    width: u32,
    /// Height
    height: u32,
    /// Show dialog flag
    show: bool,
}

impl RegionDialog {
    /// Create a new region dialog
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            show: false,
        }
    }
    
    /// Show the region dialog
    pub fn show(&mut self, ctx: &Context) -> Option<(i32, i32, u32, u32)> {
        let mut result = None;
        
        if self.show {
            egui::Window::new("Select Region")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Region Parameters");
                    
                    ui.horizontal(|ui| {
                        ui.label("Position (X, Y):");
                        ui.add(egui::DragValue::new(&mut self.x).speed(1.0));
                        ui.add(egui::DragValue::new(&mut self.y).speed(1.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Size (Width, Height):");
                        ui.add(egui::DragValue::new(&mut self.width).speed(1.0));
                        ui.add(egui::DragValue::new(&mut self.height).speed(1.0));
                    });
                    
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        // Common size presets
                        if ui.button("720p").clicked() {
                            self.width = 1280;
                            self.height = 720;
                        }
                        if ui.button("1080p").clicked() {
                            self.width = 1920;
                            self.height = 1080;
                        }
                        if ui.button("1440p").clicked() {
                            self.width = 2560;
                            self.height = 1440;
                        }
                    });
                    
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show = false;
                        }
                        
                        if ui.button("OK").clicked() {
                            result = Some((self.x, self.y, self.width, self.height));
                            self.show = false;
                        }
                    });
                });
        }
        
        result
    }
    
    /// Show or hide the dialog
    pub fn set_show(&mut self, show: bool) {
        self.show = show;
    }
    
    /// Set the region parameters
    pub fn set_region(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
    }
    
    /// Check if the dialog is currently shown
    pub fn is_shown(&self) -> bool {
        self.show
    }
} 