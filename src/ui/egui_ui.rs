let window_names = self.available_windows.clone();
if let Some(name) = window_names.get(self.selected_window_index) {
    self.set_capture_target_window_title(ctx, name);
}
for (index, name) in window_names.iter().enumerate() {
    // ... rest of your code ...
} 