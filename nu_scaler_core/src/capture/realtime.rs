pub trait RealTimeCapture {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self);
    fn get_frame(&mut self) -> Option<Vec<u8>>; // Returns raw RGB frame
}

pub struct ScreenCapture {
    running: bool,
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self { running: false }
    }
}

impl RealTimeCapture for ScreenCapture {
    fn start(&mut self) -> Result<(), String> {
        self.running = true;
        // TODO: Start capture thread
        Ok(())
    }
    fn stop(&mut self) {
        self.running = false;
        // TODO: Stop capture thread
    }
    fn get_frame(&mut self) -> Option<Vec<u8>> {
        // TODO: Return latest captured frame
        None
    }
} 