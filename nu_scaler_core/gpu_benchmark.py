#!/usr/bin/env python
"""
GPU Benchmark for Nu_Scaler Core
Tests GPU utilization and performance with our optimized implementation
"""
import sys
import time
import traceback
import numpy as np
import psutil
from threading import Thread
import gc

class GpuStatsMonitor:
    def __init__(self, upscaler):
        self.upscaler = upscaler
        self.stop_flag = False
        self.thread = Thread(target=self._monitor_loop)
        self.thread.daemon = True
        self.peak_vram = 0
        self.peak_cpu = 0
        
    def start(self):
        print("Starting GPU monitoring...")
        self.thread.start()
        
    def stop(self):
        self.stop_flag = True
        self.thread.join(timeout=1.0)
        
    def _monitor_loop(self):
        while not self.stop_flag:
            try:
                # Get CPU usage
                cpu_percent = psutil.cpu_percent(interval=None)
                if cpu_percent > self.peak_cpu:
                    self.peak_cpu = cpu_percent
                
                # Get GPU usage
                if hasattr(self.upscaler, 'get_vram_stats'):
                    stats = self.upscaler.get_vram_stats()
                    if stats.used_mb > self.peak_vram:
                        self.peak_vram = stats.used_mb
                    
                    print(f"\r[Monitor] CPU: {cpu_percent:5.1f}% (Peak: {self.peak_cpu:5.1f}%) | VRAM: {stats.used_mb:8.1f}MB / {stats.total_mb:8.1f}MB ({stats.usage_percent:5.1f}%) | Peak VRAM: {self.peak_vram:8.1f}MB", end="")
                    sys.stdout.flush()
                    
                    # Force GPU stats update
                    if hasattr(self.upscaler, 'update_gpu_stats'):
                        self.upscaler.update_gpu_stats()
            except Exception as e:
                print(f"\nMonitor error: {e}")
                
            time.sleep(0.5)

def create_test_images():
    """Create test images at different resolutions for benchmarking"""
    print("\nCreating test images...")
    images = []
    
    # Different test resolutions
    resolutions = [
        (1280, 720),    # 720p
        (1920, 1080),   # 1080p
        (2560, 1440),   # 1440p
        (3840, 2160),   # 4K
    ]
    
    for width, height in resolutions:
        print(f"Creating {width}x{height} test pattern...")
        img = np.zeros((height, width, 4), dtype=np.uint8)
        
        # Create a more complex pattern that will stress the GPU more
        for y in range(height):
            for x in range(width):
                r = int((x / width) * 255)
                g = int((y / height) * 255)
                b = int(((x + y) / (width + height)) * 255)
                
                # Add some high-frequency components (checkerboard pattern)
                if ((x // 16) % 2) == ((y // 16) % 2):
                    r = (r + 128) % 255
                    g = (g + 128) % 255
                    b = (b + 128) % 255
                    
                img[y, x, 0] = r
                img[y, x, 1] = g
                img[y, x, 2] = b
                img[y, x, 3] = 255
        
        # Convert to bytes
        img_bytes = img.tobytes()
        images.append((width, height, img_bytes))
        
        # Free memory
        img = None
        gc.collect()
    
    return images

def run_benchmark():
    try:
        import nu_scaler_core
        print(f"Loaded nu_scaler_core from: {nu_scaler_core.__file__}")
        
        # Create the advanced upscaler with our optimizations
        print("\nCreating advanced upscaler...")
        upscaler = nu_scaler_core.create_advanced_upscaler("ultra")  # Use ultra quality to maximize GPU usage
        
        # Print GPU info
        if hasattr(upscaler, 'get_gpu_info'):
            gpu_info = upscaler.get_gpu_info()
            print("\nGPU Information:")
            for key, value in gpu_info.items():
                print(f"  {key}: {value}")
        
        # Force GPU activation to maximize performance
        print("\nForcing GPU activation...")
        if hasattr(upscaler, 'force_gpu_activation'):
            upscaler.force_gpu_activation()
        
        # Start GPU monitor
        monitor = GpuStatsMonitor(upscaler)
        monitor.start()
        
        # Create test images
        test_images = create_test_images()
        
        # Run benchmark for each resolution with multiple scale factors
        print("\nRunning benchmark...")
        scale_factors = [1.5, 2.0, 3.0]
        results = []
        
        try:
            for width, height, img_bytes in test_images:
                for scale in scale_factors:
                    output_width = int(width * scale)
                    output_height = int(height * scale)
                    
                    print(f"\n\nBenchmarking {width}x{height} -> {output_width}x{output_height} (scale: {scale:.1f}x)")
                    
                    # Initialize upscaler for this resolution
                    upscaler.initialize(width, height, output_width, output_height)
                    
                    # Warm-up run to prime the GPU
                    print("Warm-up run...")
                    upscaler.upscale(img_bytes)
                    
                    # Force cleanup to ensure consistent results
                    if hasattr(upscaler, 'force_cleanup'):
                        upscaler.force_cleanup()
                    
                    # Benchmark runs
                    num_runs = 10
                    frame_times = []
                    
                    print(f"Running {num_runs} benchmark passes...")
                    for i in range(num_runs):
                        start_time = time.time()
                        result = upscaler.upscale(img_bytes)
                        elapsed = time.time() - start_time
                        frame_times.append(elapsed * 1000)  # Convert to ms
                        
                        print(f"  Pass {i+1}/{num_runs}: {elapsed*1000:.2f}ms ({len(result)} bytes)")
                    
                    # Calculate stats
                    avg_time = np.mean(frame_times)
                    min_time = np.min(frame_times)
                    max_time = np.max(frame_times)
                    fps = 1000 / avg_time
                    
                    print(f"Results for {width}x{height} -> {output_width}x{output_height}:")
                    print(f"  Average: {avg_time:.2f}ms")
                    print(f"  Min: {min_time:.2f}ms")
                    print(f"  Max: {max_time:.2f}ms")
                    print(f"  FPS: {fps:.2f}")
                    
                    # Store results
                    results.append({
                        'input_res': f"{width}x{height}",
                        'output_res': f"{output_width}x{output_height}",
                        'scale': scale,
                        'avg_ms': avg_time,
                        'min_ms': min_time,
                        'max_ms': max_time,
                        'fps': fps,
                    })
                    
                    # Force cleanup between runs
                    if hasattr(upscaler, 'force_cleanup'):
                        upscaler.force_cleanup()
        
        finally:
            # Stop the monitor
            monitor.stop()
            
        # Print summary
        print("\n\nBenchmark Summary:")
        print(f"Peak VRAM usage: {monitor.peak_vram:.2f}MB")
        print(f"Peak CPU usage: {monitor.peak_cpu:.2f}%")
        print("\nResults by resolution:")
        
        for result in results:
            print(f"{result['input_res']} -> {result['output_res']} ({result['scale']}x): " +
                  f"{result['avg_ms']:.2f}ms, {result['fps']:.2f}FPS")
        
        return True
    except Exception as e:
        print(f"Error: {e}")
        traceback.print_exc()
        return False

if __name__ == "__main__":
    run_benchmark() 