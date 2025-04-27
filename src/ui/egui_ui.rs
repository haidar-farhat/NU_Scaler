let window_names = self.available_windows.clone();
if let Some(name) = self.available_windows.get(self.selected_window_index).cloned() {
    self.set_capture_target_window_title(ctx, &name);
}
let app_state = AppState::default();
for (index, name) in window_names.iter().enumerate() {
    // ... rest of your code ...
}
if let Some(_texture) = self.textures.remove(&size) {
    // ...
} 