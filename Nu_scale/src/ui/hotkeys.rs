use anyhow::{Result, anyhow};
use global_hotkey::{GlobalHotKeyManager, HotKey};
use hotkey::modifiers;
use hotkey::keys;
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
        
        let mut modifiers = modifiers::NONE;
        let mut key = None;
        
        for part in parts.iter() {
            let part = part.trim();
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= modifiers::CONTROL,
                "alt" => modifiers |= modifiers::ALT,
                "shift" => modifiers |= modifiers::SHIFT,
                "win" | "super" | "meta" => modifiers |= modifiers::META,
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
    fn key_from_str(key_str: &str) -> Result<keys::Keys> {
        match key_str.to_lowercase().as_str() {
            "a" => Ok(keys::Keys::A),
            "b" => Ok(keys::Keys::B),
            "c" => Ok(keys::Keys::C),
            "d" => Ok(keys::Keys::D),
            "e" => Ok(keys::Keys::E),
            "f" => Ok(keys::Keys::F),
            "g" => Ok(keys::Keys::G),
            "h" => Ok(keys::Keys::H),
            "i" => Ok(keys::Keys::I),
            "j" => Ok(keys::Keys::J),
            "k" => Ok(keys::Keys::K),
            "l" => Ok(keys::Keys::L),
            "m" => Ok(keys::Keys::M),
            "n" => Ok(keys::Keys::N),
            "o" => Ok(keys::Keys::O),
            "p" => Ok(keys::Keys::P),
            "q" => Ok(keys::Keys::Q),
            "r" => Ok(keys::Keys::R),
            "s" => Ok(keys::Keys::S),
            "t" => Ok(keys::Keys::T),
            "u" => Ok(keys::Keys::U),
            "v" => Ok(keys::Keys::V),
            "w" => Ok(keys::Keys::W),
            "x" => Ok(keys::Keys::X),
            "y" => Ok(keys::Keys::Y),
            "z" => Ok(keys::Keys::Z),
            "1" | "one" => Ok(keys::Keys::NUM1),
            "2" | "two" => Ok(keys::Keys::NUM2),
            "3" | "three" => Ok(keys::Keys::NUM3),
            "4" | "four" => Ok(keys::Keys::NUM4),
            "5" | "five" => Ok(keys::Keys::NUM5),
            "6" | "six" => Ok(keys::Keys::NUM6),
            "7" | "seven" => Ok(keys::Keys::NUM7),
            "8" | "eight" => Ok(keys::Keys::NUM8),
            "9" | "nine" => Ok(keys::Keys::NUM9),
            "0" | "zero" => Ok(keys::Keys::NUM0),
            "f1" => Ok(keys::Keys::F1),
            "f2" => Ok(keys::Keys::F2),
            "f3" => Ok(keys::Keys::F3),
            "f4" => Ok(keys::Keys::F4),
            "f5" => Ok(keys::Keys::F5),
            "f6" => Ok(keys::Keys::F6),
            "f7" => Ok(keys::Keys::F7),
            "f8" => Ok(keys::Keys::F8),
            "f9" => Ok(keys::Keys::F9),
            "f10" => Ok(keys::Keys::F10),
            "f11" => Ok(keys::Keys::F11),
            "f12" => Ok(keys::Keys::F12),
            "space" => Ok(keys::Keys::SPACE),
            "escape" | "esc" => Ok(keys::Keys::ESCAPE),
            "return" | "enter" => Ok(keys::Keys::RETURN),
            "tab" => Ok(keys::Keys::TAB),
            "backspace" => Ok(keys::Keys::BACKSPACE),
            "insert" => Ok(keys::Keys::INSERT),
            "delete" => Ok(keys::Keys::DELETE),
            "home" => Ok(keys::Keys::HOME),
            "end" => Ok(keys::Keys::END),
            "pageup" => Ok(keys::Keys::PAGEUP),
            "pagedown" => Ok(keys::Keys::PAGEDOWN),
            "up" => Ok(keys::Keys::UP),
            "down" => Ok(keys::Keys::DOWN),
            "left" => Ok(keys::Keys::LEFT),
            "right" => Ok(keys::Keys::RIGHT),
            _ => Err(anyhow!("Unknown key: {}", key_str)),
        }
    }
    
    /// Get the action receiver
    pub fn get_receiver(&self) -> Receiver<HotkeyAction> {
        self.action_receiver.clone()
    }
    
    /// Start listening for hotkeys
    pub fn start_listening(self) -> Result<thread::JoinHandle<()>> {
        let manager = self.manager;
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