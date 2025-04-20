pub mod app;
pub mod settings;
pub mod profile;
pub mod hotkeys;
pub mod main_window;

use anyhow::Result;
use cpp_core::{CppBox, Ptr};
use qt_core::{QBox, QPtr, SlotNoArgs, QObject, slot};
use qt_widgets::{QApplication, QMainWindow, QWidget};
use std::rc::Rc;

/// Initialize the Qt application
pub fn init_qt_app() -> Result<QBox<QApplication>> {
    let qt_app = unsafe {
        QApplication::new_1a(
            &mut Vec::<String>::new()
                .iter_mut()
                .map(|s| s.as_ptr())
                .collect::<Vec<_>>(),
        )
    };
    
    Ok(qt_app)
}

/// Run the Qt application event loop
pub fn run_qt_app(app: &QBox<QApplication>) -> Result<i32> {
    let result = unsafe { app.exec() };
    Ok(result)
}

/// Start the UI application
pub fn run_ui() -> Result<()> {
    // Create Qt application
    let args = vec!["Nu_scale".to_string()];
    let app = QApplication::init(args)?;
    
    // Create and show main window
    let main_window = main_window::MainWindow::new()?;
    main_window.show();
    
    // Run application event loop
    let result = unsafe { app.exec() };
    
    Ok(result)
} 