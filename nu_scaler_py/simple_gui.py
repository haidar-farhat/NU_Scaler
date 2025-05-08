#!/usr/bin/env python
"""
Simplified GUI for Nu_Scaler Core to test basic functionality.
"""
import sys
import time
import traceback
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, 
    QPushButton, QLabel, QComboBox, QSlider
)
from PySide6.QtCore import Qt, QTimer
from PySide6.QtGui import QPixmap, QImage
import numpy as np
from PIL import Image
import io

class SimpleNuScalerApp(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Nu_Scaler - Simple Test")
        self.resize(800, 600)
        
        # Initialize variables
        self.upscaler = None
        self.capture = None
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.update_frame)
        self.frame_count = 0
        self.last_frame_time = None
        self.fps = 0
        
        # Create central widget
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        main_layout = QVBoxLayout(central_widget)
        
        # Create controls
        controls_layout = QHBoxLayout()
        
        # Upscaler controls
        self.quality_combo = QComboBox()
        self.quality_combo.addItems(["ultra", "quality", "balanced", "performance"])
        self.quality_combo.setCurrentText("quality")
        
        self.algo_combo = QComboBox()
        self.algo_combo.addItems(["bilinear", "bicubic", "lanczos"])
        self.algo_combo.setCurrentText("bilinear")
        
        self.scale_slider = QSlider(Qt.Horizontal)
        self.scale_slider.setRange(10, 40)  # 1.0x to 4.0x
        self.scale_slider.setValue(20)      # Default 2.0x
        self.scale_label = QLabel("2.0×")
        self.scale_slider.valueChanged.connect(
            lambda: self.scale_label.setText(f"{self.scale_slider.value()/10.0:.1f}×")
        )
        
        # Capture controls
        self.window_combo = QComboBox()
        self.window_combo.addItem("Fullscreen")
        self.refresh_btn = QPushButton("Refresh Windows")
        self.refresh_btn.clicked.connect(self.refresh_windows)
        
        # Action buttons
        self.start_btn = QPushButton("Start")
        self.start_btn.clicked.connect(self.start_capture)
        self.stop_btn = QPushButton("Stop")
        self.stop_btn.clicked.connect(self.stop_capture)
        self.stop_btn.setEnabled(False)
        
        # Add controls to layout
        controls_layout.addWidget(QLabel("Quality:"))
        controls_layout.addWidget(self.quality_combo)
        controls_layout.addWidget(QLabel("Algorithm:"))
        controls_layout.addWidget(self.algo_combo)
        controls_layout.addWidget(QLabel("Scale:"))
        controls_layout.addWidget(self.scale_slider)
        controls_layout.addWidget(self.scale_label)
        controls_layout.addWidget(QLabel("Window:"))
        controls_layout.addWidget(self.window_combo)
        controls_layout.addWidget(self.refresh_btn)
        controls_layout.addWidget(self.start_btn)
        controls_layout.addWidget(self.stop_btn)
        
        # Create image preview
        self.preview_label = QLabel()
        self.preview_label.setAlignment(Qt.AlignCenter)
        self.preview_label.setText("No preview available")
        
        # Status bar
        self.status_label = QLabel("Ready")
        
        # Add widgets to main layout
        main_layout.addLayout(controls_layout)
        main_layout.addWidget(self.preview_label, 1)
        main_layout.addWidget(self.status_label)
        
        # Try to import nu_scaler_core
        try:
            import nu_scaler_core
            self.nu_scaler_core = nu_scaler_core
            self.status_label.setText(f"Nu_Scaler Core loaded from: {nu_scaler_core.__file__}")
            self.refresh_windows()
        except ImportError as e:
            self.nu_scaler_core = None
            self.status_label.setText(f"Failed to load Nu_Scaler Core: {e}")
            self.start_btn.setEnabled(False)
    
    def refresh_windows(self):
        """Refresh the list of available windows."""
        if not self.nu_scaler_core:
            return
            
        self.window_combo.clear()
        self.window_combo.addItem("Fullscreen")
        
        try:
            if hasattr(self.nu_scaler_core, 'PyScreenCapture') and hasattr(self.nu_scaler_core.PyScreenCapture, 'list_windows'):
                windows = self.nu_scaler_core.PyScreenCapture.list_windows()
                if windows:
                    self.window_combo.addItems(windows)
        except Exception as e:
            print(f"Error listing windows: {e}")
            traceback.print_exc()
    
    def start_capture(self):
        """Start screen capture and upscaling."""
        if not self.nu_scaler_core:
            return
            
        try:
            # Create upscaler
            quality = self.quality_combo.currentText()
            algorithm = self.algo_combo.currentText()
            self.upscaler = self.nu_scaler_core.PyWgpuUpscaler(quality, algorithm)
            
            # Create capture
            self.capture = self.nu_scaler_core.PyScreenCapture()
            
            # Set capture target
            window_title = self.window_combo.currentText()
            if window_title == "Fullscreen":
                target = self.nu_scaler_core.PyCaptureTarget.FullScreen
                window = None
            else:
                target = self.nu_scaler_core.PyCaptureTarget.WindowByTitle
                window = self.nu_scaler_core.PyWindowByTitle(title=window_title)
            
            # Start capture
            self.capture.start(target, window, None)
            
            # Start timer
            self.timer.start(16)  # ~60 FPS
            
            # Update UI
            self.start_btn.setEnabled(False)
            self.stop_btn.setEnabled(True)
            self.status_label.setText("Capture started")
            self.frame_count = 0
            self.last_frame_time = time.time()
            
        except Exception as e:
            self.status_label.setText(f"Error starting capture: {e}")
            traceback.print_exc()
    
    def stop_capture(self):
        """Stop screen capture."""
        self.timer.stop()
        
        if self.capture:
            try:
                self.capture.stop()
            except Exception as e:
                print(f"Error stopping capture: {e}")
        
        self.capture = None
        self.upscaler = None
        
        # Update UI
        self.start_btn.setEnabled(True)
        self.stop_btn.setEnabled(False)
        self.status_label.setText("Capture stopped")
    
    def update_frame(self):
        """Process and display a new frame."""
        if not self.capture or not self.upscaler:
            return
            
        try:
            # Get frame
            frame_result = self.capture.get_frame()
            if not frame_result:
                return
                
            frame_bytes, width, height = frame_result
            
            # Initialize upscaler if needed
            scale = self.scale_slider.value() / 10.0
            out_width = int(width * scale)
            out_height = int(height * scale)
            
            # Check if we need to reinitialize the upscaler
            if not hasattr(self, 'last_in_width') or not hasattr(self, 'last_in_height') or \
               not hasattr(self, 'last_scale') or \
               self.last_in_width != width or self.last_in_height != height or \
               abs(self.last_scale - scale) > 0.01:
                
                self.upscaler.initialize(width, height, out_width, out_height)
                self.last_in_width = width
                self.last_in_height = height
                self.last_scale = scale
            
            # Upscale frame
            output_bytes = self.upscaler.upscale(frame_bytes)
            
            # Convert to QImage and display
            qimg = QImage(output_bytes, out_width, out_height, QImage.Format_RGBA8888)
            pixmap = QPixmap.fromImage(qimg)
            self.preview_label.setPixmap(pixmap.scaled(
                self.preview_label.width(), 
                self.preview_label.height(),
                Qt.KeepAspectRatio, 
                Qt.SmoothTransformation
            ))
            
            # Update FPS
            self.frame_count += 1
            now = time.time()
            elapsed = now - self.last_frame_time
            
            if elapsed >= 1.0:  # Update FPS once per second
                self.fps = self.frame_count / elapsed
                self.frame_count = 0
                self.last_frame_time = now
                
                # Update status
                self.status_label.setText(
                    f"FPS: {self.fps:.1f} | Input: {width}x{height} | "
                    f"Output: {out_width}x{out_height} | Scale: {scale:.1f}x"
                )
                
        except Exception as e:
            self.status_label.setText(f"Error: {e}")
            print(f"Error in update_frame: {e}")
            traceback.print_exc()
            self.stop_capture()

def main():
    app = QApplication(sys.argv)
    window = SimpleNuScalerApp()
    window.show()
    return app.exec()

if __name__ == "__main__":
    sys.exit(main()) 