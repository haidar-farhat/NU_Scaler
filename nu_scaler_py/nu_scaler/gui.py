"""
nu_scaler.gui - GUI entry point for NuScaler (to be implemented with DearPyGui or PyQt)
"""

import sys
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QLabel, QPushButton, QVBoxLayout, QHBoxLayout,
    QFileDialog, QComboBox, QSpinBox, QMessageBox
)
from PySide6.QtGui import QPixmap, QImage
from PySide6.QtCore import Qt
from PIL import Image
import numpy as np

try:
    import nu_scaler_core
except ImportError:
    nu_scaler_core = None

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("NuScaler - Basic GUI")
        self.input_image = None
        self.output_image = None
        self.upscaler = None
        self.init_ui()

    def init_ui(self):
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
        self.setCentralWidget(container)

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
