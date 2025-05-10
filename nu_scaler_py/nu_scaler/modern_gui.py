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