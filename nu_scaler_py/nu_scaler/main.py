import sys
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QLabel, QListWidget, QStackedWidget, QFrame,
    QPushButton, QComboBox, QSpinBox, QCheckBox, QSlider, QGroupBox, QFormLayout, QProgressBar, QFileDialog
)
from PySide6.QtCore import Qt, QTimer, Signal, QThread, QObject, Slot
from PySide6.QtGui import QPixmap, QImage, QAction
import time
import random
import traceback
import threading
import psutil
import os

# First, try to import the Rust core module
print("[main.py] About to import nu_scaler_core...")
try:
    import nu_scaler_core
    print(f"[main.py] Successfully imported nu_scaler_core from {nu_scaler_core.__file__}")
    print(f"[main.py] Available classes in nu_scaler_core: {dir(nu_scaler_core)}")
except ImportError as e:
    print(f"[main.py] ImportError when importing nu_scaler_core: {e}")
    nu_scaler_core = None
except Exception as e:
    print(f"[main.py] Error during nu_scaler_core import: {e}")
    traceback.print_exc()
    nu_scaler_core = None

# Then, separately try to import the benchmark module
try:
    from nu_scaler.benchmark import run_benchmark, run_comparison_benchmark, BenchmarkResult, plot_benchmark_results
except ImportError as e:
    print(f"[main.py] ImportError when importing benchmark module: {e}")
    run_benchmark = None
    run_comparison_benchmark = None
    BenchmarkResult = None
    plot_benchmark_results = None
except Exception as e:
    print(f"[main.py] Error when importing benchmark module: {e}")
    traceback.print_exc()
    run_benchmark = None
    run_comparison_benchmark = None
    BenchmarkResult = None
    plot_benchmark_results = None

print(f"[main.py] nu_scaler_core available: {nu_scaler_core is not None}")

# Add import for GPU optimization
from nu_scaler.gpu_optimizer import optimize_upscaler, force_gpu_activation

class UpscaleWorker(QObject):
    finished = Signal(bytes, int, int, float)
    error = Signal(str)

    def __init__(self, upscaler, frame, in_w, in_h, out_w, out_h, scale):
        super().__init__()
        self.upscaler = upscaler
        self.frame = frame
        self.in_w = in_w
        self.in_h = in_h
        self.out_w = out_w
        self.out_h = out_h
        self.scale = scale
        print(f'[DEBUG] UpscaleWorker created: {id(self)}')

    @Slot()
    def run(self):
        import time
        print(f"[DEBUG] UpscaleWorker.run: {id(self)} starting")
        t0 = time.perf_counter()
        try:
            print(f"[DEBUG] UpscaleWorker.run: {id(self)} before upscale")
            out = self.upscaler.upscale(self.frame)
            t1 = time.perf_counter()
            print(f"[DEBUG] UpscaleWorker.run: {id(self)} finished upscale in {t1-t0:.2f}s")
            self.finished.emit(out, self.out_w, self.out_h, (t1-t0)*1000)
        except Exception as e:
            print(f"[DEBUG] UpscaleWorker.run: {id(self)} error: {e}")
            self.error.emit(str(e))

    def __del__(self):
        print(f'[DEBUG] UpscaleWorker __del__: {id(self)}')

