import sys
# Remove sys.path.insert for pyd_test

# --- PyGetWindow Diagnostics ---
# print("--- PyGetWindow Import Diagnostics ---")
# try:
#     import pygetwindow
#     print(f"Successfully imported pygetwindow.")
#     print(f"pygetwindow version: {getattr(pygetwindow, '__version__', 'Not found')}")
#     print(f"pygetwindow file location: {getattr(pygetwindow, '__file__', 'Not found')}")
#     print(f"pygetwindow dir(): {dir(pygetwindow)}")
#     if hasattr(pygetwindow, 'getWindowsWithPid'):
#         print("getWindowsWithPid IS found in pygetwindow.")
#     else:
#         print("getWindowsWithPid IS NOT found in pygetwindow. THIS IS THE PROBLEM.")
# except ImportError as e_diag:
#     print(f"Failed to import pygetwindow: {e_diag}")
#     pygetwindow = None # Ensure it's None if import fails
# except Exception as e_diag_other:
#     print(f"An unexpected error occurred during pygetwindow diagnostics: {e_diag_other}")
#     pygetwindow = None
# print("--- End PyGetWindow Import Diagnostics ---")
# --- End PyGetWindow Diagnostics ---

from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QLabel, QListWidget, QStackedWidget, QFrame,
    QPushButton, QComboBox, QSpinBox, QCheckBox, QSlider, QGroupBox, QFormLayout, QProgressBar, QFileDialog, QSizePolicy
)
from PySide6.QtCore import Qt, QTimer, Signal, QThread, QObject, Slot, QEvent
from PySide6.QtGui import QPixmap, QImage, QAction, QKeySequence, QPainter, QColor, QFont, QShortcut
import time
import random
import traceback
import threading
import psutil
import os

# Import for Windows API access if on Windows
if os.name == 'nt':
    try:
        import win32gui
        import win32process
        import win32con # For window styles, if needed later
        print("[main.py] Successfully imported pywin32 modules (win32gui, win32process).")
    except ImportError:
        win32gui = None
        win32process = None
        win32con = None
        print("[main.py] pywin32 library not found. Process capture features for Windows will be limited.")
else:
    win32gui = None
    win32process = None
    win32con = None

# Import the Rust extension as 'nu_scaler'
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

# Import Python helper modules from nu_scaler_py
try:
    from .benchmark import run_benchmark, run_comparison_benchmark, BenchmarkResult, plot_benchmark_results
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
print(f"[main.py] DLSS available: {hasattr(nu_scaler_core, 'DlssUpscaler')}")

# Add import for GPU optimization
try:
    from .gpu_optimizer import optimize_upscaler, force_gpu_activation
except ImportError as e:
    print(f"[main.py] ImportError when importing gpu_optimizer: {e}")
    optimize_upscaler = None
    force_gpu_activation = None
except Exception as e:
    print(f"[main.py] Error when importing gpu_optimizer: {e}")
    traceback.print_exc()
    optimize_upscaler = None
    force_gpu_activation = None

class AspectRatioPreview(QLabel):
    """
    QLabel-based widget for displaying a QPixmap with aspect-ratio-aware scaling and a modern overlay.
    Supports double-click to toggle full-screen. Overlay is always visible and customizable.
    """
    doubleClicked = Signal() # Signal for when the widget is double-clicked

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setAlignment(Qt.AlignCenter)
        self.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)
        self.setStyleSheet("background: #181818; border: 1px solid #444;")
        self._pixmap = None
        self._overlay_text = ""
        self.installEventFilter(self)

    def set_pixmap(self, pixmap: QPixmap):
        """Set the pixmap to display."""
        self._pixmap = pixmap
        self.update()

    def set_overlay(self, text: str):
        """Set the overlay text."""
        self._overlay_text = text
        self.update()

    def eventFilter(self, obj, event):
        if event.type() == QEvent.MouseButtonDblClick:
            self.doubleClicked.emit() # Emit signal instead of calling toggle_fullscreen
            return True
        return super().eventFilter(obj, event)

    def paintEvent(self, event):
        super().paintEvent(event)
        painter = QPainter(self)
        # Draw the scaled pixmap centered
        if self._pixmap:
            scaled = self._pixmap.scaled(self.size(), Qt.KeepAspectRatio, Qt.SmoothTransformation)
            x = (self.width() - scaled.width()) // 2
            y = (self.height() - scaled.height()) // 2
            painter.drawPixmap(x, y, scaled)
        # Draw overlay
        if self._overlay_text:
            overlay_rect = self.rect().adjusted(12, 12, -12, -12)
            painter.setRenderHint(QPainter.Antialiasing)
            painter.setBrush(QColor(30, 30, 30, 180))
            painter.setPen(Qt.NoPen)
            painter.drawRoundedRect(overlay_rect, 12, 12)
            painter.setPen(QColor(255, 255, 255))
            font = QFont()
            font.setPointSize(12)
            painter.setFont(font)
            painter.drawText(overlay_rect, Qt.AlignTop | Qt.AlignRight, self._overlay_text)

