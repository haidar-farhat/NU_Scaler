// Import submodules
pub mod egui_ui;
pub mod profile;
pub mod settings;
pub mod hotkeys;

use anyhow::Result;

/// Run the UI
pub fn run_ui() -> Result<()> {
    egui_ui::run_app()
} 