import sys
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QLabel, QListWidget, QStackedWidget, QFrame,
    QPushButton, QComboBox, QSpinBox, QCheckBox, QSlider, QGroupBox, QFormLayout
)
from PySide6.QtCore import Qt, QTimer, Signal
from PySide6.QtGui import QPixmap, QImage
import time
import random
import traceback

try:
    import nu_scaler_core
except ImportError:
    nu_scaler_core = None

class LiveFeedScreen(QWidget):
    log_signal = Signal(str)
    profiler_signal = Signal(float, float, int, int)
    warning_signal = Signal(str, bool)
    def __init__(self):
        super().__init__()
        self.capture = None
        self.upscaler = None
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.update_frame)
        self.last_frame_time = None
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
                windows = nu_scaler_core.PyScreenCapture.list_windows()
                print(f"[GUI] Received windows: {windows}")
                if windows:
                    self.window_box.addItems(windows)
                else:
                    self.window_box.addItem("No windows found")
            except Exception as e:
                print(f"[GUI] Error listing windows: {e}")
                self.window_box.addItem("Error listing windows")
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
            self.timer.start(16) # Aim for ~60 FPS
            self.start_btn.setEnabled(False)
            self.stop_btn.setEnabled(True)
            self.status_bar.setText("Capture started")
            print("[GUI] Capture timer started.")
        except Exception as e:
            print(f"[GUI] Error starting capture: {e}")
            self.status_bar.setText(f"Error starting capture: {e}")
            self.log_signal.emit(f"Error starting capture: {e}")

    def stop_capture(self):
        if self.capture:
            self.capture.stop()
        self.timer.stop()
        self.start_btn.setEnabled(True)
        self.stop_btn.setEnabled(False)
        self.status_bar.setText("Capture stopped")

    def update_technology_ui(self, technology):
        """Update UI based on selected upscaling technology"""
        if technology == "FSR 3.0":
            # FSR works with all quality settings, but algorithm is fixed
            self.quality_box.setEnabled(True)
            self.algorithm_box.setEnabled(False)
            self.algorithm_box.setCurrentText("bilinear")  # FSR uses its own internal algorithm
        elif technology == "DLSS":
            # DLSS uses quality settings but algorithm is fixed
            self.quality_box.setEnabled(True)
            self.algorithm_box.setEnabled(False)
            self.algorithm_box.setCurrentText("bilinear")  # DLSS uses its own internal algorithm
        elif technology == "Basic":
            # Basic allows choosing the algorithm
            self.quality_box.setEnabled(True)
            self.algorithm_box.setEnabled(True)
        else:  # Auto (Best for GPU)
            # Auto mode - quality enabled, algorithm disabled
            self.quality_box.setEnabled(True)
            self.algorithm_box.setEnabled(False)
            
    def toggle_advanced_upscaling(self, state):
        """Toggle between standard and advanced upscaling"""
        self.advanced_upscaling = bool(state)
        self.upscaler = None  # Force re-initialization
        self.upscaler_initialized = False
        self.memory_strategy_box.setEnabled(self.advanced_upscaling)
        self.adaptive_quality_check.setEnabled(self.advanced_upscaling)
    
    def toggle_adaptive_quality(self, state):
        """Toggle adaptive quality mode"""
        if self.upscaler and hasattr(self.upscaler, 'set_adaptive_quality'):
            self.upscaler.set_adaptive_quality(bool(state))
    
    def set_memory_strategy(self, index):
        """Set the memory allocation strategy"""
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
        if nu_scaler_core is None:
            print("[GUI] Rust core not available for upscaler.")
            self.status_bar.setText("Rust core missing")
            return False
            
        try:
            # Get scale settings from UI
            out_w = int(in_w * scale)
            out_h = int(in_h * scale)
            quality = self.quality_box.currentText()
            algorithm = self.algorithm_box.currentText()
            technology = self.technology_box.currentText()
            
            # Use the appropriate upscaler based on settings
            if self.advanced_upscaling:
                # Use the advanced GPU-optimized upscaler with memory management
                print(f"Creating advanced upscaler with quality: {quality}")
                try:
                    self.upscaler = nu_scaler_core.create_advanced_upscaler(quality.lower())
                except (AttributeError, Exception) as e:
                    print(f"[GUI] Error initializing advanced upscaler: {e}")
                    print("[GUI] Falling back to best available upscaler")
                    self.upscaler = nu_scaler_core.create_best_upscaler(quality.lower())
                    # Disable advanced-only options when falling back
                    self.memory_strategy_box.setEnabled(False)
                    self.adaptive_quality_check.setEnabled(False)
                
                # Set adaptive quality based on checkbox
                if hasattr(self.upscaler, 'set_adaptive_quality'):
                    self.upscaler.set_adaptive_quality(self.adaptive_quality_check.isChecked())
                
                # Set memory strategy if not auto
                strategy_text = self.memory_strategy_box.currentText()
                if strategy_text != "Auto" and hasattr(self.upscaler, 'set_memory_strategy'):
                    self.upscaler.set_memory_strategy(strategy_text.lower())
            else:
                # Use appropriate upscaler based on technology selection
                if technology == "Auto (Best for GPU)":
                    # Use core's automatic detection for best technology
                    self.upscaler = nu_scaler_core.create_best_upscaler(quality.lower())
                elif technology == "FSR 3.0":
                    # Use FSR upscaler
                    self.upscaler = nu_scaler_core.create_fsr_upscaler(quality.lower())
                elif technology == "DLSS":
                    # Use DLSS upscaler
                    try:
                        self.upscaler = nu_scaler_core.create_dlss_upscaler(quality.lower())
                    except (AttributeError, Exception) as e:
                        print(f"[GUI] Error initializing DLSS upscaler: {e}")
                        print("[GUI] Falling back to best available upscaler")
                        self.upscaler = nu_scaler_core.create_best_upscaler(quality.lower())
                else:
                    # Default to basic upscaler
                    self.upscaler = nu_scaler_core.PyWgpuUpscaler(quality.lower(), algorithm.lower())
            
            # Initialize the upscaler
            self.upscaler.initialize(in_w, in_h, out_w, out_h)
            self.upscaler.set_upscale_scale(scale)
            
            print(f"Upscaler initialized: {in_w}x{in_h} -> {out_w}x{out_h}")
            self.upscaler_initialized = True
            
            # Update memory stats right away
            self.update_memory_stats()
            
            return True
        except Exception as e:
            traceback.print_exc()
            print(f"[GUI] Error initializing upscaler: {e}")
            self.status_bar.setText(f"Error: {str(e)}")
            return False

    def update_frame(self):
        if not self.capture:
            return
        t0 = time.perf_counter()
        frame_result = self.capture.get_frame()
        if frame_result is None:
            return # No frame yet

        frame_bytes_obj, in_w, in_h = frame_result
        frame = frame_bytes_obj

        # Initialize upscaler on first frame or if dimensions change
        if not self.upscaler or not self.upscaler_initialized:
            scale = self.scale_slider.value() / 10.0
            if not self.init_upscaler(in_w, in_h, scale):
                return # Stop if upscaler failed to init
        
        # Re-initialize if scale factor changes
        current_scale = self.scale_slider.value() / 10.0
        if abs(current_scale - self.upscale_scale) > 0.01:
            self.upscale_scale = current_scale
            self.upscaler.set_upscale_scale(current_scale)
            # Re-initialize with new output size
            out_w = int(in_w * current_scale)
            out_h = int(in_h * current_scale)
            try:
                self.upscaler.initialize(in_w, in_h, out_w, out_h)
            except Exception as e:
                print(f"Error re-initializing upscaler: {e}")
                return

        try:
            out_bytes = self.upscaler.upscale(frame) # Frame is now RGBA
            
            # Get scale factor from upscaler directly if possible
            if hasattr(self.upscaler, 'get_upscale_scale'):
                scale = self.upscaler.get_upscale_scale
            else:
                scale = self.upscale_scale
                
            # Calculate output dimensions
            out_w = int(in_w * scale)
            out_h = int(in_h * scale)
            
            # Clean up memory if using advanced upscaler
            if self.advanced_upscaling and hasattr(self.upscaler, 'cleanup_memory'):
                # Only call cleanup_memory occasionally to avoid constant cleanup
                if random.random() < 0.05:  # ~5% chance each frame
                    self.upscaler.cleanup_memory()
            
            # Convert result to QImage/QPixmap and display
            if out_bytes:
                qimg = QImage(out_bytes, out_w, out_h, QImage.Format_RGBA8888)
                pixmap = QPixmap.fromImage(qimg)
                self.output_preview.setPixmap(pixmap)
                
                # Update FPS
                t1 = time.perf_counter()
                dt = t1 - t0
                if dt > 0:
                    inst_fps = 1.0 / dt
                    # Smooth FPS calculation
                    self.fps = 0.95 * self.fps + 0.05 * inst_fps if self.fps > 0 else inst_fps
                self.overlay.setText(f"Input: {in_w}×{in_h}\nUpscaled: {out_w}×{out_h}\nFPS: {self.fps:.1f}")
                self.status_bar.setText(f"Frame Time: {(t1 - t0) * 1000:.1f} ms   FPS: {self.fps:.1f}   Resolution: {in_w}×{in_h} → {out_w}×{out_h}")
                self.profiler_signal.emit(dt * 1000, self.fps, in_w, in_h)
                if self.fps < 30:
                    self.warning_signal.emit(f"Warning: Low FPS ({self.fps:.1f})", True)
                else:
                    self.warning_signal.emit("", False)
                
                # Update frame time
                self.last_frame_time = t0
        except Exception as e:
            traceback.print_exc()
            print(f"[GUI] Error in upscaling: {e}")
            self.status_bar.setText(f"Error: {str(e)}")
            self.stop_capture() # Stop on error

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
    def update_profiler(self, frame_time, fps, in_w, in_h):
        self.profiler_label.setText(f"Frame: {frame_time:.1f} ms | FPS: {fps:.1f} | Input: {in_w}×{in_h}")
    def show_warning(self, msg, show):
        self.warning_label.setText(msg)
        self.warning_label.setVisible(show)

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

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Nu Scaler")
        # self.setMinimumSize(1100, 650) # <-- Commented out for testing maximization
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
        self.stack = QStackedWidget()
        self.live_feed_screen = LiveFeedScreen()
        self.debug_screen = DebugScreen()
        self.advanced_screen = AdvancedScreen(live_feed_screen=self.live_feed_screen)
        self.ui_screen = UIAccessibilityScreen()
        self.screens = {
            0: self.live_feed_screen,
            1: SettingsScreen(live_feed_screen=self.live_feed_screen),
            2: self.debug_screen,
            3: self.advanced_screen,
            4: self.ui_screen,
        }
        for i in range(5):
            self.stack.addWidget(self.screens[i])
        self.sidebar.currentRowChanged.connect(self.stack.setCurrentIndex)
        main_widget = QWidget()
        main_layout = QHBoxLayout(main_widget)
        main_layout.addWidget(self.sidebar)
        main_layout.addWidget(self.stack)
        self.setCentralWidget(main_widget)
        self.apply_theme()
        # Connect LiveFeedScreen signals to DebugScreen
        self.live_feed_screen.log_signal.connect(self.debug_screen.append_log)
        self.live_feed_screen.profiler_signal.connect(self.debug_screen.update_profiler)
        self.live_feed_screen.warning_signal.connect(self.debug_screen.show_warning)
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
    win.show() # Show first
    win.showMaximized() # Then maximize
    sys.exit(app.exec())

if __name__ == "__main__":
    run_gui()