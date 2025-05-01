"""
nu_scaler.gui - GUI entry point for NuScaler (to be implemented with DearPyGui or PyQt)
"""

import sys
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QLabel, QPushButton, QVBoxLayout, QHBoxLayout,
    QFileDialog, QComboBox, QSpinBox, QMessageBox, QTabWidget
)
from PySide6.QtGui import QPixmap, QImage
from PySide6.QtCore import Qt, QTimer
from PIL import Image
import numpy as np
import threading
import time

try:
    import nu_scaler_core
    import cv2
except ImportError:
    nu_scaler_core = None
    cv2 = None

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("NuScaler - Basic GUI")
        self.input_image = None
        self.output_image = None
        self.upscaler = None
        self.init_ui()

    def init_ui(self):
        self.tabs = QTabWidget()
        self.tabs.addTab(self.make_image_tab(), "Image Mode")
        self.tabs.addTab(self.make_game_tab(), "Game Mode")
        self.tabs.addTab(self.make_video_tab(), "Video Mode")
        self.setCentralWidget(self.tabs)

    def make_image_tab(self):
        # Widgets
        self.input_label = QLabel("Input Image")
        self.input_label.setAlignment(Qt.AlignCenter)
        self.input_pixmap = QLabel()
        self.input_pixmap.setFixedSize(256, 256)
        self.input_pixmap.setStyleSheet("border: 1px solid gray;")

        self.output_label = QLabel("Upscaled Output")
        self.output_label.setAlignment(Qt.AlignCenter)
        self.output_pixmap = QLabel()
        self.output_pixmap.setFixedSize(256, 256)
        self.output_pixmap.setStyleSheet("border: 1px solid gray;")

        self.load_btn = QPushButton("Load Image")
        self.load_btn.clicked.connect(self.load_image)

        self.save_btn = QPushButton("Save Output")
        self.save_btn.clicked.connect(self.save_output)
        self.save_btn.setEnabled(False)

        self.quality_box = QComboBox()
        self.quality_box.addItems(["ultra", "quality", "balanced", "performance"])
        self.quality_box.setCurrentText("quality")

        self.algorithm_box = QComboBox()
        self.algorithm_box.addItems(["nearest", "bilinear"])
        self.algorithm_box.setCurrentText("nearest")

        self.input_w = QSpinBox()
        self.input_w.setRange(1, 8192)
        self.input_w.setValue(256)
        self.input_h = QSpinBox()
        self.input_h.setRange(1, 8192)
        self.input_h.setValue(256)
        self.output_w = QSpinBox()
        self.output_w.setRange(1, 8192)
        self.output_w.setValue(512)
        self.output_h = QSpinBox()
        self.output_h.setRange(1, 8192)
        self.output_h.setValue(512)

        self.upscale_btn = QPushButton("Upscale")
        self.upscale_btn.clicked.connect(self.run_upscale)
        self.upscale_btn.setEnabled(False)

        # Layouts
        img_layout = QHBoxLayout()
        img_layout.addWidget(self.input_pixmap)
        img_layout.addWidget(self.output_pixmap)

        controls = QHBoxLayout()
        controls.addWidget(QLabel("Quality:"))
        controls.addWidget(self.quality_box)
        controls.addWidget(QLabel("Algorithm:"))
        controls.addWidget(self.algorithm_box)
        controls.addWidget(QLabel("Input WxH:"))
        controls.addWidget(self.input_w)
        controls.addWidget(self.input_h)
        controls.addWidget(QLabel("Output WxH:"))
        controls.addWidget(self.output_w)
        controls.addWidget(self.output_h)

        btns = QHBoxLayout()
        btns.addWidget(self.load_btn)
        btns.addWidget(self.upscale_btn)
        btns.addWidget(self.save_btn)

        main_layout = QVBoxLayout()
        main_layout.addWidget(self.input_label)
        main_layout.addLayout(img_layout)
        main_layout.addWidget(self.output_label)
        main_layout.addLayout(controls)
        main_layout.addLayout(btns)

        container = QWidget()
        container.setLayout(main_layout)
        return container

    def make_game_tab(self):
        container = QWidget()
        layout = QVBoxLayout()
        self.game_status = QLabel("Status: Idle")
        self.game_view = QLabel()
        self.game_view.setFixedSize(512, 288)
        self.game_view.setStyleSheet("border: 1px solid gray;")
        # Target selection
        self.game_target_box = QComboBox()
        self.game_target_box.addItems(["FullScreen", "Window", "Region"])
        self.game_window_box = QComboBox()
        self.game_window_box.setEnabled(False)
        self.game_region_x = QSpinBox(); self.game_region_x.setRange(0, 9999)
        self.game_region_y = QSpinBox(); self.game_region_y.setRange(0, 9999)
        self.game_region_w = QSpinBox(); self.game_region_w.setRange(1, 9999)
        self.game_region_h = QSpinBox(); self.game_region_h.setRange(1, 9999)
        for w in [self.game_region_x, self.game_region_y, self.game_region_w, self.game_region_h]:
            w.setEnabled(False)
        self.game_target_box.currentTextChanged.connect(self.update_game_target_ui)
        # List windows if possible
        try:
            from nu_scaler_core import PyScreenCapture
            cap = PyScreenCapture()
            self.game_window_box.addItems(["Window 1", "Window 2"])  # TODO: call list_windows FFI
        except Exception:
            self.game_window_box.addItems(["Window 1", "Window 2"])
        # Buttons
        self.game_start_btn = QPushButton("Start Capture")
        self.game_stop_btn = QPushButton("Stop Capture")
        self.game_stop_btn.setEnabled(False)
        self.game_start_btn.clicked.connect(self.start_game_capture)
        self.game_stop_btn.clicked.connect(self.stop_game_capture)
        btns = QHBoxLayout()
        btns.addWidget(self.game_start_btn)
        btns.addWidget(self.game_stop_btn)
        # Layout
        target_layout = QHBoxLayout()
        target_layout.addWidget(QLabel("Target:"))
        target_layout.addWidget(self.game_target_box)
        target_layout.addWidget(QLabel("Window:"))
        target_layout.addWidget(self.game_window_box)
        target_layout.addWidget(QLabel("Region x/y/w/h:"))
        target_layout.addWidget(self.game_region_x)
        target_layout.addWidget(self.game_region_y)
        target_layout.addWidget(self.game_region_w)
        target_layout.addWidget(self.game_region_h)
        layout.addWidget(self.game_status)
        layout.addWidget(self.game_view)
        layout.addLayout(target_layout)
        layout.addLayout(btns)
        container.setLayout(layout)
        return container

    def update_game_target_ui(self, text):
        if text == "FullScreen":
            self.game_window_box.setEnabled(False)
            for w in [self.game_region_x, self.game_region_y, self.game_region_w, self.game_region_h]:
                w.setEnabled(False)
        elif text == "Window":
            self.game_window_box.setEnabled(True)
            for w in [self.game_region_x, self.game_region_y, self.game_region_w, self.game_region_h]:
                w.setEnabled(False)
        elif text == "Region":
            self.game_window_box.setEnabled(False)
            for w in [self.game_region_x, self.game_region_y, self.game_region_w, self.game_region_h]:
                w.setEnabled(True)

    def start_game_capture(self):
        try:
            from nu_scaler_core import PyScreenCapture, PyWgpuUpscaler, PyCaptureTarget
        except ImportError:
            QMessageBox.critical(self, "Rust core missing", "nu_scaler_core module not found.")
            return
        self.game_capture = PyScreenCapture()
        # Determine target
        tgt_type = self.game_target_box.currentText()
        if tgt_type == "FullScreen":
            target = PyCaptureTarget.FullScreen
        elif tgt_type == "Window":
            title = self.game_window_box.currentText()
            target = PyCaptureTarget.WindowByTitle(title)
        elif tgt_type == "Region":
            x = self.game_region_x.value()
            y = self.game_region_y.value()
            w = self.game_region_w.value()
            h = self.game_region_h.value()
            target = PyCaptureTarget.Region(x, y, w, h)
        else:
            target = PyCaptureTarget.FullScreen
        try:
            self.game_capture.start(target)
        except Exception as e:
            QMessageBox.critical(self, "Capture Error", str(e))
            return
        # Use upscaler settings from image tab
        quality = self.quality_box.currentText()
        algorithm = self.algorithm_box.currentText()
        out_w = self.output_w.value()
        out_h = self.output_h.value()
        self.game_upscaler = PyWgpuUpscaler(quality, algorithm)
        # We'll get input size from the first frame
        self.game_status.setText("Status: Running")
        self.game_start_btn.setEnabled(False)
        self.game_stop_btn.setEnabled(True)
        self.game_timer = QTimer()
        self.game_timer.timeout.connect(lambda: self.update_game_frame(out_w, out_h))
        self.game_timer.start(33)  # ~30 FPS

    def update_game_frame(self, out_w, out_h):
        frame = self.game_capture.get_frame()
        if frame is None:
            return
        # Assume frame is RGB, get input size from length
        in_len = len(frame)
        # Guess width/height (from last capture)
        if not hasattr(self, 'game_in_w') or not hasattr(self, 'game_in_h'):
            # Try to infer from upscaler or default to 1920x1080
            self.game_in_w = 1920
            self.game_in_h = 1080
            if hasattr(self.game_capture, 'width') and hasattr(self.game_capture, 'height'):
                self.game_in_w = self.game_capture.width
                self.game_in_h = self.game_capture.height
        # Try to infer from frame size
        if in_len % 3 == 0:
            px = int((in_len // 3) ** 0.5)
            if px * px * 3 == in_len:
                self.game_in_w = self.game_in_h = px
        try:
            self.game_upscaler.initialize(self.game_in_w, self.game_in_h, out_w, out_h)
            out_bytes = self.game_upscaler.upscale(frame)
            img = Image.frombytes("RGB", (out_w, out_h), out_bytes)
            self.display_image(img, self.game_view)
        except Exception as e:
            self.game_status.setText(f"Error: {e}")
            self.stop_game_capture()

    def stop_game_capture(self):
        if hasattr(self, 'game_timer'):
            self.game_timer.stop()
        if hasattr(self, 'game_capture'):
            self.game_capture.stop()
        self.game_status.setText("Status: Idle")
        self.game_start_btn.setEnabled(True)
        self.game_stop_btn.setEnabled(False)

    def make_video_tab(self):
        container = QWidget()
        layout = QVBoxLayout()
        self.video_status = QLabel("Status: Idle")
        self.video_view = QLabel()
        self.video_view.setFixedSize(512, 288)
        self.video_view.setStyleSheet("border: 1px solid gray;")
        self.video_open_btn = QPushButton("Open Video")
        self.video_play_btn = QPushButton("Play")
        self.video_stop_btn = QPushButton("Stop")
        self.video_play_btn.setEnabled(False)
        self.video_stop_btn.setEnabled(False)
        self.video_open_btn.clicked.connect(self.open_video_file)
        self.video_play_btn.clicked.connect(self.start_video_playback)
        self.video_stop_btn.clicked.connect(self.stop_video_playback)
        btns = QHBoxLayout()
        btns.addWidget(self.video_open_btn)
        btns.addWidget(self.video_play_btn)
        btns.addWidget(self.video_stop_btn)
        layout.addWidget(self.video_status)
        layout.addWidget(self.video_view)
        layout.addLayout(btns)
        container.setLayout(layout)
        return container

    def open_video_file(self):
        file, _ = QFileDialog.getOpenFileName(self, "Open Video", "", "Videos (*.mp4 *.avi *.mkv)")
        if not file:
            return
        if cv2 is None:
            QMessageBox.critical(self, "OpenCV missing", "cv2 (OpenCV) is required for video mode.")
            return
        self.video_file = file
        self.video_play_btn.setEnabled(True)
        self.video_status.setText(f"Loaded: {file}")

    def start_video_playback(self):
        if not hasattr(self, 'video_file'):
            return
        self.video_cap = cv2.VideoCapture(self.video_file)
        if not self.video_cap.isOpened():
            self.video_status.setText("Error: Cannot open video.")
            return
        from nu_scaler_core import PyWgpuUpscaler
        quality = self.quality_box.currentText()
        algorithm = self.algorithm_box.currentText()
        out_w = self.output_w.value()
        out_h = self.output_h.value()
        self.video_upscaler = PyWgpuUpscaler(quality, algorithm)
        self.video_play_btn.setEnabled(False)
        self.video_stop_btn.setEnabled(True)
        self.video_status.setText("Status: Playing")
        self.video_timer = QTimer()
        self.video_timer.timeout.connect(lambda: self.update_video_frame(out_w, out_h))
        self.video_timer.start(33)  # ~30 FPS

    def update_video_frame(self, out_w, out_h):
        ret, frame = self.video_cap.read()
        if not ret:
            self.stop_video_playback()
            return
        # Convert BGR to RGB
        frame_rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
        in_h, in_w = frame_rgb.shape[:2]
        try:
            self.video_upscaler.initialize(in_w, in_h, out_w, out_h)
            out_bytes = self.video_upscaler.upscale(frame_rgb.tobytes())
            img = Image.frombytes("RGB", (out_w, out_h), out_bytes)
            self.display_image(img, self.video_view)
        except Exception as e:
            self.video_status.setText(f"Error: {e}")
            self.stop_video_playback()

    def stop_video_playback(self):
        if hasattr(self, 'video_timer'):
            self.video_timer.stop()
        if hasattr(self, 'video_cap'):
            self.video_cap.release()
        self.video_status.setText("Status: Idle")
        self.video_play_btn.setEnabled(True)
        self.video_stop_btn.setEnabled(False)

    def load_image(self):
        file, _ = QFileDialog.getOpenFileName(self, "Open Image", "", "Images (*.png *.jpg *.bmp)")
        if not file:
            return
        img = Image.open(file).convert("RGB")
        self.input_image = img
        self.input_w.setValue(img.width)
        self.input_h.setValue(img.height)
        self.display_image(img, self.input_pixmap)
        self.upscale_btn.setEnabled(True)

    def display_image(self, img, label):
        qimg = QImage(img.tobytes(), img.width, img.height, QImage.Format_RGB888)
        pixmap = QPixmap.fromImage(qimg).scaled(label.width(), label.height(), Qt.KeepAspectRatio)
        label.setPixmap(pixmap)

    def run_upscale(self):
        if self.input_image is None:
            QMessageBox.warning(self, "No input", "Please load an image first.")
            return
        if nu_scaler_core is None:
            QMessageBox.critical(self, "Rust core missing", "nu_scaler_core module not found.")
            return
        # Get parameters
        quality = self.quality_box.currentText()
        algorithm = self.algorithm_box.currentText()
        in_w = self.input_w.value()
        in_h = self.input_h.value()
        out_w = self.output_w.value()
        out_h = self.output_h.value()
        # Prepare upscaler
        try:
            self.upscaler = nu_scaler_core.PyWgpuUpscaler(quality, algorithm)
            self.upscaler.initialize(in_w, in_h, out_w, out_h)
        except Exception as e:
            QMessageBox.critical(self, "Upscaler Error", str(e))
            return
        # Prepare input bytes
        img = self.input_image.resize((in_w, in_h), Image.BILINEAR)
        img_bytes = img.tobytes()
        try:
            out_bytes = self.upscaler.upscale(img_bytes)
            out_img = Image.frombytes("RGB", (out_w, out_h), out_bytes)
            self.output_image = out_img
            self.display_image(out_img, self.output_pixmap)
            self.save_btn.setEnabled(True)
        except Exception as e:
            QMessageBox.critical(self, "Upscale Failed", str(e))

    def save_output(self):
        if self.output_image is None:
            return
        file, _ = QFileDialog.getSaveFileName(self, "Save Output", "output.png", "PNG Files (*.png)")
        if not file:
            return
        self.output_image.save(file, "PNG")


def run_gui():
    app = QApplication(sys.argv)
    win = MainWindow()
    win.show()
    sys.exit(app.exec())

if __name__ == "__main__":
    run_gui()
