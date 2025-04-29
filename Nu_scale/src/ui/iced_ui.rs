use anyhow::{Result, anyhow};
use iced::{
    widget::{button, checkbox, column, container, horizontal_space, pick_list, 
             row, scrollable, slider, text, text_input, vertical_space},
    application, executor, theme, Application, Command, Element, 
    Length, Settings, Theme, Point, Rectangle, Size,
};
use std::{
    path::{PathBuf, Path},
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    thread,
    time::{Duration, Instant},
};

use crate::capture::{CaptureTarget, ScreenCapture};
use crate::capture::common::{FrameBuffer};
use crate::upscale::{
    create_upscaler, Upscaler, UpscalingQuality, UpscalingTechnology,
    common::UpscalingAlgorithm,
};

// Import profile and settings modules
use super::profile::{Profile, UpscalingTechnology as ProfileUpscalingTechnology, UpscalingQuality as ProfileUpscalingQuality};
use super::settings::AppSettings;

// Define status message types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

// Define the messages for our application
#[derive(Debug, Clone)]
pub enum Message {
    // Profile actions
    ProfileSelected(String),
    SaveProfile,
    NewProfile,
    DeleteProfile,
    
    // Capture source actions
    SelectCaptureSource(CaptureSource),
    WindowSelected(String),
    RefreshWindows,
    SelectRegion,
    RegionUpdated(i32, i32, u32, u32),
    
    // Capture actions
    ToggleCapture,
    CaptureFrame,
    
    // Upscaling actions
    StartScaling,
    FullscreenMode,
    TechSelected(ProfileUpscalingTechnology),
    QualitySelected(ProfileUpscalingQuality),
    AlgorithmSelected(usize),
    ScaleFactorChanged(f32),
    FpsChanged(f32),
    
    // Settings actions
    ToggleAutoSaveFrames(bool),
    ToggleShowFpsCounter(bool),
    ToggleShowNotifications(bool),
    ThemeSelected(String),
    SaveSettings,
    
    // Tab navigation
    SelectTab(TabState),
    
    // Status updates
    StatusUpdate(String, MessageType),
    
    // System events
    Tick(Instant),
    Exit,
}

// Tab state enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabState {
    Capture,
    Settings,
    Advanced,
}

// Capture source enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureSource {
    Fullscreen,
    Window,
    Region,
}

// Application state structure
pub struct AppState {
    // Core application state
    profile: Profile,
    settings: AppSettings,
    is_capturing: bool,
    is_upscaling: bool,
    
    // Window properties
    available_windows: Vec<String>,
    selected_window_index: usize,
    capture_source: CaptureSource,
    region: (i32, i32, u32, u32),
    
    // Upscaling settings
    scale_factor: f32,
    fps: f32,
    
    // Capture and processing
    frame_buffer: Arc<FrameBuffer>,
    stop_signal: Arc<AtomicBool>,
    capture_status: Arc<Mutex<String>>,
    
    // Upscaling
    upscaling_buffer: Option<Arc<FrameBuffer>>,
    upscaling_stop_signal: Option<Arc<AtomicBool>>,
    upscaler: Option<Box<dyn Upscaler>>,
    
    // UI State
    current_tab: TabState,
    status_message: String,
    status_message_type: MessageType,
    
    // Available profile names
    available_profiles: Vec<String>,
}

