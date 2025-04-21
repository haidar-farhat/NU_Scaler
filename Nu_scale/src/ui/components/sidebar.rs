use egui::{Ui, RichText, Color32};
use crate::ui::tabs::TabState;

const SELECTED_COLOR: Color32 = Color32::from_rgb(0, 120, 215); // Blue accent

/// Sidebar component
pub struct Sidebar {
    /// Currently selected tab
    selected_tab: TabState,
}

impl Sidebar {
    /// Create a new sidebar
    pub fn new(selected_tab: TabState) -> Self {
        Self {
            selected_tab,
        }
    }
    
    /// Show the sidebar
    pub fn show(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.add_space(10.0);
            
            // Capture tab
            if self.tab_button(ui, "Capture", TabState::Capture) {
                self.selected_tab = TabState::Capture;
            }
            
            ui.add_space(5.0);
            
            // Settings tab
            if self.tab_button(ui, "Settings", TabState::Settings) {
                self.selected_tab = TabState::Settings;
            }
            
            ui.add_space(5.0);
            
            // Advanced tab
            if self.tab_button(ui, "Advanced", TabState::Advanced) {
                self.selected_tab = TabState::Advanced;
            }
            
            // Spacer to push bottom content down
            ui.add_space(ui.available_height() - 50.0);
            
            // Bottom help links
            ui.horizontal(|ui| {
                if ui.link("Help").clicked() {
                    // Open help documentation
                    // TODO: Implement help functionality
                }
                ui.label("|");
                if ui.link("About").clicked() {
                    // Show about dialog
                    // TODO: Implement about dialog
                }
            });
        });
    }
    
    /// Create a tab button with selection indication
    fn tab_button(&self, ui: &mut Ui, text: &str, tab: TabState) -> bool {
        let is_selected = self.selected_tab == tab;
        
        // Create button style based on selection state
        let mut button = egui::Button::new(
            RichText::new(text)
                .size(16.0)
                .color(if is_selected {
                    Color32::WHITE
                } else {
                    Color32::LIGHT_GRAY
                })
        );
        
        // Set button styling
        if is_selected {
            button = button.fill(SELECTED_COLOR);
        } else {
            button = button.fill(Color32::TRANSPARENT);
        }
        
        // Create the button with full width
        ui.add_sized([ui.available_width(), 30.0], button).clicked()
    }
    
    /// Get the current selected tab
    pub fn selected_tab(&self) -> TabState {
        self.selected_tab
    }
    
    /// Set the selected tab
    pub fn set_selected_tab(&mut self, tab: TabState) {
        self.selected_tab = tab;
    }
} 