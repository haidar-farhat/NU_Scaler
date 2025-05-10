#!/usr/bin/env python
"""
Modern, professional UI implementation for Nu_Scaler with advanced features.
"""
import sys
import time
import traceback
from typing import Optional, Tuple
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, 
    QPushButton, QLabel, QComboBox, QSlider, QSpinBox, QCheckBox,
    QSplitter, QStatusBar, QProgressBar, QDockWidget, QDialog,
    QButtonGroup, QRadioButton, QFormLayout, QGroupBox, QFrame,
    QSizePolicy, QMessageBox
)
from PySide6.QtCore import Qt, QTimer, Signal, Slot, QSize
from PySide6.QtGui import QPixmap, QImage, QPalette, QColor, QFont
import numpy as np
from PIL import Image
import io

class PreviewPanel(QFrame):
    """Custom preview panel with border and title."""
    def __init__(self, title: str, parent: Optional[QWidget] = None):
        super().__init__(parent)
        self.setFrameStyle(QFrame.StyledPanel | QFrame.Raised)
        self.setStyleSheet("""
            QFrame {
                background-color: #1B263B;
                border: 1px solid #2E8BC0;
                border-radius: 4px;
            }
        """)
        
        layout = QVBoxLayout(self)
        
        # Title
        title_label = QLabel(title)
        title_label.setStyleSheet("""
            QLabel {
                color: #E0E1DD;
                font-size: 14px;
                font-weight: bold;
                padding: 5px;
            }
        """)
        layout.addWidget(title_label)
        
        # Preview area
        self.preview = QLabel()
        self.preview.setAlignment(Qt.AlignCenter)
        self.preview.setStyleSheet("""
            QLabel {
                background-color: #0D1B2A;
                border-radius: 2px;
            }
        """)
        self.preview.setText("No preview available")
        layout.addWidget(self.preview)

class AdvancedSettingsDialog(QDialog):
    """Modal dialog for advanced interpolation settings."""
    settings_changed = Signal(dict)
    
    def __init__(self, parent: Optional[QWidget] = None):
        super().__init__(parent)
        self.setWindowTitle("Advanced Interpolation Settings")
        self.setMinimumWidth(400)
        self.setup_ui()
        
    def setup_ui(self):
        layout = QVBoxLayout(self)
        
        # Smoothness control
        smoothness_group = QGroupBox("Smoothness")
        smoothness_layout = QVBoxLayout()
        self.smoothness_slider = QSlider(Qt.Horizontal)
        self.smoothness_slider.setRange(0, 100)
        self.smoothness_slider.setValue(50)
        self.smoothness_label = QLabel("50%")
        self.smoothness_slider.valueChanged.connect(
            lambda v: self.smoothness_label.setText(f"{v}%")
        )
        smoothness_layout.addWidget(self.smoothness_slider)
        smoothness_layout.addWidget(self.smoothness_label)
        smoothness_group.setLayout(smoothness_layout)
        
        # Scene change sensitivity
        sensitivity_group = QGroupBox("Scene Change Sensitivity")
        sensitivity_layout = QVBoxLayout()
        self.sensitivity_slider = QSlider(Qt.Horizontal)
        self.sensitivity_slider.setRange(0, 100)
        self.sensitivity_slider.setValue(50)
        self.sensitivity_label = QLabel("50%")
        self.sensitivity_slider.valueChanged.connect(
            lambda v: self.sensitivity_label.setText(f"{v}%")
        )
        sensitivity_layout.addWidget(self.sensitivity_slider)
        sensitivity_layout.addWidget(self.sensitivity_label)
        sensitivity_group.setLayout(sensitivity_layout)
        
        # Blending ratio
        blending_group = QGroupBox("Blending Ratio")
        blending_layout = QVBoxLayout()
        self.blending_slider = QSlider(Qt.Horizontal)
        self.blending_slider.setRange(0, 100)
        self.blending_slider.setValue(50)
        self.blending_label = QLabel("50%")
        self.blending_slider.valueChanged.connect(
            lambda v: self.blending_label.setText(f"{v}%")
        )
        blending_layout.addWidget(self.blending_slider)
        blending_layout.addWidget(self.blending_label)
        blending_group.setLayout(blending_layout)
        
        # Shader selection
        shader_group = QGroupBox("Shader Selection")
        shader_layout = QVBoxLayout()
        self.shader_buttons = QButtonGroup()
        
        shaders = ["Default Shader", "HQ Shader", "Performance Shader"]
        for shader in shaders:
            radio = QRadioButton(shader)
            self.shader_buttons.addButton(radio)
            shader_layout.addWidget(radio)
        self.shader_buttons.buttons()[0].setChecked(True)
        shader_group.setLayout(shader_layout)
        
        # Buttons
        button_layout = QHBoxLayout()
        self.apply_btn = QPushButton("Apply")
        self.cancel_btn = QPushButton("Cancel")
        self.apply_btn.clicked.connect(self.apply_settings)
        self.cancel_btn.clicked.connect(self.reject)
        button_layout.addWidget(self.apply_btn)
        button_layout.addWidget(self.cancel_btn)
        
        # Add all groups to main layout
        layout.addWidget(smoothness_group)
        layout.addWidget(sensitivity_group)
        layout.addWidget(blending_group)
        layout.addWidget(shader_group)
        layout.addLayout(button_layout)
        
    def apply_settings(self):
        settings = {
            'smoothness': self.smoothness_slider.value(),
            'sensitivity': self.sensitivity_slider.value(),
            'blending': self.blending_slider.value(),
            'shader': self.shader_buttons.checkedButton().text()
        }
        self.settings_changed.emit(settings)
        self.accept()

