use anyhow::{Result, anyhow};
use qt_core::{QBox, QObject, QPtr, SlotNoArgs, slot, QSignalEmitter, QTimer, SlotOfInt, SlotOfString};
use qt_widgets::{
    QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QComboBox,
    QTabWidget, QGroupBox, QCheckBox, QSlider, QSpinBox, QDoubleSpinBox,
    QListWidget, QListWidgetItem, QLineEdit, QDialog, QFormLayout, QDialogButtonBox,
    QAction, QMenu, QMenuBar, QStatusBar, QInputDialog
};
use qt_gui::{QIcon, QPixmap};
use std::rc::Rc;
use std::cell::RefCell;
use std::thread;

use super::profile::{Profile, CaptureSource, SystemPlatform, UpscalingTechnology};
use super::settings::AppSettings;
use super::hotkeys::{HotkeyManager, HotkeyAction};
use crate::capture;

/// Main application window
pub struct MainWindow {
    /// Qt main window
    window: QBox<QMainWindow>,
    /// Current profile
    profile: Rc<RefCell<Profile>>,
    /// Application settings
    settings: Rc<RefCell<AppSettings>>,
    /// Is capturing
    is_capturing: Rc<RefCell<bool>>,
}

impl MainWindow {
    /// Create a new main window
    pub fn new() -> Result<Self> {
        // Load settings
        let settings = AppSettings::load()?;
        let settings_rc = Rc::new(RefCell::new(settings));
        
        // Load profile
        let profile = settings_rc.borrow().get_current_profile()?;
        let profile_rc = Rc::new(RefCell::new(profile));
        
        // Create window
        let window = QMainWindow::new_0a();
        window.set_window_title(&qt_core::QString::from_std_str("Nu Scale"));
        window.resize_2a(800, 600);
        
        // Create main widget and layout
        let central_widget = QWidget::new_0a();
        window.set_central_widget(&central_widget);
        
        let main_layout = QVBoxLayout::new_1a(&central_widget);
        
        // Create tab widget
        let tabs = QTabWidget::new_0a();
        main_layout.add_widget(&tabs);
        
        // Create tabs
        let main_tab = Self::create_main_tab(&profile_rc)?;
        tabs.add_tab_2a(&main_tab, &qt_core::QString::from_std_str("Capture"));
        
        let settings_tab = Self::create_settings_tab(&profile_rc)?;
        tabs.add_tab_2a(&settings_tab, &qt_core::QString::from_std_str("Settings"));
        
        let advanced_tab = Self::create_advanced_tab(&profile_rc)?;
        tabs.add_tab_2a(&advanced_tab, &qt_core::QString::from_std_str("Advanced"));
        
        // Create status bar
        let status_bar = QStatusBar::new_0a();
        window.set_status_bar(&status_bar);
        status_bar.show_message(&qt_core::QString::from_std_str("Ready"));
        
        // Create menu bar
        let menu_bar = QMenuBar::new_0a();
        window.set_menu_bar(&menu_bar);
        
        // File menu
        let file_menu = QMenu::new_1a(&qt_core::QString::from_std_str("File"));
        menu_bar.add_menu(&file_menu);
        
        let new_profile_action = QAction::new_1a(&qt_core::QString::from_std_str("New Profile"));
        file_menu.add_action(&new_profile_action);
        
        let load_profile_action = QAction::new_1a(&qt_core::QString::from_std_str("Load Profile"));
        file_menu.add_action(&load_profile_action);
        
        let save_profile_action = QAction::new_1a(&qt_core::QString::from_std_str("Save Profile"));
        file_menu.add_action(&save_profile_action);
        
        file_menu.add_separator();
        
        let exit_action = QAction::new_1a(&qt_core::QString::from_std_str("Exit"));
        file_menu.add_action(&exit_action);
        
        // Help menu
        let help_menu = QMenu::new_1a(&qt_core::QString::from_std_str("Help"));
        menu_bar.add_menu(&help_menu);
        
        let about_action = QAction::new_1a(&qt_core::QString::from_std_str("About"));
        help_menu.add_action(&about_action);
        
        // Create instance
        let instance = Self {
            window,
            profile: profile_rc,
            settings: settings_rc,
            is_capturing: Rc::new(RefCell::new(false)),
        };
        
        // Setup hotkey handler
        instance.setup_hotkeys()?;
        
        Ok(instance)
    }
    