impl AppState {
    // Create a new application state 
    pub fn new() -> Result<Self> {
        println!("AppState::new() called");
        
        // Load settings and profile
        let settings = AppSettings::load().unwrap_or_default();
        let profile_path = format!("{}.json", settings.current_profile);
        let profile = Profile::load(&profile_path).unwrap_or_default();
        
        // Determine capture source from profile
        let capture_source = match profile.capture_source {
            0 => CaptureSource::Fullscreen,
            1 => CaptureSource::Window,
            2 => CaptureSource::Region,
            _ => CaptureSource::Fullscreen,
        };
        
        // Get region from profile
        let region = (
            profile.region_x,
            profile.region_y,
            profile.region_width,
            profile.region_height
        );
        
        // Get available windows
        let available_windows = crate::capture::common::list_available_windows()
            .map(|windows| windows.iter().map(|w| w.title.clone()).collect())
            .unwrap_or_default();
        
        // Create frame buffer and signals
        let frame_buffer = Arc::new(FrameBuffer::new(10));
        let stop_signal = Arc::new(AtomicBool::new(false));
        let capture_status = Arc::new(Mutex::new("Idle".to_string()));
        
        // Get available profiles
        let available_profiles = super::profile::Profile::list_profiles().unwrap_or_default();
        
        Ok(Self {
            profile,
            settings,
            is_capturing: false,
            is_upscaling: false,
            available_windows,
            selected_window_index: 0,
            capture_source,
            region,
            scale_factor: profile.scale_factor,
            fps: profile.fps,
            frame_buffer,
            stop_signal,
            capture_status,
            upscaling_buffer: None,
            upscaling_stop_signal: None,
            upscaler: None,
            current_tab: TabState::Capture,
            status_message: "Ready".to_string(),
            status_message_type: MessageType::Info,
            available_profiles,
        })
    }
}