class ModernNuScalerApp(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Nu_Scaler - Professional Edition")
        self.resize(1280, 720)
        self.setup_ui()
        self.setup_connections()
        self.load_core()
        
    def setup_ui(self):
        # Create central widget with split view
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        main_layout = QHBoxLayout(central_widget)
        
        # Create preview splitter
        preview_splitter = QSplitter(Qt.Horizontal)
        self.original_preview = PreviewPanel("Original")
        self.processed_preview = PreviewPanel("Processed")
        preview_splitter.addWidget(self.original_preview)
        preview_splitter.addWidget(self.processed_preview)
        preview_splitter.setSizes([600, 600])
        
        # Create settings dock
        self.settings_dock = QDockWidget("Settings")
        self.settings_dock.setAllowedAreas(Qt.RightDockWidgetArea)
        settings_widget = QWidget()
        settings_layout = QFormLayout(settings_widget)
        
        # Device selection
        self.device_combo = QComboBox()
        self.device_combo.addItems(["GPU", "CPU"])
        settings_layout.addRow("Device:", self.device_combo)
        
        # Quality preset
        self.quality_combo = QComboBox()
        self.quality_combo.addItems(["Ultra", "Quality", "Balanced", "Performance"])
        self.quality_combo.setCurrentText("Quality")
        settings_layout.addRow("Quality:", self.quality_combo)
        
        # Algorithm selection
        self.algo_combo = QComboBox()
        self.algo_combo.addItems(["Bilinear", "Bicubic", "Lanczos"])
        self.algo_combo.setCurrentText("Bilinear")
        settings_layout.addRow("Algorithm:", self.algo_combo)
        
        # Scale control
        scale_layout = QHBoxLayout()
        self.scale_slider = QSlider(Qt.Horizontal)
        self.scale_slider.setRange(10, 40)
        self.scale_slider.setValue(20)
        self.scale_label = QLabel("2.0×")
        self.scale_slider.valueChanged.connect(
            lambda: self.scale_label.setText(f"{self.scale_slider.value()/10.0:.1f}×")
        )
        scale_layout.addWidget(self.scale_slider)
        scale_layout.addWidget(self.scale_label)
        settings_layout.addRow("Scale:", scale_layout)
        
        # Batching toggle
        self.batching_check = QCheckBox("Use Batching")
        settings_layout.addRow(self.batching_check)
        
        # Advanced settings button
        self.advanced_btn = QPushButton("Advanced Settings...")
        settings_layout.addRow(self.advanced_btn)
        
        # Window selection
        self.window_combo = QComboBox()
        self.window_combo.addItem("Fullscreen")
        settings_layout.addRow("Window:", self.window_combo)
        
        # Action buttons
        button_layout = QHBoxLayout()
        self.refresh_btn = QPushButton("Refresh Windows")
        self.start_btn = QPushButton("Start")
        self.stop_btn = QPushButton("Stop")
        self.stop_btn.setEnabled(False)
        button_layout.addWidget(self.refresh_btn)
        button_layout.addWidget(self.start_btn)
        button_layout.addWidget(self.stop_btn)
        settings_layout.addRow(button_layout)
        
        self.settings_dock.setWidget(settings_widget)
        self.addDockWidget(Qt.RightDockWidgetArea, self.settings_dock)
        
        # Add preview splitter to main layout
        main_layout.addWidget(preview_splitter)
        
        # Create status bar
        self.status_bar = QStatusBar()
        self.setStatusBar(self.status_bar)
        
        # GPU usage indicator
        self.gpu_usage = QLabel("GPU: 0%")
        self.status_bar.addPermanentWidget(self.gpu_usage)
        
        # Progress bar
        self.progress = QProgressBar()
        self.progress.setMaximumWidth(200)
        self.status_bar.addPermanentWidget(self.progress)
        
        # Status message
        self.status_label = QLabel("Ready")
        self.status_bar.addWidget(self.status_label)
        
        # Apply dark theme
        self.apply_theme()
        
    def apply_theme(self):
        """Apply professional dark theme to the application."""
        self.setStyleSheet("""
            QMainWindow {
                background-color: #0D1B2A;
            }
            QWidget {
                color: #E0E1DD;
                font-family: 'Segoe UI', Arial;
            }
            QLabel {
                color: #E0E1DD;
            }
            QPushButton {
                background-color: #2E8BC0;
                color: white;
                border: none;
                border-radius: 4px;
                padding: 5px 15px;
                min-width: 80px;
            }
            QPushButton:hover {
                background-color: #3498db;
            }
            QPushButton:disabled {
                background-color: #1B263B;
                color: #666;
            }
            QComboBox, QSpinBox {
                background-color: #1B263B;
                color: #E0E1DD;
                border: 1px solid #2E8BC0;
                border-radius: 4px;
                padding: 2px;
            }
            QSlider::groove:horizontal {
                border: 1px solid #2E8BC0;
                height: 8px;
                background: #1B263B;
                margin: 2px 0;
                border-radius: 4px;
            }
            QSlider::handle:horizontal {
                background: #2E8BC0;
                border: 1px solid #2E8BC0;
                width: 18px;
                margin: -2px 0;
                border-radius: 9px;
            }
            QCheckBox {
                spacing: 5px;
            }
            QCheckBox::indicator {
                width: 18px;
                height: 18px;
                border: 1px solid #2E8BC0;
                border-radius: 3px;
            }
            QCheckBox::indicator:checked {
                background-color: #2E8BC0;
            }
            QProgressBar {
                border: 1px solid #2E8BC0;
                border-radius: 4px;
                text-align: center;
                background-color: #1B263B;
            }
            QProgressBar::chunk {
                background-color: #2E8BC0;
                border-radius: 3px;
            }
            QStatusBar {
                background-color: #1B263B;
                color: #E0E1DD;
            }
            QDockWidget {
                titlebar-close-icon: url(close.png);
                titlebar-normal-icon: url(undock.png);
            }
            QDockWidget::title {
                text-align: center;
                background-color: #1B263B;
                padding: 5px;
            }
        """)
        
    def setup_connections(self):
        """Set up signal connections."""
        self.refresh_btn.clicked.connect(self.refresh_windows)
        self.start_btn.clicked.connect(self.start_capture)
        self.stop_btn.clicked.connect(self.stop_capture)
        self.advanced_btn.clicked.connect(self.show_advanced_settings)
        
    def load_core(self):
        """Load the Nu_Scaler core module."""
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
            self.show_error("Error listing windows", str(e))
            
    def show_advanced_settings(self):
        """Show the advanced settings dialog."""
        dialog = AdvancedSettingsDialog(self)
        dialog.settings_changed.connect(self.apply_advanced_settings)
        dialog.exec_()
        
    def apply_advanced_settings(self, settings: dict):
        """Apply advanced settings from the dialog."""
        # TODO: Implement advanced settings application
        print("Advanced settings:", settings)
        
    def start_capture(self):
        """Start screen capture and upscaling."""
        if not self.nu_scaler_core:
            return
            
        try:
            # Create upscaler
            quality = self.quality_combo.currentText().lower()
            algorithm = self.algo_combo.currentText().lower()
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
            
            # Start update timer
            self.timer = QTimer(self)
            self.timer.timeout.connect(self.update_frame)
            self.timer.start(16)  # ~60 FPS
            
            # Update UI
            self.start_btn.setEnabled(False)
            self.stop_btn.setEnabled(True)
            self.status_label.setText("Capture started")
            self.frame_count = 0
            self.last_frame_time = time.time()
            
        except Exception as e:
            self.show_error("Error starting capture", str(e))
            
    def stop_capture(self):
        """Stop screen capture."""
        if hasattr(self, 'timer'):
            self.timer.stop()
        
        if hasattr(self, 'capture') and self.capture:
            try:
                self.capture.stop()
            except Exception as e:
                self.show_error("Error stopping capture", str(e))
        
        self.capture = None
        self.upscaler = None
        
        # Update UI
        self.start_btn.setEnabled(True)
        self.stop_btn.setEnabled(False)
        self.status_label.setText("Capture stopped")
        
    def update_frame(self):
        """Process and display a new frame."""
        if not hasattr(self, 'capture') or not self.capture or not self.upscaler:
            return
            
        try:
            # Get frame
            frame_result = self.capture.get_frame()
            if not frame_result:
                return
                
            frame_bytes, width, height = frame_result
            
            # Show original frame
            original_qimg = QImage(frame_bytes, width, height, QImage.Format_RGBA8888)
            original_pixmap = QPixmap.fromImage(original_qimg)
            self.original_preview.preview.setPixmap(original_pixmap.scaled(
                self.original_preview.preview.width(),
                self.original_preview.preview.height(),
                Qt.KeepAspectRatio,
                Qt.SmoothTransformation
            ))
            
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
            
            # Show processed frame
            processed_qimg = QImage(output_bytes, out_width, out_height, QImage.Format_RGBA8888)
            processed_pixmap = QPixmap.fromImage(processed_qimg)
            self.processed_preview.preview.setPixmap(processed_pixmap.scaled(
                self.processed_preview.preview.width(),
                self.processed_preview.preview.height(),
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
                
                # Simulate GPU usage (replace with actual GPU monitoring)
                gpu_usage = min(100, int(self.fps * 2))
                self.gpu_usage.setText(f"GPU: {gpu_usage}%")
                self.progress.setValue(gpu_usage)
                
        except Exception as e:
            self.show_error("Error in update_frame", str(e))
            self.stop_capture()
            
    def show_error(self, title: str, message: str):
        """Show an error message dialog."""
        QMessageBox.critical(self, title, message)
        self.status_label.setText(f"Error: {message}")

def main():
    app = QApplication(sys.argv)
    window = ModernNuScalerApp()
    window.show()
    sys.exit(app.exec()) 