    /// Show the window
    pub fn show(&self) {
        self.window.show();
    }
    
    /// Create the main capture tab
    fn create_main_tab(profile: &Rc<RefCell<Profile>>) -> Result<QBox<QWidget>> {
        let tab = QWidget::new_0a();
        let layout = QVBoxLayout::new_1a(&tab);
        
        // Profile section
        let profile_group = QGroupBox::new_1a(&qt_core::QString::from_std_str("Profile"));
        layout.add_widget(&profile_group);
        
        let profile_layout = QHBoxLayout::new_1a(&profile_group);
        
        let profile_label = QLabel::new_1a(&qt_core::QString::from_std_str("Current Profile:"));
        profile_layout.add_widget(&profile_label);
        
        let profile_combo = QComboBox::new_0a();
        profile_layout.add_widget(&profile_combo);
        
        // Add all available profiles
        if let Ok(profiles) = Profile::list_profiles() {
            for name in profiles {
                profile_combo.add_item_q_string(&qt_core::QString::from_std_str(&name));
            }
        }
        
        // Set current profile
        let current_profile_name = profile.borrow().name.clone();
        profile_combo.set_current_text(&qt_core::QString::from_std_str(&current_profile_name));
        
        let new_profile_btn = QPushButton::new_1a(&qt_core::QString::from_std_str("New"));
        profile_layout.add_widget(&new_profile_btn);
        
        // Capture source section
        let source_group = QGroupBox::new_1a(&qt_core::QString::from_std_str("Capture Source"));
        layout.add_widget(&source_group);
        
        let source_layout = QVBoxLayout::new_1a(&source_group);
        
        // Fullscreen option
        let fullscreen_radio = QCheckBox::new_1a(&qt_core::QString::from_std_str("Fullscreen"));
        source_layout.add_widget(&fullscreen_radio);
        
        // Window option
        let window_layout = QHBoxLayout::new_0a();
        source_layout.add_layout_1a(&window_layout);
        
        let window_radio = QCheckBox::new_1a(&qt_core::QString::from_std_str("Window:"));
        window_layout.add_widget(&window_radio);
        
        let window_combo = QComboBox::new_0a();
        window_layout.add_widget(&window_combo);
        
        // Add all available windows
        if let Ok(windows) = capture::common::list_available_windows() {
            for window in windows {
                window_combo.add_item_q_string(&qt_core::QString::from_std_str(&window.title));
            }
        }
        
        let refresh_btn = QPushButton::new_1a(&qt_core::QString::from_std_str("Refresh"));
        window_layout.add_widget(&refresh_btn);
        
        // Region option
        let region_layout = QHBoxLayout::new_0a();
        source_layout.add_layout_1a(&region_layout);
        
        let region_radio = QCheckBox::new_1a(&qt_core::QString::from_std_str("Region:"));
        region_layout.add_widget(&region_radio);
        
        let region_btn = QPushButton::new_1a(&qt_core::QString::from_std_str("Select Region"));
        region_layout.add_widget(&region_btn);
        
        // Platform section
        let platform_group = QGroupBox::new_1a(&qt_core::QString::from_std_str("System Platform"));
        layout.add_widget(&platform_group);
        
        let platform_layout = QHBoxLayout::new_1a(&platform_group);
        
        let platform_label = QLabel::new_1a(&qt_core::QString::from_std_str("Platform:"));
        platform_layout.add_widget(&platform_label);
        
        let platform_combo = QComboBox::new_0a();
        platform_layout.add_widget(&platform_combo);
        
        // Add platform options
        platform_combo.add_item_q_string(&qt_core::QString::from_std_str("Auto"));
        platform_combo.add_item_q_string(&qt_core::QString::from_std_str("Windows"));
        platform_combo.add_item_q_string(&qt_core::QString::from_std_str("X11"));
        platform_combo.add_item_q_string(&qt_core::QString::from_std_str("Wayland"));
        
        // Set current platform
        match profile.borrow().platform {
            SystemPlatform::Auto => platform_combo.set_current_index(0),
            SystemPlatform::Windows => platform_combo.set_current_index(1),
            SystemPlatform::X11 => platform_combo.set_current_index(2),
            SystemPlatform::Wayland => platform_combo.set_current_index(3),
        }
        
        // Capture controls
        let controls_group = QGroupBox::new_1a(&qt_core::QString::from_std_str("Capture Controls"));
        layout.add_widget(&controls_group);
        
        let controls_layout = QHBoxLayout::new_1a(&controls_group);
        
        let start_btn = QPushButton::new_1a(&qt_core::QString::from_std_str("Start Capture"));
        controls_layout.add_widget(&start_btn);
        
        let stop_btn = QPushButton::new_1a(&qt_core::QString::from_std_str("Stop Capture"));
        controls_layout.add_widget(&stop_btn);
        
        let single_frame_btn = QPushButton::new_1a(&qt_core::QString::from_std_str("Capture Single Frame"));
        controls_layout.add_widget(&single_frame_btn);
        
        // Status
        let status_label = QLabel::new_1a(&qt_core::QString::from_std_str("Status: Not capturing"));
        layout.add_widget(&status_label);
        
        Ok(tab)
    }
    
