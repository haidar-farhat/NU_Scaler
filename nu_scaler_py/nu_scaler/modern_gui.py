#!/usr/bin/env python
"""
Nu_Scaler Modern GUI - Redesigned to match the four-pane mockup
"""
import sys
import os
import time
from pathlib import Path
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QFrame, QLabel, QDockWidget,
    QVBoxLayout, QHBoxLayout, QGridLayout, QFormLayout, QSplitter,
    QPushButton, QToolBar, QStatusBar, QComboBox, QCheckBox, QSlider,
    QSpinBox, QDoubleSpinBox, QButtonGroup, QRadioButton, QDialog,
    QFileDialog, QProgressBar, QToolButton, QGraphicsDropShadowEffect,
    QGraphicsView, QGraphicsScene, QStyle, QStyleFactory, QStackedLayout
)
from PySide6.QtCore import Qt, QTimer, QSize, QThread, Signal, Slot, QEvent
from PySide6.QtGui import QPixmap, QImage, QColor, QPalette, QIcon, QAction, QDrag, QFont

# Try to import Nu_Scaler core and utilities
try:
    import nu_scaler_core
    from .benchmark import run_benchmark
    print(f"Nu_Scaler Core loaded from {nu_scaler_core.__file__}")
except ImportError as e:
    print(f"Warning: Nu_Scaler core import error: {e}")
    nu_scaler_core = None

# Constants for styling
COLORS = {
    "background_dark": "#0D1B2A",     # Deep navy
    "background_medium": "#1B263B",   # Charcoal
    "accent_primary": "#2E8BC0",      # Teal
    "accent_secondary": "#A3D977",    # Neon-green
    "text_light": "#E0E1DD",          # Light grey for text
    "text_disabled": "#778DA9",       # Muted blue-grey
    "button_hover": "#3A86C5",        # Slightly lighter teal
    "button_pressed": "#236E9B",      # Slightly darker teal
    "error": "#E63946",               # Error/warning red
    "surface": "#152536",             # Slightly lighter than background_dark
    "border": "#2C3E50"               # Border color
}

# Global stylesheet
STYLESHEET = f"""
* {{
    font-family: 'Segoe UI', 'Roboto', sans-serif;
    color: {COLORS["text_light"]};
}}

QMainWindow, QDialog, QDockWidget, QWidget {{
    background-color: {COLORS["background_dark"]};
}}

QFrame, QToolBar, QStatusBar {{
    background-color: {COLORS["background_medium"]};
    border-radius: 6px;
    border: 1px solid {COLORS["border"]};
}}

QPushButton, QToolButton {{
    background-color: {COLORS["accent_primary"]};
    color: {COLORS["text_light"]};
    border-radius: 6px;
    padding: 6px 12px;
    border: none;
    font-weight: bold;
}}

QPushButton:hover, QToolButton:hover {{
    background-color: {COLORS["button_hover"]};
}}

QPushButton:pressed, QToolButton:pressed {{
    background-color: {COLORS["button_pressed"]};
}}

QPushButton:disabled, QToolButton:disabled {{
    background-color: {COLORS["background_medium"]};
    color: {COLORS["text_disabled"]};
}}

QComboBox {{
    background-color: {COLORS["background_medium"]};
    border-radius: 6px;
    padding: 4px 10px;
    border: 1px solid {COLORS["border"]};
    min-height: 24px;
}}

QComboBox:drop-down {{
    border: none;
    background-color: {COLORS["accent_primary"]};
    width: 24px;
    border-top-right-radius: 6px;
    border-bottom-right-radius: 6px;
}}

QCheckBox {{
    spacing: 8px;
}}

QCheckBox::indicator {{
    width: 18px;
    height: 18px;
    border-radius: 3px;
    border: 1px solid {COLORS["border"]};
    background-color: {COLORS["background_dark"]};
}}

QCheckBox::indicator:checked {{
    background-color: {COLORS["accent_secondary"]};
    border: 1px solid {COLORS["accent_secondary"]};
}}

QRadioButton {{
    spacing: 8px;
}}

QRadioButton::indicator {{
    width: 18px;
    height: 18px;
    border-radius: 9px;
    border: 1px solid {COLORS["border"]};
    background-color: {COLORS["background_dark"]};
}}

QRadioButton::indicator:checked {{
    background-color: {COLORS["accent_secondary"]};
    border: 1px solid {COLORS["accent_secondary"]};
}}

QSlider::groove:horizontal {{
    border-radius: 3px;
    height: 6px;
    background-color: {COLORS["background_dark"]};
}}

QSlider::handle:horizontal {{
    background-color: {COLORS["accent_secondary"]};
    border-radius: 7px;
    width: 14px;
    margin: -4px 0;
}}

QProgressBar {{
    border-radius: 3px;
    background-color: {COLORS["background_dark"]};
    text-align: center;
    height: 10px;
}}

QProgressBar::chunk {{
    background-color: {COLORS["accent_secondary"]};
    border-radius: 3px;
}}

QLabel#statusLabel {{
    padding: 2px 8px;
    min-width: 120px;
}}

QGraphicsView, QLabel#previewLabel {{
    background-color: {COLORS["background_dark"]};
    border-radius: 6px;
    border: 1px solid {COLORS["border"]};
}}

QSplitter::handle {{
    background-color: {COLORS["border"]};
    width: 1px;
    height: 1px;
}}

QDockWidget::title {{
    background-color: {COLORS["background_medium"]};
    padding: 6px;
    text-align: center;
}}

QDockWidget::close-button, QDockWidget::float-button {{
    background-color: {COLORS["accent_primary"]};
    border-radius: 6px;
}}
"""

