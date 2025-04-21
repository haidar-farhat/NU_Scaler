// UI component modules
use egui::Ui;

/// Status message type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusMessageType {
    Info,
    Success,
    Warning,
    Error,
}

// Stub components for compilation
pub struct StatusBar {}
impl StatusBar {
    pub fn new(_message: String, _message_type: StatusMessageType) -> Self {
        Self {}
    }
    pub fn show(&self, _ui: &mut Ui) {}
    pub fn set_message(&mut self, _message: String, _message_type: StatusMessageType) {}
}

pub struct TopBar {}
impl TopBar {
    pub fn new(_is_fullscreen: bool, _is_capturing: bool) -> Self {
        Self {}
    }
    pub fn show(&mut self, _ui: &mut Ui) -> TopBarAction {
        TopBarAction::None
    }
    pub fn set_fullscreen(&mut self, _is_fullscreen: bool) {}
    pub fn set_capturing(&mut self, _is_capturing: bool) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopBarAction {
    None,
    ToggleFullscreen,
    ToggleCapture,
}

pub struct Sidebar {}
impl Sidebar {
    pub fn new(_selected_tab: crate::ui::tabs::TabState) -> Self {
        Self {}
    }
    pub fn show(&mut self, _ui: &mut Ui) {}
    pub fn selected_tab(&self) -> crate::ui::tabs::TabState {
        crate::ui::tabs::TabState::Capture
    }
}

pub struct RegionDialog {}
impl RegionDialog {
    pub fn new(_x: i32, _y: i32, _width: u32, _height: u32) -> Self {
        Self {}
    }
    pub fn show(&mut self, _ctx: &egui::Context) -> Option<(i32, i32, u32, u32)> {
        None
    }
    pub fn set_show(&mut self, _show: bool) {}
}

/// Create a card-style frame for UI elements
pub fn card_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(32, 34, 37))
        .rounding(egui::Rounding::same(5.0))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(54, 57, 63)))
        .inner_margin(egui::style::Margin::same(10.0))
        .outer_margin(egui::style::Margin::same(5.0))
} 