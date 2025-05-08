use egui::{Ui, Color32, RichText};
use crate::ui::components::StatusMessageType;

/// Status bar component
pub struct StatusBar {
    /// Status message
    message: String,
    /// Status message type
    message_type: StatusMessageType,
}

impl StatusBar {
    /// Create a new status bar
    pub fn new(message: String, message_type: StatusMessageType) -> Self {
        Self {
            message,
            message_type,
        }
    }
    
    /// Set the status message
    pub fn set_message(&mut self, message: String, message_type: StatusMessageType) {
        self.message = message;
        self.message_type = message_type;
    }
    
    /// Show the status bar
    pub fn show(&self, ui: &mut Ui) {
        // Status message color based on type
        let color = match self.message_type {
            StatusMessageType::Info => Color32::from_rgb(255, 255, 255),
            StatusMessageType::Success => Color32::from_rgb(25, 170, 88),
            StatusMessageType::Warning => Color32::from_rgb(235, 165, 0),
            StatusMessageType::Error => Color32::from_rgb(209, 43, 43),
        };
        
        ui.horizontal(|ui| {
            ui.label(RichText::new(&self.message).color(color));
            
            // Push everything else to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Version info
                ui.label(format!("NU Scale v{}", env!("CARGO_PKG_VERSION")));
            });
        });
    }
    
    /// Get the current status message
    pub fn message(&self) -> &str {
        &self.message
    }
    
    /// Get the current status message type
    pub fn message_type(&self) -> StatusMessageType {
        self.message_type
    }
} 