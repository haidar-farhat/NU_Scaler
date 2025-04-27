                    self.drag_current = None;
                } else if response.drag_stopped() {
                    self.dragging = false;
                    self.drag_start = None;
                    self.drag_current = None;
                } 