    /// Create the settings tab
    fn create_settings_tab(profile: &Rc<RefCell<Profile>>) -> Result<QBox<QWidget>> {
        let tab = QWidget::new_0a();
        let layout = QVBoxLayout::new_1a(&tab);
        
        // Upscaling settings
        let upscaling_group = QGroupBox::new_1a(&qt_core::QString::from_std_str("Upscaling Settings"));
        layout.add_widget(&upscaling_group);
        
        let upscaling_layout = QVBoxLayout::new_1a(&upscaling_group);
        
        // Scale factor
        let scale_layout = QHBoxLayout::new_0a();
        upscaling_layout.add_layout_1a(&scale_layout);
        
        let scale_label = QLabel::new_1a(&qt_core::QString::from_std_str("Scale Factor:"));
        scale_layout.add_widget(&scale_label);
        
        let scale_slider = QSlider::new_1a(qt_core::Orientation::Horizontal);
        scale_slider.set_minimum(100);
        scale_slider.set_maximum(400);
        scale_slider.set_value((profile.borrow().scale_factor * 100.0) as i32);
        scale_slider.set_tick_interval(50);
        scale_slider.set_tick_position(qt_widgets::QSlider::TickPosition::TicksBelow);
        scale_layout.add_widget(&scale_slider);
        
        let scale_spin = QDoubleSpinBox::new_0a();
        scale_spin.set_minimum(1.0);
        scale_spin.set_maximum(4.0);
        scale_spin.set_single_step(0.1);
        scale_spin.set_value(profile.borrow().scale_factor as f64);
        scale_layout.add_widget(&scale_spin);
        
        // Upscaling technology
        let tech_layout = QHBoxLayout::new_0a();
        upscaling_layout.add_layout_1a(&tech_layout);
        
        let tech_label = QLabel::new_1a(&qt_core::QString::from_std_str("Upscaling Technology:"));
        tech_layout.add_widget(&tech_label);
        
        let tech_combo = QComboBox::new_0a();
        tech_layout.add_widget(&tech_combo);
        
        // Add technology options
        tech_combo.add_item_q_string(&qt_core::QString::from_std_str("None"));
        tech_combo.add_item_q_string(&qt_core::QString::from_std_str("FSR (AMD FidelityFX)"));
        tech_combo.add_item_q_string(&qt_core::QString::from_std_str("NIS (NVIDIA Image Scaling)"));
        tech_combo.add_item_q_string(&qt_core::QString::from_std_str("Custom"));
        
        // Set current technology
        match profile.borrow().upscaling_tech {
            UpscalingTechnology::None => tech_combo.set_current_index(0),
            UpscalingTechnology::FSR => tech_combo.set_current_index(1),
            UpscalingTechnology::NIS => tech_combo.set_current_index(2),
            UpscalingTechnology::Custom => tech_combo.set_current_index(3),
        }
        
        // Hotkey settings
        let hotkey_group = QGroupBox::new_1a(&qt_core::QString::from_std_str("Hotkey Settings"));
        layout.add_widget(&hotkey_group);
        
        let hotkey_layout = QFormLayout::new_1a(&hotkey_group);
        
        // Capture toggle hotkey
        let toggle_edit = QLineEdit::new_0a();
        toggle_edit.set_text(&qt_core::QString::from_std_str(&profile.borrow().hotkey));
        hotkey_layout.add_row_q_string_q_widget(
            &qt_core::QString::from_std_str("Start/Stop Capture:"),
            &toggle_edit,
        );
        
        // Single frame hotkey
        let single_frame_edit = QLineEdit::new_0a();
        single_frame_edit.set_text(&qt_core::QString::from_std_str("Ctrl+Alt+S"));
        hotkey_layout.add_row_q_string_q_widget(
            &qt_core::QString::from_std_str("Capture Single Frame:"),
            &single_frame_edit,
        );
        
        // Overlay toggle hotkey
        let overlay_edit = QLineEdit::new_0a();
        overlay_edit.set_text(&qt_core::QString::from_std_str("Ctrl+Alt+O"));
        hotkey_layout.add_row_q_string_q_widget(
            &qt_core::QString::from_std_str("Toggle Overlay:"),
            &overlay_edit,
        );
        
        // FPS settings
        let fps_group = QGroupBox::new_1a(&qt_core::QString::from_std_str("Capture FPS"));
        layout.add_widget(&fps_group);
        
        let fps_layout = QHBoxLayout::new_1a(&fps_group);
        
        let fps_label = QLabel::new_1a(&qt_core::QString::from_std_str("Target FPS:"));
        fps_layout.add_widget(&fps_label);
        
        let fps_spin = QSpinBox::new_0a();
        fps_spin.set_minimum(1);
        fps_spin.set_maximum(240);
        fps_spin.set_value(profile.borrow().fps as i32);
        fps_layout.add_widget(&fps_spin);
        
        // Overlay settings
        let overlay_group = QGroupBox::new_1a(&qt_core::QString::from_std_str("Overlay"));
        layout.add_widget(&overlay_group);
        
        let overlay_layout = QVBoxLayout::new_1a(&overlay_group);
        
        let enable_overlay = QCheckBox::new_1a(&qt_core::QString::from_std_str("Enable Overlay"));
        enable_overlay.set_checked(profile.borrow().enable_overlay);
        overlay_layout.add_widget(&enable_overlay);
        
        Ok(tab)
    }
    
