use anyhow::{Result, anyhow};
use global_hotkey::hotkey::{HotKey, Modifiers, Code};
use global_hotkey::{GlobalHotKeyManager, GlobalHotKeyEvent};
use std::collections::HashMap;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

/// Hotkey action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotkeyAction {
    /// Start/stop capture
    ToggleCapture,
    /// Capture single frame
    CaptureFrame,
    /// Toggle overlay
    ToggleOverlay,
    /// Quit application
    Quit,
}

/// Manages global hotkeys
pub struct HotkeyManager {
    /// Global hotkey manager
    manager: GlobalHotKeyManager,
    /// Registered hotkeys with their actions
    hotkeys: HashMap<u32, HotkeyAction>,
    /// Action sender
    action_sender: Sender<HotkeyAction>,
    /// Action receiver
    action_receiver: Receiver<HotkeyAction>,
    /// Is the listener thread running
    listener_running: bool,
}

impl HotkeyManager {
    /// Create a new hotkey manager
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new()?;
        let (action_sender, action_receiver) = mpsc::channel();
        
        Ok(Self {
            manager,
            hotkeys: HashMap::new(),
            action_sender,
            action_receiver,
            listener_running: false,
        })
    }
    
    /// Register a hotkey
    pub fn register(&mut self, hotkey_str: &str, action: HotkeyAction) -> Result<()> {
        // Parse hotkey string into components
        let hotkey = Self::parse_hotkey(hotkey_str)?;
        
        // Register with the manager
        self.manager.register(hotkey)?;
        
        // Store for later lookup
        self.hotkeys.insert(hotkey.id(), action);
        
        Ok(())
    }
    
    /// Unregister a hotkey
    pub fn unregister(&mut self, hotkey: HotKey) -> Result<()> {
        self.manager.unregister(hotkey)?;
        self.hotkeys.remove(&hotkey.id());
        Ok(())
    }
    
    /// Start listening for hotkey events
    pub fn start_listening(&mut self) -> Result<()> {
        if self.listener_running {
            return Ok(());
        }
        
        // Clone hotkeys map and sender for the thread
        let hotkeys = self.hotkeys.clone();
        let sender = self.action_sender.clone();
        
        // Spawn listener thread
        thread::spawn(move || Self::listener_thread(sender, hotkeys));
        
        self.listener_running = true;
        
        Ok(())
    }
    
    /// Get the action receiver
    pub fn receiver(&self) -> &Receiver<HotkeyAction> {
        &self.action_receiver
    }
    
    /// Parse a hotkey string like "Ctrl+Alt+S" into a HotKey
    pub fn parse_hotkey(hotkey_str: &str) -> Result<HotKey> {
        let parts: Vec<&str> = hotkey_str.split('+').collect();
        
        if parts.is_empty() {
            return Err(anyhow!("Invalid hotkey format"));
        }
        
        // The last part is the key
        let key_str = parts.last().unwrap().trim();
        
        // The rest are modifiers
        let modifier_parts = &parts[0..parts.len() - 1];
        let modifiers = Self::modifiers_from_parts(modifier_parts)?;
        
        // Parse key
        let key_code = Self::key_from_str(key_str)?;
        
        match key_code {
            Some(key_code) => Ok(HotKey::new(Some(modifiers), key_code)),
            None => Err(anyhow!("Unknown key: {}", key_str)),
        }
    }
    
    /// Convert key string to key code
    fn key_from_str(key_str: &str) -> Result<Option<Code>> {
        // Handle special keys and letters
        match key_str.to_lowercase().as_str() {
            "c" => Ok(Some(Code::KeyC)),
            "s" => Ok(Some(Code::KeyS)),
            "o" => Ok(Some(Code::KeyO)),
            "q" => Ok(Some(Code::KeyQ)),
            "f1" => Ok(Some(Code::F1)),
            "f2" => Ok(Some(Code::F2)),
            "f3" => Ok(Some(Code::F3)),
            "f4" => Ok(Some(Code::F4)),
            "f5" => Ok(Some(Code::F5)),
            "f6" => Ok(Some(Code::F6)),
            "f7" => Ok(Some(Code::F7)),
            "f8" => Ok(Some(Code::F8)),
            "f9" => Ok(Some(Code::F9)),
            "f10" => Ok(Some(Code::F10)),
            "f11" => Ok(Some(Code::F11)),
            "f12" => Ok(Some(Code::F12)),
            "escape" | "esc" => Ok(Some(Code::Escape)),
            "space" => Ok(Some(Code::Space)),
            "tab" => Ok(Some(Code::Tab)),
            "enter" | "return" => Ok(Some(Code::Enter)),
            _ => {
                // Single character keys
                if key_str.len() == 1 {
                    // Try to convert directly to key code
                    match key_str.chars().next().unwrap() {
                        'a' | 'A' => Ok(Some(Code::KeyA)),
                        'b' | 'B' => Ok(Some(Code::KeyB)),
                        'c' | 'C' => Ok(Some(Code::KeyC)),
                        'd' | 'D' => Ok(Some(Code::KeyD)),
                        'e' | 'E' => Ok(Some(Code::KeyE)),
                        'f' | 'F' => Ok(Some(Code::KeyF)),
                        'g' | 'G' => Ok(Some(Code::KeyG)),
                        'h' | 'H' => Ok(Some(Code::KeyH)),
                        'i' | 'I' => Ok(Some(Code::KeyI)),
                        'j' | 'J' => Ok(Some(Code::KeyJ)),
                        'k' | 'K' => Ok(Some(Code::KeyK)),
                        'l' | 'L' => Ok(Some(Code::KeyL)),
                        'm' | 'M' => Ok(Some(Code::KeyM)),
                        'n' | 'N' => Ok(Some(Code::KeyN)),
                        'o' | 'O' => Ok(Some(Code::KeyO)),
                        'p' | 'P' => Ok(Some(Code::KeyP)),
                        'q' | 'Q' => Ok(Some(Code::KeyQ)),
                        'r' | 'R' => Ok(Some(Code::KeyR)),
                        's' | 'S' => Ok(Some(Code::KeyS)),
                        't' | 'T' => Ok(Some(Code::KeyT)),
                        'u' | 'U' => Ok(Some(Code::KeyU)),
                        'v' | 'V' => Ok(Some(Code::KeyV)),
                        'w' | 'W' => Ok(Some(Code::KeyW)),
                        'x' | 'X' => Ok(Some(Code::KeyX)),
                        'y' | 'Y' => Ok(Some(Code::KeyY)),
                        'z' | 'Z' => Ok(Some(Code::KeyZ)),
                        '0' => Ok(Some(Code::Digit0)),
                        '1' => Ok(Some(Code::Digit1)),
                        '2' => Ok(Some(Code::Digit2)),
                        '3' => Ok(Some(Code::Digit3)),
                        '4' => Ok(Some(Code::Digit4)),
                        '5' => Ok(Some(Code::Digit5)),
                        '6' => Ok(Some(Code::Digit6)),
                        '7' => Ok(Some(Code::Digit7)),
                        '8' => Ok(Some(Code::Digit8)),
                        '9' => Ok(Some(Code::Digit9)),
                        _ => Err(anyhow!("Unsupported key: {}", key_str)),
                    }
                } else {
                    Err(anyhow!("Unknown key: {}", key_str))
                }
            }
        }
    }
    
    /// Parse modifier strings to Modifiers
    fn modifiers_from_parts(parts: &[&str]) -> Result<Modifiers> {
        let mut modifiers = Modifiers::empty();
        
        for part in parts {
            match part.trim().to_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
                "alt" => modifiers |= Modifiers::ALT,
                "shift" => modifiers |= Modifiers::SHIFT,
                "super" | "win" | "windows" => modifiers |= Modifiers::SUPER,
                _ => return Err(anyhow!("Unknown modifier: {}", part)),
            }
        }
        
        Ok(modifiers)
    }
    
    /// Listener thread for hotkey events
    fn listener_thread(tx: Sender<HotkeyAction>, hotkeys: HashMap<u32, HotkeyAction>) {
        let event_channel = GlobalHotKeyEvent::receiver();
        
        // Listen for hotkey events
        while let Ok(event) = event_channel.recv() {
            if let Some(action) = hotkeys.get(&event.id) {
                // Send the event to the main thread
                let _ = tx.send(*action);
            }
        }
    }
}

// Define hotkey constants
pub const KEY_TOGGLE_CAPTURE: &str = "Ctrl+Shift+C";
pub const KEY_CAPTURE_FRAME: &str = "Ctrl+Shift+F";
pub const KEY_TOGGLE_OVERLAY: &str = "Ctrl+Shift+O";

/// Register a global hotkey
pub fn register_global_hotkey(_key: &str, _action: &str) -> bool {
    // Stub implementation
    true
} 