class PreviewPane(QFrame):
    """Custom widget for displaying original and processed image/video previews"""
    
    def __init__(self, title, parent=None):
        super().__init__(parent)
        self.setObjectName("previewPane")
        self.setAcceptDrops(True)
        
        # Create layout
        layout = QVBoxLayout(self)
        
        # Title label
        self.title_label = QLabel(title)
        self.title_label.setAlignment(Qt.AlignCenter)
        self.title_label.setStyleSheet(f"font-size: 16px; font-weight: bold; color: {COLORS['text_light']};")
        
        # Preview area (using QLabel for now, could be replaced with QGraphicsView for more complex needs)
        self.preview = QLabel()
        self.preview.setObjectName("previewLabel")
        self.preview.setAlignment(Qt.AlignCenter)
        self.preview.setText(f"Drag & drop an image/video\nor click to select")
        self.preview.setWordWrap(True)
        self.preview.setMinimumSize(320, 240)
        self.preview.setStyleSheet("padding: 20px;")
        
        # Add shadow effect
        shadow = QGraphicsDropShadowEffect()
        shadow.setBlurRadius(15)
        shadow.setColor(QColor(0, 0, 0, 100))
        shadow.setOffset(0, 2)
        self.preview.setGraphicsEffect(shadow)
        
        # Add widgets to layout
        layout.addWidget(self.title_label)
        layout.addWidget(self.preview, 1)  # 1 = stretch factor
        layout.setContentsMargins(12, 12, 12, 12)
        
    def setPixmap(self, pixmap):
        """Set the preview image"""
        if pixmap and not pixmap.isNull():
            # Scale pixmap to fit the label while maintaining aspect ratio
            scaled_pixmap = pixmap.scaled(
                self.preview.size(),
                Qt.KeepAspectRatio,
                Qt.SmoothTransformation
            )
            self.preview.setPixmap(scaled_pixmap)
            # Clear text once we have an image
            self.preview.setText("")
        else:
            self.preview.clear()
            self.preview.setText("No image/video to display")
            
    def dragEnterEvent(self, event):
        """Handle drag enter events for drag & drop functionality"""
        if event.mimeData().hasUrls():
            event.acceptProposedAction()
            
    def dropEvent(self, event):
        """Handle drop events for drag & drop functionality"""
        if event.mimeData().hasUrls():
            url = event.mimeData().urls()[0]
            file_path = url.toLocalFile()
            
            # Simple check for image files (could be expanded for videos)
            if file_path.lower().endswith(('.png', '.jpg', '.jpeg', '.bmp', '.gif')):
                pixmap = QPixmap(file_path)
                if not pixmap.isNull():
                    self.setPixmap(pixmap)
                    # Emit a signal here to notify parent of new image
            
            event.acceptProposedAction()
    
    def mousePressEvent(self, event):
        """Handle mouse press events for file selection dialog"""
        if event.button() == Qt.LeftButton:
            file_path, _ = QFileDialog.getOpenFileName(
                self, 
                "Open Image/Video", 
                "", 
                "Images (*.png *.jpg *.jpeg *.bmp);;Videos (*.mp4 *.avi *.mov);;All Files (*)"
            )
            
            if file_path:
                if file_path.lower().endswith(('.png', '.jpg', '.jpeg', '.bmp')):
                    pixmap = QPixmap(file_path)
                    if not pixmap.isNull():
                        self.setPixmap(pixmap)
                        # Emit a signal here to notify parent of new image
                # Handle video files here

