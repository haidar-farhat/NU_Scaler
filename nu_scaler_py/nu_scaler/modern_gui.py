#!/usr/bin/env python
"""
Nu_Scaler Modern GUI - Professional-grade interface with four-pane design
"""
import sys
import os
import time
from pathlib import Path
from typing import Dict, Any, Optional, List, Tuple, Union

from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QFrame, QLabel, QDockWidget,
    QVBoxLayout, QHBoxLayout, QGridLayout, QFormLayout, QSplitter,
    QPushButton, QToolBar, QStatusBar, QComboBox, QCheckBox, QSlider,
    QSpinBox, QDoubleSpinBox, QButtonGroup, QRadioButton, QDialog,
    QFileDialog, QProgressBar, QToolButton, QGraphicsDropShadowEffect,
    QGraphicsView, QGraphicsScene, QStyle, QStyleFactory, QStackedLayout,
    QMenu, QAction, QGraphicsOpacityEffect, QScrollArea, QSizePolicy,
    QLineEdit, QTabWidget
)
from PySide6.QtCore import (
    Qt, QTimer, QSize, QThread, Signal, Slot, QEvent, QRect, QPoint, 
    QEasingCurve, QPropertyAnimation, QParallelAnimationGroup, QObject,
    QFileInfo
)
from PySide6.QtGui import (
    QPixmap, QImage, QColor, QPalette, QIcon, QAction as QGuiAction, 
    QDrag, QFont, QFontMetrics, QPainter, QBrush, QPen, QGradient,
    QLinearGradient, QCursor, QKeySequence, QShortcut
)

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
    # Base colors
    "background_dark": "#0D1B2A",      # Deep navy
    "background_medium": "#1B263B",    # Charcoal
    "background_light": "#415A77",     # Medium blue-grey
    "accent_primary": "#2E8BC0",       # Teal
    "accent_secondary": "#A3D977",     # Neon-green
    "text_light": "#E0E1DD",           # Light grey for text
    "text_medium": "#B0B1BD",          # Medium grey for secondary text
    "text_disabled": "#778DA9",        # Muted blue-grey
    
    # Interactive states
    "button_hover": "#3A86C5",         # Slightly lighter teal
    "button_pressed": "#236E9B",       # Slightly darker teal
    "secondary_hover": "#ABDE85",      # Lighter green
    "secondary_pressed": "#8EC463",    # Darker green
    
    # Semantic colors
    "error": "#E63946",                # Error/warning red
    "success": "#57CC99",              # Success green
    "warning": "#F9C74F",              # Warning yellow
    "info": "#4CC9F0",                 # Info blue
    
    # Additional UI elements
    "surface": "#152536",              # Slightly lighter than background_dark
    "border": "#2C3E50",               # Border color
    "shadow": "rgba(0, 0, 0, 0.5)",    # Shadow color
    "overlay": "rgba(13, 27, 42, 0.7)" # Overlay color with transparency
}

# Font settings
FONTS = {
    "primary": "'Segoe UI', 'Roboto', 'Arial', sans-serif",
    "monospace": "'Cascadia Code', 'Consolas', 'Courier New', monospace",
    "size_small": "9pt",
    "size_normal": "10pt",
    "size_medium": "12pt",
    "size_large": "14pt",
    "size_xlarge": "18pt",
    "weight_normal": "normal",
    "weight_medium": "500",
    "weight_bold": "bold"
}

# Spacing and sizing
SPACING = {
    "xs": "4px",
    "sm": "8px",
    "md": "12px",
    "lg": "16px",
    "xl": "24px",
    "xxl": "32px"
}

# Animation durations
ANIMATION = {
    "fast": 150,     # ms
    "normal": 250,   # ms
    "slow": 350      # ms
}

# Borders and shadows
EFFECTS = {
    "border_radius_sm": "4px",
    "border_radius_md": "6px",
    "border_radius_lg": "8px",
    "border_radius_full": "50%",
    "shadow_sm": "0 2px 4px rgba(0, 0, 0, 0.15)",
    "shadow_md": "0 4px 8px rgba(0, 0, 0, 0.2)",
    "shadow_lg": "0 8px 16px rgba(0, 0, 0, 0.25)"
}

# Global stylesheet - now with more refinements, gradients, and transitions
STYLESHEET = f"""
* {{
    font-family: {FONTS["primary"]};
    font-size: {FONTS["size_normal"]};
    color: {COLORS["text_light"]};
}}

QMainWindow, QDialog, QDockWidget, QWidget {{
    background-color: {COLORS["background_dark"]};
}}

QMenuBar {{
    background-color: {COLORS["background_dark"]};
    border-bottom: 1px solid {COLORS["border"]};
    padding: 2px;
}}

QMenuBar::item {{
    background-color: transparent;
    padding: 4px 12px;
    border-radius: {EFFECTS["border_radius_sm"]};
}}

QMenuBar::item:selected {{
    background-color: {COLORS["background_light"]};
}}

QMenu {{
    background-color: {COLORS["background_medium"]};
    border: 1px solid {COLORS["border"]};
    border-radius: {EFFECTS["border_radius_md"]};
    padding: 4px;
}}

QMenu::item {{
    padding: 6px 24px 6px 12px;
    border-radius: {EFFECTS["border_radius_sm"]};
}}

QMenu::item:selected {{
    background-color: {COLORS["background_light"]};
}}

QToolTip {{
    background-color: {COLORS["background_medium"]};
    color: {COLORS["text_light"]};
    border: 1px solid {COLORS["border"]};
    border-radius: {EFFECTS["border_radius_sm"]};
    padding: 6px;
    font-size: {FONTS["size_small"]};
}}

QMainWindow::separator {{
    width: 1px;
    height: 1px;
    background-color: {COLORS["border"]};
}}

QFrame, QToolBar, QStatusBar {{
    background-color: {COLORS["background_medium"]};
    border-radius: {EFFECTS["border_radius_md"]};
    border: 1px solid {COLORS["border"]};
}}

QFrame[frameShape="4"] {{
    background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 rgba(27, 38, 59, 0.95),
                stop:1 rgba(27, 38, 59, 0.85));
    border-radius: {EFFECTS["border_radius_md"]};
    border: 1px solid {COLORS["border"]};
}}

QFrame#previewPane {{
    background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 rgba(21, 37, 54, 0.95),
                stop:1 rgba(13, 27, 42, 0.85));
    border-radius: {EFFECTS["border_radius_lg"]};
    border: 1px solid {COLORS["border"]};
}}

QPushButton, QToolButton {{
    background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {COLORS["accent_primary"]},
                stop:1 {COLORS["button_pressed"]});
    color: {COLORS["text_light"]};
    border-radius: {EFFECTS["border_radius_md"]};
    padding: 8px 16px;
    border: none;
    font-weight: {FONTS["weight_bold"]};
    min-height: 20px;
    text-align: center;
}}

QPushButton:hover, QToolButton:hover {{
    background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {COLORS["button_hover"]},
                stop:1 {COLORS["accent_primary"]});
}}

QPushButton:pressed, QToolButton:pressed {{
    background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {COLORS["button_pressed"]},
                stop:1 {COLORS["button_pressed"]});
    padding: 9px 15px 7px 17px;
}}

QPushButton:disabled, QToolButton:disabled {{
    background: {COLORS["background_medium"]};
    color: {COLORS["text_disabled"]};
    border: 1px solid {COLORS["border"]};
}}

QPushButton#accentButton, QToolButton#accentButton {{
    background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {COLORS["accent_secondary"]},
                stop:1 {COLORS["secondary_pressed"]});
}}

QPushButton#accentButton:hover, QToolButton#accentButton:hover {{
    background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {COLORS["secondary_hover"]},
                stop:1 {COLORS["accent_secondary"]});
}}

QPushButton#accentButton:pressed, QToolButton#accentButton:pressed {{
    background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {COLORS["secondary_pressed"]},
                stop:1 {COLORS["secondary_pressed"]});
    padding: 9px 15px 7px 17px;
}}

QComboBox {{
    background-color: {COLORS["background_medium"]};
    border-radius: {EFFECTS["border_radius_md"]};
    padding: 6px 12px;
    border: 1px solid {COLORS["border"]};
    min-height: 28px;
    selection-background-color: {COLORS["accent_primary"]};
}}

QComboBox:hover {{
    border: 1px solid {COLORS["accent_primary"]};
}}

QComboBox:focus {{
    border: 1px solid {COLORS["accent_secondary"]};
}}

QComboBox::drop-down {{
    border: none;
    background-color: {COLORS["accent_primary"]};
    width: 28px;
    border-top-right-radius: {EFFECTS["border_radius_md"]};
    border-bottom-right-radius: {EFFECTS["border_radius_md"]};
}}

QComboBox::drop-down:hover {{
    background-color: {COLORS["button_hover"]};
}}

QComboBox QAbstractItemView {{
    background-color: {COLORS["background_medium"]};
    border: 1px solid {COLORS["border"]};
    border-radius: {EFFECTS["border_radius_md"]};
    selection-background-color: {COLORS["background_light"]};
}}

QCheckBox, QRadioButton {{
    spacing: 10px;
    padding: 3px;
    font-size: {FONTS["size_normal"]};
}}

QCheckBox:hover, QRadioButton:hover {{
    color: {COLORS["accent_secondary"]};
}}

QCheckBox::indicator {{
    width: 20px;
    height: 20px;
    border-radius: {EFFECTS["border_radius_sm"]};
    border: 1px solid {COLORS["border"]};
    background-color: {COLORS["background_dark"]};
}}

QCheckBox::indicator:hover {{
    border: 1px solid {COLORS["accent_primary"]};
}}

QCheckBox::indicator:checked {{
    background-color: {COLORS["accent_secondary"]};
    border: 1px solid {COLORS["accent_secondary"]};
    image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 24 24' fill='none' stroke='%23ffffff' stroke-width='3' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='20 6 9 17 4 12'%3E%3C/polyline%3E%3C/svg%3E");
}}

QRadioButton::indicator {{
    width: 20px;
    height: 20px;
    border-radius: {EFFECTS["border_radius_full"]};
    border: 1px solid {COLORS["border"]};
    background-color: {COLORS["background_dark"]};
}}

QRadioButton::indicator:hover {{
    border: 1px solid {COLORS["accent_primary"]};
}}

QRadioButton::indicator:checked {{
    background-color: {COLORS["background_dark"]};
    border: 1px solid {COLORS["accent_secondary"]};
}}

QRadioButton::indicator:checked::after {{
    content: "";
    display: block;
    width: 12px;
    height: 12px;
    border-radius: {EFFECTS["border_radius_full"]};
    background-color: {COLORS["accent_secondary"]};
    position: absolute;
    top: 4px;
    left: 4px;
}}

QSlider::groove:horizontal {{
    border-radius: {EFFECTS["border_radius_sm"]};
    height: 8px;
    background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {COLORS["background_dark"]},
                stop:1 {COLORS["surface"]});
    margin: 2px 0;
}}

QSlider::handle:horizontal {{
    background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {COLORS["accent_secondary"]},
                stop:1 {COLORS["secondary_pressed"]});
    border-radius: {EFFECTS["border_radius_full"]};
    width: 16px;
    height: 16px;
    margin: -4px 0;
}}

QSlider::handle:horizontal:hover {{
    background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {COLORS["secondary_hover"]},
                stop:1 {COLORS["accent_secondary"]});
    width: 18px;
    height: 18px;
    margin: -5px 0;
}}

QSlider::sub-page:horizontal {{
    background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {COLORS["accent_primary"]},
                stop:1 {COLORS["button_pressed"]});
    border-radius: {EFFECTS["border_radius_sm"]};
}}

QProgressBar {{
    border-radius: {EFFECTS["border_radius_sm"]};
    background-color: {COLORS["background_dark"]};
    text-align: center;
    color: {COLORS["text_light"]};
    font-size: {FONTS["size_small"]};
    height: 14px;
}}

QProgressBar::chunk {{
    background: qlineargradient(x1:0, y1:0, x2:1, y2:0,
                stop:0 {COLORS["button_pressed"]},
                stop:1 {COLORS["accent_primary"]});
    border-radius: {EFFECTS["border_radius_sm"]};
}}

QLineEdit, QSpinBox, QDoubleSpinBox {{
    background-color: {COLORS["background_dark"]};
    border: 1px solid {COLORS["border"]};
    border-radius: {EFFECTS["border_radius_md"]};
    padding: 6px 10px;
    selection-background-color: {COLORS["accent_primary"]};
}}

QLineEdit:hover, QSpinBox:hover, QDoubleSpinBox:hover {{
    border: 1px solid {COLORS["accent_primary"]};
}}

QLineEdit:focus, QSpinBox:focus, QDoubleSpinBox:focus {{
    border: 1px solid {COLORS["accent_secondary"]};
}}

QSpinBox::up-button, QDoubleSpinBox::up-button,
QSpinBox::down-button, QDoubleSpinBox::down-button {{
    background-color: {COLORS["background_light"]};
    width: 20px;
    border-radius: 0;
}}

QSpinBox::up-button:hover, QDoubleSpinBox::up-button:hover,
QSpinBox::down-button:hover, QDoubleSpinBox::down-button:hover {{
    background-color: {COLORS["accent_primary"]};
}}

QLabel {{
    font-family: {FONTS["primary"]};
}}

QLabel#statusLabel {{
    padding: 4px 10px;
    min-width: 120px;
    background-color: {COLORS["background_dark"]};
    border-radius: {EFFECTS["border_radius_sm"]};
}}

QLabel#title {{
    font-size: {FONTS["size_large"]};
    font-weight: {FONTS["weight_bold"]};
    color: {COLORS["text_light"]};
    padding: 8px;
}}

QLabel#subtitle {{
    font-size: {FONTS["size_medium"]};
    color: {COLORS["text_medium"]};
    padding: 4px;
}}

QGraphicsView, QLabel#previewLabel {{
    background-color: {COLORS["background_dark"]};
    border-radius: {EFFECTS["border_radius_md"]};
    border: 1px solid {COLORS["border"]};
}}

QSplitter::handle {{
    background-color: {COLORS["border"]};
    width: 1px;
    height: 1px;
}}

QTabWidget::pane {{
    border: 1px solid {COLORS["border"]};
    border-radius: {EFFECTS["border_radius_md"]};
    top: -1px;
}}

QTabBar::tab {{
    background-color: {COLORS["background_medium"]};
    color: {COLORS["text_medium"]};
    border-top-left-radius: {EFFECTS["border_radius_sm"]};
    border-top-right-radius: {EFFECTS["border_radius_sm"]};
    padding: 8px 12px;
    border: 1px solid {COLORS["border"]};
    border-bottom: none;
    min-width: 80px;
}}

QTabBar::tab:selected {{
    background-color: {COLORS["background_dark"]};
    color: {COLORS["text_light"]};
    border-bottom: 2px solid {COLORS["accent_primary"]};
}}

QTabBar::tab:!selected {{
    margin-top: 2px;
}}

QTabBar::tab:hover:!selected {{
    background-color: {COLORS["background_light"]};
    color: {COLORS["text_light"]};
}}

QDockWidget::title {{
    background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {COLORS["background_light"]},
                stop:1 {COLORS["background_medium"]});
    padding: 8px;
    text-align: center;
    font-weight: {FONTS["weight_medium"]};
    font-size: {FONTS["size_medium"]};
}}

QDockWidget::close-button, QDockWidget::float-button {{
    background-color: {COLORS["accent_primary"]};
    border-radius: {EFFECTS["border_radius_sm"]};
    padding: 2px;
}}

QDockWidget::close-button:hover, QDockWidget::float-button:hover {{
    background-color: {COLORS["button_hover"]};
}}

QScrollBar:vertical {{
    border: none;
    background: {COLORS["background_dark"]};
    width: 10px;
    margin: 0;
}}

QScrollBar::handle:vertical {{
    background: {COLORS["background_light"]};
    min-height: 30px;
    border-radius: 4px;
}}

QScrollBar::handle:vertical:hover {{
    background: {COLORS["accent_primary"]};
}}

QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {{
    height: 0px;
}}

QScrollBar:horizontal {{
    border: none;
    background: {COLORS["background_dark"]};
    height: 10px;
    margin: 0;
}}

QScrollBar::handle:horizontal {{
    background: {COLORS["background_light"]};
    min-width: 30px;
    border-radius: 4px;
}}

QScrollBar::handle:horizontal:hover {{
    background: {COLORS["accent_primary"]};
}}

QScrollBar::add-line:horizontal, QScrollBar::sub-line:horizontal {{
    width: 0px;
}}

QStatusBar::item {{
    border: none;
}}
"""

