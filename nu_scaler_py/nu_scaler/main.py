import sys
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QLabel, QListWidget, QStackedWidget, QFrame,
    QPushButton, QComboBox, QSpinBox, QCheckBox, QSlider, QGroupBox, QFormLayout
)
from PySide6.QtCore import Qt, QTimer
from PySide6.QtGui import QPixmap, QImage
import time

try:
    import nu_scaler_core
except ImportError:
    nu_scaler_core = None

class LiveFeedScreen(QWidget):
    def __init__(self):
        super().__init__()
        self.capture = None
        self.upscaler = None
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.update_frame)
        self.last_frame_time = None
        self.fps = 0.0
        self.init_ui()

    def init_ui(self):
        layout = QHBoxLayout(self)
        # Left: Live preview and controls
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
        self.overlay = QLabel("Input: --\nUpscaled: --\nFPS: 0.0")
        self.overlay.setStyleSheet("background: rgba(30,30,30,180); color: #fff; padding: 8px; border-radius: 8px;")
        self.overlay.setAlignment(Qt.AlignRight | Qt.AlignTop)
        left_layout.addWidget(self.overlay)
        # Controls
        controls = QGroupBox("Capture Controls")
        form = QFormLayout(controls)
        self.source_box = QComboBox()
        self.source_box.addItems(["Screen", "Window", "Region"])
        self.backend_box = QComboBox()
        self.backend_box.addItems(["Auto", "Win32", "X11", "Wayland"])
        self.window_box = QComboBox()
        self.window_box.setEnabled(False)
        self.source_box.currentTextChanged.connect(self.update_source_ui)
        self.refresh_btn = QPushButton("Refresh Windows")
        self.refresh_btn.clicked.connect(self.refresh_windows)
        self.start_btn = QPushButton("Start")
        self.start_btn.clicked.connect(self.start_capture)
        self.stop_btn = QPushButton("Stop")
        self.stop_btn.clicked.connect(self.stop_capture)
        self.stop_btn.setEnabled(False)
        form.addRow("Input Source:", self.source_box)
        form.addRow("Backend:", self.backend_box)
        form.addRow("Window:", self.window_box)
        form.addRow(self.refresh_btn)
        form.addRow(self.start_btn, self.stop_btn)
        left_layout.addWidget(controls)
        left_layout.addStretch()
        # Right: Upscaled output and upscaling controls
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
        # Upscaling controls
        upscale_controls = QGroupBox("Upscaling Settings")
        upscale_form = QFormLayout(upscale_controls)
        self.quality_box = QComboBox()
        self.quality_box.addItems(["ultra", "quality", "balanced", "performance"])
        self.algorithm_box = QComboBox()
        self.algorithm_box.addItems(["nearest", "bilinear"])
        self.scale_slider = QSlider(Qt.Horizontal)
        self.scale_slider.setRange(10, 40)
        self.scale_slider.setValue(20)
        self.scale_slider.valueChanged.connect(self.update_scale_label)
        self.scale_label = QLabel("2.0×")
        upscale_form.addRow("Quality:", self.quality_box)
        upscale_form.addRow("Algorithm:", self.algorithm_box)
        upscale_form.addRow("Scale Factor:", self.scale_slider)
        upscale_form.addRow("", self.scale_label)
        right_layout.addWidget(upscale_controls)
        right_layout.addStretch()
        # Status bar
        self.status_bar = QLabel("Frame Time: -- ms   FPS: --   Resolution: --")
        self.status_bar.setStyleSheet("background: #181818; color: #aaa; padding: 4px;")
        right_layout.addWidget(self.status_bar)
        # Layout
        layout.addWidget(left_panel)
        layout.addWidget(right_panel)
        self.refresh_windows()
        self.update_scale_label()

    def update_source_ui(self, text):
        if text == "Window":
            self.window_box.setEnabled(True)
        else:
            self.window_box.setEnabled(False)

    def refresh_windows(self):
        self.window_box.clear()
        if nu_scaler_core is not None:
            try:
                windows = nu_scaler_core.PyScreenCapture.list_windows()
                if windows:
                    self.window_box.addItems(windows)
                else:
                    self.window_box.addItem("No windows found")
            except Exception:
                self.window_box.addItem("Error listing windows")
        else:
            self.window_box.addItem("Rust core missing")

    def update_scale_label(self):
        val = self.scale_slider.value() / 10.0
        self.scale_label.setText(f"{val:.1f}×")

    def start_capture(self):
        if nu_scaler_core is None:
            self.status_bar.setText("Rust core missing")
            return
        try:
            self.capture = nu_scaler_core.PyScreenCapture()
            source = self.source_box.currentText()
            if source == "Screen":
                target = nu_scaler_core.PyCaptureTarget.FullScreen
                window = None
                region = None
            elif source == "Window":
                target = nu_scaler_core.PyCaptureTarget.WindowByTitle
                window = nu_scaler_core.PyWindowByTitle(title=self.window_box.currentText())
                region = None
            else:
                target = nu_scaler_core.PyCaptureTarget.Region
                window = None
                # For demo, use a fixed region
                region = nu_scaler_core.PyRegion(x=100, y=100, width=640, height=480)
            self.capture.start(target, window, region)
            self.init_upscaler()
            self.timer.start(33)  # ~30 FPS
            self.start_btn.setEnabled(False)
            self.stop_btn.setEnabled(True)
            self.status_bar.setText("Capture started")
        except Exception as e:
            self.status_bar.setText(f"Error: {e}")

    def stop_capture(self):
        if self.capture:
            self.capture.stop()
        self.timer.stop()
        self.start_btn.setEnabled(True)
        self.stop_btn.setEnabled(False)
        self.status_bar.setText("Capture stopped")

    def init_upscaler(self):
        quality = self.quality_box.currentText()
        algorithm = self.algorithm_box.currentText()
        scale = self.scale_slider.value() / 10.0
        self.upscaler = nu_scaler_core.PyWgpuUpscaler(quality, algorithm)
        # We'll set input/output size on first frame
        self.upscale_scale = scale

    def update_frame(self):
        if not self.capture or not self.upscaler:
            return
        t0 = time.perf_counter()
        frame = self.capture.get_frame()
        if frame is None:
            return
        # Assume frame is RGB, try to infer input size
        in_len = len(frame)
        # For demo, guess square
        in_w = in_h = int((in_len // 3) ** 0.5)
        out_w = int(in_w * self.upscale_scale)
        out_h = int(in_h * self.upscale_scale)
        try:
            self.upscaler.initialize(in_w, in_h, out_w, out_h)
            out_bytes = self.upscaler.upscale(frame)
            img = QImage(out_bytes, out_w, out_h, QImage.Format_RGB888)
            pixmap = QPixmap.fromImage(img).scaled(self.output_preview.width(), self.output_preview.height(), Qt.KeepAspectRatio)
            self.output_preview.setPixmap(pixmap)
            # Show input as well
            img_in = QImage(frame, in_w, in_h, QImage.Format_RGB888)
            pixmap_in = QPixmap.fromImage(img_in).scaled(self.input_preview.width(), self.input_preview.height(), Qt.KeepAspectRatio)
            self.input_preview.setPixmap(pixmap_in)
            # Overlay and status
            t1 = time.perf_counter()
            frame_time = (t1 - t0) * 1000
            self.fps = 1000.0 / frame_time if frame_time > 0 else 0.0
            self.overlay.setText(f"Input: {in_w}×{in_h}\nUpscaled: {out_w}×{out_h}\nFPS: {self.fps:.1f}")
            self.status_bar.setText(f"Frame Time: {frame_time:.1f} ms   FPS: {self.fps:.1f}   Resolution: {in_w}×{in_h} → {out_w}×{out_h}")
        except Exception as e:
            self.status_bar.setText(f"Upscale error: {e}")
            self.stop_capture()

class SettingsScreen(QWidget):
    def __init__(self):
        super().__init__()
        layout = QVBoxLayout(self)
        # Input & Capture
        input_group = QGroupBox("Input & Capture")
        input_form = QFormLayout(input_group)
        self.input_source = QComboBox()
        self.input_source.addItems(["Screen Capture", "Video File", "Static Image"])
        self.backend = QComboBox()
        self.backend.addItems(["Auto", "Win32", "X11", "Wayland"])
        self.capture_btn = QPushButton("Capture Frame")
        self.refresh_btn = QPushButton("Refresh Devices")
        input_form.addRow("Input Source:", self.input_source)
        input_form.addRow("Backend:", self.backend)
        input_form.addRow(self.capture_btn, self.refresh_btn)
        # Upscaling Settings
        upscale_group = QGroupBox("Upscaling Settings")
        upscale_form = QFormLayout(upscale_group)
        self.scale_slider = QSlider(Qt.Horizontal)
        self.scale_slider.setRange(10, 40)
        self.scale_slider.setValue(20)
        self.scale_label = QLabel("2.0×")
        self.scale_slider.valueChanged.connect(lambda: self.scale_label.setText(f"{self.scale_slider.value()/10.0:.1f}×"))
        self.method = QComboBox()
        self.method.addItems(["AMD FSR", "NVIDIA NIS", "Pure Rust Interpolation"])
        self.advanced_btn = QPushButton("Advanced Algorithm Settings")
        upscale_form.addRow("Scale Factor:", self.scale_slider)
        upscale_form.addRow("", self.scale_label)
        upscale_form.addRow("Upscaling Method:", self.method)
        upscale_form.addRow(self.advanced_btn)
        # Interpolation Settings
        interp_group = QGroupBox("Interpolation Settings")
        interp_form = QFormLayout(interp_group)
        self.motion_slider = QSlider(Qt.Horizontal)
        self.motion_slider.setRange(0, 100)
        self.motion_slider.setValue(50)
        self.blend_slider = QSlider(Qt.Horizontal)
        self.blend_slider.setRange(0, 100)
        self.blend_slider.setValue(50)
        self.smooth_slider = QSlider(Qt.Horizontal)
        self.smooth_slider.setRange(0, 100)
        self.smooth_slider.setValue(50)
        self.gpu_shader = QCheckBox("Use GPU Shader")
        self.reload_shader = QPushButton("Reload Shader")
        interp_form.addRow("Motion Sensitivity:", self.motion_slider)
        interp_form.addRow("Blending Ratio:", self.blend_slider)
        interp_form.addRow("Smoothing Factor:", self.smooth_slider)
        interp_form.addRow(self.gpu_shader, self.reload_shader)
        # Compute Settings
        compute_group = QGroupBox("Compute Settings")
        compute_form = QFormLayout(compute_group)
        self.render_mode = QComboBox()
        self.render_mode.addItems(["GPU Accelerated", "CPU-only"])
        self.optimize_perf = QCheckBox("Optimize for Performance")
        compute_form.addRow("Rendering Mode:", self.render_mode)
        compute_form.addRow(self.optimize_perf)
        # Control Buttons
        control_group = QGroupBox("Controls")
        control_layout = QHBoxLayout(control_group)
        self.start_btn = QPushButton("Start")
        self.pause_btn = QPushButton("Pause/Resume")
        self.stop_btn = QPushButton("Stop")
        self.export_btn = QPushButton("Export Frame")
        self.export_format = QComboBox()
        self.export_format.addItems(["PNG", "JPG", "BMP"])
        control_layout.addWidget(self.start_btn)
        control_layout.addWidget(self.pause_btn)
        control_layout.addWidget(self.stop_btn)
        control_layout.addWidget(self.export_btn)
        control_layout.addWidget(self.export_format)
        # Add all groups to layout
        layout.addWidget(input_group)
        layout.addWidget(upscale_group)
        layout.addWidget(interp_group)
        layout.addWidget(compute_group)
        layout.addWidget(control_group)
        layout.addStretch()

class DebugScreen(QWidget):
    def __init__(self):
        super().__init__()
        layout = QVBoxLayout(self)
        # Collapsible log view
        self.log_group = QGroupBox("Log (Collapsible)")
        self.log_group.setCheckable(True)
        self.log_group.setChecked(True)
        log_layout = QVBoxLayout(self.log_group)
        self.log_view = QLabel("[Logs will appear here]")
        self.log_view.setStyleSheet("background: #222; color: #f88; font-family: monospace; padding: 8px;")
        self.log_view.setWordWrap(True)
        log_layout.addWidget(self.log_view)
        # Profiler graph placeholder
        self.profiler_group = QGroupBox("Profiler")
        profiler_layout = QVBoxLayout(self.profiler_group)
        self.profiler_label = QLabel("[Profiler graph/timeline placeholder]")
        self.profiler_label.setStyleSheet("background: #222; color: #8ff; padding: 8px;")
        profiler_layout.addWidget(self.profiler_label)
        # Overlay warnings
        self.warning_label = QLabel("[Overlay warnings: FPS drop, errors, etc.]")
        self.warning_label.setStyleSheet("background: #400; color: #fff; padding: 6px; border-radius: 6px;")
        self.warning_label.setVisible(False)
        # Layout
        layout.addWidget(self.log_group)
        layout.addWidget(self.profiler_group)
        layout.addWidget(self.warning_label)
        layout.addStretch()

class AdvancedScreen(QWidget):
    def __init__(self):
        super().__init__()
        layout = QVBoxLayout(self)
        # Shader & Engine
        shader_group = QGroupBox("Shader & Engine")
        shader_form = QFormLayout(shader_group)
        self.shader_path = QLabel("[WGSL Shader Path]")
        self.reload_shader = QPushButton("Reload Shader")
        self.hot_reload = QCheckBox("Enable Hot Reload")
        shader_form.addRow("Custom WGSL Shader Path:", self.shader_path)
        shader_form.addRow(self.reload_shader)
        shader_form.addRow(self.hot_reload)
        # Concurrency
        concurrency_group = QGroupBox("Concurrency")
        concurrency_form = QFormLayout(concurrency_group)
        self.thread_count = QSpinBox()
        self.thread_count.setRange(1, 64)
        self.thread_count.setValue(4)
        self.auto_scale = QCheckBox("Auto-scale threads")
        self.rayon_toggle = QCheckBox("Use Rayon/Crossbeam backend")
        concurrency_form.addRow("Thread Count:", self.thread_count)
        concurrency_form.addRow(self.auto_scale)
        concurrency_form.addRow(self.rayon_toggle)
        # Memory Options
        memory_group = QGroupBox("Memory Options")
        memory_form = QFormLayout(memory_group)
        self.buffer_pool = QSpinBox()
        self.buffer_pool.setRange(1, 32)
        self.buffer_pool.setValue(4)
        self.gpu_allocator = QComboBox()
        self.gpu_allocator.addItems(["Default", "Aggressive", "Conservative"])
        memory_form.addRow("Buffer Pool Size:", self.buffer_pool)
        memory_form.addRow("GPU Allocator Preset:", self.gpu_allocator)
        # Layout
        layout.addWidget(shader_group)
        layout.addWidget(concurrency_group)
        layout.addWidget(memory_group)
        layout.addStretch()

class UIAccessibilityScreen(QWidget):
    def __init__(self):
        super().__init__()
        layout = QVBoxLayout(self)
        # Theme
        theme_group = QGroupBox("Theme & Appearance")
        theme_form = QFormLayout(theme_group)
        self.theme_select = QComboBox()
        self.theme_select.addItems(["Dark", "Light", "System Default"])
        self.font_scale = QSlider(Qt.Horizontal)
        self.font_scale.setRange(8, 32)
        self.font_scale.setValue(14)
        self.font_label = QLabel("14pt")
        self.font_scale.valueChanged.connect(lambda: self.font_label.setText(f"{self.font_scale.value()}pt"))
        theme_form.addRow("Theme:", self.theme_select)
        theme_form.addRow("Font Scale:", self.font_scale)
        theme_form.addRow("", self.font_label)
        # Shortcuts
        shortcut_group = QGroupBox("Keyboard Shortcuts")
        shortcut_layout = QVBoxLayout(shortcut_group)
        self.shortcut_label = QLabel("[Shortcuts view/editor placeholder]")
        shortcut_layout.addWidget(self.shortcut_label)
        # Config
        config_group = QGroupBox("Configuration")
        config_layout = QHBoxLayout(config_group)
        self.save_btn = QPushButton("Save Config")
        self.load_btn = QPushButton("Load Config")
        config_layout.addWidget(self.save_btn)
        config_layout.addWidget(self.load_btn)
        # Layout
        layout.addWidget(theme_group)
        layout.addWidget(shortcut_group)
        layout.addWidget(config_group)
        layout.addStretch()

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
            1: SettingsScreen(),
            2: DebugScreen(),
            3: AdvancedScreen(),
            4: UIAccessibilityScreen(),
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
            QFrame[frameShape=\"4\"] { border: 1px solid #444; border-radius: 8px; }
        """)

def run_gui():
    app = QApplication(sys.argv)
    win = MainWindow()
    win.show()
    sys.exit(app.exec())

if __name__ == "__main__":
    run_gui() 