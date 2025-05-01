import sys
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QLabel, QListWidget, QStackedWidget, QFrame
)
from PySide6.QtCore import Qt

# Placeholder for future screens
class LiveFeedScreen(QWidget):
    def __init__(self):
        super().__init__()
        layout = QHBoxLayout(self)
        # Left: Live preview
        left_panel = QFrame()
        left_panel.setFrameShape(QFrame.StyledPanel)
        left_layout = QVBoxLayout(left_panel)
        self.input_label = QLabel("Live Feed Preview")
        self.input_label.setAlignment(Qt.AlignCenter)
        self.input_label.setStyleSheet("font-size: 18px; color: #ccc;")
        self.input_preview = QLabel()
        self.input_preview.setFixedSize(480, 270)
        self.input_preview.setStyleSheet("background: #222; border: 1px solid #555;")
        left_layout.addWidget(self.input_label)
        left_layout.addWidget(self.input_preview)
        # Overlay (floating)
        self.overlay = QLabel("Input: 1280×720\nUpscaled: 2560×1440\nFPS: 0.0")
        self.overlay.setStyleSheet("background: rgba(30,30,30,180); color: #fff; padding: 8px; border-radius: 8px;")
        self.overlay.setAlignment(Qt.AlignRight | Qt.AlignTop)
        left_layout.addWidget(self.overlay)
        left_layout.addStretch()
        # Right: Upscaled output
        right_panel = QFrame()
        right_panel.setFrameShape(QFrame.StyledPanel)
        right_layout = QVBoxLayout(right_panel)
        self.output_label = QLabel("Upscaled Output Preview")
        self.output_label.setAlignment(Qt.AlignCenter)
        self.output_label.setStyleSheet("font-size: 18px; color: #ccc;")
        self.output_preview = QLabel()
        self.output_preview.setFixedSize(480, 270)
        self.output_preview.setStyleSheet("background: #222; border: 1px solid #555;")
        right_layout.addWidget(self.output_label)
        right_layout.addWidget(self.output_preview)
        right_layout.addStretch()
        # Status bar
        self.status_bar = QLabel("Frame Time: -- ms   FPS: --   Resolution: --")
        self.status_bar.setStyleSheet("background: #181818; color: #aaa; padding: 4px;")
        right_layout.addWidget(self.status_bar)
        # Layout
        layout.addWidget(left_panel)
        layout.addWidget(right_panel)

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Nu Scaler")
        self.setMinimumSize(1100, 650)
        # Sidebar navigation
        self.sidebar = QListWidget()
        self.sidebar.addItems([
            "Live Feed",
            "Settings",
            "Debug",
            "Advanced",
            "UI & Accessibility"
        ])
        self.sidebar.setFixedWidth(180)
        self.sidebar.setStyleSheet("background: #232323; color: #bbb; font-size: 16px;")
        # Stacked widget for screens
        self.stack = QStackedWidget()
        self.screens = {
            0: LiveFeedScreen(),
            # Placeholders for other screens
            1: QWidget(),
            2: QWidget(),
            3: QWidget(),
            4: QWidget(),
        }
        for i in range(5):
            self.stack.addWidget(self.screens[i])
        self.sidebar.currentRowChanged.connect(self.stack.setCurrentIndex)
        # Main layout
        main_widget = QWidget()
        main_layout = QHBoxLayout(main_widget)
        main_layout.addWidget(self.sidebar)
        main_layout.addWidget(self.stack)
        self.setCentralWidget(main_widget)
        self.apply_theme()
    def apply_theme(self):
        self.setStyleSheet("""
            QMainWindow { background: #181818; }
            QLabel { font-family: 'Segoe UI', 'Arial', sans-serif; }
            QListWidget::item:selected { background: #444; color: #fff; }
            QFrame[frameShape="4"] { border: 1px solid #444; border-radius: 8px; }
        """)

def run_gui():
    app = QApplication(sys.argv)
    win = MainWindow()
    win.show()
    sys.exit(app.exec())

if __name__ == "__main__":
    run_gui() 