class SettingsPanel(QWidget):
    """Dockable settings panel with form layout and controls"""
    
    # Signals
    settingsChanged = Signal(dict)  # Emitted when settings change
    advancedRequested = Signal()    # Emitted when advanced button is clicked
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.initUI()
        
    def initUI(self):
        """Initialize the user interface"""
        layout = QVBoxLayout(self)
        
        # Create form layout for settings
        form_layout = QFormLayout()
        form_layout.setSpacing(16)
        form_layout.setContentsMargins(12, 12, 12, 12)
        
        # Upscaling method
        self.method_combo = QComboBox()
        self.method_combo.addItems(["WGPU Bilinear", "WGPU Nearest", "DLSS"])
        form_layout.addRow("Upscaling Method:", self.method_combo)
        
        # Quality preset
        self.quality_combo = QComboBox()
        self.quality_combo.addItems(["Ultra", "Quality", "Balanced", "Performance"])
        form_layout.addRow("Quality Preset:", self.quality_combo)
        
        # Scale factor
        scale_layout = QHBoxLayout()
        self.scale_slider = QSlider(Qt.Horizontal)
        self.scale_slider.setRange(10, 40)  # 1.0x to 4.0x
        self.scale_slider.setValue(20)      # Default 2.0x
        self.scale_label = QLabel("2.0×")
        self.scale_slider.valueChanged.connect(
            lambda: self.scale_label.setText(f"{self.scale_slider.value()/10.0:.1f}×")
        )
        scale_layout.addWidget(self.scale_slider)
        scale_layout.addWidget(self.scale_label)
        form_layout.addRow("Scale Factor:", scale_layout)
        
        # GPU Batch Size
        self.batch_size_spin = QSpinBox()
        self.batch_size_spin.setRange(1, 16)
        self.batch_size_spin.setValue(1)
        form_layout.addRow("GPU Batch Size:", self.batch_size_spin)
        
        # Checkboxes
        self.use_tensor_cores = QCheckBox("Use Tensor Cores (NVIDIA GPUs)")
        self.enable_interpolation = QCheckBox("Enable Frame Interpolation")
        self.auto_optimize = QCheckBox("Auto-Optimize for GPU")
        
        form_layout.addRow("", self.use_tensor_cores)
        form_layout.addRow("", self.enable_interpolation)
        form_layout.addRow("", self.auto_optimize)
        
        # Add form to main layout
        layout.addLayout(form_layout)
        
        # Advanced button
        self.advanced_btn = QPushButton("Advanced Interpolation Settings...")
        self.advanced_btn.clicked.connect(self.advancedRequested.emit)
        layout.addWidget(self.advanced_btn)
        
        # Enable/disable logic
        self.method_combo.currentTextChanged.connect(self.updateControlState)
        self.enable_interpolation.toggled.connect(self.updateControlState)
        
        # Add stretch to push everything to the top
        layout.addStretch(1)
        
        # Connect signals
        self.method_combo.currentTextChanged.connect(self.emitSettingsChanged)
        self.quality_combo.currentTextChanged.connect(self.emitSettingsChanged)
        self.scale_slider.valueChanged.connect(self.emitSettingsChanged)
        self.batch_size_spin.valueChanged.connect(self.emitSettingsChanged)
        self.use_tensor_cores.toggled.connect(self.emitSettingsChanged)
        self.enable_interpolation.toggled.connect(self.emitSettingsChanged)
        self.auto_optimize.toggled.connect(self.emitSettingsChanged)
        
        # Initial state update
        self.updateControlState()
        
    def updateControlState(self):
        """Update enabled/disabled state of controls based on current selections"""
        # Tensor cores only available with DLSS
        is_dlss = self.method_combo.currentText() == "DLSS"
        self.use_tensor_cores.setEnabled(is_dlss)
        
        # Advanced button only enabled if interpolation is enabled
        self.advanced_btn.setEnabled(self.enable_interpolation.isChecked())
        
    def emitSettingsChanged(self):
        """Emit signal with current settings as dictionary"""
        settings = {
            "method": self.method_combo.currentText(),
            "quality": self.quality_combo.currentText(),
            "scale": self.scale_slider.value() / 10.0,
            "batch_size": self.batch_size_spin.value(),
            "use_tensor_cores": self.use_tensor_cores.isChecked(),
            "enable_interpolation": self.enable_interpolation.isChecked(),
            "auto_optimize": self.auto_optimize.isChecked()
        }
        self.settingsChanged.emit(settings)

