use egui::{Ui, RichText, Color32};

/// Top bar component
pub struct TopBar {
    /// Is fullscreen mode active
    is_fullscreen: bool,
    /// Is capturing active
    is_capturing: bool,
}

impl TopBar {
    /// Create a new top bar
    pub fn new(is_fullscreen: bool, is_capturing: bool) -> Self {
        Self {
            is_fullscreen,
            is_capturing,
        }
    }
    
    /// Show the top bar
    pub fn show(&mut self, ui: &mut Ui) -> TopBarAction {
        let mut action = TopBarAction::None;
        
        ui.horizontal(|ui| {
            // App title
            ui.label(RichText::new("NU Scale").size(18.0).strong());
            
            // Status indicator
            let status_text = if self.is_capturing {
                RichText::new("● CAPTURING")
                    .color(Color32::from_rgb(25, 170, 88))
                    .strong()
            } else {
                RichText::new("■ IDLE")
                    .color(Color32::from_rgb(200, 200, 200))
            };
            ui.label(status_text);
            
            // Spacer to push buttons to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Fullscreen button
                let fullscreen_text = if self.is_fullscreen {
                    "Exit Fullscreen"
                } else {
                    "Fullscreen"
                };
                
                if ui.button(fullscreen_text).clicked() {
                    action = TopBarAction::ToggleFullscreen;
                }
                
                // Capture button
                let capture_text = if self.is_capturing {
                    "Stop Capture"
                } else {
                    "Start Capture"
                };
                
                if ui.button(capture_text).clicked() {
                    action = TopBarAction::ToggleCapture;
                }
            });
        });
        
        action
    }
    
    /// Set the fullscreen mode
    pub fn set_fullscreen(&mut self, is_fullscreen: bool) {
        self.is_fullscreen = is_fullscreen;
    }
    
    /// Set the capturing mode
    pub fn set_capturing(&mut self, is_capturing: bool) {
        self.is_capturing = is_capturing;
    }
}

/// Actions that can be triggered from the top bar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopBarAction {
    /// No action
    None,
    /// Toggle fullscreen mode
    ToggleFullscreen,
    /// Toggle capture mode
    ToggleCapture,
} 