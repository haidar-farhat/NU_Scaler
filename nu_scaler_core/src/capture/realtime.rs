use scrap::{Capturer, Display};
use std::io::ErrorKind;

pub trait RealTimeCapture {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self);
    fn get_frame(&mut self) -> Option<Vec<u8>>; // Returns raw RGB frame
}

pub struct ScreenCapture {
    running: bool,
    capturer: Option<Capturer>,
    width: usize,
    height: usize,
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self {
            running: false,
            capturer: None,
            width: 0,
            height: 0,
        }
    }
}

impl RealTimeCapture for ScreenCapture {
    fn start(&mut self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let display = Display::primary().map_err(|e| e.to_string())?;
            let width = display.width();
            let height = display.height();
            let capturer = Capturer::new(display).map_err(|e| e.to_string())?;
            self.width = width as usize;
            self.height = height as usize;
            self.capturer = Some(capturer);
            self.running = true;
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        {
            // TODO: Implement for Linux (X11/Wayland)
            Err("Screen capture not implemented for this OS".to_string())
        }
    }
    fn stop(&mut self) {
        self.running = false;
        self.capturer = None;
    }
    fn get_frame(&mut self) -> Option<Vec<u8>> {
        if !self.running {
            return None;
        }
        let capturer = self.capturer.as_mut()?;
        match capturer.frame() {
            Ok(frame) => {
                // Convert BGRA to RGB
                let mut rgb = Vec::with_capacity(self.width * self.height * 3);
                for chunk in frame.chunks(4) {
                    if chunk.len() == 4 {
                        rgb.push(chunk[2]); // R
                        rgb.push(chunk[1]); // G
                        rgb.push(chunk[0]); // B
                    }
                }
                Some(rgb)
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => None, // No new frame yet
            Err(_) => None,
        }
    }
} 