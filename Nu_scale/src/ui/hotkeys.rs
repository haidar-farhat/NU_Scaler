use anyhow::{Result, anyhow};
use global_hotkey::GlobalHotKeyManager;
use global_hotkey::hotkey::{self, HotKey, Modifiers};
use std::str::FromStr;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;

/// Hotkey action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    /// Start/Stop capture
    ToggleCapture,
    /// Take single screenshot
    TakeScreenshot,
    /// Toggle overlay
    ToggleOverlay,
}

/// Hotkey manager
pub struct HotkeyManager {
    /// Global hotkey manager
    manager: GlobalHotKeyManager,
    /// Sender for hotkey actions
    action_sender: Sender<HotkeyAction>,
    /// Receiver for hotkey actions
    action_receiver: Receiver<HotkeyAction>,
    /// Registered hotkeys
    hotkeys: Vec<(HotKey, HotkeyAction)>,
}

impl HotkeyManager {
    /// Create a new hotkey manager
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new()?;
        let (action_sender, action_receiver) = channel();
        
        Ok(Self {
            manager,
            action_sender,
            action_receiver,
            hotkeys: Vec::new(),
        })
    }
    
    /// Register a hotkey
    pub fn register_hotkey(&mut self, hotkey_str: &str, action: HotkeyAction) -> Result<()> {
        // Parse hotkey string (e.g., "Ctrl+Alt+C")
        let hotkey = Self::parse_hotkey(hotkey_str)?;
        
        // Register hotkey with the system
        self.manager.register(hotkey)?;
        
        // Save hotkey and action
        self.hotkeys.push((hotkey, action));
        
        Ok(())
    }
    
    /// Unregister a hotkey
    pub fn unregister_hotkey(&mut self, hotkey_str: &str) -> Result<()> {
        let hotkey = Self::parse_hotkey(hotkey_str)?;
        
        // Find and remove the hotkey
        if let Some(index) = self.hotkeys.iter().position(|(h, _)| *h == hotkey) {
            self.hotkeys.remove(index);
            self.manager.unregister(hotkey)?;
        }
        
        Ok(())
    }
    
    /// Parse a hotkey string
    pub fn parse_hotkey(hotkey_str: &str) -> Result<HotKey> {
        let parts: Vec<&str> = hotkey_str.split('+').collect();
        if parts.is_empty() {
            return Err(anyhow!("Invalid hotkey format"));
        }
        
        // Initialize modifiers with no modifiers
        let mut modifiers = Modifiers::empty();
        let mut key = None;
        
        for part in parts.iter() {
            let part = part.trim();
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
                "alt" => modifiers |= Modifiers::ALT,
                "shift" => modifiers |= Modifiers::SHIFT,
                "win" | "super" | "meta" => modifiers |= Modifiers::META,
                key_name => {
                    // It's a key, try to parse it
                    if key.is_some() {
                        return Err(anyhow!("Multiple keys in hotkey"));
                    }
                    
                    // Try to convert to a key code
                    if let Ok(key_code) = Self::key_from_str(key_name) {
                        key = Some(key_code);
                    } else {
                        return Err(anyhow!("Unknown key: {}", key_name));
                    }
                }
            }
        }
        
        match key {
            Some(key_code) => Ok(HotKey::new(Some(modifiers), key_code)),
            None => Err(anyhow!("No key specified in hotkey")),
        }
    }
    
    /// Convert a key name to a key code
    fn key_from_str(key_str: &str) -> Result<hotkey::Code> {
        use hotkey::Code;
        
        match key_str.to_lowercase().as_str() {
            "a" => Ok(Code::KeyA),
            "b" => Ok(Code::KeyB),
            "c" => Ok(Code::KeyC),
            "d" => Ok(Code::KeyD),
            "e" => Ok(Code::KeyE),
            "f" => Ok(Code::KeyF),
            "g" => Ok(Code::KeyG),
            "h" => Ok(Code::KeyH),
            "i" => Ok(Code::KeyI),
            "j" => Ok(Code::KeyJ),
            "k" => Ok(Code::KeyK),
            "l" => Ok(Code::KeyL),
            "m" => Ok(Code::KeyM),
            "n" => Ok(Code::KeyN),
            "o" => Ok(Code::KeyO),
            "p" => Ok(Code::KeyP),
            "q" => Ok(Code::KeyQ),
            "r" => Ok(Code::KeyR),
            "s" => Ok(Code::KeyS),
            "t" => Ok(Code::KeyT),
            "u" => Ok(Code::KeyU),
            "v" => Ok(Code::KeyV),
            "w" => Ok(Code::KeyW),
            "x" => Ok(Code::KeyX),
            "y" => Ok(Code::KeyY),
            "z" => Ok(Code::KeyZ),
            "1" | "one" => Ok(Code::Digit1),
            "2" | "two" => Ok(Code::Digit2),
            "3" | "three" => Ok(Code::Digit3),
            "4" | "four" => Ok(Code::Digit4),
            "5" | "five" => Ok(Code::Digit5),
            "6" | "six" => Ok(Code::Digit6),
            "7" | "seven" => Ok(Code::Digit7),
            "8" | "eight" => Ok(Code::Digit8),
            "9" | "nine" => Ok(Code::Digit9),
            "0" | "zero" => Ok(Code::Digit0),
            "f1" => Ok(Code::F1),
            "f2" => Ok(Code::F2),
            "f3" => Ok(Code::F3),
            "f4" => Ok(Code::F4),
            "f5" => Ok(Code::F5),
            "f6" => Ok(Code::F6),
            "f7" => Ok(Code::F7),
            "f8" => Ok(Code::F8),
            "f9" => Ok(Code::F9),
            "f10" => Ok(Code::F10),
            "f11" => Ok(Code::F11),
            "f12" => Ok(Code::F12),
            "space" => Ok(Code::Space),
            "escape" | "esc" => Ok(Code::Escape),
            "return" | "enter" => Ok(Code::Enter),
            "tab" => Ok(Code::Tab),
            "backspace" => Ok(Code::Backspace),
            "insert" => Ok(Code::Insert),
            "delete" => Ok(Code::Delete),
            "home" => Ok(Code::Home),
            "end" => Ok(Code::End),
            "pageup" => Ok(Code::PageUp),
            "pagedown" => Ok(Code::PageDown),
            "up" => Ok(Code::ArrowUp),
            "down" => Ok(Code::ArrowDown),
            "left" => Ok(Code::ArrowLeft),
            "right" => Ok(Code::ArrowRight),
            _ => Err(anyhow!("Unknown key: {}", key_str)),
        }
    }
    
    /// Get the action receiver
    pub fn get_receiver(&self) -> &Receiver<HotkeyAction> {
        &self.action_receiver
    }
    
    /// Start listening for hotkeys
    pub fn start_listening(self) -> Result<thread::JoinHandle<()>> {
        let sender = self.action_sender;
        let hotkeys = self.hotkeys;
        
        let handle = thread::spawn(move || {
            loop {
                if let Ok(event) = global_hotkey::GlobalHotKeyEvent::receiver().recv() {
                    // Find the hotkey and send the action
                    for (hotkey, action) in &hotkeys {
                        if hotkey.id() == event.id {
                            let _ = sender.send(*action);
                            break;
                        }
                    }
                }
            }
        });
        
        Ok(handle)
    }
} 