impl Application for AppState {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        match Self::new() {
            Ok(app) => (app, Command::none()),
            Err(e) => {
                eprintln!("Failed to create app state: {}", e);
                // Create a default app state and set an error message
                let mut default_app = AppState {
                    profile: Profile::default(),
                    settings: AppSettings::default(),
                    is_capturing: false,
                    is_upscaling: false,
                    available_windows: Vec::new(),
                    selected_window_index: 0,
                    capture_source: CaptureSource::Fullscreen,
                    region: (0, 0, 1920, 1080),
                    scale_factor: 2.0,
                    fps: 60.0,
                    frame_buffer: Arc::new(FrameBuffer::new(10)),
                    stop_signal: Arc::new(AtomicBool::new(false)),
                    capture_status: Arc::new(Mutex::new("Error".to_string())),
                    upscaling_buffer: None,
                    upscaling_stop_signal: None,
                    upscaler: None,
                    current_tab: TabState::Capture,
                    status_message: format!("Error: {}", e),
                    status_message_type: MessageType::Error,
                    available_profiles: Vec::new(),
                };
                
                (default_app, Command::none())
            }
        }
    }

    fn title(&self) -> String {
        String::from("NU Scaler")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SelectTab(tab) => {
                self.current_tab = tab;
            }
            Message::ToggleCapture => {
                self.is_capturing = !self.is_capturing;
                if self.is_capturing {
                    self.status_message = "Capture started".to_string();
                    self.status_message_type = MessageType::Success;
                } else {
                    self.status_message = "Capture stopped".to_string();
                    self.status_message_type = MessageType::Info;
                }
            }
            Message::CaptureFrame => {
                self.status_message = "Frame captured".to_string();
                self.status_message_type = MessageType::Success;
            }
            Message::StartScaling => {
                self.status_message = "Scaling started".to_string();
                self.status_message_type = MessageType::Success;
            }
            Message::ProfileSelected(name) => {
                // Load the selected profile
                let profile_path = format!("{}.json", name);
                match Profile::load(&profile_path) {
                    Ok(profile) => {
                        self.profile = profile;
                        self.settings.current_profile = name;
                        let _ = self.settings.save();
                        
                        // Update UI state from profile
                        self.capture_source = match self.profile.capture_source {
                            0 => CaptureSource::Fullscreen,
                            1 => CaptureSource::Window,
                            2 => CaptureSource::Region,
                            _ => CaptureSource::Fullscreen,
                        };
                        self.region = (
                            self.profile.region_x,
                            self.profile.region_y,
                            self.profile.region_width,
                            self.profile.region_height
                        );
                        self.scale_factor = self.profile.scale_factor;
                        self.fps = self.profile.fps;
                        
                        self.status_message = format!("Loaded profile: {}", name);
                        self.status_message_type = MessageType::Success;
                    }
                    Err(e) => {
                        self.status_message = format!("Error loading profile: {}", e);
                        self.status_message_type = MessageType::Error;
                    }
                }
            }
            Message::SaveProfile => {
                if let Err(e) = self.profile.save(None) {
                    self.status_message = format!("Error saving profile: {}", e);
                    self.status_message_type = MessageType::Error;
                } else {
                    self.status_message = "Profile saved".to_string();
                    self.status_message_type = MessageType::Success;
                    
                    // Refresh profile list
                    if let Ok(profiles) = super::profile::Profile::list_profiles() {
                        self.available_profiles = profiles;
                    }
                }
            }
            Message::NewProfile => {
                let new_name = format!("Profile_{}", self.available_profiles.len() + 1);
                self.profile = Profile::new(&new_name);
                if let Err(e) = self.profile.save(None) {
                    self.status_message = format!("Error creating profile: {}", e);
                    self.status_message_type = MessageType::Error;
                } else {
                    self.status_message = "New profile created".to_string();
                    self.status_message_type = MessageType::Success;
                    
                    // Refresh profile list
                    if let Ok(profiles) = super::profile::Profile::list_profiles() {
                        self.available_profiles = profiles;
                    }
                }
            }
            Message::DeleteProfile => {
                let profile_to_delete = self.profile.name.clone();
                if profile_to_delete != "Default" && self.available_profiles.len() > 1 {
                    let profile_path = format!("{}.json", profile_to_delete);
                    if std::fs::remove_file(profile_path).is_ok() {
                        self.status_message = "Profile deleted".to_string();
                        self.status_message_type = MessageType::Success;
                        
                        // Refresh profile list
                        if let Ok(profiles) = super::profile::Profile::list_profiles() {
                            self.available_profiles = profiles;
                            
                            // Load the first available profile
                            if let Some(next_profile) = self.available_profiles.first() {
                                let next_path = format!("{}.json", next_profile);
                                if let Ok(loaded_profile) = Profile::load(&next_path) {
                                    self.profile = loaded_profile;
                                    self.settings.current_profile = next_profile.clone();
                                    let _ = self.settings.save();
                                }
                            }
                        }
                    } else {
                        self.status_message = "Error deleting profile".to_string();
                        self.status_message_type = MessageType::Error;
                    }
                } else {
                    self.status_message = "Cannot delete the last or default profile".to_string();
                    self.status_message_type = MessageType::Warning;
                }
            }
            Message::SelectCaptureSource(source) => {
                self.capture_source = source;
                self.profile.capture_source = match source {
                    CaptureSource::Fullscreen => 0,
                    CaptureSource::Window => 1,
                    CaptureSource::Region => 2,
                };
            }
            Message::WindowSelected(window_name) => {
                // Find the index of the selected window
                if let Some(index) = self.available_windows.iter().position(|w| w == &window_name) {
                    self.selected_window_index = index;
                    self.profile.window_title = window_name;
                }
            }
            Message::RefreshWindows => {
                if let Ok(windows) = crate::capture::common::list_available_windows() {
                    self.available_windows = windows.iter().map(|w| w.title.clone()).collect();
                    
                    // Reset selection if out of bounds
                    if self.selected_window_index >= self.available_windows.len() {
                        self.selected_window_index = 0;
                        if self.capture_source == CaptureSource::Window && !self.available_windows.is_empty() {
                            self.profile.window_title = self.available_windows[0].clone();
                        }
                    }
                }
            }
            Message::SelectRegion => {
                // This would open a dialog to select a region
                // For now we just log that it was triggered
                self.status_message = "Select region dialog triggered".to_string();
                self.status_message_type = MessageType::Info;
            }
            Message::RegionUpdated(x, y, width, height) => {
                self.region = (x, y, width, height);
                self.profile.region_x = x;
                self.profile.region_y = y;
                self.profile.region_width = width;
                self.profile.region_height = height;
            }
            Message::TechSelected(tech) => {
                self.profile.upscaling_tech = match tech {
                    ProfileUpscalingTechnology::None => 0,
                    ProfileUpscalingTechnology::FSR => 1,
                    ProfileUpscalingTechnology::DLSS => 2,
                    ProfileUpscalingTechnology::Custom => 3,
                    ProfileUpscalingTechnology::Fallback => 4,
                };
            }
            Message::QualitySelected(quality) => {
                self.profile.upscaling_quality = match quality {
                    ProfileUpscalingQuality::Ultra => 0,
                    ProfileUpscalingQuality::Quality => 1,
                    ProfileUpscalingQuality::Balanced => 2,
                    ProfileUpscalingQuality::Performance => 3,
                };
            }
            Message::AlgorithmSelected(algorithm) => {
                self.profile.upscaling_algorithm = algorithm;
            }
            Message::ScaleFactorChanged(factor) => {
                self.scale_factor = factor;
                self.profile.scale_factor = factor;
            }
            Message::FpsChanged(fps) => {
                self.fps = fps;
                self.profile.fps = fps;
            }
            Message::ToggleAutoSaveFrames(enabled) => {
                self.settings.auto_save_frames = enabled;
            }
            Message::ToggleShowFpsCounter(enabled) => {
                self.settings.show_fps_counter = enabled;
            }
            Message::ToggleShowNotifications(enabled) => {
                self.settings.show_notifications = enabled;
            }
            Message::ThemeSelected(theme) => {
                self.settings.theme = theme;
            }
            Message::SaveSettings => {
                if let Err(e) = self.settings.save() {
                    self.status_message = format!("Error saving settings: {}", e);
                    self.status_message_type = MessageType::Error;
                } else {
                    self.status_message = "Settings saved".to_string();
                    self.status_message_type = MessageType::Success;
                }
            }
            Message::StatusUpdate(message, message_type) => {
                self.status_message = message;
                self.status_message_type = message_type;
            }
            Message::Tick(_now) => {
                // Update time-sensitive information
            }
            Message::Exit => {
                return Command::none();
            }
            // Implement other message handlers
            _ => {}
        }
        
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let content = match self.current_tab {
            TabState::Capture => self.view_capture_tab(),
            TabState::Settings => self.view_settings_tab(),
            TabState::Advanced => self.view_advanced_tab(),
        };
        
        // Main layout with header, sidebar, content, and status bar
        let layout = column![
            self.view_header(),
            row![
                self.view_sidebar(),
                content,
            ]
            .height(Length::Fill),
            self.view_status_bar(),
        ]
        .padding(10)
        .spacing(10);
        
        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl AppState {
    // View the header
    fn view_header(&self) -> Element<Message> {
        let title = text("NU Scale")
            .size(28)
            .style(theme::Text::Color(iced::Color::from_rgb(0.0, 0.47, 0.84)));
            
        let save_profile_button = button("💾 Save Profile")
            .on_press(Message::SaveProfile);
            
        let new_profile_button = button("+ New Profile")
            .on_press(Message::NewProfile);
            
        let capture_button = if self.is_capturing {
            button("⏹ Stop Capture")
                .on_press(Message::ToggleCapture)
                .style(theme::Button::Destructive)
        } else {
            button("▶ Start Capture")
                .on_press(Message::ToggleCapture)
                .style(theme::Button::Primary)
        };
        
        let capture_frame_button = button("📷 Capture Frame")
            .on_press(Message::CaptureFrame);
            
        let fullscreen_button = button("🖥️ Fullscreen Mode")
            .on_press(Message::FullscreenMode);
            
        let scale_button = button("🔍 Scale")
            .on_press(Message::StartScaling);
            
        row![
            title,
            horizontal_space(10),
            save_profile_button,
            new_profile_button,
            horizontal_space(Length::Fill),
            capture_button,
            capture_frame_button,
            fullscreen_button,
            scale_button,
        ]
        .spacing(10)
        .align_items(iced::Alignment::Center)
        .into()
    }
    
    // View the sidebar
    fn view_sidebar(&self) -> Element<Message> {
        let capture_tab = button(
            row![
                text("📷 Capture").size(16)
            ]
            .width(Length::Fill)
        )
        .width(Length::Fill)
        .style(if self.current_tab == TabState::Capture {
            theme::Button::Primary
        } else {
            theme::Button::Text
        })
        .on_press(Message::SelectTab(TabState::Capture));
        
        let settings_tab = button(
            row![
                text("⚙️ Settings").size(16)
            ]
            .width(Length::Fill)
        )
        .width(Length::Fill)
        .style(if self.current_tab == TabState::Settings {
            theme::Button::Primary
        } else {
            theme::Button::Text
        })
        .on_press(Message::SelectTab(TabState::Settings));
        
        let advanced_tab = button(
            row![
                text("🔧 Advanced").size(16)
            ]
            .width(Length::Fill)
        )
        .width(Length::Fill)
        .style(if self.current_tab == TabState::Advanced {
            theme::Button::Primary
        } else {
            theme::Button::Text
        })
        .on_press(Message::SelectTab(TabState::Advanced));
        
        let version_text = column![
            text("NU Scale").size(14),
            text("v1.0.0").size(12),
        ];
        
        column![
            text("Navigation").size(18),
            capture_tab,
            settings_tab,
            advanced_tab,
            vertical_space(Length::Fill),
            version_text,
        ]
        .spacing(10)
        .width(200)
        .height(Length::Fill)
        .into()
    }
    
    // View the capture tab
    fn view_capture_tab(&self) -> Element<Message> {
        let title = text("Capture Settings").size(24);
        
        // Profile selection panel
        let profile_selection = container(
            column![
                text("Profile Selection").size(18),
                row![
                    text("Current Profile:"),
                    pick_list(
                        self.available_profiles.clone(),
                        Some(self.profile.name.clone()),
                        Message::ProfileSelected,
                    )
                    .width(200),
                ],
                row![
                    button("Save").on_press(Message::SaveProfile),
                    button("📋 New").on_press(Message::NewProfile),
                    button("❌ Delete").on_press(Message::DeleteProfile),
                ],
            ]
            .spacing(10)
            .padding(10)
        )
        .style(theme::Container::Box)
        .width(Length::Fill);
        
        // Capture source panel
        let capture_source = container(
            column![
                text("Capture Source").size(18),
                // Fullscreen radio button
                button(
                    row![
                        checkbox(
                            self.capture_source == CaptureSource::Fullscreen,
                            "🖥️ Fullscreen",
                            |_| Message::SelectCaptureSource(CaptureSource::Fullscreen),
                        ),
                    ]
                )
                .style(theme::Button::Text),
                
                // Window radio button
                row![
                    checkbox(
                        self.capture_source == CaptureSource::Window,
                        "🪟 Window",
                        |_| Message::SelectCaptureSource(CaptureSource::Window),
                    ),
                    
                    // Only show window dropdown if Window is selected
                    if self.capture_source == CaptureSource::Window {
                        row![
                            pick_list(
                                self.available_windows.clone(),
                                self.available_windows.get(self.selected_window_index).cloned(),
                                Message::WindowSelected,
                            )
                            .width(240),
                            button("🔄 Refresh").on_press(Message::RefreshWindows),
                        ]
                        .spacing(8)
                        .into()
                    } else {
                        row![].into()
                    }
                ],
                
                // Region radio button
                row![
                    checkbox(
                        self.capture_source == CaptureSource::Region,
                        "📏 Region",
                        |_| Message::SelectCaptureSource(CaptureSource::Region),
                    ),
                    
                    // Only show region selector if Region is selected
                    if self.capture_source == CaptureSource::Region {
                        row![
                            button("Select Region").on_press(Message::SelectRegion),
                            text(format!(
                                "({}, {}, {}x{})",
                                self.region.0, self.region.1, self.region.2, self.region.3
                            )),
                        ]
                        .spacing(8)
                        .into()
                    } else {
                        row![].into()
                    }
                ],
            ]
            .spacing(10)
            .padding(10)
        )
        .style(theme::Container::Box)
        .width(Length::Fill);
        
        column![
            title,
            vertical_space(10),
            profile_selection,
            vertical_space(10),
            capture_source,
        ]
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    // View the settings tab
    fn view_settings_tab(&self) -> Element<Message> {
        let title = text("Settings").size(24);
        
        // Upscaling settings panel
        let upscaling_settings = container(
            column![
                text("Upscaling Settings").size(18),
                
                // Scale factor slider
                row![
                    text("Scale Factor:"),
                    slider(1.0..=4.0, self.scale_factor, Message::ScaleFactorChanged)
                        .step(0.1)
                        .width(Length::Fill),
                    text(format!("{:.1}×", self.scale_factor)),
                ],
                
                // Upscaling technology
                row![
                    text("Upscaling Technology:"),
                    pick_list(
                        vec![
                            ("Auto", ProfileUpscalingTechnology::None),
                            ("AMD FSR", ProfileUpscalingTechnology::FSR),
                            ("NVIDIA DLSS", ProfileUpscalingTechnology::DLSS),
                            ("GPU (Vulkan)", ProfileUpscalingTechnology::Custom),
                            ("Fallback/Basic", ProfileUpscalingTechnology::Fallback),
                        ],
                        Some((
                            match self.profile.upscaling_tech {
                                0 => "Auto",
                                1 => "AMD FSR",
                                2 => "NVIDIA DLSS",
                                3 => "GPU (Vulkan)",
                                4 => "Fallback/Basic",
                                _ => "Auto",
                            }, 
                            match self.profile.upscaling_tech {
                                0 => ProfileUpscalingTechnology::None,
                                1 => ProfileUpscalingTechnology::FSR,
                                2 => ProfileUpscalingTechnology::DLSS,
                                3 => ProfileUpscalingTechnology::Custom,
                                4 => ProfileUpscalingTechnology::Fallback,
                                _ => ProfileUpscalingTechnology::None,
                            }
                        )),
                        |(_label, tech)| Message::TechSelected(tech),
                    ),
                ],
                
                // Upscaling quality
                row![
                    text("Upscaling Quality:"),
                    pick_list(
                        vec![
                            ("Ultra Quality", ProfileUpscalingQuality::Ultra),
                            ("Quality", ProfileUpscalingQuality::Quality),
                            ("Balanced", ProfileUpscalingQuality::Balanced),
                            ("Performance", ProfileUpscalingQuality::Performance),
                        ],
                        Some((
                            match self.profile.upscaling_quality {
                                0 => "Ultra Quality",
                                1 => "Quality",
                                2 => "Balanced",
                                3 => "Performance",
                                _ => "Quality",
                            },
                            match self.profile.upscaling_quality {
                                0 => ProfileUpscalingQuality::Ultra,
                                1 => ProfileUpscalingQuality::Quality,
                                2 => ProfileUpscalingQuality::Balanced,
                                3 => ProfileUpscalingQuality::Performance,
                                _ => ProfileUpscalingQuality::Quality,
                            }
                        )),
                        |(_label, quality)| Message::QualitySelected(quality),
                    ),
                ],
                
                // Upscaling algorithm (only shown for GPU or Fallback)
                if self.profile.upscaling_tech == 3 || self.profile.upscaling_tech == 4 {
                    column![
                        row![
                            text("Upscaling Algorithm:"),
                            pick_list(
                                vec![
                                    ("Lanczos (a=3)", 0),
                                    ("Bicubic", 1),
                                    ("Bilinear", 2),
                                    ("Nearest-Neighbor", 3),
                                ],
                                Some((
                                    match self.profile.upscaling_algorithm {
                                        0 => "Lanczos (a=3)",
                                        1 => "Bicubic",
                                        2 => "Bilinear",
                                        3 => "Nearest-Neighbor",
                                        _ => "Lanczos (a=3)",
                                    },
                                    self.profile.upscaling_algorithm
                                )),
                                |(_label, algorithm)| Message::AlgorithmSelected(algorithm),
                            ),
                        ],
                        
                        // Algorithm description
                        text(
                            match self.profile.upscaling_algorithm {
                                0 => "Windowed sinc filter over a 6×6 kernel. Best edge preservation among traditional kernels, heavier compute.",
                                1 => "Uses cubic convolution on a 4×4 neighborhood to preserve more edge sharpness than bilinear, at moderate cost.",
                                2 => "Computes a weighted average of the four nearest input pixels. Fast and smooth, but tends to blur sharp edges.",
                                3 => "Copies each input pixel to an N×N block. Zero smoothing, zero blur, but aliased.",
                                _ => "",
                            }
                        )
                        .size(12)
                        .style(theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7))),
                    ]
                    .into()
                } else {
                    row![].into()
                },
            ]
            .spacing(10)
            .padding(10)
        )
        .style(theme::Container::Box)
        .width(Length::Fill);
        
        // FPS settings panel
        let fps_settings = container(
            column![
                text("Capture FPS").size(18),
                
                // FPS slider
                row![
                    text("Target FPS:"),
                    slider(1.0..=240.0, self.fps, Message::FpsChanged)
                        .step(1.0)
                        .width(Length::Fill),
                    text(format!("{} fps", self.fps as u32)),
                ],
            ]
            .spacing(10)
            .padding(10)
        )
        .style(theme::Container::Box)
        .width(Length::Fill);
        
        column![
            title,
            vertical_space(10),
            upscaling_settings,
            vertical_space(10),
            fps_settings,
        ]
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    // View the advanced tab
    fn view_advanced_tab(&self) -> Element<Message> {
        let title = text("Advanced").size(24);
        
        // Application settings panel
        let app_settings = container(
            column![
                text("Application Settings").size(18),
                
                // Checkboxes for application settings
                checkbox(
                    self.settings.auto_save_frames,
                    "Auto-save Captured Frames",
                    Message::ToggleAutoSaveFrames,
                ),
                
                checkbox(
                    self.settings.show_fps_counter,
                    "Show FPS counter",
                    Message::ToggleShowFpsCounter,
                ),
                
                checkbox(
                    self.settings.show_notifications,
                    "Show notifications",
                    Message::ToggleShowNotifications,
                ),
                
                // Theme selection
                row![
                    text("Theme:"),
                    pick_list(
                        vec!["light", "dark"],
                        Some(self.settings.theme.as_str()),
                        |theme| Message::ThemeSelected(theme.to_string()),
                    ),
                ],
            ]
            .spacing(10)
            .padding(10)
        )
        .style(theme::Container::Box)
        .width(Length::Fill);
        
        // Advanced options panel
        let advanced_options = container(
            column![
                text("Advanced Options").size(18),
                text("Advanced settings will be available in future versions."),
            ]
            .spacing(10)
            .padding(10)
        )
        .style(theme::Container::Box)
        .width(Length::Fill);
        
        // Save settings button
        let save_settings_button = button("Save Application Settings")
            .on_press(Message::SaveSettings);
            
        column![
            title,
            vertical_space(10),
            app_settings,
            vertical_space(10),
            advanced_options,
            vertical_space(10),
            save_settings_button,
        ]
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    // View the status bar
    fn view_status_bar(&self) -> Element<Message> {
        let status_color = match self.status_message_type {
            MessageType::Info => iced::Color::from_rgb(0.7, 0.7, 0.7),
            MessageType::Success => iced::Color::from_rgb(0.1, 0.7, 0.3),
            MessageType::Warning => iced::Color::from_rgb(0.9, 0.65, 0.0),
            MessageType::Error => iced::Color::from_rgb(0.8, 0.2, 0.2),
        };
        
        let status_text = text(&self.status_message)
            .style(theme::Text::Color(status_color));
            
        let capturing_indicator = if self.is_capturing {
            text("● CAPTURING")
                .style(theme::Text::Color(iced::Color::from_rgb(0.1, 0.7, 0.3)))
                .size(14)
        } else {
            text("")
        };
            
        row![
            status_text,
            horizontal_space(Length::Fill),
            capturing_indicator,
        ]
        .spacing(10)
        .align_items(iced::Alignment::Center)
        .into()
    }
}

/// Run the Iced application
pub fn run_app() -> Result<()> {
    println!("Iced run_app() started");
    
    let settings = Settings {
        window: iced::window::Settings {
            size: (1200, 800),
            position: iced::window::Position::Centered,
            ..Default::default()
        },
        ..Default::default()
    };
    
    // Run the application
    AppState::run(settings).map_err(|e| anyhow!("Iced application failed: {}", e))
} 