class FullScreenDisplayWindow(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("NuScaler - Full Screen Output")
        self.setWindowFlags(Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.Tool)
        self.setStyleSheet("background-color: black;") # Ensure no gaps

        self._layout = QVBoxLayout(self)
        self._layout.setContentsMargins(0,0,0,0)
        self.preview_widget = AspectRatioPreview(self)
        self._layout.addWidget(self.preview_widget)
        self.setLayout(self._layout)

        # Close on double click on its own preview widget
        self.preview_widget.doubleClicked.connect(self.close)

    def set_pixmap(self, pixmap: QPixmap):
        if pixmap and not pixmap.isNull():
            self.preview_widget.set_pixmap(pixmap)
        else:
            # Optionally, clear or set a placeholder if pixmap is None/Null
            self.preview_widget.set_pixmap(QPixmap()) # Clear

    def set_overlay(self, text: str):
        self.preview_widget.set_overlay(text)

    def keyPressEvent(self, event: QEvent): # QKeyEvent is more specific but QEvent works for key() check
        if event.key() == Qt.Key_Escape:
            self.close()
        else:
            super().keyPressEvent(event)

    def get_current_pixmap(self): # Helper for LiveFeedScreen if needed
        return self.preview_widget._pixmap

    def get_current_overlay_text(self): # Helper for LiveFeedScreen if needed
        return self.preview_widget._overlay_text

class CornerOverlayWindow(QWidget):
    """Window to display upscaled content in a corner while allowing click-through to the desktop"""
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("NuScaler - Corner Overlay")
        # Set flags for click-through, always-on-top window with no frame
        self.setWindowFlags(Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.Tool)
        self.setAttribute(Qt.WA_TranslucentBackground)
        self.setAttribute(Qt.WA_TransparentForMouseEvents, True)  # Makes the window click-through
        
        # Create layout with no margins for the preview
        self._layout = QVBoxLayout(self)
        self._layout.setContentsMargins(0, 0, 0, 0)
        self.preview_widget = AspectRatioPreview(self)
        self._layout.addWidget(self.preview_widget)
        self.setLayout(self._layout)
        
        # Set the default position and size (bottom-right corner, 25% of screen)
        screen = QApplication.primaryScreen().size()
        width = screen.width() // 4
        height = screen.height() // 4
        self.resize(width, height)
        self.move(screen.width() - width, screen.height() - height)
        
        # We don't connect doubleClicked here since the window is click-through
        # However, we'll add a close shortcut
        self.close_shortcut = QShortcut(QKeySequence(Qt.Key_Escape), self)
        self.close_shortcut.activated.connect(self.close)

    def set_pixmap(self, pixmap: QPixmap):
        if pixmap and not pixmap.isNull():
            self.preview_widget.set_pixmap(pixmap)
        else:
            self.preview_widget.set_pixmap(QPixmap())  # Clear

    def set_overlay(self, text: str):
        self.preview_widget.set_overlay(text)

    def get_current_pixmap(self):
        return self.preview_widget._pixmap

    def get_current_overlay_text(self):
        return self.preview_widget._overlay_text

class UpscaleWorker(QObject):
    finished = Signal(bytes, int, int, float, str, float)
    error = Signal(str)

    def __init__(self, upscaler, frame, in_w, in_h, out_w, out_h, scale, interpolation_status: str, interpolation_cpu_time_ms: float):
        super().__init__()
        self.upscaler = upscaler
        self.frame = frame
        self.in_w = in_w
        self.in_h = in_h
        self.out_w = out_w
        self.out_h = out_h
        self.scale = scale
        self.interpolation_status = interpolation_status
        self.interpolation_cpu_time_ms = interpolation_cpu_time_ms
        print(f'[DEBUG] UpscaleWorker created: {id(self)}')

    @Slot()
    def run(self):
        import time
        print(f"[PYTHON DEBUG] About to call upscaler.upscale: {self.upscaler!r}")
        print(f"[PYTHON DEBUG] type(self.upscaler): {type(self.upscaler)}")
        t0 = time.perf_counter()
        try:
            print("[DEBUG] UpscaleWorker: Before upscale")
            result = self.upscaler.upscale(self.frame)
            print("[DEBUG] UpscaleWorker: After upscale")
            t1 = time.perf_counter()
            upscale_gpu_time_ms = (t1 - t0) * 1000
            self.finished.emit(result, self.out_w, self.out_h, upscale_gpu_time_ms, self.interpolation_status, self.interpolation_cpu_time_ms)
        except Exception as e:
            print(f"[DEBUG] UpscaleWorker: Exception: {e}")
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
        self.interpolator = None
        if nu_scaler_core and hasattr(nu_scaler_core, 'WgpuFrameInterpolator'):
            try:
                # Default workgroup preset is Wide32x8
                self.interpolator = nu_scaler_core.WgpuFrameInterpolator()
                print("[LiveFeedScreen] WgpuFrameInterpolator initialized successfully.")
                if hasattr(self, 'log_signal') and self.log_signal is not None: # Check if log_signal is connected
                    self.log_signal.emit("Frame Interpolator: Initialized")
            except Exception as e:
                print(f"[LiveFeedScreen] Failed to initialize WgpuFrameInterpolator: {e}")
                traceback.print_exc()
                if hasattr(self, 'log_signal') and self.log_signal is not None:
                    self.log_signal.emit(f"Frame Interpolator: Failed to init: {e}")
        else:
            print("[LiveFeedScreen] WgpuFrameInterpolator not available in nu_scaler_core.")
            if hasattr(self, 'log_signal') and self.log_signal is not None:
                 self.log_signal.emit("Frame Interpolator: Not available in core library")
        
        self.prev_frame_data = None # Stores (bytes, width, height) for interpolation
        self.interpolation_enabled = False
        self.timer = QTimer(self)
        self.timer.setInterval(100)  # Lowered to 10 FPS for diagnosis
        self.timer.timeout.connect(self.update_frame)
        
        # --- FPS Calculation Attributes ---
        self.last_frame_time = None # For scaled FPS
        self.fps = 0.0 # Scaled FPS
        
        self.base_fps = 0.0 # FPS of frames coming from capture source
        self.last_base_frame_time = None
        self.base_frame_count_for_fps = 0
        # --- END FPS Calculation Attributes ---
        
        self.last_timer_time = None # Not currently used, but was there
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
        self.fullscreen_display_window = None # For dedicated fullscreen output
        self.corner_overlay_window = None # For corner overlay output
        self.display_mode = "embedded" # "embedded", "fullscreen", or "corner"
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
        self.preview_label = AspectRatioPreview()
        left_layout.addWidget(self.input_label)
        left_layout.addWidget(self.preview_label, 1)
        # Controls
        controls = QGroupBox("Capture Controls")
        form = QFormLayout(controls)
        self.source_box = QComboBox()
        self.source_box.addItems(["Screen", "Process", "Region"])
        self.backend_box = QComboBox()
        self.backend_box.addItems(["Auto", "Win32", "X11", "Wayland"])
        self.target_box = QComboBox()
        self.target_box.setEnabled(False)
        self.source_box.currentTextChanged.connect(self.update_source_ui)
        self.refresh_targets_btn = QPushButton("Refresh Targets")
        self.refresh_targets_btn.clicked.connect(self.refresh_targets)
        self.start_btn = QPushButton("Start")
        self.start_btn.clicked.connect(self.start_capture)
        self.stop_btn = QPushButton("Stop")
        self.stop_btn.clicked.connect(self.stop_capture)
        self.stop_btn.setEnabled(False)
        form.addRow("Input Source:", self.source_box)
        form.addRow("Backend:", self.backend_box)
        form.addRow("Target:", self.target_box)
        form.addRow(self.refresh_targets_btn)
        form.addRow(self.start_btn, self.stop_btn)

        # Interpolation Checkbox - CORRECT PLACEMENT
        self.interpolation_checkbox = QCheckBox("Enable Frame Interpolation")
        self.interpolation_checkbox.setChecked(self.interpolation_enabled)
        self.interpolation_checkbox.toggled.connect(self.toggle_interpolation)
        form.addRow(self.interpolation_checkbox) # Add to the form layout

        left_layout.addWidget(controls)
        left_layout.addStretch()
        # Right: Upscaled output and upscaling controls
        right_panel = QFrame()
        right_panel.setFrameShape(QFrame.StyledPanel)
        right_layout = QVBoxLayout(right_panel)
        self.output_label = QLabel("Upscaled Output Preview")
        self.output_label.setAlignment(Qt.AlignCenter)
        self.output_label.setStyleSheet("font-size: 18px; color: #ccc;")
        # Modern maximized aspect-ratio-aware preview
        self.output_preview = AspectRatioPreview()
        self.output_preview.setMinimumSize(320, 180)
        self.output_preview.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)
        self.output_preview.doubleClicked.connect(self.handle_dedicated_fullscreen_toggle) # Connect to new handler
        right_layout.addWidget(self.output_label)
        right_layout.addWidget(self.output_preview, 1)
        # Upscaling controls
        upscale_controls = QGroupBox("Upscaling Settings")
        upscale_form = QFormLayout(upscale_controls)
        self.method_box = QComboBox()
        methods = []
        if hasattr(nu_scaler_core, 'DlssUpscaler'):
            methods.append("DLSS")
        if hasattr(nu_scaler_core, 'PyWgpuUpscaler'):
            methods.append("WGPU Nearest")
            methods.append("WGPU Bilinear")
        # Add FSR, etc. as needed
        self.method_box.addItems(methods)
        self.quality_box = QComboBox()
        self.quality_box.addItems(["ultra", "quality", "balanced", "performance"])
        self.quality_box.setToolTip("Select the upscaling quality.")
        self.scale_slider = QSlider(Qt.Horizontal)
        self.scale_slider.setRange(10, 40)
        self.scale_slider.setValue(20)
        self.scale_slider.valueChanged.connect(self.update_scale_label)
        self.scale_label = QLabel("2.0×")
        upscale_form.addRow("Method:", self.method_box)
        upscale_form.addRow("Quality:", self.quality_box)
        upscale_form.addRow("Scale Factor:", self.scale_slider)
        upscale_form.addRow("", self.scale_label)
        right_layout.addWidget(upscale_controls)
        right_layout.addStretch()
        # Status bar
        self.status_bar = QLabel("Frame Time: -- ms   FPS: --   Resolution: --")
        self.status_bar.setStyleSheet("background: #181818; color: #aaa; padding: 4px;")
        right_layout.addWidget(self.status_bar)
        layout.addWidget(left_panel)
        layout.addWidget(right_panel, 1)
        self.refresh_targets()
        self.update_scale_label()
        self.advanced_check = QCheckBox("Advanced GPU Optimization", self)
        self.advanced_check.setChecked(self.advanced_upscaling)
        self.advanced_check.stateChanged.connect(self.toggle_advanced_upscaling)
        upscale_controls.layout().addRow(self.advanced_check)
        self.memory_stats_label = QLabel("VRAM: 0.0 MB / 0.0 MB (0%)", self)
        upscale_controls.layout().addRow(self.memory_stats_label)
        memory_strategy_layout = QHBoxLayout()
        memory_strategy_layout.addWidget(QLabel("Memory Strategy:"))
        self.memory_strategy_box = QComboBox(self)
        self.memory_strategy_box.addItems(["Auto", "Aggressive", "Balanced", "Conservative", "Minimal"])
        self.memory_strategy_box.setCurrentText("Auto")
        self.memory_strategy_box.currentIndexChanged.connect(self.set_memory_strategy)
        memory_strategy_layout.addWidget(self.memory_strategy_box)
        upscale_controls.layout().addRow("", memory_strategy_layout)
        # Hotkey: Alt+S to start/stop (customizable in settings placeholder)
        self.start_stop_shortcut = QShortcut(QKeySequence("Alt+S"), self)
        self.start_stop_shortcut.activated.connect(self.toggle_start_stop)
        # Placeholder for hotkey customization in settings
        # TODO: Integrate with settings dialog/config

        # Output display mode controls
        self.display_btn = QPushButton("Fullscreen Mode")
        self.display_btn.setToolTip("Toggle between display modes (embedded, fullscreen, corner overlay)")
        self.display_btn.clicked.connect(self.toggle_display_mode)
        
        # Add to form layout
        form.addRow("Display Mode:", self.display_btn)

    def update_source_ui(self, text):
        self.target_box.clear()
        if text == "Process":
            self.target_box.setEnabled(True)
            self.refresh_targets_btn.setEnabled(True)
            self.refresh_targets()
        elif text == "Screen":
            self.target_box.setEnabled(False)
            self.refresh_targets_btn.setEnabled(False)
            self.target_box.addItem("N/A - Captures primary screen")
        elif text == "Region":
            self.target_box.setEnabled(False)
            self.refresh_targets_btn.setEnabled(False)
            self.target_box.addItem("N/A - Uses fixed region coordinates")
        else:
            self.target_box.setEnabled(False)
            self.refresh_targets_btn.setEnabled(False)
            self.target_box.addItem("N/A - Invalid Source")

    def refresh_targets(self):
        current_source_type = self.source_box.currentText()
        self.target_box.clear()
        print(f"[GUI] Refreshing targets for source type: {current_source_type}")
        
        if current_source_type == "Process":
            apps = {}
            if os.name == 'nt' and win32gui and win32process:
                print("[GUI] Using pywin32 to find 'App' processes (with visible windows).")
                try:
                    def enum_windows_callback(hwnd, lParam):
                        if win32gui.IsWindowVisible(hwnd) and win32gui.GetWindowText(hwnd):
                            _, pid = win32process.GetWindowThreadProcessId(hwnd)
                            # We store the title with the PID in case we need it for WindowByTitle fallback
                            # and to ensure we only list a PID once even if it has multiple such windows.
                            if pid not in lParam:
                                lParam[pid] = win32gui.GetWindowText(hwnd) # Store first non-empty title found for this PID
                        return True # Continue enumeration

                    window_owning_pids_with_titles = {}
                    win32gui.EnumWindows(enum_windows_callback, window_owning_pids_with_titles)

                    if not window_owning_pids_with_titles:
                        self.target_box.addItem("No processes with visible windows found (via pywin32).")
                        self.target_box.setEnabled(False)
                        return

                    # Now cross-reference with psutil to get process names
                    final_apps_list = []
                    for proc in psutil.process_iter(['pid', 'name']):
                        try:
                            pid = proc.info['pid']
                            if pid in window_owning_pids_with_titles:
                                name = proc.info['name'] or "N/A"
                                # Using the title obtained from win32gui as it might be more accurate for the main window
                                # For display, we show process name and PID.
                                final_apps_list.append(f"{name} (PID: {pid})") 
                        except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                            continue
                    
                    if final_apps_list:
                        self.target_box.addItems(sorted(list(set(final_apps_list)))) # Set for uniqueness, then sort
                    else:
                        # This case should be rare if window_owning_pids_with_titles was populated
                        self.target_box.addItem("Could not match PIDs to process names.")
                        self.target_box.setEnabled(False)

                except Exception as e_win32:
                    print(f"[GUI] Error using pywin32 for process listing: {e_win32}")
                    traceback.print_exc()
                    self.target_box.addItem("Error listing processes with pywin32.")
                    self.log_signal.emit(f"pywin32 error: {e_win32}")
                    self.target_box.setEnabled(False)
            
            else: # Not on Windows or pywin32 not available
                if os.name == 'nt': # Specifically on Windows but pywin32 failed to import
                    msg = "pywin32 missing for App list; showing basic process list."
                    self.log_signal.emit("Warning: pywin32 not found. Process list may include background tasks.")
                else: # Not on Windows
                    msg = "Process capture not optimized for non-Windows; showing basic process list."
                    self.log_signal.emit("Info: Process listing uses basic psutil iteration on non-Windows.")
                
                print(f"[GUI] {msg}")
                self.target_box.addItem(msg) # Add as first item to inform user

                # Fallback to basic psutil listing (all processes with a name and exe)
                psutil_apps = []
                for proc in psutil.process_iter(['pid', 'name', 'exe']):
                    try:
                        proc_name = proc.info['name'] or "N/A"
                        if proc_name and proc.info.get('exe'):
                           psutil_apps.append(f"{proc_name} (PID: {proc.info['pid']})")
                    except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                        continue
                if psutil_apps:
                    self.target_box.addItems(sorted(list(set(psutil_apps))))
                else: # If basic list also empty (very unlikely)
                    self.target_box.addItem("No processes found via psutil.")
                    self.target_box.setEnabled(False) # Only disable if truly nothing found
        
        else: # Screen or Region
            pass # target_box is handled by update_source_ui

    def update_scale_label(self):
        val = self.scale_slider.value() / 10.0
        self.scale_label.setText(f"{val:.1f}×")

    def start_capture(self):
        print("[GUI] Start capture requested.")
        if nu_scaler_core is None:
            self.log_signal.emit("Error: Rust core (nu_scaler_core) not available for capture.")
            self.status_bar.setText("Rust core missing")
            return
        try:
            source = self.source_box.currentText()
            target_selection = self.target_box.currentText()
            print(f"[GUI] Source: {source}, Target Selection: '{target_selection}'")

            capture_target_type = None
            capture_target_param = None

            if source == "Screen":
                # Check if core supports FullScreen capture
                # Corrected check using hasattr:
                if hasattr(nu_scaler_core, "PyCaptureTarget") and hasattr(nu_scaler_core.PyCaptureTarget, "FullScreen"):
                    capture_target_type = nu_scaler_core.PyCaptureTarget.FullScreen
                    print("[GUI] Using FullScreen target.")
                else:
                    self.log_signal.emit("Error: FullScreen capture target not available in nu_scaler_core.")
                    self.status_bar.setText("FullScreen target missing in core")
                    return

            elif source == "Process":
                if not target_selection or target_selection.startswith("No suitable") or target_selection.startswith("Error") or target_selection.startswith("pywin32 missing") or target_selection.startswith("Process capture not optimized"):
                    self.log_signal.emit("Process capture: No valid process selected from the list.")
                    self.status_bar.setText("Invalid process selection")
                    return
                try:
                    pid_str = target_selection[target_selection.rfind("(PID: ") + 6:-1]
                    pid = int(pid_str)
                    
                    # Ideal scenario: Core supports capturing by PID directly
                    # Check if PyCaptureTarget exists and has a WindowByPid attribute/member
                    # Also check if the corresponding PyWindowByPid struct/class exists.
                    if hasattr(nu_scaler_core, "PyCaptureTarget") and hasattr(nu_scaler_core.PyCaptureTarget, "WindowByPid") and hasattr(nu_scaler_core, "PyWindowByPid"):
                        capture_target_type = nu_scaler_core.PyCaptureTarget.WindowByPid
                        capture_target_param = nu_scaler_core.PyWindowByPid(pid=pid)
                        print(f"[GUI] Using WindowByPid target (from core): {pid}")
                        self.log_signal.emit(f"Attempting capture for PID {pid} using core WindowByPid.")
                    # Fallback: If core doesn't support WindowByPid, we might need to find a window title
                    # using pywin32 (if available and on Windows) and use WindowByTitle.
                    # This part makes the assumption that if WindowByPid is not in core, we still need a window title.
                    elif os.name == 'nt' and win32gui and win32process: # Check if on Windows and pywin32 is available
                        print(f"[GUI] Core WindowByPid not found. Attempting to find main window title for PID: {pid} using pywin32 for WindowByTitle fallback.")
                        found_title_for_pid = None
                        try:
                            # Callback to find a suitable window for the specific PID
                            def find_pid_window_callback(hwnd, lParam):
                                target_pid, storage = lParam # lParam is a tuple (target_pid, [list_to_store_title])
                                if win32gui.IsWindowVisible(hwnd):
                                    _, current_pid = win32process.GetWindowThreadProcessId(hwnd)
                                    if current_pid == target_pid:
                                        title = win32gui.GetWindowText(hwnd)
                                        if title: # Found a visible window with a title for our PID
                                            storage.append(title)
                                            return False # Stop enumeration, we found one
                                return True # Continue enumeration
                            
                            window_titles_for_pid = []
                            win32gui.EnumWindows(find_pid_window_callback, (pid, window_titles_for_pid))

                            if window_titles_for_pid:
                                found_title_for_pid = window_titles_for_pid[0] # Take the first one found
                                print(f"[GUI] pywin32 found window title '{found_title_for_pid}' for PID {pid}.")

                            # Check if core supports WindowByTitle capture
                            if found_title_for_pid and hasattr(nu_scaler_core, "PyCaptureTarget") and hasattr(nu_scaler_core.PyCaptureTarget, "WindowByTitle") and hasattr(nu_scaler_core, "PyWindowByTitle"):
                                capture_target_type = nu_scaler_core.PyCaptureTarget.WindowByTitle
                                capture_target_param = nu_scaler_core.PyWindowByTitle(title=found_title_for_pid)
                                print(f"[GUI] Fallback: Capturing PID {pid} via window title '{found_title_for_pid}' (found with pywin32).")
                                self.log_signal.emit(f"Fallback: Capturing PID {pid} using pywin32-found window title: '{found_title_for_pid}'.")
                            else:
                                if not found_title_for_pid:
                                    self.log_signal.emit(f"Could not find a suitable window title for PID {pid} using pywin32 for fallback.")
                                    self.status_bar.setText(f"No window title for PID {pid} (pywin32)")
                                else: # Found title but WindowByTitle target is missing in core
                                    self.log_signal.emit(f"Error: WindowByTitle capture not available in nu_scaler_core for PID {pid} pywin32 fallback.")
                                    self.status_bar.setText("WindowByTitle missing for PID fallback")
                                return
                        except Exception as e_gw_fallback:
                            self.log_signal.emit(f"Error during pywin32 fallback for PID {pid} window search: {e_gw_fallback}")
                            self.status_bar.setText(f"Error finding window for PID {pid} (pywin32)")
                            traceback.print_exc()
                            return
                    else:
                        # Core WindowByPid not available, AND (not on Windows OR pywin32 not available)
                        self.log_signal.emit(f"Error: Cannot capture PID {pid}. Core WindowByPid not supported, and no suitable Python fallback available.")
                        self.status_bar.setText("Process capture: Core/fallback missing.")
                        return
                except ValueError: # For int(pid_str)
                    self.log_signal.emit(f"Error: Could not parse PID from selection '{target_selection}'.")
                    self.status_bar.setText("Invalid process selection (PID parsing failed)")
                    return
            
            elif source == "Region":
                if hasattr(nu_scaler_core, "PyCaptureTarget") and "Region" in nu_scaler_core.PyCaptureTarget.__members__ and hasattr(nu_scaler_core, "PyRegion"):
                    capture_target_type = nu_scaler_core.PyCaptureTarget.Region
                    capture_target_param = nu_scaler_core.PyRegion(x=100, y=100, width=800, height=600)
                    print(f"[GUI] Using Region target: x={capture_target_param.x}, y={capture_target_param.y}, w={capture_target_param.width}, h={capture_target_param.height}")
                else:
                    self.log_signal.emit("Error: Region capture not available/configured in nu_scaler_core.")
                    self.status_bar.setText("Region target missing/misconfigured")
                    return
            else:
                self.log_signal.emit(f"Error: Unknown capture source '{source}'.")
                self.status_bar.setText(f"Unknown source: {source}")
                return

            if capture_target_type is None:
                self.log_signal.emit("Error: Capture target type was not set.")
                self.status_bar.setText("Capture target type not set.")
                return

            if self.capture:
                self.stop_capture(silent=True)

            print(f"[GUI] Initializing PyScreenCapture for target type: {capture_target_type}")
            self.capture = nu_scaler_core.PyScreenCapture()
            
            print(f"[GUI] Calling self.capture.start(target_type={capture_target_type}, target_param={capture_target_param})")
            self.capture.start(capture_target_type, capture_target_param)
            print("[GUI] capture.start() returned.")

            self.upscaler_initialized = False
            self.upscaler = None 
            self.timer.start() 
            self.start_btn.setEnabled(False)
            self.stop_btn.setEnabled(True)
            self.source_box.setEnabled(False)
            self.target_box.setEnabled(False)
            self.refresh_targets_btn.setEnabled(False)
            self.status_bar.setText(f"Capture started ({source})")
            self.log_signal.emit(f"Capture started. Source: {source}, Target: {target_selection if capture_target_param else 'N/A'}")
            print("[GUI] Capture timer started.")

        except ImportError:
             self.log_signal.emit("Fatal Error: nu_scaler_core module is not available. Cannot start capture.")
             self.status_bar.setText("ERROR: nu_scaler_core MISSING!")
             print("[GUI] FATAL: nu_scaler_core not imported.")
        except AttributeError as ae:
            error_message = f"Core AttributeError: {ae}. Functions in nu_scaler_core might be missing or named differently."
            print(f"[GUI] {error_message}")
            self.log_signal.emit(error_message)
            self.status_bar.setText("Core library error (Attribute)")
            traceback.print_exc()
        except Exception as e:
            error_message = f"Error starting capture: {e}"
            print(f"[GUI] {error_message}")
            self.log_signal.emit(error_message)
            self.status_bar.setText(f"Error starting capture")
            traceback.print_exc()
            self.start_btn.setEnabled(True)
            self.stop_btn.setEnabled(False)
            self.source_box.setEnabled(True)
            self.update_source_ui(self.source_box.currentText()) 

    def stop_capture(self, silent=False):
        print(f'[DEBUG] stop_capture: called (silent={silent})')
        # Stop the frame processing timer first
        self.timer.stop()
        print('[DEBUG] stop_capture: timer stopped')

        # Stop the capture object
        if self.capture:
            try:
                self.capture.stop()
                print('[DEBUG] stop_capture: capture.stop() called')
            except Exception as e:
                print(f'[DEBUG] stop_capture: error stopping capture object: {e}')
                if hasattr(self, 'log_signal') and self.log_signal:
                    self.log_signal.emit(f"Error stopping capture device: {e}")
            self.capture = None
        
        # Clean up worker and thread
        if hasattr(self, '_upscale_thread') and self._upscale_thread is not None:
            if self._upscale_thread.isRunning():
                print(f'[DEBUG] stop_capture: Quitting upscale thread (state: {self._upscale_thread.isRunning()})')
                self._upscale_thread.quit()
                if not self._upscale_thread.wait(2000): # Wait for 2 seconds
                    print('[DEBUG] stop_capture: Warning - upscale thread did not quit in time.')
                else:
                    print('[DEBUG] stop_capture: Upscale thread quit and waited')
            self._upscale_thread = None
        
        if hasattr(self, '_upscale_worker') and self._upscale_worker is not None:
            # Worker should be managed by its thread's lifecycle (e.g., deleteLater)
            # but explicitly setting to None here after thread is stopped.
            self._upscale_worker = None
            print('[DEBUG] stop_capture: Upscale worker cleared')

        # Reset upscaler related attributes
        if self.upscaler:
            print('[DEBUG] stop_capture: Clearing upscaler instance')
            # If upscaler has a specific release method, it should be called before/during del
            # e.g., if hasattr(self.upscaler, 'release'): self.upscaler.release()
            self.upscaler = None # Allow it to be garbage collected
        self.upscaler_initialized = False

        # Update UI elements
        self.start_btn.setEnabled(True)
        self.stop_btn.setEnabled(False)
        self.source_box.setEnabled(True) 
        
        # update_source_ui will correctly set enable state for target_box and refresh_targets_btn
        # and also populate target_box if needed (e.g. for Window/Process)
        self.update_source_ui(self.source_box.currentText()) 

        if not silent:
            self.status_bar.setText("Capture stopped")
            if hasattr(self, 'log_signal') and self.log_signal:
                self.log_signal.emit("Capture stopped")
        
        # Close dedicated fullscreen window if it's open and managed by LiveFeedScreen
        if hasattr(self, 'fullscreen_display_window') and self.fullscreen_display_window and self.fullscreen_display_window.isVisible():
            print('[DEBUG] stop_capture: Closing fullscreen display window.')
            self.fullscreen_display_window.close()
            # Optionally set to None if it's managed this way:
            # self.fullscreen_display_window = None 

        # Optional: Force garbage collection if memory issues were observed, though usually not necessary
        # import gc
        # gc.collect()
        # print(f'[DEBUG] stop_capture: gc collected (if uncommented)')

        print(f'[DEBUG] stop_capture: finished')

    def toggle_advanced_upscaling(self, state):
        try:
            self.advanced_upscaling = bool(state)
            self.upscaler = None
            self.upscaler_initialized = False
            self.memory_strategy_box.setEnabled(self.advanced_upscaling)
        except Exception as e:
            print(f'[DEBUG] toggle_advanced_upscaling: {e}')

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
        """Create and initialize the appropriate upscaler based on settings."""
        if not nu_scaler_core:
            self.log_signal.emit("Error: nu_scaler_core not loaded.")
            return None

        self.log_signal.emit(f"Attempting to initialize upscaler for {in_w}x{in_h} -> scale {scale:.1f}x")
        self.upscaler_initialized = False
        out_w = int(in_w * scale)
        out_h = int(in_h * scale)

        # Get selected quality and method
        quality = self.quality_box.currentText()
        method = self.method_box.currentText()

        try:
            if method == "DLSS":
                if hasattr(nu_scaler_core, 'DlssUpscaler'):
                    self.log_signal.emit(f"Creating DLSS Upscaler (Quality: {quality})")
                    self.upscaler = nu_scaler_core.DlssUpscaler(quality)
                    self.upscaler.initialize(in_w, in_h, out_w, out_h)
                    self.advanced_upscaling = False
                else:
                    self.log_signal.emit("Error: DlssUpscaler not found in nu_scaler_core.")
                    return None
            elif method == "WGPU Nearest":
                if hasattr(nu_scaler_core, 'PyWgpuUpscaler'):
                    self.log_signal.emit(f"Creating WGPU Upscaler (nearest) (Quality: {quality})")
                    self.upscaler = nu_scaler_core.PyWgpuUpscaler(quality, "nearest")
                    self.upscaler.initialize(in_w, in_h, out_w, out_h)
                else:
                    self.log_signal.emit("Error: PyWgpuUpscaler not found in nu_scaler_core.")
                    return None
            elif method == "WGPU Bilinear":
                if hasattr(nu_scaler_core, 'PyWgpuUpscaler'):
                    self.log_signal.emit(f"Creating WGPU Upscaler (bilinear) (Quality: {quality})")
                    self.upscaler = nu_scaler_core.PyWgpuUpscaler(quality, "bilinear")
                    self.upscaler.initialize(in_w, in_h, out_w, out_h)
                else:
                    self.log_signal.emit("Error: PyWgpuUpscaler not found in nu_scaler_core.")
                    return None
            else:
                self.log_signal.emit(f"Error: Unknown upscaling method selected: {method}")
                return None

            self.upscaler_initialized = True
            self.log_signal.emit(f"Upscaler '{self.upscaler.name}' initialized ({in_w}x{in_h} -> {out_w}x{out_h})")
            self._last_in_w = in_w
            self._last_in_h = in_h
            self._last_scale = scale
            self._last_quality = quality
            self._last_method = method
            return self.upscaler

        except Exception as e:
            error_msg = f"Error initializing upscaler ({method}, {quality}): {e}"
            print(error_msg)
            traceback.print_exc()
            self.log_signal.emit(error_msg)
            self.upscaler = None
            self.upscaler_initialized = False
            return None

    def update_frame(self):
        # print("[TRACE] update_frame called") # Optional: Uncomment for very verbose tracing
        try:
            print("[TRACE] update_frame ENTERED try block") # Moved inside try
            if not self.capture:
                print("[DEBUG] update_frame: No capture object, returning early.") # Added detail
                return
            
            print("[DEBUG] update_frame: Attempting self.capture.get_frame()...") # Added before call
            frame_result = self.capture.get_frame()
            print(f"[DEBUG] update_frame: self.capture.get_frame() returned: {type(frame_result)}") # Added after call

            if frame_result is None:
                # print("[TRACE] update_frame: get_frame() returned None, returning.") # Keep commented unless needed
                return # No frame yet
            
            # This print is now redundant if the one above works
            # print(f"[DEBUG] update_frame: Got frame_result.") 

            # --- Base FPS Calculation START ---
            now_for_base_fps = time.perf_counter()
            if self.last_base_frame_time is None:
                self.last_base_frame_time = now_for_base_fps
            
            self.base_frame_count_for_fps += 1
            base_fps_elapsed = now_for_base_fps - self.last_base_frame_time
            
            if base_fps_elapsed >= 1.0: # Update base_fps roughly every second
                self.base_fps = self.base_frame_count_for_fps / base_fps_elapsed
                self.base_frame_count_for_fps = 0
                self.last_base_frame_time = now_for_base_fps
            # --- Base FPS Calculation END ---

            frame_bytes_obj, in_w, in_h = frame_result
            print(f"[DEBUG] update_frame: Frame details - Size={in_w}x{in_h}, Bytes type={type(frame_bytes_obj)}") # DEBUG PRINT
            current_captured_frame_bytes = frame_bytes_obj # Keep original for prev_frame_data

            # --- Frame Interpolation Logic START ---
            frame_to_process = current_captured_frame_bytes
            interpolation_status_for_frame = "Captured (Interp Off)" # Default status
            interpolation_cpu_time_ms_for_frame = 0.0

            if self.interpolation_enabled and self.interpolator:
                if self.prev_frame_data:
                    prev_frame_bytes, prev_w, prev_h = self.prev_frame_data
                    if prev_w == in_w and prev_h == in_h:
                        try:
                            interp_start_time = time.perf_counter()
                            interpolated_frame_bytes = self.interpolator.interpolate_py(
                                prev_frame_bytes, 
                                current_captured_frame_bytes, 
                                in_w, 
                                in_h, 
                                time_t=0.5
                            )
                            interpolation_cpu_time_ms_for_frame = (time.perf_counter() - interp_start_time) * 1000
                            if interpolated_frame_bytes:
                                frame_to_process = interpolated_frame_bytes
                                interpolation_status_for_frame = "Interpolated"
                            else:
                                self.log_signal.emit("Frame Interpolation: Call returned None")
                                print("[DEBUG] Frame interpolation: interpolate_py returned None")
                                interpolation_status_for_frame = "Captured (Interp Failed)"
                        except Exception as e:
                            error_msg = f"Frame Interpolation Error: {e}"
                            print(f"[DEBUG] {error_msg}")
                            self.log_signal.emit(error_msg)
                            traceback.print_exc()
                            interpolation_status_for_frame = "Captured (Interp Error)"
                            # Fallback to current_captured_frame_bytes (already set as frame_to_process)
                    else:
                        print("[DEBUG] Frame interpolation skipped (dimension mismatch).")
                        self.log_signal.emit("Frame Interpolation: Skipped (dimension mismatch)")
                        interpolation_status_for_frame = "Captured (Interp Skipped - Dim Mismatch)"
                        self.prev_frame_data = None # Reset due to stream change
                else:
                    interpolation_status_for_frame = "Captured (Interp Skipped - No Prev Frame)"
                    # self.log_signal.emit("Frame Interpolation: Skipped (no previous frame yet)") # Can be spammy
            elif self.interpolator and not self.interpolation_enabled:
                 interpolation_status_for_frame = "Captured (Interp Off)"
            elif not self.interpolator:
                interpolation_status_for_frame = "Captured (Interpolator N/A)"
            
            self.prev_frame_data = (current_captured_frame_bytes, in_w, in_h)
            # --- Frame Interpolation Logic END ---
            print(f"[DEBUG] update_frame: Interpolation status for frame: {interpolation_status_for_frame}") # DEBUG PRINT

            # Only re-initialize upscaler if input size or scale changes
            scale = self.scale_slider.value() / 10.0
            reinit_needed = False
            print(f"[DEBUG] update_frame: Checking if upscaler re-init needed. Current state: Initialized={self.upscaler_initialized}, Upscaler Obj={self.upscaler is not None}") # DEBUG PRINT
            if not self.upscaler or not self.upscaler_initialized:
                print(f"[DEBUG] update_frame: Re-init needed - Upscaler not initialized or None.") # DEBUG PRINT
                reinit_needed = True
            elif (self._last_in_w != in_w or self._last_in_h != in_h):
                print(f"[DEBUG] update_frame: Re-init needed - Input size changed ({self._last_in_w}x{self._last_in_h} -> {in_w}x{in_h})") # DEBUG PRINT
                reinit_needed = True
            elif (self._last_scale != scale):
                print(f"[DEBUG] update_frame: Re-init needed - Scale changed ({self._last_scale} -> {scale})") # DEBUG PRINT
                reinit_needed = True
            # Add check for method/quality changes as well?
            # elif (self._last_method != self.method_box.currentText()) or (self._last_quality != self.quality_box.currentText()): 
            #     print(f"[DEBUG] update_frame: Re-init needed - Method/Quality changed")
            #     reinit_needed = True
            else:
                 print(f"[DEBUG] update_frame: Re-init NOT needed.") # DEBUG PRINT

            if reinit_needed:
                self._last_in_w = in_w
                self._last_in_h = in_h
                self._last_scale = scale
                # self._last_method = self.method_box.currentText()
                # self._last_quality = self.quality_box.currentText()
                print(f"[DEBUG] update_frame: Calling init_upscaler({in_w}, {in_h}, {scale})") # DEBUG PRINT
                upscaler_instance = self.init_upscaler(in_w, in_h, scale)
                print(f"[DEBUG] update_frame: init_upscaler returned: {type(upscaler_instance)}") # DEBUG PRINT
                if not upscaler_instance:
                    print(f"[DEBUG] update_frame: init_upscaler failed, returning.") # DEBUG PRINT
                    return # Stop if upscaler failed to init
                else:
                    print(f"[DEBUG] update_frame: init_upscaler succeeded.") # DEBUG PRINT

            # Only start a new upscale if no worker is running
            print(f"[DEBUG] update_frame: Checking existing upscale thread: {self._upscale_thread is not None}") # DEBUG PRINT
            if self._upscale_thread is not None:
                print("[DEBUG] update_frame: Skipping frame: upscale worker thread already exists and presumably running.") # DEBUG PRINT
                return

            # Calculate output dimensions (using self.upscale_scale which might differ from slider during transition)
            current_scale = self.upscale_scale 
            out_w = int(in_w * current_scale)
            out_h = int(in_h * current_scale)
            print(f"[DEBUG] update_frame: Preparing UpscaleWorker for {in_w}x{in_h} -> {out_w}x{out_h} (Scale: {current_scale})") # DEBUG PRINT

            # Start worker thread for upscaling
            self._upscale_thread = QThread()
            self._upscale_worker = UpscaleWorker(self.upscaler, frame_to_process, in_w, in_h, out_w, out_h, current_scale, interpolation_status_for_frame, interpolation_cpu_time_ms_for_frame)
            self._upscale_worker.moveToThread(self._upscale_thread)
            self._upscale_thread.started.connect(self._upscale_worker.run)
            self._upscale_worker.finished.connect(self.on_upscale_finished)
            self._upscale_worker.error.connect(self.on_upscale_error)
            self._upscale_worker.finished.connect(self._upscale_thread.quit)
            self._upscale_worker.finished.connect(self._upscale_worker.deleteLater)
            self._upscale_thread.finished.connect(self._upscale_thread.deleteLater)
            self._upscale_thread.finished.connect(self._clear_upscale_thread)
            print(f"[DEBUG] update_frame: Starting upscale thread...") # DEBUG PRINT
            self._upscale_thread.start()
            print(f"[DEBUG] update_frame: Upscale thread started.") # DEBUG PRINT
        except Exception as e:
            # Enhanced exception printing
            print(f"[EXCEPTION] An error occurred within update_frame loop:")
            print(f"[EXCEPTION] Type: {type(e).__name__}")
            print(f"[EXCEPTION] Error: {e}")
            print(f"[EXCEPTION] Traceback:")
            import traceback
            traceback.print_exc() # Print full traceback to console
            # Optionally, emit to log signal as well
            if hasattr(self, 'log_signal') and self.log_signal:
                self.log_signal.emit(f"Error in update_frame: {e}")
            # Decide if we should stop capture on error, or just log and continue?
            # self.stop_capture() # Uncomment to stop capture automatically on update_frame error

    def _clear_upscale_thread(self):
        self._upscale_thread = None
        self._upscale_worker = None

    def on_upscale_finished(self, out_bytes, out_w, out_h, upscale_gpu_time_ms, interpolation_status, interpolation_cpu_time_ms):
        # Note: `elapsed` from worker is already in ms, renamed to upscale_gpu_time_ms
        # print(f'[DEBUG] on_upscale_finished: {id(self)}')
        # print(f"[DEBUG] Upscale finished in {upscale_gpu_time_ms:.2f} ms at {time.strftime('%H:%M:%S')}")
        # print(f"[DEBUG] Interpolation status: {interpolation_status}, CPU time: {interpolation_cpu_time_ms:.2f} ms")
        if out_bytes:
            try:
                qimg = QImage(out_bytes, out_w, out_h, QImage.Format_RGBA8888)
                pixmap = QPixmap.fromImage(qimg)
                self.output_preview.set_pixmap(pixmap)
                
                # Scaled FPS calculation (based on upscaler output rate)
                inst_scaled_fps = 1000.0 / upscale_gpu_time_ms if upscale_gpu_time_ms > 0 else 0.0
                self.fps = 0.95 * self.fps + 0.05 * inst_scaled_fps if self.fps > 0 else inst_scaled_fps
                
                vram_str = self.memory_stats_label.text()
                
                overlay_lines = [
                    f"Base Frame: {out_w//self.upscale_scale:.0f}×{out_h//self.upscale_scale:.0f}",
                    f"Scaled Frame: {out_w}×{out_h}",
                    f"Base FPS: {self.base_fps:.1f}",       # Display calculated base FPS
                    f"Scaled FPS: {self.fps:.1f}",     # This is the existing self.fps
                    f"{vram_str}",
                    f"Upscale GPU Time: {upscale_gpu_time_ms:.1f} ms"
                ]

                if self.interpolation_enabled and self.interpolator:
                    overlay_lines.append(f"Frame Source: {interpolation_status}")
                    if interpolation_status == "Interpolated" and interpolation_cpu_time_ms > 0:
                        overlay_lines.append(f"Interp CPU Time: {interpolation_cpu_time_ms:.1f} ms")
                else:
                    overlay_lines.append("Frame Source: Captured (Interp Off)") # Or use interpolation_status if more detailed

                overlay = "\n".join(overlay_lines)
                self.output_preview.set_overlay(overlay)
                
                status_bar_text = (
                    f"Base: {out_w//self.upscale_scale:.0f}×{out_h//self.upscale_scale:.0f} @ {self.base_fps:.1f}FPS | "
                    f"Scaled: {out_w}×{out_h} @ {self.fps:.1f}FPS ({upscale_gpu_time_ms:.1f}ms GPU)"
                )
                if self.interpolation_enabled and self.interpolator and interpolation_status == "Interpolated" and interpolation_cpu_time_ms > 0:
                    status_bar_text += f" | Interp CPU: {interpolation_cpu_time_ms:.1f}ms ({interpolation_status})"
                elif self.interpolation_enabled and self.interpolator:
                    status_bar_text += f" | Interp: {interpolation_status}" 

                self.status_bar.setText(status_bar_text)
                self.profiler_signal.emit(upscale_gpu_time_ms, self.fps, out_w//self.upscale_scale, out_h//self.upscale_scale)
                
                self.last_frame_time = time.perf_counter()

                # Update display windows based on current mode
                if self.display_mode == "fullscreen" and self.fullscreen_display_window and self.fullscreen_display_window.isVisible():
                    self.fullscreen_display_window.set_pixmap(pixmap)
                    self.fullscreen_display_window.set_overlay(overlay)
                elif self.display_mode == "corner" and self.corner_overlay_window and self.corner_overlay_window.isVisible():
                    self.corner_overlay_window.set_pixmap(pixmap)
                    self.corner_overlay_window.set_overlay(overlay)

            except Exception as e:
                print(f"[ERROR] Failed to update output preview: {e}")
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

    def toggle_start_stop(self):
        """Toggle start/stop capture via hotkey."""
        if self.start_btn.isEnabled():
            self.start_capture()
        elif self.stop_btn.isEnabled():
            self.stop_capture()

    def handle_dedicated_fullscreen_toggle(self):
        # This is the original fullscreen toggle - kept for backward compatibility
        # Instead we'll delegate to the more flexible toggle_display_mode
        self.toggle_display_mode("fullscreen")

    def toggle_display_mode(self, mode=None):
        """Toggle between embedded, fullscreen, and corner overlay display modes
        
        Args:
            mode (str, optional): Force a specific mode ("embedded", "fullscreen", "corner")
                                 If None, cycles through modes
        """
        current_mode = self.display_mode
        
        # If no mode specified, cycle to the next one
        if mode is None:
            if current_mode == "embedded":
                mode = "fullscreen"
            elif current_mode == "fullscreen":
                mode = "corner"
            else:  # "corner"
                mode = "embedded"
        
        # Exit current mode
        if current_mode == "fullscreen" and self.fullscreen_display_window and self.fullscreen_display_window.isVisible():
            self.fullscreen_display_window.hide()
            # Restore embedded preview 
            if self.fullscreen_display_window.get_current_overlay_text():
                self.output_preview.set_overlay(self.fullscreen_display_window.get_current_overlay_text())
            if self.fullscreen_display_window.get_current_pixmap():
                self.output_preview.set_pixmap(self.fullscreen_display_window.get_current_pixmap())
            if hasattr(self.output_preview, '_original_text_when_fullscreen'):
                del self.output_preview._original_text_when_fullscreen
                
        elif current_mode == "corner" and self.corner_overlay_window and self.corner_overlay_window.isVisible():
            self.corner_overlay_window.hide()
            # Restore embedded preview
            if self.corner_overlay_window.get_current_overlay_text():
                self.output_preview.set_overlay(self.corner_overlay_window.get_current_overlay_text())
            if self.corner_overlay_window.get_current_pixmap():
                self.output_preview.set_pixmap(self.corner_overlay_window.get_current_pixmap())
            if hasattr(self.output_preview, '_original_text_when_corner'):
                del self.output_preview._original_text_when_corner
        
        # Enter new mode
        self.display_mode = mode
        
        if mode == "embedded":
            self.display_btn.setText("Fullscreen Mode")
            # Ensure overlay window is hidden and main preview is visible
            if self.fullscreen_display_window:
                self.fullscreen_display_window.hide()
            if self.corner_overlay_window:
                self.corner_overlay_window.hide()
            
        elif mode == "fullscreen":
            self.display_btn.setText("Corner Mode")
            
            if not self.fullscreen_display_window:
                self.fullscreen_display_window = FullScreenDisplayWindow()
                
            # Get current content from embedded preview
            current_pixmap = self.output_preview._pixmap
            current_overlay_text = self.output_preview._overlay_text
            
            if current_pixmap and not current_pixmap.isNull():
                self.fullscreen_display_window.set_pixmap(current_pixmap)
            else:
                self.fullscreen_display_window.set_pixmap(QPixmap())
                
            self.fullscreen_display_window.set_overlay(current_overlay_text)
            self.fullscreen_display_window.showFullScreen()
            
            # Change embedded preview appearance
            self.output_preview._original_text_when_fullscreen = current_overlay_text
            self.output_preview.set_pixmap(QPixmap())
            self.output_preview.set_overlay("Output in fullscreen mode\n(Press Esc to exit)")
            
        elif mode == "corner":
            self.display_btn.setText("Embedded Mode")
            
            if not self.corner_overlay_window:
                self.corner_overlay_window = CornerOverlayWindow()
                
            # Get current content from embedded preview
            current_pixmap = self.output_preview._pixmap
            current_overlay_text = self.output_preview._overlay_text
            
            if current_pixmap and not current_pixmap.isNull():
                self.corner_overlay_window.set_pixmap(current_pixmap)
            else:
                self.corner_overlay_window.set_pixmap(QPixmap())
                
            self.corner_overlay_window.set_overlay(current_overlay_text)
            self.corner_overlay_window.show()
            
            # Change embedded preview appearance
            self.output_preview._original_text_when_corner = current_overlay_text
            self.output_preview.set_pixmap(QPixmap())
            self.output_preview.set_overlay("Output in corner overlay mode\n(Press Esc to exit)")
            
        print(f"[LiveFeedScreen] Display mode changed to: {mode}")
        if hasattr(self, 'log_signal') and self.log_signal is not None:
            self.log_signal.emit(f"Display mode: {mode}")
            
    def toggle_interpolation(self, checked):
        self.interpolation_enabled = checked
        self.prev_frame_data = None # Reset on toggle to ensure clean start
        if hasattr(self, 'log_signal') and self.log_signal is not None:
            status = "Enabled" if checked else "Disabled"
            self.log_signal.emit(f"Frame Interpolation: {status}")
        print(f"[LiveFeedScreen] Frame interpolation {status}")

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
        self.scale_slider.valueChanged.connect(self.update_scale_label)
        self.scale_slider.setToolTip("Set the upscaling factor (1.0× to 4.0×).")
        self.method_box = QComboBox()
        self.method_box.addItems(["DLSS", "WGPU Nearest", "WGPU Bilinear"])
        self.method_box.setToolTip("Select the upscaling method.")
        self.quality_box = QComboBox()
        self.quality_box.addItems(["ultra", "quality", "balanced", "performance"])
        self.quality_box.setToolTip("Select the upscaling quality.")
        self.scale_slider.valueChanged.connect(self.update_scale_label)
        upscale_form.addRow("Method:", self.method_box)
        upscale_form.addRow("Quality:", self.quality_box)
        upscale_form.addRow("Scale Factor:", self.scale_slider)
        upscale_form.addRow("", self.scale_label)
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
            self.live_feed_screen.refresh_targets()

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

    def update_scale_label(self):
        self.scale_label.setText(f"{self.scale_slider.value()/10.0:.1f}×")

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
            QMainWindow {
                background-color: #0D1B2A;
            }
            QWidget {
                color: #E0E1DD;
                font-family: 'Segoe UI', Arial, sans-serif;
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
            QListWidget {
                background: #232323;
                color: #bbb;
                font-size: 16px;
            }
            QListWidget::item:selected {
                background: #2E8BC0;
                color: #fff;
            }
            QFrame[frameShape='4'] {
                border: 1px solid #2E8BC0;
                border-radius: 8px;
            }
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