class LiveFeedScreen(QWidget):
    log_signal = Signal(str)
    profiler_signal = Signal(float, float, int, int)
    warning_signal = Signal(str, bool)
    def __init__(self, parent=None):
        super().__init__(parent)
        self.capture = None
        self.upscaler = None
        self.timer = QTimer(self)
        self.timer.setInterval(100)  # Lowered to 10 FPS for diagnosis
        self.timer.timeout.connect(self.update_frame)
        self.last_frame_time = None
        self.last_timer_time = None
        self.fps = 0.0
        self.upscaler_initialized = False
        self.upscale_scale = 2.0  # Default scale factor
        self.advanced_upscaling = True  # Use advanced upscaler by default
        self.memory_monitor_timer = QTimer(self)
        self.memory_monitor_timer.timeout.connect(self.update_memory_stats)
        self.memory_monitor_timer.start(2000)  # Update every 2 seconds
        self.vram_usage = 0.0
        self.total_vram = 0.0
        self.show_memory_stats = True
        self._upscale_thread = None
        self._upscale_worker = None
        self._last_in_w = None
        self._last_in_h = None
        self._last_scale = None
        print('[DEBUG] LiveFeedScreen: Before init_ui')
        self.init_ui()
        print('[DEBUG] LiveFeedScreen: After init_ui')
        print('[DEBUG] LiveFeedScreen: Before refresh_windows')
        # self.refresh_windows()  # Commented out for diagnosis
        print('[DEBUG] LiveFeedScreen: After refresh_windows')
        print('[DEBUG] LiveFeedScreen: Before update_scale_label')
        self.update_scale_label()
        print('[DEBUG] LiveFeedScreen: After update_scale_label')
        # Heartbeat timer
        self.heartbeat_timer = QTimer(self)
        self.heartbeat_timer.setInterval(1000)
        self.heartbeat_timer.timeout.connect(self._heartbeat)
        self.heartbeat_timer.start()
        # Resource monitor timer
        self.resource_timer = QTimer(self)
        self.resource_timer.setInterval(1000)
        self.resource_timer.timeout.connect(self._resource_debug)
        self.resource_timer.start()
        # Start watchdog thread
        self._watchdog_running = True
        self._watchdog_thread = threading.Thread(target=self._watchdog_loop, daemon=True)
        self._watchdog_thread.start()

    def _watchdog_loop(self):
        while self._watchdog_running:
            try:
                print(f"[WATCHDOG] Still alive at {time.strftime('%H:%M:%S')}")
                process = psutil.Process(os.getpid())
                mem = process.memory_info().rss / (1024 * 1024)
                thread_count = threading.active_count()
                print(f"[WATCHDOG] Memory: {mem:.1f} MB | Threads: {thread_count}")
            except Exception as e:
                print(f"[WATCHDOG] Error: {e}")
            time.sleep(1)

    def closeEvent(self, event):
        self._watchdog_running = False
        super().closeEvent(event)

    def _heartbeat(self):
        print(f"[HEARTBEAT] GUI event loop alive at {time.strftime('%H:%M:%S')}")

    def _resource_debug(self):
        try:
            process = psutil.Process(os.getpid())
            mem = process.memory_info().rss / (1024 * 1024)
            thread_count = threading.active_count()
            print(f"[RESOURCE] Memory: {mem:.1f} MB | Threads: {thread_count}")
        except Exception as e:
            print(f"[RESOURCE] Error: {e}")

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
        
        # Add technology selector
        self.technology_box = QComboBox()
        self.technology_box.addItems(["Auto (Best for GPU)", "FSR 3.0", "DLSS", "Basic"])
        
        self.quality_box = QComboBox()
        self.quality_box.addItems(["ultra", "quality", "balanced", "performance"])
        self.algorithm_box = QComboBox()
        self.algorithm_box.addItems(["nearest", "bilinear"])
        self.scale_slider = QSlider(Qt.Horizontal)
        self.scale_slider.setRange(10, 40)
        self.scale_slider.setValue(20)
        self.scale_slider.valueChanged.connect(self.update_scale_label)
        self.scale_label = QLabel("2.0×")
        
        # Technology selector is first option
        upscale_form.addRow("Technology:", self.technology_box)
        upscale_form.addRow("Quality:", self.quality_box)
        upscale_form.addRow("Algorithm:", self.algorithm_box)
        upscale_form.addRow("Scale Factor:", self.scale_slider)
        upscale_form.addRow("", self.scale_label)
        
        # Connect technology selector to handle enabling/disabling appropriate options
        self.technology_box.currentTextChanged.connect(self.update_technology_ui)
        
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

        # Add advanced upscaling option
        self.advanced_check = QCheckBox("Advanced GPU Optimization", self)
        self.advanced_check.setChecked(self.advanced_upscaling)
        self.advanced_check.stateChanged.connect(self.toggle_advanced_upscaling)
        upscale_controls.layout().addRow(self.advanced_check)
        
        # Add memory stats display
        self.memory_stats_label = QLabel("VRAM: 0.0 MB / 0.0 MB (0%)", self)
        upscale_controls.layout().addRow(self.memory_stats_label)
        
        # Add memory management strategy dropdown
        memory_strategy_layout = QHBoxLayout()
        memory_strategy_layout.addWidget(QLabel("Memory Strategy:"))
        self.memory_strategy_box = QComboBox(self)
        self.memory_strategy_box.addItems(["Auto", "Aggressive", "Balanced", "Conservative", "Minimal"])
        self.memory_strategy_box.setCurrentText("Auto")
        self.memory_strategy_box.currentIndexChanged.connect(self.set_memory_strategy)
        memory_strategy_layout.addWidget(self.memory_strategy_box)
        upscale_controls.layout().addRow("", memory_strategy_layout)
        
        # Add adaptive quality checkbox
        self.adaptive_quality_check = QCheckBox("Adaptive Quality", self)
        self.adaptive_quality_check.setChecked(True)
        self.adaptive_quality_check.stateChanged.connect(self.toggle_adaptive_quality)
        upscale_controls.layout().addRow(self.adaptive_quality_check)

    def update_source_ui(self, text):
        if text == "Window":
            self.window_box.setEnabled(True)
        else:
            self.window_box.setEnabled(False)

    def refresh_windows(self):
        print("[GUI] Refreshing windows list...")
        self.window_box.clear()
        if nu_scaler_core is not None:
            try:
                # Make sure we have a PyScreenCapture class
                if hasattr(nu_scaler_core, 'PyScreenCapture') and hasattr(nu_scaler_core.PyScreenCapture, 'list_windows'):
                    windows = nu_scaler_core.PyScreenCapture.list_windows()
                    print(f"[GUI] Received windows: {windows}")
                    if windows:
                        self.window_box.addItems(windows)
                    else:
                        self.window_box.addItem("No windows found")
                else:
                    print("[GUI] PyScreenCapture.list_windows method not available")
                    self.window_box.addItem("API method missing")
            except Exception as e:
                print(f"[GUI] Error listing windows: {e}")
                traceback.print_exc()
                self.window_box.addItem(f"Error: {str(e)[:30]}...")
        else:
            print("[GUI] Rust core not available for listing windows.")
            self.window_box.addItem("Rust core missing")

    def update_scale_label(self):
        val = self.scale_slider.value() / 10.0
        self.scale_label.setText(f"{val:.1f}×")

    def start_capture(self):
        print("[GUI] Start capture requested.")
        if nu_scaler_core is None:
            print("[GUI] Rust core not available for capture.")
            self.status_bar.setText("Rust core missing")
            return
        try:
            # Determine target based on GUI selection
            source = self.source_box.currentText()
            window_title = self.window_box.currentText() if source == "Window" else None
            print(f"[GUI] Source: {source}, Window Title: {window_title}")
            # --- Remove Forced FullScreen --- 
            # print("[GUI - DEBUG] Forcing FullScreen capture mode.")
            # source = "Screen" # Override
            # window_title = None # Override
            # --- End Remove Forced FullScreen ---

            if source == "Screen":
                target = nu_scaler_core.PyCaptureTarget.FullScreen
                window = None
                region = None
                print("[GUI] Using FullScreen target.")
            elif source == "Window" and window_title and window_title != "No windows found" and window_title != "Error listing windows":
                target = nu_scaler_core.PyCaptureTarget.WindowByTitle
                window = nu_scaler_core.PyWindowByTitle(title=window_title)
                region = None
                print(f"[GUI] Using WindowByTitle target: {window_title}")
            elif source == "Region": # Fixed region for demo
                target = nu_scaler_core.PyCaptureTarget.Region
                window = None
                region = nu_scaler_core.PyRegion(x=100, y=100, width=640, height=480)
                print(f"[GUI] Using Region target: {region.x},{region.y} {region.width}x{region.height}")
            else:
                print("[GUI] Invalid capture configuration.")
                self.status_bar.setText("Invalid capture config")
                return

            print("[GUI] Calling capture.start()...")
            self.capture = nu_scaler_core.PyScreenCapture()
            self.capture.start(target, window, region)
            print("[GUI] capture.start() returned.")

            # Remove the delay
            # print("[GUI] Waiting 2 seconds before starting frame timer...")
            # time.sleep(2.0)
            # print("[GUI] Starting frame timer.")

            self.upscaler_initialized = False # Reset upscaler state
            self.upscaler = None
            self.timer.start()  # Start the timer (throttled)
            self.start_btn.setEnabled(False)
            self.stop_btn.setEnabled(True)
            self.status_bar.setText("Capture started")
            print("[GUI] Capture timer started.")
        except Exception as e:
            print(f"[GUI] Error starting capture: {e}")
            self.status_bar.setText(f"Error starting capture: {e}")
            self.log_signal.emit(f"Error starting capture: {e}")

    def stop_capture(self):
        import threading as _threading
        import gc
        import psutil
        print('[DEBUG] stop_capture: called')
        print(f'[DEBUG] stop_capture: active threads before: {_threading.active_count()}')
        if self.capture:
            try:
                self.capture.stop()
                print('[DEBUG] stop_capture: capture stopped')
            except Exception as e:
                print(f'[DEBUG] stop_capture: error stopping capture: {e}')
            self.capture = None
        self.timer.stop()
        print('[DEBUG] stop_capture: timer stopped')
        self.start_btn.setEnabled(True)
        self.stop_btn.setEnabled(False)
        self.status_bar.setText("Capture stopped")
        # Clean up worker and thread
        if hasattr(self, '_upscale_thread') and self._upscale_thread is not None:
            print(f'[DEBUG] stop_capture: upscale thread state: {self._upscale_thread.isRunning()}')
            self._upscale_thread.quit()
            self._upscale_thread.wait(2000)
            print('[DEBUG] stop_capture: upscale thread quit and waited')
            self._upscale_thread = None
        if hasattr(self, '_upscale_worker') and self._upscale_worker is not None:
            print('[DEBUG] stop_capture: deleting upscale worker')
            self._upscale_worker = None
        print(f'[DEBUG] stop_capture: active threads after: {_threading.active_count()}')
        # Explicitly delete upscaler and capture
        if hasattr(self, 'upscaler') and self.upscaler is not None:
            print('[DEBUG] stop_capture: deleting upscaler')
            del self.upscaler
            self.upscaler = None
        if hasattr(self, 'capture') and self.capture is not None:
            print('[DEBUG] stop_capture: deleting capture')
            del self.capture
            self.capture = None
        gc.collect()
        print(f'[DEBUG] stop_capture: gc collected, objects: {len(gc.get_objects())}')
        print(f'[DEBUG] stop_capture: OS handle count: {psutil.Process().num_handles()}')
        # Delayed watchdogs
        import threading
        def watchdog(msg):
            print(f'[WATCHDOG] {msg}')
        threading.Timer(2, watchdog, args=("2s after stop_capture",)).start()
        threading.Timer(5, watchdog, args=("5s after stop_capture",)).start()
        threading.Timer(10, watchdog, args=("10s after stop_capture",)).start()

    def update_technology_ui(self, technology):
        try:
            if technology == "FSR 3.0":
                self.quality_box.setEnabled(True)
                self.algorithm_box.setEnabled(False)
                self.algorithm_box.setCurrentText("bilinear")
            elif technology == "DLSS":
                self.quality_box.setEnabled(True)
                self.algorithm_box.setEnabled(False)
                self.algorithm_box.setCurrentText("bilinear")
            elif technology == "Basic":
                self.quality_box.setEnabled(True)
                self.algorithm_box.setEnabled(True)
            else:
                self.quality_box.setEnabled(True)
                self.algorithm_box.setEnabled(False)
        except Exception as e:
            print(f'[DEBUG] update_technology_ui: {e}')

    def toggle_advanced_upscaling(self, state):
        try:
            self.advanced_upscaling = bool(state)
            self.upscaler = None
            self.upscaler_initialized = False
            self.memory_strategy_box.setEnabled(self.advanced_upscaling)
            self.adaptive_quality_check.setEnabled(self.advanced_upscaling)
        except Exception as e:
            print(f'[DEBUG] toggle_advanced_upscaling: {e}')

    def toggle_adaptive_quality(self, state):
        try:
            if self.upscaler and hasattr(self.upscaler, 'set_adaptive_quality'):
                self.upscaler.set_adaptive_quality(bool(state))
        except Exception as e:
            print(f'[DEBUG] toggle_adaptive_quality: {e}')

    def set_memory_strategy(self, index):
        try:
            if not self.upscaler or not hasattr(self.upscaler, 'set_memory_strategy'):
                return
            strategies = ["Auto", "Aggressive", "Balanced", "Conservative", "Minimal"]
            if index >= 0 and index < len(strategies):
                strategy = strategies[index].lower()
                try:
                    self.upscaler.set_memory_strategy(strategy)
                    print(f"Memory strategy set to: {strategy}")
                except Exception as e:
                    print(f"Failed to set memory strategy: {e}")
        except Exception as e:
            print(f'[DEBUG] set_memory_strategy: {e}')
    
    def update_memory_stats(self):
        """Update GPU memory usage statistics"""
        try:
            if self.upscaler and hasattr(self.upscaler, 'get_vram_stats'):
                stats = self.upscaler.get_vram_stats()
                if stats:
                    self.vram_usage = stats.used_mb
                    self.total_vram = stats.total_mb
                    percentage = stats.usage_percent
                    
                    # Update label
                    self.memory_stats_label.setText(
                        f"VRAM: {self.vram_usage:.1f} MB / {self.total_vram:.1f} MB ({percentage:.1f}%)"
                    )
                    
                    # Set color based on usage
                    if percentage > 90:
                        self.memory_stats_label.setStyleSheet("color: red; font-weight: bold")
                    elif percentage > 75:
                        self.memory_stats_label.setStyleSheet("color: orange")
                    else:
                        self.memory_stats_label.setStyleSheet("color: green")
        except Exception as e:
            print(f"Error updating memory stats: {e}")
    
    def init_upscaler(self, in_w, in_h, scale):
        """
        Initialize the upscaler with the given dimensions and scale factor.
        """
        print("[GUI] Initializing upscaler...")
        out_w = int(in_w * scale)
        out_h = int(in_h * scale)
        
        # First check if Rust core is available
        if nu_scaler_core is None:
            print("[GUI] Rust core not available, using dummy upscaler")
            self.upscaler = DummyUpscaler()
        else:
            # Use advanced upscaler if available
            if hasattr(nu_scaler_core, 'create_advanced_upscaler'):
                quality = self.quality_box.currentText().lower()
                self.upscaler = nu_scaler_core.create_advanced_upscaler(quality)
                print(f"[GUI] Created advanced upscaler with quality: {quality}")
            else:
                # Fall back to regular upscaler
                print(f"[GUI] Created regular upscaler")
                self.upscaler = nu_scaler_core.PyWgpuUpscaler("quality", "bilinear")
                
        # Force GPU activation to maximize performance
        if hasattr(nu_scaler_core, 'PyAdvancedWgpuUpscaler') and isinstance(self.upscaler, nu_scaler_core.PyAdvancedWgpuUpscaler):
            print("[GUI] Forcing GPU activation for better performance...")
            force_gpu_activation(self.upscaler)
                
        # Initialize the upscaler
        self.upscaler.initialize(in_w, in_h, out_w, out_h)
        # Don't try to call set_upscale_scale which doesn't exist
        # Instead, just note that the upscaler scale is set during initialization
        # based on the dimensions
        print(f"Upscaler scale factor: {self.upscaler.upscale_scale}")
            
        self.upscaler_initialized = True
        self.upscale_scale = scale
        return True

    def update_frame(self):
        try:
            if not self.capture:
                return
            frame_result = self.capture.get_frame()
            if frame_result is None:
                return # No frame yet

            frame_bytes_obj, in_w, in_h = frame_result
            frame = frame_bytes_obj

            # Only re-initialize upscaler if input size or scale changes
            scale = self.scale_slider.value() / 10.0
            reinit_needed = False
            if not self.upscaler or not self.upscaler_initialized:
                print(f"[DEBUG] Upscaler needs init: not initialized yet")
                reinit_needed = True
            elif (self._last_in_w != in_w or self._last_in_h != in_h):
                print(f"[DEBUG] Upscaler needs re-init: input size changed {self._last_in_w}x{self._last_in_h} -> {in_w}x{in_h}")
                reinit_needed = True
            elif (self._last_scale != scale):
                print(f"[DEBUG] Upscaler needs re-init: scale changed {self._last_scale} -> {scale}")
                reinit_needed = True

            if reinit_needed:
                self._last_in_w = in_w
                self._last_in_h = in_h
                self._last_scale = scale
                if not self.init_upscaler(in_w, in_h, scale):
                    return # Stop if upscaler failed to init

            # Only start a new upscale if no worker is running
            if self._upscale_thread is not None:
                print("[DEBUG] Skipping frame: upscale worker still running")
                return

            # Calculate output dimensions
            scale = self.upscale_scale
            out_w = int(in_w * scale)
            out_h = int(in_h * scale)

            # Start worker thread for upscaling
            print(f"[DEBUG] Starting upscale worker at {time.strftime('%H:%M:%S')}")
            self._upscale_thread = QThread()
            self._upscale_worker = UpscaleWorker(self.upscaler, frame, in_w, in_h, out_w, out_h, scale)
            self._upscale_worker.moveToThread(self._upscale_thread)
            self._upscale_thread.started.connect(self._upscale_worker.run)
            self._upscale_worker.finished.connect(self.on_upscale_finished)
            self._upscale_worker.error.connect(self.on_upscale_error)
            self._upscale_worker.finished.connect(self._upscale_thread.quit)
            self._upscale_worker.finished.connect(self._upscale_worker.deleteLater)
            self._upscale_thread.finished.connect(self._upscale_thread.deleteLater)
            self._upscale_thread.finished.connect(self._clear_upscale_thread)
            self._upscale_thread.start()
        except Exception as e:
            print(f"[EXCEPTION] update_frame: {e}")
            import traceback
            traceback.print_exc()

    def _clear_upscale_thread(self):
        self._upscale_thread = None
        self._upscale_worker = None

    def on_upscale_finished(self, out_bytes, out_w, out_h, elapsed):
        print(f'[DEBUG] on_upscale_finished: {id(self)}')
        print(f"[DEBUG] Upscale finished in {elapsed*1000:.2f} ms at {time.strftime('%H:%M:%S')}")
        # Update the GUI with the upscaled image
        if out_bytes:
            qimg = QImage(out_bytes, out_w, out_h, QImage.Format_RGBA8888)
            pixmap = QPixmap.fromImage(qimg)
            self.output_preview.setPixmap(pixmap)
            # Update FPS
            inst_fps = 1.0 / elapsed if elapsed > 0 else 0.0
            self.fps = 0.95 * self.fps + 0.05 * inst_fps if self.fps > 0 else inst_fps
            self.overlay.setText(f"Input: {out_w//self.upscale_scale:.0f}×{out_h//self.upscale_scale:.0f}\nUpscaled: {out_w}×{out_h}\nFPS: {self.fps:.1f}")
            self.status_bar.setText(f"Frame Time: {elapsed * 1000:.1f} ms   FPS: {self.fps:.1f}   Resolution: {out_w//self.upscale_scale:.0f}×{out_h//self.upscale_scale:.0f} → {out_w}×{out_h}")
            self.profiler_signal.emit(elapsed * 1000, self.fps, out_w//self.upscale_scale, out_h//self.upscale_scale)
            if self.fps < 30:
                self.warning_signal.emit(f"Warning: Low FPS ({self.fps:.1f})", True)
            else:
                self.warning_signal.emit("", False)
            self.last_frame_time = time.perf_counter()
        # The timer will continue to fire at the set interval

    def on_upscale_error(self, error_msg):
        print(f'[DEBUG] on_upscale_error: {id(self)}')
        import traceback
        print(f"[GUI] Error in upscaling: {error_msg}")
        self.status_bar.setText(f"Error: {str(error_msg)}")
        self.upscaler = None
        self.upscaler_initialized = False
        traceback.print_exc()
        # The timer will continue to fire at the set interval

class SettingsScreen(QWidget):
    def __init__(self, live_feed_screen=None):
        super().__init__()
        self.live_feed_screen = live_feed_screen
        layout = QVBoxLayout(self)
        # Input & Capture
        input_group = QGroupBox("Input & Capture")
        input_form = QFormLayout(input_group)
        self.input_source = QComboBox()
        self.input_source.addItems(["Screen Capture", "Video File", "Static Image"])
        self.input_source.setToolTip("Select the input source for upscaling.")
        self.backend = QComboBox()
        self.backend.addItems(["Auto", "Win32", "X11", "Wayland"])
        self.backend.setToolTip("Select the backend for capture (platform dependent).")
        self.capture_btn = QPushButton("Capture Frame")
        self.capture_btn.setToolTip("Capture a single frame from the selected input.")
        self.capture_btn.clicked.connect(self.capture_frame)
        self.refresh_btn = QPushButton("Refresh Devices")
        self.refresh_btn.setToolTip("Refresh the list of available input devices/windows.")
        self.refresh_btn.clicked.connect(self.refresh_devices)
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
        self.scale_slider.setToolTip("Set the upscaling factor (1.0× to 4.0×).")
        self.method = QComboBox()
        self.method.addItems(["AMD FSR", "NVIDIA NIS", "Pure Rust Interpolation"])
        self.method.setToolTip("Select the upscaling algorithm.")
        self.advanced_btn = QPushButton("Advanced Algorithm Settings")
        self.advanced_btn.setToolTip("Open advanced settings for the selected algorithm.")
        self.advanced_btn.clicked.connect(self.open_advanced_settings)
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
        self.motion_slider.setToolTip("Adjust motion sensitivity for optical flow.")
        self.blend_slider = QSlider(Qt.Horizontal)
        self.blend_slider.setRange(0, 100)
        self.blend_slider.setValue(50)
        self.blend_slider.setToolTip("Adjust blending ratio for interpolation.")
        self.smooth_slider = QSlider(Qt.Horizontal)
        self.smooth_slider.setRange(0, 100)
        self.smooth_slider.setValue(50)
        self.smooth_slider.setToolTip("Adjust smoothing factor for interpolation.")
        self.gpu_shader = QCheckBox("Use GPU Shader")
        self.gpu_shader.setToolTip("Enable GPU shader for interpolation.")
        self.reload_shader = QPushButton("Reload Shader")
        self.reload_shader.setToolTip("Reload the current shader from disk.")
        interp_form.addRow("Motion Sensitivity:", self.motion_slider)
        interp_form.addRow("Blending Ratio:", self.blend_slider)
        interp_form.addRow("Smoothing Factor:", self.smooth_slider)
        interp_form.addRow(self.gpu_shader, self.reload_shader)
        # Compute Settings
        compute_group = QGroupBox("Compute Settings")
        compute_form = QFormLayout(compute_group)
        self.render_mode = QComboBox()
        self.render_mode.addItems(["GPU Accelerated", "CPU-only"])
        self.render_mode.setToolTip("Choose between GPU or CPU rendering.")
        self.optimize_perf = QCheckBox("Optimize for Performance")
        self.optimize_perf.setToolTip("Trade off quality for speed.")
        compute_form.addRow("Rendering Mode:", self.render_mode)
        compute_form.addRow(self.optimize_perf)
        # Control Buttons
        control_group = QGroupBox("Controls")
        control_layout = QHBoxLayout(control_group)
        self.start_btn = QPushButton("Start")
        self.start_btn.setToolTip("Start real-time upscaling.")
        self.start_btn.clicked.connect(self.start_pipeline)
        self.pause_btn = QPushButton("Pause/Resume")
        self.pause_btn.setToolTip("Pause or resume the pipeline.")
        self.pause_btn.clicked.connect(self.pause_pipeline)
        self.stop_btn = QPushButton("Stop")
        self.stop_btn.setToolTip("Stop the pipeline.")
        self.stop_btn.clicked.connect(self.stop_pipeline)
        self.export_btn = QPushButton("Export Frame")
        self.export_btn.setToolTip("Export the current upscaled frame.")
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

    def capture_frame(self):
        # Example: trigger a single frame capture in LiveFeedScreen
        if self.live_feed_screen:
            self.live_feed_screen.start_capture()
            self.live_feed_screen.timer.singleShot(100, self.live_feed_screen.stop_capture)

    def refresh_devices(self):
        if self.live_feed_screen:
            self.live_feed_screen.refresh_windows()

    def open_advanced_settings(self):
        # Placeholder: show a message or open a modal
        from PySide6.QtWidgets import QMessageBox
        QMessageBox.information(self, "Advanced Settings", "Advanced algorithm settings coming soon!")

    def start_pipeline(self):
        if self.live_feed_screen:
            self.live_feed_screen.start_capture()

    def pause_pipeline(self):
        # Placeholder: implement pause/resume logic
        from PySide6.QtWidgets import QMessageBox
        QMessageBox.information(self, "Pause/Resume", "Pause/Resume not yet implemented.")

    def stop_pipeline(self):
        if self.live_feed_screen:
            self.live_feed_screen.stop_capture()

class DebugScreen(QWidget):
    def __init__(self):
        super().__init__()
        layout = QVBoxLayout(self)
        self.log_group = QGroupBox("Log (Collapsible)")
        self.log_group.setCheckable(True)
        self.log_group.setChecked(True)
        log_layout = QVBoxLayout(self.log_group)
        self.log_view = QLabel("[Logs will appear here]")
        self.log_view.setStyleSheet("background: #222; color: #f88; font-family: monospace; padding: 8px;")
        self.log_view.setWordWrap(True)
        log_layout.addWidget(self.log_view)
        self.profiler_group = QGroupBox("Profiler")
        profiler_layout = QVBoxLayout(self.profiler_group)
        self.profiler_label = QLabel("[Profiler graph/timeline placeholder]")
        self.profiler_label.setStyleSheet("background: #222; color: #8ff; padding: 8px;")
        profiler_layout.addWidget(self.profiler_label)
        self.warning_label = QLabel("[Overlay warnings: FPS drop, errors, etc.]")
        self.warning_label.setStyleSheet("background: #400; color: #fff; padding: 6px; border-radius: 6px;")
        self.warning_label.setVisible(False)
        layout.addWidget(self.log_group)
        layout.addWidget(self.profiler_group)
        layout.addWidget(self.warning_label)
        layout.addStretch()
    def append_log(self, msg):
        prev = self.log_view.text()
        self.log_view.setText(prev + "\n" + msg)
        print(f"[GUI LOG] {msg}")
    def update_profiler(self, frame_time, fps, in_w, in_h):
        self.profiler_label.setText(f"Frame: {frame_time:.1f} ms | FPS: {fps:.1f} | Input: {in_w}×{in_h}")
        print(f"[PROFILER] Frame: {frame_time:.1f} ms | FPS: {fps:.1f} | Input: {in_w}×{in_h}")
    def show_warning(self, msg, show):
        self.warning_label.setText(msg)
        self.warning_label.setVisible(show)
        if show and msg:
            print(f"[GUI WARNING] {msg}")

class AdvancedScreen(QWidget):
    def __init__(self, live_feed_screen=None):
        super().__init__()
        self.live_feed_screen = live_feed_screen
        layout = QVBoxLayout(self)
        shader_group = QGroupBox("Shader & Engine")
        shader_form = QFormLayout(shader_group)
        self.shader_path = QLabel("[WGSL Shader Path]")
        self.reload_shader = QPushButton("Reload Shader")
        self.reload_shader.clicked.connect(self.reload_shader_backend)
        self.hot_reload = QCheckBox("Enable Hot Reload")
        shader_form.addRow("Custom WGSL Shader Path:", self.shader_path)
        shader_form.addRow(self.reload_shader)
        shader_form.addRow(self.hot_reload)
        concurrency_group = QGroupBox("Concurrency")
        concurrency_form = QFormLayout(concurrency_group)
        self.thread_count = QSpinBox()
        self.thread_count.setRange(1, 64)
        self.thread_count.setValue(4)
        self.thread_count.valueChanged.connect(self.update_threads)
        self.auto_scale = QCheckBox("Auto-scale threads")
        self.rayon_toggle = QCheckBox("Use Rayon/Crossbeam backend")
        concurrency_form.addRow("Thread Count:", self.thread_count)
        concurrency_form.addRow(self.auto_scale)
        concurrency_form.addRow(self.rayon_toggle)
        memory_group = QGroupBox("Memory Options")
        memory_form = QFormLayout(memory_group)
        self.buffer_pool = QSpinBox()
        self.buffer_pool.setRange(1, 32)
        self.buffer_pool.setValue(4)
        self.buffer_pool.valueChanged.connect(self.update_buffer_pool)
        self.gpu_allocator = QComboBox()
        self.gpu_allocator.addItems(["Default", "Aggressive", "Conservative"])
        self.gpu_allocator.currentTextChanged.connect(self.update_gpu_allocator)
        memory_form.addRow("Buffer Pool Size:", self.buffer_pool)
        memory_form.addRow("GPU Allocator Preset:", self.gpu_allocator)
        layout.addWidget(shader_group)
        layout.addWidget(concurrency_group)
        layout.addWidget(memory_group)
        layout.addStretch()
    def get_upscaler(self):
        if self.live_feed_screen and self.live_feed_screen.upscaler:
            return self.live_feed_screen.upscaler
        return None
    def reload_shader_backend(self):
        upscaler = self.get_upscaler()
        if upscaler:
            # For demo, use a placeholder path
            path = self.shader_path.text() or "shader.wgsl"
            try:
                upscaler.reload_shader(path)
            except Exception as e:
                from PySide6.QtWidgets import QMessageBox
                QMessageBox.warning(self, "Reload Shader", f"Error: {e}")
        else:
            from PySide6.QtWidgets import QMessageBox
            QMessageBox.warning(self, "Reload Shader", "No upscaler instance available.")
    def update_threads(self, val):
        upscaler = self.get_upscaler()
        if upscaler:
            try:
                upscaler.set_thread_count(val)
            except Exception as e:
                from PySide6.QtWidgets import QMessageBox
                QMessageBox.warning(self, "Thread Count", f"Error: {e}")
    def update_buffer_pool(self, val):
        upscaler = self.get_upscaler()
        if upscaler:
            try:
                upscaler.set_buffer_pool_size(val)
            except Exception as e:
                from PySide6.QtWidgets import QMessageBox
                QMessageBox.warning(self, "Buffer Pool", f"Error: {e}")
    def update_gpu_allocator(self, val):
        upscaler = self.get_upscaler()
        if upscaler:
            try:
                upscaler.set_gpu_allocator(val)
            except Exception as e:
                from PySide6.QtWidgets import QMessageBox
                QMessageBox.warning(self, "GPU Allocator", f"Error: {e}")

class UIAccessibilityScreen(QWidget):
    def __init__(self):
        super().__init__()
        layout = QVBoxLayout(self)
        theme_group = QGroupBox("Theme & Appearance")
        theme_form = QFormLayout(theme_group)
        self.theme_select = QComboBox()
        self.theme_select.addItems(["Dark", "Light", "System Default"])
        self.theme_select.currentTextChanged.connect(self.apply_theme)
        self.font_scale = QSlider(Qt.Horizontal)
        self.font_scale.setRange(8, 32)
        self.font_scale.setValue(14)
        self.font_label = QLabel("14pt")
        self.font_scale.valueChanged.connect(lambda: self.font_label.setText(f"{self.font_scale.value()}pt"))
        self.font_scale.valueChanged.connect(self.apply_font_scale)
        theme_form.addRow("Theme:", self.theme_select)
        theme_form.addRow("Font Scale:", self.font_scale)
        theme_form.addRow("", self.font_label)
        shortcut_group = QGroupBox("Keyboard Shortcuts")
        shortcut_layout = QVBoxLayout(shortcut_group)
        self.shortcut_label = QLabel("[Shortcuts view/editor placeholder]")
        shortcut_layout.addWidget(self.shortcut_label)
        config_group = QGroupBox("Configuration")
        config_layout = QHBoxLayout(config_group)
        self.save_btn = QPushButton("Save Config")
        self.save_btn.clicked.connect(self.save_config)
        self.load_btn = QPushButton("Load Config")
        self.load_btn.clicked.connect(self.load_config)
        config_layout.addWidget(self.save_btn)
        config_layout.addWidget(self.load_btn)
        layout.addWidget(theme_group)
        layout.addWidget(shortcut_group)
        layout.addWidget(config_group)
        layout.addStretch()
    def apply_theme(self, theme):
        # Apply theme globally
        if theme == "Dark":
            QApplication.instance().setStyleSheet("QMainWindow { background: #181818; } QLabel { color: #ccc; }")
        elif theme == "Light":
            QApplication.instance().setStyleSheet("QMainWindow { background: #f8f8f8; } QLabel { color: #222; }")
        else:
            QApplication.instance().setStyleSheet("")
    def apply_font_scale(self, val):
        QApplication.instance().setStyleSheet(QApplication.instance().styleSheet() + f" QLabel {{ font-size: {val}px; }}")
    def save_config(self):
        # Placeholder: save config to file
        from PySide6.QtWidgets import QMessageBox
        QMessageBox.information(self, "Save Config", "Config save not yet implemented.")
    def load_config(self):
        # Placeholder: load config from file
        from PySide6.QtWidgets import QMessageBox
        QMessageBox.information(self, "Load Config", "Config load not yet implemented.")

class BenchmarkWorker(QObject):
    progress = Signal(int)
    finished = Signal(list)
    error = Signal(str)
    
    def __init__(self, config):
        super().__init__()
        self.config = config
    
    def run_single_benchmark(self):
        try:
            result = run_benchmark(
                technology=self.config['technology'],
                quality=self.config['quality'],
                input_width=self.config['input_width'],
                input_height=self.config['input_height'],
                scale_factor=self.config['scale_factor'],
                frame_count=self.config['frame_count']
            )
            self.finished.emit([result] if result else [])
        except Exception as e:
            traceback.print_exc()
            self.error.emit(f"Benchmark error: {str(e)}")
    
    def run_comparison(self):
        try:
            # Emit progress updates as we go
            self.progress.emit(10)  # Started
            
            results = run_comparison_benchmark(
                input_width=self.config['input_width'],
                input_height=self.config['input_height'],
                scale_factor=self.config['scale_factor'],
                frame_count=self.config['frame_count']
            )
            
            self.progress.emit(100)  # Completed
            self.finished.emit(results)
        except Exception as e:
            traceback.print_exc()
            self.error.emit(f"Comparison benchmark error: {str(e)}")

class BenchmarkScreen(QWidget):
    def __init__(self):
        super().__init__()
        self.thread = None
        self.worker = None
        self.results = []
        
        # Main layout
        layout = QVBoxLayout(self)
        
        # Configuration section
        config_group = QGroupBox("Benchmark Configuration")
        config_form = QFormLayout(config_group)
        
        # Upscaling technology selection
        self.tech_combo = QComboBox()
        self.tech_combo.addItems(["Auto (Best for GPU)", "FSR", "DLSS", "Basic"])
        
        # Quality selection
        self.quality_combo = QComboBox()
        self.quality_combo.addItems(["ultra", "quality", "balanced", "performance"])
        
        # Resolution settings
        self.width_spin = QSpinBox()
        self.width_spin.setRange(640, 3840)
        self.width_spin.setValue(1920)
        self.width_spin.setSingleStep(160)
        
        self.height_spin = QSpinBox()
        self.height_spin.setRange(480, 2160)
        self.height_spin.setValue(1080)
        self.height_spin.setSingleStep(90)
        
        # Resolution presets
        self.res_preset = QComboBox()
        self.res_preset.addItems(["Custom", "720p", "1080p", "1440p", "4K"])
        self.res_preset.currentTextChanged.connect(self.apply_resolution_preset)
        
        # Scale factor
        self.scale_slider = QSlider(Qt.Horizontal)
        self.scale_slider.setRange(10, 40)
        self.scale_slider.setValue(20)
        self.scale_label = QLabel("2.0×")
        self.scale_slider.valueChanged.connect(
            lambda: self.scale_label.setText(f"{self.scale_slider.value()/10.0:.1f}×")
        )
        
        # Frame count
        self.frame_count = QSpinBox()
        self.frame_count.setRange(10, 1000)
        self.frame_count.setValue(100)
        self.frame_count.setSingleStep(10)
        
        # Add all to config form
        config_form.addRow("Technology:", self.tech_combo)
        config_form.addRow("Quality:", self.quality_combo)
        config_form.addRow("Resolution preset:", self.res_preset)
        
        res_layout = QHBoxLayout()
        res_layout.addWidget(self.width_spin)
        res_layout.addWidget(QLabel("×"))
        res_layout.addWidget(self.height_spin)
        config_form.addRow("Resolution:", res_layout)
        
        config_form.addRow("Scale Factor:", self.scale_slider)
        config_form.addRow("", self.scale_label)
        config_form.addRow("Frame Count:", self.frame_count)
        
        # Benchmark buttons
        button_layout = QHBoxLayout()
        self.run_btn = QPushButton("Run Benchmark")
        self.run_btn.clicked.connect(self.run_single_benchmark)
        
        self.compare_btn = QPushButton("Run Comparison")
        self.compare_btn.clicked.connect(self.run_comparison_benchmark)
        
        self.plot_btn = QPushButton("Plot Results")
        self.plot_btn.clicked.connect(self.plot_results)
        self.plot_btn.setEnabled(False)  # Disabled until we have results
        
        self.export_btn = QPushButton("Export Results")
        self.export_btn.clicked.connect(self.export_results)
        self.export_btn.setEnabled(False)  # Disabled until we have results
        
        button_layout.addWidget(self.run_btn)
        button_layout.addWidget(self.compare_btn)
        button_layout.addWidget(self.plot_btn)
        button_layout.addWidget(self.export_btn)
        
        # Progress bar
        self.progress_bar = QProgressBar()
        self.progress_bar.setRange(0, 100)
        self.progress_bar.setValue(0)
        
        # Results display
        self.results_group = QGroupBox("Benchmark Results")
        results_layout = QVBoxLayout(self.results_group)
        self.results_text = QLabel("Run a benchmark to see results here.")
        self.results_text.setWordWrap(True)
        results_layout.addWidget(self.results_text)
        
        # Add all widgets to main layout
        layout.addWidget(config_group)
        layout.addLayout(button_layout)
        layout.addWidget(self.progress_bar)
        layout.addWidget(self.results_group)
        
        # Check if benchmarking is available
        if run_benchmark is None:
            self.run_btn.setEnabled(False)
            self.compare_btn.setEnabled(False)
            self.results_text.setText("ERROR: Benchmarking not available. nu_scaler_core module is missing.")
    
    def apply_resolution_preset(self, preset):
        """Apply a resolution preset."""
        if preset == "720p":
            self.width_spin.setValue(1280)
            self.height_spin.setValue(720)
        elif preset == "1080p":
            self.width_spin.setValue(1920)
            self.height_spin.setValue(1080)
        elif preset == "1440p":
            self.width_spin.setValue(2560)
            self.height_spin.setValue(1440)
        elif preset == "4K":
            self.width_spin.setValue(3840)
            self.height_spin.setValue(2160)
        # For "Custom", do nothing and let the user set values
    
    def get_config(self):
        """Get benchmark configuration from UI."""
        tech_map = {
            "Auto (Best for GPU)": "auto",
            "FSR": "fsr",
            "DLSS": "dlss",
            "Basic": "wgpu"
        }
        
        return {
            'technology': tech_map.get(self.tech_combo.currentText(), "auto"),
            'quality': self.quality_combo.currentText(),
            'input_width': self.width_spin.value(),
            'input_height': self.height_spin.value(),
            'scale_factor': self.scale_slider.value() / 10.0,
            'frame_count': self.frame_count.value()
        }
    
    def run_single_benchmark(self):
        """Run a single benchmark with the current configuration."""
        if run_benchmark is None:
            return
        
        self.set_ui_running(True)
        self.results_text.setText("Running benchmark...")
        self.progress_bar.setValue(0)
        
        # Create worker and thread
        config = self.get_config()
        self.worker = BenchmarkWorker(config)
        self.thread = QThread()
        
        # Move worker to thread
        self.worker.moveToThread(self.thread)
        
        # Connect signals
        self.thread.started.connect(self.worker.run_single_benchmark)
        self.worker.finished.connect(self.on_benchmark_finished)
        self.worker.error.connect(self.on_benchmark_error)
        self.worker.finished.connect(self.thread.quit)
        self.worker.finished.connect(self.worker.deleteLater)
        self.thread.finished.connect(self.thread.deleteLater)
        
        # Start the thread
        self.thread.start()
    
    def run_comparison_benchmark(self):
        """Run a comparison benchmark across technologies."""
        if run_comparison_benchmark is None:
            return
        
        self.set_ui_running(True)
        self.results_text.setText("Running comparison benchmark across upscaling technologies...")
        self.progress_bar.setValue(0)
        
        # Create worker and thread
        config = self.get_config()
        self.worker = BenchmarkWorker(config)
        self.thread = QThread()
        
        # Move worker to thread
        self.worker.moveToThread(self.thread)
        
        # Connect signals
        self.thread.started.connect(self.worker.run_comparison)
        self.worker.progress.connect(self.progress_bar.setValue)
        self.worker.finished.connect(self.on_benchmark_finished)
        self.worker.error.connect(self.on_benchmark_error)
        self.worker.finished.connect(self.thread.quit)
        self.worker.finished.connect(self.worker.deleteLater)
        self.thread.finished.connect(self.thread.deleteLater)
        
        # Start the thread
        self.thread.start()
    
    def on_benchmark_finished(self, results):
        """Handle benchmark completion."""
        self.results = results
        self.set_ui_running(False)
        
        if not results:
            self.results_text.setText("Benchmark completed but no results were returned.")
            return
        
        # Format results for display
        text = ""
        for i, result in enumerate(results):
            text += f"--- Result {i+1} ---\n{str(result)}\n\n"
        
        self.results_text.setText(text)
        self.plot_btn.setEnabled(True)
        self.export_btn.setEnabled(True)
    
    def on_benchmark_error(self, error_msg):
        """Handle benchmark errors."""
        self.set_ui_running(False)
        self.results_text.setText(f"ERROR: {error_msg}")
    
    def set_ui_running(self, is_running):
        """Update UI state based on whether benchmark is running."""
        self.run_btn.setEnabled(not is_running)
        self.compare_btn.setEnabled(not is_running)
        self.tech_combo.setEnabled(not is_running)
        self.quality_combo.setEnabled(not is_running)
        self.width_spin.setEnabled(not is_running)
        self.height_spin.setEnabled(not is_running)
        self.res_preset.setEnabled(not is_running)
        self.scale_slider.setEnabled(not is_running)
        self.frame_count.setEnabled(not is_running)
    
    def plot_results(self):
        """Plot benchmark results using matplotlib."""
        if not self.results:
            return
        
        try:
            plot_benchmark_results(self.results, "Nu Scaler Benchmark Results")
        except Exception as e:
            from PySide6.QtWidgets import QMessageBox
            QMessageBox.warning(self, "Plot Error", f"Error plotting results: {str(e)}")
    
    def export_results(self):
        """Export results to a file."""
        if not self.results:
            return
        
        try:
            from PySide6.QtWidgets import QFileDialog
            
            filename, _ = QFileDialog.getSaveFileName(
                self, "Export Results", "", "CSV Files (*.csv);;Text Files (*.txt);;All Files (*)"
            )
            
            if not filename:
                return
            
            if filename.endswith('.csv'):
                self.export_to_csv(filename)
            else:
                self.export_to_text(filename)
                
            from PySide6.QtWidgets import QMessageBox
            QMessageBox.information(self, "Export", f"Results exported to {filename}")
                
        except Exception as e:
            from PySide6.QtWidgets import QMessageBox
            QMessageBox.warning(self, "Export Error", f"Error exporting results: {str(e)}")
    
    def export_to_csv(self, filename):
        """Export results to CSV format."""
        with open(filename, 'w') as f:
            # Write header
            f.write("Upscaler,Technology,Quality,InputWidth,InputHeight,OutputWidth,OutputHeight,"
                   "ScaleFactor,FrameTimeMs,FPS,FramesProcessed,TotalDurationMs\n")
            
            # Write data rows
            for result in self.results:
                f.write(f"{result.upscaler_name},{result.technology},{result.quality},"
                        f"{result.input_width},{result.input_height},{result.output_width},"
                        f"{result.output_height},{result.scale_factor},{result.avg_frame_time_ms},"
                        f"{result.fps},{result.frames_processed},{result.total_duration_ms}\n")
    
    def export_to_text(self, filename):
        """Export results to plain text format."""
        with open(filename, 'w') as f:
            f.write("Nu Scaler Benchmark Results\n")
            f.write("===========================\n\n")
            
            for i, result in enumerate(self.results):
                f.write(f"Result {i+1}:\n")
                f.write(str(result))
                f.write("\n\n")

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Nu_Scaler")
        self.resize(1024, 768)
        
        # Create the main widget
        self.main_widget = LiveFeedScreen(self)

        # Create debug screen
        self.debug_screen = DebugScreen()
        # Create advanced screen
        self.advanced_screen = AdvancedScreen(live_feed_screen=self.main_widget)
        # Create UI accessibility screen
        self.ui_screen = UIAccessibilityScreen()
        # Create benchmark screen
        self.benchmark_screen = BenchmarkScreen()

        # Create stacked widget
        self.stack = QStackedWidget()
        self.screens = {
            0: self.main_widget,
            1: SettingsScreen(live_feed_screen=self.main_widget),
            2: self.benchmark_screen,
            3: self.debug_screen,
            4: self.advanced_screen,
            5: self.ui_screen,
        }
        for i in range(6):
            self.stack.addWidget(self.screens[i])
        self.sidebar = QListWidget()
        self.sidebar.addItems([
            "Live Feed",
            "Settings",
            "Benchmark",
            "Debug",
            "Advanced",
            "UI & Accessibility"
        ])
        self.sidebar.setFixedWidth(180)
        self.sidebar.setStyleSheet("background: #232323; color: #bbb; font-size: 16px;")
        self.sidebar.currentRowChanged.connect(self.stack.setCurrentIndex)
        main_layout = QHBoxLayout()
        main_layout.addWidget(self.sidebar)
        main_layout.addWidget(self.stack)
        main_widget = QWidget(self)
        main_widget.setLayout(main_layout)
        self.setCentralWidget(main_widget)
        self.apply_theme()
        # Connect LiveFeedScreen signals to DebugScreen
        self.main_widget.log_signal.connect(self.debug_screen.append_log)
        self.main_widget.profiler_signal.connect(self.debug_screen.update_profiler)
        self.main_widget.warning_signal.connect(self.debug_screen.show_warning)

        # Create menu bar
        self.menu_bar = self.menuBar()
        self.file_menu = self.menu_bar.addMenu("File")
        self.help_menu = self.menu_bar.addMenu("Help")
        # Create exit action
        exit_action = QAction("Exit", self)
        exit_action.triggered.connect(qApp.quit)
        self.file_menu.addAction(exit_action)
        # Create about action
        about_action = QAction("About", self)
        about_action.triggered.connect(self.show_about_dialog)
        self.help_menu.addAction(about_action)
        print('[DEBUG] MainWindow: Before upscaler optimization')
        # Heavy call: optimize_upscaler(self.upscaler)
        # Heavy call: force_gpu_activation(self.upscaler)
        # if nu_scaler_core is not None:
        #     try:
        #         if hasattr(nu_scaler_core, 'create_advanced_upscaler'):
        #             self.upscaler = nu_scaler_core.create_advanced_upscaler('quality')
        #             optimize_upscaler(self.upscaler)
        #             print("[GUI] Application startup: GPU optimizations applied")
        #     except Exception as e:
        #         print(f"[GUI] Error initializing optimized upscaler: {e}")
        print('[DEBUG] MainWindow: After upscaler optimization')

    def apply_theme(self):
        self.setStyleSheet("""
            QMainWindow { background: #181818; }
            QLabel { font-family: 'Segoe UI', 'Arial', sans-serif; }
            QListWidget::item:selected { background: #444; color: #fff; }
            QFrame[frameShape=\"4\"] { border: 1px solid #444; border-radius: 8px; }
        """)

    def show_about_dialog(self):
        from PySide6.QtWidgets import QMessageBox
        QMessageBox.information(self, "About Nu_Scaler", "Nu_Scaler is a high-performance upscaling application.")

def run_gui():
    app = QApplication(sys.argv)
    win = MainWindow()
    win.show() # Show first
    win.showMaximized() # Then maximize
    sys.exit(app.exec())

if __name__ == "__main__":
    run_gui()