class InterpolationDialog(QDialog):
    """Modal dialog for advanced interpolation settings"""
    
    # Signal emitted when settings are applied
    settingsApplied = Signal(dict)
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Advanced Interpolation Settings")
        self.setMinimumWidth(450)
        self.initUI()
        
        # Apply shadow effect to the dialog
        shadow = QGraphicsDropShadowEffect()
        shadow.setBlurRadius(30)
        shadow.setColor(QColor(0, 0, 0, 120))
        shadow.setOffset(0, 5)
        self.setGraphicsEffect(shadow)
        
    def initUI(self):
        """Initialize the user interface"""
        layout = QVBoxLayout(self)
        layout.setSpacing(16)
        layout.setContentsMargins(24, 24, 24, 24)
        
        # Title
        title = QLabel("Advanced Interpolation Settings")
        title.setStyleSheet(f"font-size: 18px; font-weight: bold; color: {COLORS['text_light']};")
        title.setAlignment(Qt.AlignCenter)
        layout.addWidget(title)
        
        # Motion Sensitivity slider
        motion_layout = QVBoxLayout()
        self.motion_slider = QSlider(Qt.Horizontal)
        self.motion_slider.setRange(0, 100)
        self.motion_slider.setValue(50)
        self.motion_label = QLabel("Motion Sensitivity: 50%")
        self.motion_slider.valueChanged.connect(
            lambda val: self.motion_label.setText(f"Motion Sensitivity: {val}%")
        )
        motion_layout.addWidget(self.motion_label)
        motion_layout.addWidget(self.motion_slider)
        layout.addLayout(motion_layout)
        
        # Detail Preservation slider
        detail_layout = QVBoxLayout()
        self.detail_slider = QSlider(Qt.Horizontal)
        self.detail_slider.setRange(0, 100)
        self.detail_slider.setValue(75)
        self.detail_label = QLabel("Detail Preservation: 75%")
        self.detail_slider.valueChanged.connect(
            lambda val: self.detail_label.setText(f"Detail Preservation: {val}%")
        )
        detail_layout.addWidget(self.detail_label)
        detail_layout.addWidget(self.detail_slider)
        layout.addLayout(detail_layout)
        
        # Artifact Reduction slider
        artifact_layout = QVBoxLayout()
        self.artifact_slider = QSlider(Qt.Horizontal)
        self.artifact_slider.setRange(0, 100)
        self.artifact_slider.setValue(25)
        self.artifact_label = QLabel("Artifact Reduction: 25%")
        self.artifact_slider.valueChanged.connect(
            lambda val: self.artifact_label.setText(f"Artifact Reduction: {val}%")
        )
        artifact_layout.addWidget(self.artifact_label)
        artifact_layout.addWidget(self.artifact_slider)
        layout.addLayout(artifact_layout)
        
        # Shader selection section
        shader_group_box = QFrame()
        shader_group_box.setStyleSheet(f"background-color: {COLORS['background_medium']}; padding: 12px; border-radius: 6px;")
        shader_layout = QVBoxLayout(shader_group_box)
        
        shader_title = QLabel("Interpolation Shader")
        shader_title.setStyleSheet("font-weight: bold;")
        shader_layout.addWidget(shader_title)
        
        # Radio buttons for shader selection
        self.shader_group = QButtonGroup(self)
        self.optical_flow_radio = QRadioButton("Optical Flow (Best for Videos)")
        self.rife_radio = QRadioButton("RIFE (Best for Gaming)")
        self.blend_radio = QRadioButton("Simple Blend (Low GPU Usage)")
        
        self.shader_group.addButton(self.optical_flow_radio, 1)
        self.shader_group.addButton(self.rife_radio, 2)
        self.shader_group.addButton(self.blend_radio, 3)
        
        # Default selection
        self.optical_flow_radio.setChecked(True)
        
        shader_layout.addWidget(self.optical_flow_radio)
        shader_layout.addWidget(self.rife_radio)
        shader_layout.addWidget(self.blend_radio)
        
        layout.addWidget(shader_group_box)
        
        # Button layout
        button_layout = QHBoxLayout()
        button_layout.setSpacing(12)
        
        # Apply and Cancel buttons
        self.cancel_btn = QPushButton("Cancel")
        self.cancel_btn.clicked.connect(self.reject)
        
        self.apply_btn = QPushButton("Apply")
        self.apply_btn.setStyleSheet(f"background-color: {COLORS['accent_secondary']};")
        self.apply_btn.clicked.connect(self.applySettings)
        
        button_layout.addWidget(self.cancel_btn)
        button_layout.addWidget(self.apply_btn)
        
        layout.addLayout(button_layout)
        
    def applySettings(self):
        """Collect settings and emit signal before closing"""
        settings = {
            "motion_sensitivity": self.motion_slider.value(),
            "detail_preservation": self.detail_slider.value(),
            "artifact_reduction": self.artifact_slider.value(),
            "shader": self.getSelectedShader()
        }
        self.settingsApplied.emit(settings)
        self.accept()
        
    def getSelectedShader(self):
        """Get the currently selected shader"""
        if self.optical_flow_radio.isChecked():
            return "optical_flow"
        elif self.rife_radio.isChecked():
            return "rife"
        else:
            return "blend"

