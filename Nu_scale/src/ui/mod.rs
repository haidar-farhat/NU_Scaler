pub mod profile;
pub mod settings;
pub mod hotkeys;
pub mod egui_ui;

use anyhow::Result;
use profile::Profile;
use settings::AppSettings;

/// Start the UI application
pub fn run_ui() -> Result<()> {
    // Create and run the egui application
    egui_ui::run_app()
} 