class PreviewPane(QFrame):
    """
    Enhanced widget for displaying original and processed image/video previews
    with drag-and-drop support, animations, and visual feedback.
    """
    # Signals
    fileDropped = Signal(str)  # Emitted when a file is dropped onto the widget
    fileSelected = Signal(str) # Emitted when a file is selected via dialog
    
    def __init__(self, title: str, parent=None):
        super().__init__(parent)
        self.setObjectName("previewPane")
        self.setAcceptDrops(True)
        self.current_file_path = None
        
        # Create layout
        layout = QVBoxLayout(self)
        layout.setContentsMargins(int(SPACING["md"].replace("px", "")), 
                                  int(SPACING["md"].replace("px", "")),
                                  int(SPACING["md"].replace("px", "")),
                                  int(SPACING["md"].replace("px", "")))
        layout.setSpacing(int(SPACING["md"].replace("px", "")))
        
        # Header container with background
        header_container = QFrame()
        header_container.setObjectName("headerContainer")
        header_container.setStyleSheet(f"""
            QFrame#headerContainer {{
                background-color: {COLORS["background_dark"]};
                border-radius: {EFFECTS["border_radius_sm"]};
                padding: 4px;
            }}
        """)
        header_layout = QHBoxLayout(header_container)
        header_layout.setContentsMargins(8, 4, 8, 4)
        
        # Title label
        self.title_label = QLabel(title)
        self.title_label.setObjectName("title")
        self.title_label.setAlignment(Qt.AlignCenter)
        header_layout.addWidget(self.title_label)
        
        # Control buttons
        btn_layout = QHBoxLayout()
        btn_layout.setSpacing(8)
        
        # Reset button
        self.reset_btn = QToolButton()
        self.reset_btn.setIcon(self._create_icon("reset"))
        self.reset_btn.setToolTip("Reset view")
        self.reset_btn.setFixedSize(24, 24)
        self.reset_btn.setStyleSheet("""
            QToolButton {
                background-color: transparent;
                border: none;
                border-radius: 4px;
                padding: 2px;
            }
            QToolButton:hover {
                background-color: rgba(255, 255, 255, 0.1);
            }
            QToolButton:pressed {
                background-color: rgba(0, 0, 0, 0.1);
            }
        """)
        self.reset_btn.clicked.connect(self.reset_view)
        
        # Export button
        self.export_btn = QToolButton()
        self.export_btn.setIcon(self._create_icon("export"))
        self.export_btn.setToolTip("Export image")
        self.export_btn.setFixedSize(24, 24)
        self.export_btn.setStyleSheet("""
            QToolButton {
                background-color: transparent;
                border: none;
                border-radius: 4px;
                padding: 2px;
            }
            QToolButton:hover {
                background-color: rgba(255, 255, 255, 0.1);
            }
            QToolButton:pressed {
                background-color: rgba(0, 0, 0, 0.1);
            }
        """)
        self.export_btn.clicked.connect(self.export_image)
        
        btn_layout.addWidget(self.reset_btn)
        btn_layout.addWidget(self.export_btn)
        header_layout.addLayout(btn_layout)
        
        # Add header to main layout
        layout.addWidget(header_container)
        
        # Preview area using QLabel with enhanced style
        self.preview_container = QFrame()
        self.preview_container.setObjectName("previewContainer")
        preview_container_layout = QVBoxLayout(self.preview_container)
        preview_container_layout.setContentsMargins(0, 0, 0, 0)
        
        self.preview = QLabel()
        self.preview.setObjectName("previewLabel")
        self.preview.setAlignment(Qt.AlignCenter)
        self.preview.setText("Drag & drop an image/video\nor click to select")
        self.preview.setWordWrap(True)
        self.preview.setMinimumSize(320, 240)
        self.preview.setStyleSheet(f"""
            QLabel#previewLabel {{
                background-color: {COLORS["background_dark"]};
                color: {COLORS["text_medium"]};
                border-radius: {EFFECTS["border_radius_md"]};
                padding: 20px;
                font-size: {FONTS["size_medium"]};
            }}
        """)
        
        # Add shadow effect
        shadow = QGraphicsDropShadowEffect()
        shadow.setBlurRadius(20)
        shadow.setColor(QColor(0, 0, 0, 100))
        shadow.setOffset(0, 5)
        self.preview.setGraphicsEffect(shadow)
        
        preview_container_layout.addWidget(self.preview)
        
        # Add info label for metadata
        self.info_label = QLabel("")
        self.info_label.setObjectName("subtitle")
        self.info_label.setAlignment(Qt.AlignCenter)
        self.info_label.setWordWrap(True)
        self.info_label.setTextFormat(Qt.RichText)
        
        # Add widgets to layout
        layout.addWidget(self.preview_container, 1)  # 1 = stretch factor
        layout.addWidget(self.info_label)
        
        # Create context menu
        self.setContextMenuPolicy(Qt.CustomContextMenu)
        self.customContextMenuRequested.connect(self.show_context_menu)
        
        # Initialize buttons state
        self.export_btn.setEnabled(False)
        
    def _create_icon(self, icon_type):
        """Create SVG icons programmatically"""
        if icon_type == "reset":
            # Refresh/reset icon
            return QIcon(self._get_svg_path("refresh"))
        elif icon_type == "export":
            # Export/download icon
            return QIcon(self._get_svg_path("download"))
        elif icon_type == "zoom_in":
            # Zoom in icon
            return QIcon(self._get_svg_path("zoom-in"))
        elif icon_type == "zoom_out":
            # Zoom out icon
            return QIcon(self._get_svg_path("zoom-out"))
        else:
            # Default empty icon
            pixmap = QPixmap(24, 24)
            pixmap.fill(Qt.transparent)
            return QIcon(pixmap)
            
    def _get_svg_path(self, icon_name):
        """Generate SVG data for common icons"""
        svg_icons = {
            "refresh": """
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#E0E1DD" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M23 4v6h-6"></path>
                    <path d="M1 20v-6h6"></path>
                    <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10"></path>
                    <path d="M20.49 15a9 9 0 0 1-14.85 3.36L1 14"></path>
                </svg>
            """,
            "download": """
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#E0E1DD" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                    <polyline points="7 10 12 15 17 10"></polyline>
                    <line x1="12" y1="15" x2="12" y2="3"></line>
                </svg>
            """,
            "zoom-in": """
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#E0E1DD" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="11" cy="11" r="8"></circle>
                    <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                    <line x1="11" y1="8" x2="11" y2="14"></line>
                    <line x1="8" y1="11" x2="14" y2="11"></line>
                </svg>
            """,
            "zoom-out": """
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#E0E1DD" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="11" cy="11" r="8"></circle>
                    <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                    <line x1="8" y1="11" x2="14" y2="11"></line>
                </svg>
            """
        }
        
        # Create a temporary file with the SVG data
        if icon_name in svg_icons:
            import tempfile
            import os
            
            temp = tempfile.NamedTemporaryFile(suffix='.svg', delete=False)
            temp.write(svg_icons[icon_name].encode('utf-8'))
            temp.close()
            
            file_path = temp.name
            return file_path
        
        return ""
    
    def setPixmap(self, pixmap):
        """Set the preview image with animation effect"""
        if pixmap and not pixmap.isNull():
            # Store the original pixmap
            self._original_pixmap = pixmap
            
            # Create the fade effect
            self.fade_effect = QGraphicsOpacityEffect(self.preview)
            self.preview.setGraphicsEffect(self.fade_effect)
            
            # Set up animation
            self.fade_animation = QPropertyAnimation(self.fade_effect, b"opacity")
            self.fade_animation.setDuration(ANIMATION["normal"])
            self.fade_animation.setStartValue(0.3)
            self.fade_animation.setEndValue(1.0)
            self.fade_animation.setEasingCurve(QEasingCurve.OutCubic)
            
            # Connect animation finished signal to restore the shadow effect
            self.fade_animation.finished.connect(self._restore_shadow)
            
            # Scale pixmap to fit the label while maintaining aspect ratio
            self._update_preview()
            
            # Start the animation
            self.fade_animation.start()
            
            # Update info label with image details
            self._update_info()
            
            # Enable export button
            self.export_btn.setEnabled(True)
        else:
            self.preview.clear()
            self.preview.setText("No image/video to display")
            self.info_label.setText("")
            self.export_btn.setEnabled(False)
            self._original_pixmap = None
    
    def _restore_shadow(self):
        """Restore the shadow effect after animation completes"""
        shadow = QGraphicsDropShadowEffect()
        shadow.setBlurRadius(20)
        shadow.setColor(QColor(0, 0, 0, 100))
        shadow.setOffset(0, 5)
        self.preview.setGraphicsEffect(shadow)
    
    def _update_preview(self):
        """Update the preview with the current pixmap scaled to fit"""
        if not hasattr(self, '_original_pixmap') or self._original_pixmap is None:
            return
            
        # Scale pixmap to fit the label while maintaining aspect ratio
        scaled_pixmap = self._original_pixmap.scaled(
            self.preview.size(),
            Qt.KeepAspectRatio,
            Qt.SmoothTransformation
        )
        self.preview.setPixmap(scaled_pixmap)
        # Clear text once we have an image
        self.preview.setText("")
    
    def _update_info(self):
        """Update the info label with image metadata"""
        if not hasattr(self, '_original_pixmap') or self._original_pixmap is None:
            self.info_label.setText("")
            return
            
        # Get image dimensions
        width = self._original_pixmap.width()
        height = self._original_pixmap.height()
        
        # Format file info if available
        file_info = ""
        if self.current_file_path:
            file_name = os.path.basename(self.current_file_path)
            file_size = os.path.getsize(self.current_file_path) / 1024  # KB
            
            if file_size >= 1024:
                file_size = file_size / 1024  # Convert to MB
                file_size_str = f"{file_size:.1f} MB"
            else:
                file_size_str = f"{file_size:.1f} KB"
                
            file_info = f"<b>{file_name}</b> ({file_size_str})"
        
        # Set the info text
        self.info_label.setText(f"{file_info}<br>{width} × {height} pixels")
    
    def reset_view(self):
        """Reset the view to its initial state"""
        if hasattr(self, '_original_pixmap') and self._original_pixmap:
            self._update_preview()
    
    def export_image(self):
        """Export the current image to a file"""
        if not hasattr(self, '_original_pixmap') or self._original_pixmap is None:
            return
            
        # Get save path from user
        suggested_name = "output.png"
        if self.current_file_path:
            file_info = QFileInfo(self.current_file_path)
            suggested_name = f"{file_info.baseName()}_processed.png"
            
        file_path, _ = QFileDialog.getSaveFileName(
            self,
            "Save Image",
            suggested_name,
            "PNG Images (*.png);;JPEG Images (*.jpg *.jpeg);;All Files (*)"
        )
        
        if file_path:
            # Save the original pixmap to preserve quality
            self._original_pixmap.save(file_path)
            
            # Show quick animation feedback
            self._show_save_animation()
    
    def _show_save_animation(self):
        """Show a quick animation to indicate successful save"""
        save_indicator = QLabel("✓ Saved", self)
        save_indicator.setStyleSheet(f"""
            background-color: {COLORS["success"]};
            color: white;
            padding: 8px 16px;
            border-radius: {EFFECTS["border_radius_md"]};
            font-weight: bold;
        """)
        save_indicator.setAlignment(Qt.AlignCenter)
        
        # Position the indicator
        save_indicator.move(
            (self.width() - save_indicator.sizeHint().width()) // 2,
            (self.height() - save_indicator.sizeHint().height()) // 2
        )
        save_indicator.show()
        
        # Create fade animation
        opacity_effect = QGraphicsOpacityEffect(save_indicator)
        save_indicator.setGraphicsEffect(opacity_effect)
        
        fade_animation = QPropertyAnimation(opacity_effect, b"opacity")
        fade_animation.setDuration(1500)  # 1.5 seconds
        fade_animation.setStartValue(1.0)
        fade_animation.setEndValue(0.0)
        fade_animation.setEasingCurve(QEasingCurve.OutCubic)
        
        # Connect animation finished signal to remove the indicator
        fade_animation.finished.connect(save_indicator.deleteLater)
        
        # Start the animation
        fade_animation.start()
    
    def show_context_menu(self, position):
        """Show context menu with options"""
        context_menu = QMenu(self)
        
        # Only add these actions if we have an image
        if hasattr(self, '_original_pixmap') and self._original_pixmap:
            reset_action = context_menu.addAction("Reset View")
            reset_action.triggered.connect(self.reset_view)
            
            export_action = context_menu.addAction("Export Image...")
            export_action.triggered.connect(self.export_image)
            
            context_menu.addSeparator()
        
        # These actions are always available
        open_action = context_menu.addAction("Open File...")
        open_action.triggered.connect(self.open_file_dialog)
        
        # Show the context menu
        context_menu.exec_(self.mapToGlobal(position))
    
    def dragEnterEvent(self, event):
        """Handle drag enter events for drag & drop functionality"""
        if event.mimeData().hasUrls():
            event.acceptProposedAction()
            # Change styling to indicate drop is possible
            self.setStyleSheet(f"""
                QFrame#previewPane {{
                    background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                                stop:0 rgba(46, 139, 192, 0.2),
                                stop:1 rgba(13, 27, 42, 0.85));
                    border-radius: {EFFECTS["border_radius_lg"]};
                    border: 2px dashed {COLORS["accent_primary"]};
                }}
            """)
    
    def dragLeaveEvent(self, event):
        """Handle drag leave events"""
        # Reset styling
        self.setStyleSheet("")
    
    def dragMoveEvent(self, event):
        """Handle drag move events"""
        if event.mimeData().hasUrls():
            event.acceptProposedAction()
    
    def dropEvent(self, event):
        """Handle drop events for drag & drop functionality"""
        # Reset styling
        self.setStyleSheet("")
        
        if event.mimeData().hasUrls():
            url = event.mimeData().urls()[0]
            file_path = url.toLocalFile()
            
            # Simple check for image files (could be expanded for videos)
            if file_path.lower().endswith(('.png', '.jpg', '.jpeg', '.bmp', '.gif', '.webp')):
                pixmap = QPixmap(file_path)
                if not pixmap.isNull():
                    self.current_file_path = file_path
                    self.setPixmap(pixmap)
                    self.fileDropped.emit(file_path)
            
            event.acceptProposedAction()
    
    def open_file_dialog(self):
        """Open a file dialog to select an image"""
        file_path, _ = QFileDialog.getOpenFileName(
            self, 
            "Open Image/Video", 
            "", 
            "Images (*.png *.jpg *.jpeg *.bmp *.webp *.gif);;Videos (*.mp4 *.avi *.mov);;All Files (*)"
        )
        
        if file_path:
            if file_path.lower().endswith(('.png', '.jpg', '.jpeg', '.bmp', '.gif', '.webp')):
                pixmap = QPixmap(file_path)
                if not pixmap.isNull():
                    self.current_file_path = file_path
                    self.setPixmap(pixmap)
                    self.fileSelected.emit(file_path)
            # Video file handling can be added here
    
    def mousePressEvent(self, event):
        """Handle mouse press events for file selection dialog"""
        if event.button() == Qt.LeftButton:
            # Only open dialog if we don't already have an image
            if not hasattr(self, '_original_pixmap') or self._original_pixmap is None:
                self.open_file_dialog()
        
        super().mousePressEvent(event)
    
    def resizeEvent(self, event):
        """Handle resize events to update the preview scaling"""
        super().resizeEvent(event)
        self._update_preview()
        
    def cleanup(self):
        """Clean up any temporary files created for icons"""
        # If we created temporary SVG files, delete them
        # This would be called when the widget is destroyed
        pass