class ComputeControlsPane(QWidget):
    """Widget containing graphics & compute controls with viewport and toolbar"""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.initUI()
        
    def initUI(self):
        """Initialize the user interface"""
        layout = QVBoxLayout(self)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setSpacing(12)
        
        # Title
        title = QLabel("Graphics & Compute Controls")
        title.setStyleSheet(f"font-size: 16px; font-weight: bold; color: {COLORS['text_light']};")
        title.setAlignment(Qt.AlignCenter)
        layout.addWidget(title)
        
        # Main viewport (placeholder)
        self.viewport = QGraphicsView()
        self.scene = QGraphicsScene()
        self.viewport.setScene(self.scene)
        self.viewport.setMinimumHeight(120)
        
        # Add shadow effect to viewport
        shadow = QGraphicsDropShadowEffect()
        shadow.setBlurRadius(15)
        shadow.setColor(QColor(0, 0, 0, 100))
        shadow.setOffset(0, 2)
        self.viewport.setGraphicsEffect(shadow)
        
        layout.addWidget(self.viewport)
        
        # Toolbar
        toolbar_frame = QFrame()
        toolbar_layout = QHBoxLayout(toolbar_frame)
        toolbar_layout.setContentsMargins(8, 8, 8, 8)
        toolbar_layout.setSpacing(8)
        
        # Toolbar buttons
        self.debug_btn = QPushButton("Debug View")
        self.performance_btn = QPushButton("Performance")
        self.export_btn = QPushButton("Export")
        self.reset_btn = QPushButton("Reset")
        
        # Style the export button differently
        self.export_btn.setStyleSheet(f"background-color: {COLORS['accent_secondary']};")
        
        toolbar_layout.addWidget(self.debug_btn)
        toolbar_layout.addWidget(self.performance_btn)
        toolbar_layout.addStretch(1)
        toolbar_layout.addWidget(self.export_btn)
        toolbar_layout.addWidget(self.reset_btn)
        
        layout.addWidget(toolbar_frame)