    /// Create the advanced tab
    fn create_advanced_tab(profile: &Rc<RefCell<Profile>>) -> Result<QBox<QWidget>> {
        let tab = QWidget::new_0a();
        let layout = QVBoxLayout::new_1a(&tab);
        
        // Advanced settings
        let advanced_group = QGroupBox::new_1a(&qt_core::QString::from_std_str("Advanced Settings"));
        layout.add_widget(&advanced_group);
        
        let advanced_layout = QVBoxLayout::new_1a(&advanced_group);
        
        // Placeholder for advanced settings
        let label = QLabel::new_1a(&qt_core::QString::from_std_str("Advanced settings will be available in future versions."));
        advanced_layout.add_widget(&label);
        
        Ok(tab)
    }
    
    /// Setup hotkey handler
    fn setup_hotkeys(&self) -> Result<()> {
        let profile = self.profile.clone();
        let is_capturing = self.is_capturing.clone();
        
        // Start hotkey handler in a separate thread
        thread::spawn(move || {
            if let Ok(mut hotkey_manager) = HotkeyManager::new() {
                // Register the capture toggle hotkey
                let hotkey_str = profile.borrow().hotkey.clone();
                let _ = hotkey_manager.register_hotkey(&hotkey_str, HotkeyAction::ToggleCapture);
                
                // Start listening for hotkeys
                if let Ok(handle) = hotkey_manager.start_listening() {
                    // Wait for the thread to finish (this will never happen in practice)
                    let _ = handle.join();
                }
            }
        });
        
        Ok(())
    }
} 