class SettingsPanel(QWidget):
    """
    Enhanced dockable settings panel with form layout, tooltips, and animations.
    Provides more granular control over upscaling parameters.
    """
    
    # Signals
    settingsChanged = Signal(dict)  # Emitted when settings change
    advancedRequested = Signal()    # Emitted when advanced button is clicked
    profileSelected = Signal(str)   # Emitted when a profile is selected
    
    def __init__(self, parent=None):
        super().__init__(parent)
        
        # Setup UI
        self.initUI()
        
        # Setup keyboard shortcuts
        self.setup_shortcuts()
        
        # Load default profiles
        self.load_default_profiles()
        
    def initUI(self):
        """Initialize the user interface with improved layout and style"""
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(int(SPACING["md"].replace("px", "")), 
                                      int(SPACING["md"].replace("px", "")),
                                      int(SPACING["md"].replace("px", "")),
                                      int(SPACING["md"].replace("px", "")))
        main_layout.setSpacing(int(SPACING["md"].replace("px", "")))
        
        # Add scroll area for small screens
        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setFrameShape(QFrame.NoFrame)
        
        # Container widget for the scroll area
        container = QWidget()
        container_layout = QVBoxLayout(container)
        container_layout.setContentsMargins(0, 0, 0, 0)
        container_layout.setSpacing(int(SPACING["lg"].replace("px", "")))
        
        # ====== Profile Selection ======
        profile_frame = QFrame()
        profile_frame.setStyleSheet(f"""
            QFrame {{
                background-color: {COLORS["surface"]};
                border-radius: {EFFECTS["border_radius_md"]};
                padding: 4px;
            }}
        """)
        
        profile_layout = QVBoxLayout(profile_frame)
        profile_layout.setContentsMargins(int(SPACING["sm"].replace("px", "")), 
                                        int(SPACING["sm"].replace("px", "")),
                                        int(SPACING["sm"].replace("px", "")),
                                        int(SPACING["sm"].replace("px", "")))
        
        profile_header = QHBoxLayout()
        
        profile_title = QLabel("Preset Profiles")
        profile_title.setStyleSheet(f"""
            font-size: {FONTS["size_medium"]};
            font-weight: {FONTS["weight_bold"]};
        """)
        
        self.profile_combo = QComboBox()
        self.profile_combo.setToolTip("Select a predefined profile for common scenarios")
        
        profile_header.addWidget(profile_title)
        profile_header.addStretch()
        
        profile_selector = QHBoxLayout()
        profile_selector.addWidget(self.profile_combo, 1)
        
        save_profile_btn = QPushButton("Save")
        save_profile_btn.setFixedWidth(60)
        save_profile_btn.setToolTip("Save current settings as a new profile")
        save_profile_btn.clicked.connect(self.save_current_profile)
        
        profile_selector.addWidget(save_profile_btn)
        
        profile_layout.addLayout(profile_header)
        profile_layout.addLayout(profile_selector)
        
        container_layout.addWidget(profile_frame)
        
        # ====== Upscaling Settings ======
        # Create form layout for settings
        form_frame = QFrame()
        form_frame.setStyleSheet(f"""
            QFrame {{
                background-color: {COLORS["surface"]};
                border-radius: {EFFECTS["border_radius_md"]};
                padding: 4px;
            }}
        """)
        
        form_layout = QFormLayout(form_frame)
        form_layout.setSpacing(int(SPACING["lg"].replace("px", "")))
        form_layout.setContentsMargins(int(SPACING["md"].replace("px", "")), 
                                     int(SPACING["md"].replace("px", "")),
                                     int(SPACING["md"].replace("px", "")),
                                     int(SPACING["md"].replace("px", "")))
        form_layout.setLabelAlignment(Qt.AlignLeft)
        form_layout.setFieldGrowthPolicy(QFormLayout.AllNonFixedFieldsGrow)
        
        settings_title = QLabel("Upscaling Settings")
        settings_title.setStyleSheet(f"""
            font-size: {FONTS["size_medium"]};
            font-weight: {FONTS["weight_bold"]};
        """)
        form_layout.addRow(settings_title)
        
        # Upscaling method with improved styling
        self.method_combo = QComboBox()
        self.method_combo.addItems(["WGPU Bilinear", "WGPU Nearest", "DLSS"])
        self.method_combo.setToolTip(
            "Select upscaling algorithm:\n"
            "• WGPU Bilinear: Smooth interpolation, good all-around\n"
            "• WGPU Nearest: Pixel-perfect for pixel art\n"
            "• DLSS: NVIDIA's AI upscaling (requires compatible GPU)"
        )
        method_label = QLabel("Upscaling Method:")
        method_label.setBuddy(self.method_combo)
        form_layout.addRow(method_label, self.method_combo)
        
        # Quality preset
        self.quality_combo = QComboBox()
        self.quality_combo.addItems(["Ultra", "Quality", "Balanced", "Performance"])
        self.quality_combo.setToolTip(
            "Quality vs. performance tradeoff:\n"
            "• Ultra: Maximum quality, highest GPU usage\n"
            "• Quality: High quality with good performance\n"
            "• Balanced: Good balance of quality and speed\n"
            "• Performance: Fastest option with acceptable quality"
        )
        quality_label = QLabel("Quality Preset:")
        quality_label.setBuddy(self.quality_combo)
        form_layout.addRow(quality_label, self.quality_combo)
        
        # Scale factor with improved slider
        scale_layout = QVBoxLayout()
        
        scale_header = QHBoxLayout()
        self.scale_value_label = QLabel("2.0×")
        self.scale_value_label.setAlignment(Qt.AlignRight | Qt.AlignVCenter)
        self.scale_value_label.setMinimumWidth(40)
        self.scale_value_label.setStyleSheet(f"""
            color: {COLORS["accent_secondary"]};
            font-weight: {FONTS['weight_bold']};
        """)
        
        scale_header.addWidget(QLabel("Scale Factor:"))
        scale_header.addStretch()
        scale_header.addWidget(self.scale_value_label)
        
        scale_layout.addLayout(scale_header)
        
        self.scale_slider = QSlider(Qt.Horizontal)
        self.scale_slider.setRange(10, 40)  # 1.0x to 4.0x
        self.scale_slider.setValue(20)      # Default 2.0x
        self.scale_slider.setTickPosition(QSlider.TicksBelow)
        self.scale_slider.setTickInterval(5)
        self.scale_slider.setToolTip("Drag to adjust the scaling factor (1.0x to 4.0x)")
        
        # Use a more sophisticated function to update the label with animations
        self.scale_slider.valueChanged.connect(self.update_scale_label)
        
        scale_marks = QHBoxLayout()
        scale_marks.setContentsMargins(0, 0, 0, 0)
        for mark, pos in [("1.0×", 0), ("2.0×", 33), ("3.0×", 66), ("4.0×", 100)]:
            mark_label = QLabel(mark)
            mark_label.setStyleSheet(f"color: {COLORS['text_medium']}; font-size: {FONTS['size_small']};")
            mark_label.setAlignment(Qt.AlignCenter)
            scale_marks.addWidget(mark_label, pos)
        
        scale_layout.addWidget(self.scale_slider)
        scale_layout.addLayout(scale_marks)
        form_layout.addRow("", scale_layout)
        
        # GPU Batch Size with more useful range and tooltip
        batch_layout = QHBoxLayout()
        self.batch_size_spin = QSpinBox()
        self.batch_size_spin.setRange(1, 16)
        self.batch_size_spin.setValue(1)
        self.batch_size_spin.setToolTip(
            "Number of frames to process in each GPU batch\n"
            "Higher values improve performance but use more VRAM"
        )
        
        # Add presets next to the spin box
        batch_label = QLabel("GPU Batch Size:")
        
        batch_presets = QHBoxLayout()
        for preset in [1, 4, 8]:
            preset_btn = QPushButton(str(preset))
            preset_btn.setFixedWidth(30)
            preset_btn.setFixedHeight(26)
            preset_btn.setStyleSheet("""
                QPushButton {
                    padding: 2px;
                    font-size: 9pt;
                }
            """)
            preset_btn.clicked.connect(lambda checked, p=preset: self.batch_size_spin.setValue(p))
            batch_presets.addWidget(preset_btn)
        
        batch_layout.addWidget(self.batch_size_spin)
        batch_layout.addLayout(batch_presets)
        form_layout.addRow(batch_label, batch_layout)
        
        # Advanced Options Section
        advanced_title = QLabel("Advanced Options")
        advanced_title.setStyleSheet(f"""
            font-size: {FONTS["size_medium"]};
            font-weight: {FONTS["weight_bold"]};
            margin-top: 10px;
        """)
        form_layout.addRow(advanced_title)
        
        # Checkboxes with improved styling and tooltips
        options_layout = QVBoxLayout()
        options_layout.setSpacing(int(SPACING["md"].replace("px", "")))
        
        self.use_tensor_cores = QCheckBox("Use Tensor Cores (NVIDIA GPUs)")
        self.use_tensor_cores.setToolTip(
            "Enable hardware acceleration with NVIDIA Tensor Cores\n"
            "Only available on RTX series GPUs"
        )
        
        self.enable_interpolation = QCheckBox("Enable Frame Interpolation")
        self.enable_interpolation.setToolTip(
            "Generate intermediate frames to increase smoothness\n"
            "Useful for video content but increases processing time"
        )
        
        self.auto_optimize = QCheckBox("Auto-Optimize for GPU")
        self.auto_optimize.setToolTip(
            "Automatically adjust settings based on your GPU capabilities\n"
            "Recommended for best performance"
        )
        self.auto_optimize.setChecked(True)
        
        self.reduce_memory = QCheckBox("Reduce Memory Usage")
        self.reduce_memory.setToolTip(
            "Optimize memory usage at the cost of slightly lower performance\n"
            "Useful for systems with limited VRAM"
        )
        
        options_layout.addWidget(self.use_tensor_cores)
        options_layout.addWidget(self.enable_interpolation)
        options_layout.addWidget(self.auto_optimize)
        options_layout.addWidget(self.reduce_memory)
        
        form_layout.addRow("", options_layout)
        
        container_layout.addWidget(form_frame)
        
        # Advanced settings button
        self.advanced_btn = QPushButton("Advanced Interpolation Settings...")
        self.advanced_btn.setObjectName("accentButton")
        self.advanced_btn.setToolTip("Configure detailed interpolation parameters")
        self.advanced_btn.clicked.connect(self.advancedRequested.emit)
        self.advanced_btn.setIcon(self._create_icon("settings"))
        
        # Apply button
        self.apply_btn = QPushButton("Apply Settings")
        self.apply_btn.setToolTip("Apply current settings to the upscaler")
        self.apply_btn.clicked.connect(self.apply_settings)
        self.apply_btn.setIcon(self._create_icon("check"))
        
        # Add buttons to main layout
        button_layout = QHBoxLayout()
        button_layout.addWidget(self.advanced_btn)
        button_layout.addWidget(self.apply_btn)
        
        container_layout.addLayout(button_layout)
        
        # Add stretch to push everything to the top
        container_layout.addStretch(1)
        
        # Add container to scroll area
        scroll_area.setWidget(container)
        main_layout.addWidget(scroll_area)
        
        # Enable/disable logic
        self.method_combo.currentTextChanged.connect(self.updateControlState)
        self.enable_interpolation.toggled.connect(self.updateControlState)
        self.auto_optimize.toggled.connect(self.updateControlState)
        
        # Connect signals for settings changes
        self.method_combo.currentTextChanged.connect(lambda: self.on_setting_changed("method"))
        self.quality_combo.currentTextChanged.connect(lambda: self.on_setting_changed("quality"))
        self.scale_slider.valueChanged.connect(lambda: self.on_setting_changed("scale"))
        self.batch_size_spin.valueChanged.connect(lambda: self.on_setting_changed("batch_size"))
        self.use_tensor_cores.toggled.connect(lambda: self.on_setting_changed("use_tensor_cores"))
        self.enable_interpolation.toggled.connect(lambda: self.on_setting_changed("enable_interpolation"))
        self.auto_optimize.toggled.connect(lambda: self.on_setting_changed("auto_optimize"))
        self.reduce_memory.toggled.connect(lambda: self.on_setting_changed("reduce_memory"))
        
        # Profile selection changes
        self.profile_combo.currentTextChanged.connect(self.load_profile)
        
        # Initial state update
        self.updateControlState()
    
    def _create_icon(self, icon_type):
        """Create SVG icons programmatically"""
        svg_icons = {
            "settings": """
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#E0E1DD" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="3"></circle>
                    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
                </svg>
            """,
            "check": """
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#E0E1DD" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="20 6 9 17 4 12"></polyline>
                </svg>
            """
        }
        
        if icon_type in svg_icons:
            import tempfile
            
            temp = tempfile.NamedTemporaryFile(suffix='.svg', delete=False)
            temp.write(svg_icons[icon_type].encode('utf-8'))
            temp.close()
            
            file_path = temp.name
            return QIcon(file_path)
        
        return QIcon()
    
    def update_scale_label(self, value=None):
        """Update the scale label with the current slider value and animation"""
        if value is None:
            value = self.scale_slider.value()
            
        scale_value = value / 10.0
        
        # Update the label with a smooth color transition based on value
        # Higher values get a more prominent color
        color_intensity = min(1.0, (scale_value - 1.0) / 3.0 * 1.5)
        
        # Interpolate between text color and accent color
        r1, g1, b1 = int(COLORS["text_light"][1:3], 16), int(COLORS["text_light"][3:5], 16), int(COLORS["text_light"][5:7], 16)
        r2, g2, b2 = int(COLORS["accent_secondary"][1:3], 16), int(COLORS["accent_secondary"][3:5], 16), int(COLORS["accent_secondary"][5:7], 16)
        
        r = int(r1 + (r2 - r1) * color_intensity)
        g = int(g1 + (g2 - g1) * color_intensity)
        b = int(b1 + (b2 - b1) * color_intensity)
        
        color_hex = f"#{r:02x}{g:02x}{b:02x}"
        
        # Create an emphasized style for higher values
        font_size = 10 + min(4, (scale_value - 1.0) * 2)
        
        self.scale_value_label.setStyleSheet(f"""
            color: {color_hex};
            font-weight: {FONTS['weight_bold']};
            font-size: {font_size}pt;
        """)
        
        self.scale_value_label.setText(f"{scale_value:.1f}×")
    
    def updateControlState(self):
        """Update enabled/disabled state of controls based on current selections"""
        # Tensor cores only available with DLSS
        is_dlss = self.method_combo.currentText() == "DLSS"
        self.use_tensor_cores.setEnabled(is_dlss)
        
        # Advanced interpolation button only enabled if interpolation is enabled
        self.advanced_btn.setEnabled(self.enable_interpolation.isChecked())
        
        # If auto-optimize is checked, disable some manual settings
        auto_optimize = self.auto_optimize.isChecked()
        self.batch_size_spin.setEnabled(not auto_optimize)
        
        # Update apply button style based on whether settings have changed
        self.apply_btn.setStyleSheet(f"""
            background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                        stop:0 {COLORS['accent_secondary']},
                        stop:1 {COLORS['secondary_pressed']});
        """)
    
    def on_setting_changed(self, setting_name):
        """Called when any setting is changed"""
        # Update control states
        self.updateControlState()
        
        # You can implement additional logic here if needed
        # For now, we don't emit settingsChanged on every change,
        # but wait for the Apply button to be clicked
    
    def apply_settings(self):
        """Apply the current settings and emit the settingsChanged signal"""
        settings = self.get_current_settings()
        
        # Provide visual feedback
        self.apply_btn.setEnabled(False)
        self.apply_btn.setText("Applied!")
        
        # Reset button after delay
        QTimer.singleShot(1000, lambda: self.apply_btn.setText("Apply Settings"))
        QTimer.singleShot(1000, lambda: self.apply_btn.setEnabled(True))
        
        # Emit signal with settings
        self.settingsChanged.emit(settings)
    
    def get_current_settings(self):
        """Get the current settings as a dictionary"""
        return {
            "method": self.method_combo.currentText(),
            "quality": self.quality_combo.currentText(),
            "scale": self.scale_slider.value() / 10.0,
            "batch_size": self.batch_size_spin.value(),
            "use_tensor_cores": self.use_tensor_cores.isChecked(),
            "enable_interpolation": self.enable_interpolation.isChecked(),
            "auto_optimize": self.auto_optimize.isChecked(),
            "reduce_memory": self.reduce_memory.isChecked()
        }
    
    def load_default_profiles(self):
        """Load default profiles"""
        self.profiles = {
            "Gaming": {
                "method": "DLSS" if hasattr(self.method_combo, "_items") and "DLSS" in self.method_combo._items else "WGPU Bilinear",
                "quality": "Performance",
                "scale": 2.0,
                "batch_size": 1,
                "use_tensor_cores": True,
                "enable_interpolation": False,
                "auto_optimize": True,
                "reduce_memory": False
            },
            "Video": {
                "method": "WGPU Bilinear",
                "quality": "Quality",
                "scale": 1.5,
                "batch_size": 4,
                "use_tensor_cores": False,
                "enable_interpolation": True,
                "auto_optimize": True,
                "reduce_memory": False
            },
            "Low-End Hardware": {
                "method": "WGPU Bilinear",
                "quality": "Performance",
                "scale": 1.3,
                "batch_size": 1,
                "use_tensor_cores": False,
                "enable_interpolation": False,
                "auto_optimize": True,
                "reduce_memory": True
            },
            "Max Quality": {
                "method": "WGPU Bilinear",
                "quality": "Ultra",
                "scale": 2.5,
                "batch_size": 1,
                "use_tensor_cores": True,
                "enable_interpolation": True,
                "auto_optimize": False,
                "reduce_memory": False
            }
        }
        
        # Populate the combobox
        self.profile_combo.clear()
        self.profile_combo.addItem("Custom")
        for profile_name in self.profiles.keys():
            self.profile_combo.addItem(profile_name)
    
    def load_profile(self, profile_name):
        """Load a profile and apply its settings"""
        if profile_name == "Custom" or profile_name not in self.profiles:
            return
            
        profile = self.profiles[profile_name]
        
        # Block signals during updates to avoid triggering change events
        self.blockSignals(True)
        
        # Update UI controls
        self.method_combo.setCurrentText(profile["method"])
        self.quality_combo.setCurrentText(profile["quality"])
        self.scale_slider.setValue(int(profile["scale"] * 10))
        self.update_scale_label()
        self.batch_size_spin.setValue(profile["batch_size"])
        self.use_tensor_cores.setChecked(profile["use_tensor_cores"])
        self.enable_interpolation.setChecked(profile["enable_interpolation"])
        self.auto_optimize.setChecked(profile["auto_optimize"])
        self.reduce_memory.setChecked(profile["reduce_memory"])
        
        # Unblock signals
        self.blockSignals(False)
        
        # Update control states
        self.updateControlState()
        
        # Emit signal
        self.profileSelected.emit(profile_name)
    
    def save_current_profile(self):
        """Save current settings as a new profile"""
        from PySide6.QtWidgets import QInputDialog
        
        profile_name, ok = QInputDialog.getText(
            self, 
            "Save Profile", 
            "Profile name:", 
            text="My Profile"
        )
        
        if ok and profile_name:
            # Save current settings
            self.profiles[profile_name] = self.get_current_settings()
            
            # Add to combo box if not exists
            if self.profile_combo.findText(profile_name) == -1:
                self.profile_combo.addItem(profile_name)
            
            # Select the new profile
            self.profile_combo.setCurrentText(profile_name)
    
    def setup_shortcuts(self):
        """Setup keyboard shortcuts"""
        # Apply settings with Ctrl+Enter
        apply_shortcut = QShortcut(QKeySequence("Ctrl+Return"), self)
        apply_shortcut.activated.connect(self.apply_settings)
        
        # Reset scale to 1.0x with Ctrl+1
        scale_10_shortcut = QShortcut(QKeySequence("Ctrl+1"), self)
        scale_10_shortcut.activated.connect(lambda: self.scale_slider.setValue(10))
        
        # Set scale to 2.0x with Ctrl+2
        scale_20_shortcut = QShortcut(QKeySequence("Ctrl+2"), self)
        scale_20_shortcut.activated.connect(lambda: self.scale_slider.setValue(20))
        
        # Set scale to 3.0x with Ctrl+3
        scale_30_shortcut = QShortcut(QKeySequence("Ctrl+3"), self)
        scale_30_shortcut.activated.connect(lambda: self.scale_slider.setValue(30))
        
        # Set scale to 4.0x with Ctrl+4
        scale_40_shortcut = QShortcut(QKeySequence("Ctrl+4"), self)
        scale_40_shortcut.activated.connect(lambda: self.scale_slider.setValue(40))