class MainWindow(QMainWindow):
    """Main application window implementing the four-pane mockup design"""
    
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Nu_Scaler")
        self.resize(1200, 800)
        self.initUI()
        
        # Start statusbar update timer
        self.status_timer = QTimer(self)
        self.status_timer.timeout.connect(self.updateStatusBar)
        self.status_timer.start(1000)  # Update every second
        
    def initUI(self):
        """Initialize the user interface"""
        # Set central widget
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        
        # Main layout
        main_layout = QVBoxLayout(central_widget)
        main_layout.setContentsMargins(12, 12, 12, 12)
        main_layout.setSpacing(12)
        
        # Splitter for preview panes
        splitter = QSplitter(Qt.Horizontal)
        
        # Create preview panes
        self.original_pane = PreviewPane("Original Input")
        self.processed_pane = PreviewPane("Processed Output")
        
        # Add to splitter
        splitter.addWidget(self.original_pane)
        splitter.addWidget(self.processed_pane)
        splitter.setSizes([600, 600])  # Equal initial size
        
        # Add splitter to main layout
        main_layout.addWidget(splitter, 3)  # 3 = relative stretch factor
        
        # Graphics & Compute controls pane
        self.compute_pane = ComputeControlsPane()
        main_layout.addWidget(self.compute_pane, 1)  # 1 = relative stretch factor
        
        # Create settings panel as dock widget
        self.settings_panel = SettingsPanel()
        self.settings_dock = QDockWidget("Settings", self)
        self.settings_dock.setWidget(self.settings_panel)
        self.settings_dock.setFeatures(QDockWidget.DockWidgetClosable | QDockWidget.DockWidgetFloatable)
        self.settings_dock.setAllowedAreas(Qt.RightDockWidgetArea | Qt.LeftDockWidgetArea)
        
        # Add dock widget to the right
        self.addDockWidget(Qt.RightDockWidgetArea, self.settings_dock)
        
        # Create interpolation dialog (initially hidden)
        self.interpolation_dialog = InterpolationDialog(self)
        
        # Create status bar
        self.statusBar = QStatusBar()
        self.setStatusBar(self.statusBar)
        
        # Add status indicators
        self.fps_label = QLabel("FPS: 0")
        self.fps_label.setObjectName("statusLabel")
        
        self.gpu_label = QLabel("GPU: 0%")
        self.gpu_label.setObjectName("statusLabel")
        
        self.time_label = QLabel("Process Time: 0ms")
        self.time_label.setObjectName("statusLabel")
        
        self.progress_bar = QProgressBar()
        self.progress_bar.setRange(0, 100)
        self.progress_bar.setValue(0)
        self.progress_bar.setMaximumWidth(200)
        
        self.status_label = QLabel("Ready")
        
        # Add widgets to status bar
        self.statusBar.addWidget(self.fps_label)
        self.statusBar.addWidget(self.gpu_label)
        self.statusBar.addWidget(self.time_label)
        self.statusBar.addWidget(self.progress_bar)
        self.statusBar.addPermanentWidget(self.status_label, 1)  # Stretch=1, align right
        
        # Connect signals
        self.settings_panel.advancedRequested.connect(self.showInterpolationDialog)
        self.settings_panel.settingsChanged.connect(self.onSettingsChanged)
        self.interpolation_dialog.settingsApplied.connect(self.onInterpolationSettingsApplied)
        
    def showInterpolationDialog(self):
        """Show the advanced interpolation settings dialog"""
        self.interpolation_dialog.exec()
        
    def onSettingsChanged(self, settings):
        """Handle changes to the settings panel"""
        # Update status
        self.status_label.setText(f"Settings changed: {settings['method']} at {settings['scale']}x scale")
        
        # Additional processing logic would go here
        
    def onInterpolationSettingsApplied(self, settings):
        """Handle application of interpolation settings"""
        # Update status
        self.status_label.setText(f"Interpolation settings applied: {settings['shader']} shader")
        
        # Additional processing logic would go here
        
    def updateStatusBar(self):
        """Update status bar with current values (simulated for now)"""
        # In a real app, these would come from the processing engine
        fps = 60.0
        gpu_usage = 45
        process_time = 8.5
        
        self.fps_label.setText(f"FPS: {fps:.1f}")
        self.gpu_label.setText(f"GPU: {gpu_usage}%")
        self.time_label.setText(f"Process Time: {process_time:.1f}ms")
        self.progress_bar.setValue(gpu_usage)

def run_gui():
    """Start the application"""
    app = QApplication(sys.argv)
    
    # Set global stylesheet
    app.setStyleSheet(STYLESHEET)
    
    window = MainWindow()
    window.show()
    return app.exec()

if __name__ == "__main__":
    sys.exit(run_gui()) 