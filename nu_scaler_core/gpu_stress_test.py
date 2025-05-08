#!/usr/bin/env python
"""
GPU Stress Test for Nu_Scaler Core
This script will push the GPU to its limits by processing multiple large 8K images simultaneously
"""
import sys
import time
import traceback
import numpy as np
from threading import Thread
import concurrent.futures
import psutil
import gc

# GPU monitor class to track usage during the test
class GpuMonitor:
    def __init__(self, upscaler):
        self.upscaler = upscaler
        self.stop_flag = False
        self.max_vram_usage = 0
        self.thread = Thread(target=self._monitor_loop)
        self.thread.daemon = True
        
    def start(self):
        self.thread.start()
        
    def stop(self):
        self.stop_flag = True
        self.thread.join(timeout=1.0)
        
    def _monitor_loop(self):
        while not self.stop_flag:
            try:
                if hasattr(self.upscaler, 'get_vram_stats'):
                    stats = self.upscaler.get_vram_stats()
                    if stats.used_mb > self.max_vram_usage:
                        self.max_vram_usage = stats.used_mb
                    
                    print(f"\r[GPU] VRAM: {stats.used_mb:.1f}MB / {stats.total_mb:.1f}MB ({stats.usage_percent:.1f}%) | Peak: {self.max_vram_usage:.1f}MB | CPU: {psutil.cpu_percent(interval=None):.1f}%", end="")
                    sys.stdout.flush()
                    
                    # Force GPU stats update
                    if hasattr(self.upscaler, 'update_gpu_stats'):
                        self.upscaler.update_gpu_stats()
            except Exception as e:
                print(f"\nMonitor error: {e}")
                
            time.sleep(0.5)

# Create extremely large test images at different resolutions
def create_test_images():
    print("Creating test images at various resolutions...")
    images = []
    
    # 4K, 5K, 6K, 8K test images
    resolutions = [
        (3840, 2160),  # 4K
        (5120, 2880),  # 5K
        (6016, 3384),  # 6K
        (7680, 4320),  # 8K
    ]
    
    for width, height in resolutions:
        print(f"Creating {width}x{height} test image...")
        img = np.zeros((height, width, 4), dtype=np.uint8)
        
        # Create a complex pattern (gradient with some high-frequency detail)
        for y in range(height):
            for x in range(width):
                # Base gradient
                r = int((x / width) * 255)
                g = int((y / height) * 255)
                b = int(((x + y) / (width + height)) * 255)
                
                # Add some high-frequency detail (checkerboard pattern)
                if ((x // 16) % 2) == ((y // 16) % 2):
                    r = (r + 128) % 255
                    g = (g + 128) % 255
                    b = (b + 128) % 255
                
                img[y, x, 0] = r  # R
                img[y, x, 1] = g  # G
                img[y, x, 2] = b  # B
                img[y, x, 3] = 255  # Alpha
        
        # Convert to byte array
        img_bytes = img.tobytes()
        images.append((width, height, img_bytes))
        
        # Force garbage collection to free up RAM
        img = None
        gc.collect()
    
    return images

def process_image(args):
    upscaler, width, height, img_bytes, scale_factor = args
    
    try:
        # Calculate output dimensions
        output_width = int(width * scale_factor)
        output_height = int(height * scale_factor)
        
        print(f"\nProcessing {width}x{height} -> {output_width}x{output_height}")
        
        # Initialize upscaler for this image
        upscaler.initialize(width, height, output_width, output_height)
        
        # Process the image
        start_time = time.time()
        result = upscaler.upscale(img_bytes)
        elapsed = time.time() - start_time
        
        print(f"Processed {width}x{height} image in {elapsed:.2f}s ({len(result)} bytes)")
        return True
    except Exception as e:
        print(f"Error processing image {width}x{height}: {e}")
        traceback.print_exc()
        return False

def run_gpu_stress_test():
    try:
        import nu_scaler_core
        print(f"Loaded nu_scaler_core module from: {nu_scaler_core.__file__}")
        
        # Create an advanced upscaler with maximum memory usage
        print("\nCreating advanced upscaler...")
        upscaler = nu_scaler_core.create_advanced_upscaler("ultra")  # Use ultra quality for maximum stress
        
        # Print GPU info
        if hasattr(upscaler, 'get_gpu_info'):
            gpu_info = upscaler.get_gpu_info()
            print("\nGPU Information:")
            for key, value in gpu_info.items():
                print(f"  {key}: {value}")
        
        # Start GPU monitor
        monitor = GpuMonitor(upscaler)
        monitor.start()
        
        # Create test images (this will take a moment)
        test_images = create_test_images()
        
        # Process each image multiple times with different scale factors
        print("\nStarting GPU stress test - processing multiple large images...")
        
        # Use different scale factors to stress different aspects of the GPU
        scale_factors = [1.25, 1.5, 2.0, 3.0]
        
        try:
            # Process images multiple times to stress the GPU
            for iteration in range(5):
                print(f"\nIteration {iteration+1}/5:")
                
                # Process each image with each scale factor
                tasks = []
                for width, height, img_bytes in test_images:
                    for scale in scale_factors:
                        tasks.append((upscaler, width, height, img_bytes, scale))
                
                # Process tasks in parallel
                with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:
                    results = list(executor.map(process_image, tasks))
                
                # Force cleanup after each iteration
                if hasattr(upscaler, 'force_cleanup'):
                    print("\nForcing memory cleanup...")
                    upscaler.force_cleanup()
                
                # Allow GPU to cool down between iterations
                time.sleep(2.0)
            
        finally:
            monitor.stop()
            
        # Print final statistics
        print(f"\n\nGPU Stress Test Complete")
        print(f"Peak VRAM usage: {monitor.max_vram_usage:.2f}MB")
        
        # Get current VRAM stats
        if hasattr(upscaler, 'get_vram_stats'):
            stats = upscaler.get_vram_stats()
            print(f"Final VRAM usage: {stats.used_mb:.2f}MB / {stats.total_mb:.2f}MB ({stats.usage_percent:.2f}%)")
        
        return True
        
    except Exception as e:
        print(f"Error: {e}")
        traceback.print_exc()
        return False

if __name__ == "__main__":
    try:
        import psutil
    except ImportError:
        print("Installing required package: psutil")
        import subprocess
        subprocess.check_call([sys.executable, "-m", "pip", "install", "psutil"])
        import psutil
        
    success = run_gpu_stress_test()
    sys.exit(0 if success else 1) 