class InterpolationDialog(QDialog):
    """
    Enhanced modal dialog for advanced interpolation settings with
    animated sliders, dynamic previews, and responsive layout.
    """
    
    # Signal emitted when settings are applied
    settingsApplied = Signal(dict)
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Advanced Interpolation Settings")
        self.setMinimumSize(500, 500)
        
        # Track whether settings have changed
        self.settings_changed = False
        
        # Initialize the UI
        self.initUI()
        
        # Apply window effects
        self.apply_effects()
    
    def apply_effects(self):
        """Apply visual effects to the dialog"""
        # Apply shadow effect to the dialog
        shadow = QGraphicsDropShadowEffect()
        shadow.setBlurRadius(30)
        shadow.setColor(QColor(0, 0, 0, 120))
        shadow.setOffset(0, 5)
        self.setGraphicsEffect(shadow)
        
    def initUI(self):
        """Initialize the user interface with an improved layout and visuals"""
        # Main layout
        layout = QVBoxLayout(self)
        layout.setSpacing(int(SPACING["xl"].replace("px", "")))
        layout.setContentsMargins(int(SPACING["xl"].replace("px", "")), 
                                int(SPACING["xl"].replace("px", "")),
                                int(SPACING["xl"].replace("px", "")),
                                int(SPACING["xl"].replace("px", "")))
        
        # Title with icon
        title_layout = QHBoxLayout()
        
        title_icon = QLabel()
        title_icon.setPixmap(self._get_icon_pixmap("interpolation"))
        title_icon.setFixedSize(32, 32)
        
        title = QLabel("Advanced Interpolation Settings")
        title.setStyleSheet(f"""
            font-size: {FONTS["size_xlarge"]};
            font-weight: {FONTS["weight_bold"]};
            color: {COLORS["text_light"]};
        """)
        
        title_layout.addWidget(title_icon)
        title_layout.addWidget(title)
        title_layout.addStretch()
        
        layout.addLayout(title_layout)
        
        # Description text
        description = QLabel(
            "These settings control how intermediate frames are generated when "
            "frame interpolation is enabled. Adjust the parameters to balance "
            "between smooth motion and visual artifacts."
        )
        description.setWordWrap(True)
        description.setStyleSheet(f"""
            color: {COLORS["text_medium"]};
            font-size: {FONTS["size_normal"]};
            margin-bottom: 10px;
        """)
        layout.addWidget(description)
        
        # Tab widget for different settings categories
        tabs = QTabWidget()
        tabs.setStyleSheet(f"""
            QTabWidget::pane {{
                border: 1px solid {COLORS["border"]};
                border-radius: {EFFECTS["border_radius_md"]};
                background-color: {COLORS["surface"]};
            }}
        """)
        
        # === Motion Tab ===
        motion_tab = QWidget()
        motion_layout = QVBoxLayout(motion_tab)
        motion_layout.setSpacing(int(SPACING["lg"].replace("px", "")))
        
        # Motion Sensitivity slider with improved visuals and feedback
        motion_frame = self._create_slider_frame(
            "Motion Sensitivity",
            "Controls how aggressively the algorithm detects motion. Higher values capture more subtle movements.",
            0, 100, 50,
            self._update_motion_label
        )
        self.motion_slider = motion_frame.findChild(QSlider)
        self.motion_label = motion_frame.findChild(QLabel, "valueLabel")
        
        # Detail Preservation slider
        detail_frame = self._create_slider_frame(
            "Detail Preservation",
            "Preserves fine details in moving areas. Higher values retain more details but may increase artifacts.",
            0, 100, 75,
            self._update_detail_label
        )
        self.detail_slider = detail_frame.findChild(QSlider)
        self.detail_label = detail_frame.findChild(QLabel, "valueLabel")
        
        # Artifact Reduction slider
        artifact_frame = self._create_slider_frame(
            "Artifact Reduction",
            "Reduces visual glitches in interpolated frames. Higher values are smoother but may blur details.",
            0, 100, 25,
            self._update_artifact_label
        )
        self.artifact_slider = artifact_frame.findChild(QSlider)
        self.artifact_label = artifact_frame.findChild(QLabel, "valueLabel")
        
        # Add to motion tab
        motion_layout.addWidget(motion_frame)
        motion_layout.addWidget(detail_frame)
        motion_layout.addWidget(artifact_frame)
        motion_layout.addStretch(1)
        
        # === Shader Tab ===
        shader_tab = QWidget()
        shader_layout = QVBoxLayout(shader_tab)
        shader_layout.setSpacing(int(SPACING["lg"].replace("px", "")))
        
        # Create a group for shader selection
        shader_frame = QFrame()
        shader_frame.setStyleSheet(f"""
            QFrame {{
                background-color: {COLORS["background_medium"]};
                border-radius: {EFFECTS["border_radius_md"]};
                padding: 15px;
            }}
        """)
        
        shader_content_layout = QVBoxLayout(shader_frame)
        
        shader_title = QLabel("Interpolation Shader")
        shader_title.setStyleSheet(f"""
            font-weight: {FONTS["weight_bold"]};
            font-size: {FONTS["size_medium"]};
            color: {COLORS["text_light"]};
            margin-bottom: 5px;
        """)
        shader_content_layout.addWidget(shader_title)
        
        shader_desc = QLabel(
            "Select the algorithm used for generating intermediate frames. "
            "Different shaders are optimized for specific types of content."
        )
        shader_desc.setWordWrap(True)
        shader_desc.setStyleSheet(f"color: {COLORS['text_medium']};")
        shader_content_layout.addWidget(shader_desc)
        
        # Radio buttons for shader selection with improved layout and icons
        self.shader_group = QButtonGroup(self)
        
        # Optical Flow option
        optical_layout = QHBoxLayout()
        optical_icon = QLabel()
        optical_icon.setPixmap(self._get_icon_pixmap("video"))
        optical_icon.setFixedSize(24, 24)
        
        self.optical_flow_radio = QRadioButton("Optical Flow")
        self.optical_flow_radio.setStyleSheet(f"""
            QRadioButton {{
                font-weight: {FONTS["weight_medium"]};
            }}
        """)
        
        optical_desc = QLabel("Best for video content. Analyzes motion between frames with high accuracy.")
        optical_desc.setWordWrap(True)
        optical_desc.setStyleSheet(f"color: {COLORS['text_medium']}; font-size: {FONTS['size_small']};")
        
        optical_layout.addWidget(optical_icon)
        optical_layout.addWidget(self.optical_flow_radio)
        optical_layout.addStretch()
        
        optical_container = QFrame()
        optical_container_layout = QVBoxLayout(optical_container)
        optical_container_layout.setContentsMargins(0, 0, 0, 0)
        optical_container_layout.addLayout(optical_layout)
        optical_container_layout.addWidget(optical_desc)
        
        # RIFE option
        rife_layout = QHBoxLayout()
        rife_icon = QLabel()
        rife_icon.setPixmap(self._get_icon_pixmap("gaming"))
        rife_icon.setFixedSize(24, 24)
        
        self.rife_radio = QRadioButton("RIFE")
        self.rife_radio.setStyleSheet(f"""
            QRadioButton {{
                font-weight: {FONTS["weight_medium"]};
            }}
        """)
        
        rife_desc = QLabel("Best for gaming and CGI. Uses neural network for faster processing with good quality.")
        rife_desc.setWordWrap(True)
        rife_desc.setStyleSheet(f"color: {COLORS['text_medium']}; font-size: {FONTS['size_small']};")
        
        rife_layout.addWidget(rife_icon)
        rife_layout.addWidget(self.rife_radio)
        rife_layout.addStretch()
        
        rife_container = QFrame()
        rife_container_layout = QVBoxLayout(rife_container)
        rife_container_layout.setContentsMargins(0, 0, 0, 0)
        rife_container_layout.addLayout(rife_layout)
        rife_container_layout.addWidget(rife_desc)
        
        # Blend option
        blend_layout = QHBoxLayout()
        blend_icon = QLabel()
        blend_icon.setPixmap(self._get_icon_pixmap("blend"))
        blend_icon.setFixedSize(24, 24)
        
        self.blend_radio = QRadioButton("Simple Blend")
        self.blend_radio.setStyleSheet(f"""
            QRadioButton {{
                font-weight: {FONTS["weight_medium"]};
            }}
        """)
        
        blend_desc = QLabel("Lowest GPU usage. Basic frame blending suitable for static content.")
        blend_desc.setWordWrap(True)
        blend_desc.setStyleSheet(f"color: {COLORS['text_medium']}; font-size: {FONTS['size_small']};")
        
        blend_layout.addWidget(blend_icon)
        blend_layout.addWidget(self.blend_radio)
        blend_layout.addStretch()
        
        blend_container = QFrame()
        blend_container_layout = QVBoxLayout(blend_container)
        blend_container_layout.setContentsMargins(0, 0, 0, 0)
        blend_container_layout.addLayout(blend_layout)
        blend_container_layout.addWidget(blend_desc)
        
        # Add to button group
        self.shader_group.addButton(self.optical_flow_radio, 1)
        self.shader_group.addButton(self.rife_radio, 2)
        self.shader_group.addButton(self.blend_radio, 3)
        
        # Default selection
        self.optical_flow_radio.setChecked(True)
        
        # Add radio containers with spacing
        radio_layout = QVBoxLayout()
        radio_layout.setSpacing(15)
        radio_layout.addWidget(optical_container)
        radio_layout.addWidget(rife_container)
        radio_layout.addWidget(blend_container)
        
        shader_content_layout.addLayout(radio_layout)
        shader_layout.addWidget(shader_frame)
        
        # Frame generation options
        frames_frame = QFrame()
        frames_frame.setStyleSheet(f"""
            QFrame {{
                background-color: {COLORS["background_medium"]};
                border-radius: {EFFECTS["border_radius_md"]};
                padding: 15px;
            }}
        """)
        
        frames_layout = QVBoxLayout(frames_frame)
        
        frames_title = QLabel("Frame Generation")
        frames_title.setStyleSheet(f"""
            font-weight: {FONTS["weight_bold"]};
            font-size: {FONTS["size_medium"]};
            color: {COLORS["text_light"]};
            margin-bottom: 5px;
        """)
        frames_layout.addWidget(frames_title)
        
        # Frame multiplier slider
        self.frame_multi_slider = QSlider(Qt.Horizontal)
        self.frame_multi_slider.setRange(1, 4)  # 1x to 4x frames
        self.frame_multi_slider.setValue(2)     # Default 2x (doubles frames)
        self.frame_multi_slider.setTickPosition(QSlider.TicksBelow)
        self.frame_multi_slider.setTickInterval(1)
        
        self.frame_multi_label = QLabel("2× (60 → 120 FPS)")
        self.frame_multi_label.setAlignment(Qt.AlignCenter)
        self.frame_multi_label.setStyleSheet(f"""
            color: {COLORS["accent_secondary"]};
            font-weight: {FONTS["weight_bold"]};
        """)
        
        self.frame_multi_slider.valueChanged.connect(self._update_frame_multi_label)
        
        frames_layout.addWidget(self.frame_multi_label)
        frames_layout.addWidget(self.frame_multi_slider)
        
        # Add tick labels
        ticks_layout = QHBoxLayout()
        ticks_layout.setContentsMargins(0, 0, 0, 0)
        
        for i, label in enumerate(["1× (Original)", "2× (Double)", "3× (Triple)", "4× (Quadruple)"]):
            tick_label = QLabel(label)
            tick_label.setStyleSheet(f"color: {COLORS['text_medium']}; font-size: {FONTS['size_small']};")
            tick_label.setAlignment(Qt.AlignCenter)
            ticks_layout.addWidget(tick_label)
        
        frames_layout.addLayout(ticks_layout)
        
        shader_layout.addWidget(frames_frame)
        shader_layout.addStretch(1)
        
        # Add tabs
        tabs.addTab(motion_tab, "Motion Control")
        tabs.addTab(shader_tab, "Shader & Frames")
        
        layout.addWidget(tabs)
        
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
    """
    Enhanced widget for graphics & compute controls with data visualization,
    performance monitoring, and interactive viewport.
    """
    
    # Signals
    debugViewToggled = Signal(bool)
    performanceViewToggled = Signal(bool)
    exportRequested = Signal()
    resetRequested = Signal()
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.initUI()
        
        # Initialize performance data
        self.performance_data = {
            "gpu_usage": [0] * 60,
            "fps": [0] * 60,
            "vram": [0] * 60
        }
        
        # Start update timer for animated display
        self.update_timer = QTimer(self)
        self.update_timer.timeout.connect(self.updateViewport)
        self.update_timer.start(1000 // 30)  # 30 FPS updates
    
    def initUI(self):
        """Initialize the user interface with improved layout and graphics"""
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(int(SPACING["md"].replace("px", "")), 
                                    int(SPACING["md"].replace("px", "")),
                                    int(SPACING["md"].replace("px", "")),
                                    int(SPACING["md"].replace("px", "")))
        main_layout.setSpacing(int(SPACING["md"].replace("px", "")))
        
        # Header area with title and view options
        header_layout = QHBoxLayout()
        
        title = QLabel("Graphics & Compute Controls")
        title.setObjectName("title")
        title.setAlignment(Qt.AlignLeft | Qt.AlignVCenter)
        
        # View mode tabs
        view_tabs = QTabWidget()
        view_tabs.setObjectName("viewTabs")
        view_tabs.setStyleSheet(f"""
            QTabWidget#viewTabs {{
                min-height: 150px;
            }}
            QTabWidget::pane {{
                border: none;
            }}
        """)
        
        # Main viewport
        self.viewport = QGraphicsView()
        self.viewport.setRenderHint(QPainter.Antialiasing)
        self.viewport.setRenderHint(QPainter.SmoothPixmapTransform)
        self.viewport.setFrameShape(QFrame.NoFrame)
        self.viewport.setStyleSheet(f"""
            background-color: {COLORS["background_dark"]};
            border-radius: {EFFECTS["border_radius_md"]};
        """)
        
        self.scene = QGraphicsScene()
        self.scene.setBackgroundBrush(QBrush(QColor(COLORS["background_dark"])))
        self.viewport.setScene(self.scene)
        
        # Debug view
        self.debug_view = QGraphicsView()
        self.debug_view.setRenderHint(QPainter.Antialiasing)
        self.debug_view.setFrameShape(QFrame.NoFrame)
        self.debug_view.setStyleSheet(f"""
            background-color: {COLORS["background_dark"]};
            border-radius: {EFFECTS["border_radius_md"]};
        """)
        
        self.debug_scene = QGraphicsScene()
        self.debug_scene.setBackgroundBrush(QBrush(QColor(COLORS["background_dark"])))
        self.debug_view.setScene(self.debug_scene)
        
        # Performance view
        self.perf_view = QGraphicsView()
        self.perf_view.setRenderHint(QPainter.Antialiasing)
        self.perf_view.setFrameShape(QFrame.NoFrame)
        self.perf_view.setStyleSheet(f"""
            background-color: {COLORS["background_dark"]};
            border-radius: {EFFECTS["border_radius_md"]};
        """)
        
        self.perf_scene = QGraphicsScene()
        self.perf_scene.setBackgroundBrush(QBrush(QColor(COLORS["background_dark"])))
        self.perf_view.setScene(self.perf_scene)
        
        # Add views to tabs
        view_tabs.addTab(self.viewport, "Standard View")
        view_tabs.addTab(self.debug_view, "Debug View")
        view_tabs.addTab(self.perf_view, "Performance")
        
        # Add shadow effect to the viewport tabs
        shadow = QGraphicsDropShadowEffect()
        shadow.setBlurRadius(20)
        shadow.setColor(QColor(0, 0, 0, 100))
        shadow.setOffset(0, 5)
        view_tabs.setGraphicsEffect(shadow)
        
        header_layout.addWidget(title)
        
        # Add the header and viewport to main layout
        main_layout.addLayout(header_layout)
        main_layout.addWidget(view_tabs, 1)  # 1 = stretch factor
        
        # Toolbar with stylized buttons
        toolbar_frame = QFrame()
        toolbar_frame.setStyleSheet(f"""
            QFrame {{
                background-color: {COLORS["background_medium"]};
                border-radius: {EFFECTS["border_radius_md"]};
                padding: 4px;
            }}
        """)
        
        toolbar_layout = QHBoxLayout(toolbar_frame)
        toolbar_layout.setContentsMargins(int(SPACING["sm"].replace("px", "")), 
                                        int(SPACING["sm"].replace("px", "")),
                                        int(SPACING["sm"].replace("px", "")),
                                        int(SPACING["sm"].replace("px", "")))
        toolbar_layout.setSpacing(int(SPACING["sm"].replace("px", "")))
        
        # Toolbar buttons with icons
        self.debug_btn = self._create_tool_button("Debug View", "debug")
        self.debug_btn.setCheckable(True)
        self.debug_btn.clicked.connect(lambda checked: self.debugViewToggled.emit(checked))
        
        self.performance_btn = self._create_tool_button("Performance", "chart")
        self.performance_btn.setCheckable(True)
        self.performance_btn.clicked.connect(lambda checked: self.performanceViewToggled.emit(checked))
        
        self.export_btn = self._create_tool_button("Export", "export")
        self.export_btn.setObjectName("accentButton")
        self.export_btn.clicked.connect(self.exportRequested.emit)
        
        self.reset_btn = self._create_tool_button("Reset", "reset")
        self.reset_btn.clicked.connect(self.resetRequested.emit)
        
        # GPU Usage indicator
        self.gpu_indicator = QProgressBar()
        self.gpu_indicator.setRange(0, 100)
        self.gpu_indicator.setValue(0)
        self.gpu_indicator.setFormat("GPU: %p%")
        self.gpu_indicator.setTextVisible(True)
        self.gpu_indicator.setFixedWidth(120)
        self.gpu_indicator.setToolTip("Current GPU utilization")
        
        # FPS counter
        self.fps_label = QLabel("60 FPS")
        self.fps_label.setStyleSheet(f"""
            background-color: {COLORS["background_dark"]};
            color: {COLORS["accent_primary"]};
            padding: 4px 8px;
            border-radius: {EFFECTS["border_radius_sm"]};
            font-weight: {FONTS["weight_bold"]};
        """)
        self.fps_label.setFixedWidth(70)
        self.fps_label.setAlignment(Qt.AlignCenter)
        self.fps_label.setToolTip("Current frames per second")
        
        toolbar_layout.addWidget(self.debug_btn)
        toolbar_layout.addWidget(self.performance_btn)
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

class MainWindow(QMainWindow):
    """
    Enhanced main application window with improved layouts, animations,
    and professional visual styling.
    """
    
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Nu_Scaler")
        self.resize(1280, 800)
        
        # Track app state
        self.processing_active = False
        self.current_file = None
        
        # Initialize UI
        self.initUI()
        
        # Set up timers
        self.setupTimers()
        
        # Load initial settings
        self.loadSettings()
        
        # Set up keyboard shortcuts
        self.setupShortcuts()
        
        # Show a welcome message
        self.showWelcomeMessage()
    
    def initUI(self):
        """Initialize the user interface with improved layout and components"""
        # Create menu bar with standard menus
        self.createMenus()
        
        # Set central widget
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        
        # Main layout with proper spacing
        main_layout = QVBoxLayout(central_widget)
        main_layout.setContentsMargins(int(SPACING["md"].replace("px", "")), 
                                     int(SPACING["md"].replace("px", "")),
                                     int(SPACING["md"].replace("px", "")),
                                     int(SPACING["md"].replace("px", "")))
        main_layout.setSpacing(int(SPACING["md"].replace("px", "")))
        
        # Create splitter for preview panes
        splitter = QSplitter(Qt.Horizontal)
        splitter.setHandleWidth(1)
        splitter.setChildrenCollapsible(False)
        
        # Create preview panes with better styling and drop shadows
        self.original_pane = PreviewPane("Original Input")
        self.processed_pane = PreviewPane("Processed Output")
        
        # Add panes to splitter
        splitter.addWidget(self.original_pane)
        splitter.addWidget(self.processed_pane)
        splitter.setSizes([int(self.width()/2), int(self.width()/2)])  # Equal initial size
        
        # Add splitter to main layout with stretch factor
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
        
        # Create enhanced status bar
        self.createStatusBar()
        
        # Connect signals for better interactivity
        self.connectSignals()
    
    def createMenus(self):
        """Create application menus with standard options and keyboard shortcuts"""
        # Main menu bar
        menu_bar = self.menuBar()
        
        # File menu
        file_menu = menu_bar.addMenu("&File")
        
        # Open action
        open_action = QAction("&Open Image/Video...", self)
        open_action.setShortcut(QKeySequence.Open)
        open_action.triggered.connect(self.openFile)
        file_menu.addAction(open_action)
        
        # Save result action
        save_action = QAction("&Save Processed Result...", self)
        save_action.setShortcut(QKeySequence.Save)
        save_action.triggered.connect(self.saveResult)
        save_action.setEnabled(False)  # Initially disabled
        self.save_action = save_action  # Store reference to update enabled state
        file_menu.addAction(save_action)
        
        file_menu.addSeparator()
        
        # Settings submenu
        settings_submenu = file_menu.addMenu("Settings")
        
        # Load settings action
        load_settings_action = QAction("Load Settings...", self)
        load_settings_action.triggered.connect(self.loadSettingsFromFile)
        settings_submenu.addAction(load_settings_action)
        
        # Save settings action
        save_settings_action = QAction("Save Settings...", self)
        save_settings_action.triggered.connect(self.saveSettingsToFile)
        settings_submenu.addAction(save_settings_action)
        
        file_menu.addSeparator()
        
        # Exit action
        exit_action = QAction("E&xit", self)
        exit_action.setShortcut(QKeySequence.Quit)
        exit_action.triggered.connect(self.close)
        file_menu.addAction(exit_action)
        
        # Edit menu
        edit_menu = menu_bar.addMenu("&Edit")
        
        # Copy action
        copy_action = QAction("&Copy Processed Image", self)
        copy_action.setShortcut(QKeySequence.Copy)
        copy_action.triggered.connect(self.copyToClipboard)
        copy_action.setEnabled(False)  # Initially disabled
        self.copy_action = copy_action  # Store reference to update enabled state
        edit_menu.addAction(copy_action)
        
        # View menu
        view_menu = menu_bar.addMenu("&View")
        
        # Toggle settings panel action
        toggle_settings_action = QAction("&Settings Panel", self)
        toggle_settings_action.setCheckable(True)
        toggle_settings_action.setChecked(True)
        toggle_settings_action.triggered.connect(self.toggleSettingsPanel)
        view_menu.addAction(toggle_settings_action)
        
        # Reset layout action
        reset_layout_action = QAction("Reset &Layout", self)
        reset_layout_action.triggered.connect(self.resetLayout)
        view_menu.addAction(reset_layout_action)
        
        # Processing menu
        processing_menu = menu_bar.addMenu("&Processing")
        
        # Start/stop processing action
        self.toggle_processing_action = QAction("&Start Processing", self)
        self.toggle_processing_action.setShortcut("F5")
        self.toggle_processing_action.triggered.connect(self.toggleProcessing)
        processing_menu.addAction(self.toggle_processing_action)
        
        # Advanced interpolation action
        advanced_action = QAction("Advanced &Interpolation Settings...", self)
        advanced_action.triggered.connect(self.showInterpolationDialog)
        processing_menu.addAction(advanced_action)
        
        # Help menu
        help_menu = menu_bar.addMenu("&Help")
        
        # About action
        about_action = QAction("&About Nu_Scaler", self)
        about_action.triggered.connect(self.showAboutDialog)
        help_menu.addAction(about_action)
        
        # Documentation action
        docs_action = QAction("&Documentation", self)
        docs_action.triggered.connect(self.openDocumentation)
        help_menu.addAction(docs_action)
    
    def createStatusBar(self):
        """Create an enhanced status bar with animated indicators"""
        # Create custom status bar with better styling
        self.statusBar = QStatusBar()
        self.statusBar.setStyleSheet(f"""
            QStatusBar {{
                background-color: {COLORS["background_medium"]};
                border-top: 1px solid {COLORS["border"]};
                padding: 2px;
            }}
        """)
        self.setStatusBar(self.statusBar)
        
        # Add status indicators with improved styling
        # FPS indicator
        self.fps_label = QLabel("FPS: 0")
        self.fps_label.setObjectName("statusLabel")
        self.fps_label.setToolTip("Current processing frame rate")
        
        # GPU usage indicator
        gpu_layout = QHBoxLayout()
        gpu_layout.setContentsMargins(0, 0, 0, 0)
        gpu_layout.setSpacing(4)
        
        gpu_text = QLabel("GPU:")
        
        self.gpu_progress = QProgressBar()
        self.gpu_progress.setRange(0, 100)
        self.gpu_progress.setValue(0)
        self.gpu_progress.setFormat("%p%")
        self.gpu_progress.setTextVisible(True)
        self.gpu_progress.setMinimumWidth(80)
        self.gpu_progress.setMaximumWidth(100)
        self.gpu_progress.setMaximumHeight(16)
        self.gpu_progress.setToolTip("Current GPU utilization")
        
        gpu_container = QWidget()
        gpu_layout.addWidget(gpu_text)
        gpu_layout.addWidget(self.gpu_progress)
        gpu_container.setLayout(gpu_layout)
        
        # Process time indicator
        self.time_label = QLabel("Process: 0ms")
        self.time_label.setObjectName("statusLabel")
        self.time_label.setToolTip("Frame processing time")
        
        # Overall progress bar
        self.progress_bar = QProgressBar()
        self.progress_bar.setRange(0, 100)
        self.progress_bar.setValue(0)
        self.progress_bar.setFormat("%p%")
        self.progress_bar.setTextVisible(True)
        self.progress_bar.setMaximumWidth(150)
        self.progress_bar.setMaximumHeight(16)
        self.progress_bar.setToolTip("Overall processing progress")
        
        # Status message label
        self.status_label = QLabel("Ready")
        self.status_label.setStyleSheet(f"color: {COLORS['text_light']};")
        
        # Add widgets to status bar
        self.statusBar.addWidget(self.fps_label)
        self.statusBar.addWidget(gpu_container)
        self.statusBar.addWidget(self.time_label)
        self.statusBar.addWidget(self.progress_bar)
        self.statusBar.addPermanentWidget(self.status_label, 1)  # Stretch=1, align right
    
    def connectSignals(self):
        """Connect signals to slots for all UI components"""
        # Settings panel signals
        self.settings_panel.advancedRequested.connect(self.showInterpolationDialog)
        self.settings_panel.settingsChanged.connect(self.onSettingsChanged)
        self.settings_panel.profileSelected.connect(self.onProfileSelected)
        
        # Interpolation dialog signals
        self.interpolation_dialog.settingsApplied.connect(self.onInterpolationSettingsApplied)
        
        # Preview pane signals
        self.original_pane.fileDropped.connect(self.onFileDropped)
        self.original_pane.fileSelected.connect(self.onFileSelected)
        
        # Compute pane signals
        self.compute_pane.debugViewToggled.connect(self.onDebugViewToggled)
        self.compute_pane.performanceViewToggled.connect(self.onPerformanceViewToggled)
        self.compute_pane.exportRequested.connect(self.saveResult)
        self.compute_pane.resetRequested.connect(self.resetProcessing)
    
    def setupTimers(self):
        """Set up timers for UI updates and animations"""
        # Status bar update timer
        self.status_timer = QTimer(self)
        self.status_timer.timeout.connect(self.updateStatusBar)
        self.status_timer.start(500)  # Update every 500ms
        
        # Processing simulation timer (for demo purposes)
        self.process_timer = QTimer(self)
        self.process_timer.timeout.connect(self.simulateProcessing)
    
    def setupShortcuts(self):
        """Set up keyboard shortcuts for the application"""
        # F11 for full screen toggle
        fullscreen_shortcut = QShortcut(QKeySequence("F11"), self)
        fullscreen_shortcut.activated.connect(self.toggleFullScreen)
        
        # Ctrl+R to reset processing
        reset_shortcut = QShortcut(QKeySequence("Ctrl+R"), self)
        reset_shortcut.activated.connect(self.resetProcessing)
        
        # Esc to exit fullscreen
        esc_shortcut = QShortcut(QKeySequence("Esc"), self)
        esc_shortcut.activated.connect(self.exitFullScreen)
    
    def loadSettings(self):
        """Load application settings"""
        # This would normally load from QSettings or a config file
        # For now, we'll just use defaults
        pass
    
    def toggleFullScreen(self):
        """Toggle fullscreen mode"""
        if self.isFullScreen():
            self.showNormal()
        else:
            self.showFullScreen()
    
    def exitFullScreen(self):
        """Exit fullscreen mode if active"""
        if self.isFullScreen():
            self.showNormal()
    
    def showInterpolationDialog(self):
        """Show the advanced interpolation settings dialog"""
        self.interpolation_dialog.exec()
    
    def onSettingsChanged(self, settings):
        """Handle changes to the settings panel"""
        # Update status
        self.status_label.setText(f"Settings applied: {settings['method']} at {settings['scale']}x scale")
        
        # Enable the processing button
        self.toggle_processing_action.setEnabled(True)
        
        # Additional processing logic would go here
    
    def onInterpolationSettingsApplied(self, settings):
        """Handle application of interpolation settings"""
        # Update status
        self.status_label.setText(f"Interpolation settings applied: {settings['shader']} shader")
        
        # Additional processing logic would go here
    
    def onProfileSelected(self, profile_name):
        """Handle selection of a preset profile"""
        self.status_label.setText(f"Profile '{profile_name}' loaded")
    
    def onFileDropped(self, file_path):
        """Handle file dropped on the original preview pane"""
        self.current_file = file_path
        self.status_label.setText(f"Loaded file: {os.path.basename(file_path)}")
        
        # Enable processing actions
        self.toggle_processing_action.setEnabled(True)
        
        # If we were processing, restart with the new file
        if self.processing_active:
            self.stopProcessing()
            self.startProcessing()
    
    def onFileSelected(self, file_path):
        """Handle file selected through dialog"""
        self.onFileDropped(file_path)
    
    def onDebugViewToggled(self, enabled):
        """Handle debug view toggle"""
        if enabled:
            self.status_label.setText("Debug view enabled")
        else:
            self.status_label.setText("Debug view disabled")
    
    def onPerformanceViewToggled(self, enabled):
        """Handle performance view toggle"""
        if enabled:
            self.status_label.setText("Performance monitoring enabled")
        else:
            self.status_label.setText("Performance monitoring disabled")
    
    def toggleSettingsPanel(self, visible):
        """Toggle the visibility of the settings panel"""
        self.settings_dock.setVisible(visible)
    
    def resetLayout(self):
        """Reset the application layout to defaults"""
        # Reset dock widgets
        self.settings_dock.setVisible(True)
        self.addDockWidget(Qt.RightDockWidgetArea, self.settings_dock)
        
        # Reset splitter proportions
        splitter = self.findChild(QSplitter)
        if splitter:
            splitter.setSizes([int(self.width()/2), int(self.width()/2)])
        
        self.status_label.setText("Layout reset to defaults")
    
    def toggleProcessing(self):
        """Toggle processing start/stop"""
        if self.processing_active:
            self.stopProcessing()
        else:
            self.startProcessing()
    
    def startProcessing(self):
        """Start the processing simulation"""
        # Update UI state
        self.processing_active = True
        self.toggle_processing_action.setText("&Stop Processing")
        self.status_label.setText("Processing started")
        
        # Start processing timer
        self.process_timer.start(100)  # Update every 100ms for simulation
        
        # Reset progress
        self.progress_bar.setValue(0)
    
    def stopProcessing(self):
        """Stop the processing simulation"""
        # Update UI state
        self.processing_active = False
        self.toggle_processing_action.setText("&Start Processing")
        self.status_label.setText("Processing stopped")
        
        # Stop processing timer
        self.process_timer.stop()
    
    def resetProcessing(self):
        """Reset the processing state"""
        # Stop processing if active
        if self.processing_active:
            self.stopProcessing()
        
        # Clear the processed pane
        self.processed_pane.setPixmap(None)
        
        # Reset progress
        self.progress_bar.setValue(0)
        
        # Update copy/save actions
        self.copy_action.setEnabled(False)
        self.save_action.setEnabled(False)
        
        self.status_label.setText("Processing reset")
    
    def simulateProcessing(self):
        """Simulate processing for demonstration purposes"""
        import random
        
        # Only proceed if we have an input image
        if not hasattr(self.original_pane, '_original_pixmap') or self.original_pane._original_pixmap is None:
            self.stopProcessing()
            return
        
        # Increment progress
        current_progress = self.progress_bar.value()
        new_progress = min(100, current_progress + random.randint(1, 5))
        self.progress_bar.setValue(new_progress)
        
        # When complete, update the processed image
        if new_progress >= 100:
            self.stopProcessing()
            self.status_label.setText("Processing completed")
            
            # Create a simulated "processed" image from the original
            if hasattr(self.original_pane, '_original_pixmap'):
                # This is where actual processing would happen
                # For demo, just create a modified copy of the original
                processed_pixmap = self.original_pane._original_pixmap.copy()
                
                # Apply a simple brightness/contrast adjustment (just for demo)
                image = processed_pixmap.toImage()
                for y in range(image.height()):
                    for x in range(image.width()):
                        color = QColor(image.pixel(x, y))
                        h, s, v, a = color.getHsv()
                        
                        # Increase saturation and value
                        new_s = min(255, int(s * 1.2))
                        new_v = min(255, int(v * 1.1))
                        
                        color.setHsv(h, new_s, new_v, a)
                        image.setPixelColor(x, y, color)
                
                processed_pixmap = QPixmap.fromImage(image)
                
                # Update the processed pane
                self.processed_pane.setPixmap(processed_pixmap)
                self.processed_pane.current_file_path = None  # Clear file path as this is a generated image
                
                # Enable copy/save actions
                self.copy_action.setEnabled(True)
                self.save_action.setEnabled(True)
    
    def updateStatusBar(self):
        """Update status bar with current values"""
        if self.processing_active:
            # Generate realistic but simulated values when processing
            import random
            import math
            import time
            
            # Sine wave variation for more realistic appearance
            t = time.time()
            
            # FPS between 45-60 with sine wave variation
            fps = 55 + 5 * math.sin(t * 2) + random.uniform(-2, 2)
            
            # GPU usage between 40-90% with sine wave variation
            gpu_usage = 65 + 20 * math.sin(t * 0.5) + random.uniform(-5, 5)
            
            # Process time 5-15ms with sine wave variation
            process_time = 10 + 5 * math.sin(t * 1.5) + random.uniform(-2, 2)
            
            self.fps_label.setText(f"FPS: {fps:.1f}")
            self.gpu_progress.setValue(int(gpu_usage))
            self.time_label.setText(f"Process: {process_time:.1f}ms")
        else:
            # Static values when not processing
            self.fps_label.setText("FPS: --")
            self.gpu_progress.setValue(0)
            self.time_label.setText("Process: --")
    
    def openFile(self):
        """Open a file through menu action"""
        file_path, _ = QFileDialog.getOpenFileName(
            self, 
            "Open Image/Video", 
            "", 
            "Images (*.png *.jpg *.jpeg *.bmp *.webp);;Videos (*.mp4 *.avi *.mov);;All Files (*)"
        )
        
        if file_path:
            self.onFileSelected(file_path)
    
    def saveResult(self):
        """Save the processed result"""
        if not hasattr(self.processed_pane, '_original_pixmap') or self.processed_pane._original_pixmap is None:
            return
            
        # Get save path from user
        suggested_name = "output.png"
        if self.current_file:
            file_info = QFileInfo(self.current_file)
            suggested_name = f"{file_info.baseName()}_processed.png"
            
        file_path, _ = QFileDialog.getSaveFileName(
            self,
            "Save Processed Image",
            suggested_name,
            "PNG Images (*.png);;JPEG Images (*.jpg *.jpeg);;All Files (*)"
        )
        
        if file_path:
            # Save the processed image
            self.processed_pane._original_pixmap.save(file_path)
            self.status_label.setText(f"Saved processed image to: {os.path.basename(file_path)}")
    
    def copyToClipboard(self):
        """Copy the processed image to clipboard"""
        if not hasattr(self.processed_pane, '_original_pixmap') or self.processed_pane._original_pixmap is None:
            return
            
        # Copy to clipboard
        from PySide6.QtGui import QGuiApplication
        clipboard = QGuiApplication.clipboard()
        clipboard.setPixmap(self.processed_pane._original_pixmap)
        
        self.status_label.setText("Copied processed image to clipboard")
    
    def loadSettingsFromFile(self):
        """Load settings from a file"""
        # This would load from a settings file
        self.status_label.setText("Settings loaded from file")
    
    def saveSettingsToFile(self):
        """Save current settings to a file"""
        # This would save to a settings file
        self.status_label.setText("Settings saved to file")
    
    def showAboutDialog(self):
        """Show the about dialog"""
        from PySide6.QtWidgets import QMessageBox
        
        QMessageBox.about(
            self,
            "About Nu_Scaler",
            f"""<h1>Nu_Scaler</h1>
            <p>Version 1.0</p>
            <p>A real-time upscaling application for video and games.</p>
            <p>© 2023 Nu_Scaler Team</p>"""
        )
    
    def openDocumentation(self):
        """Open the documentation"""
        # This would open documentation in browser
        self.status_label.setText("Opening documentation...")
    
    def showWelcomeMessage(self):
        """Show a welcome message in the status bar with animation"""
        messages = [
            "Welcome to Nu_Scaler",
            "Drag & drop an image to get started",
            "Ready"
        ]
        
        # Use a timer to show each message in sequence
        for i, message in enumerate(messages):
            QTimer.singleShot(i * 1500, lambda msg=message: self.status_label.setText(msg))

def run_gui():
    """Start the application with proper error handling and styling"""
    app = QApplication(sys.argv)
    
    # Set application info
    app.setApplicationName("Nu_Scaler")
    app.setApplicationVersion("1.0")
    app.setOrganizationName("Nu_Scaler Team")
    
    # Apply global stylesheet
    app.setStyleSheet(STYLESHEET)
    
    try:
        # Create and show main window
        window = MainWindow()
        window.show()
        
        # Start the event loop
        return app.exec()
    except Exception as e:
        from PySide6.QtWidgets import QMessageBox
        
        # Show error dialog
        error_box = QMessageBox()
        error_box.setIcon(QMessageBox.Critical)
        error_box.setWindowTitle("Application Error")
        error_box.setText("An error occurred while starting Nu_Scaler")
        error_box.setDetailedText(str(e))
        error_box.exec()
        
        return 1

if __name__ == "__main__":
    sys.exit(run_gui()) 