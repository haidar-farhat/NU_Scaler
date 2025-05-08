#!/usr/bin/env python
"""
GPU Monitoring for Nu_Scaler Core
This script will create an upscaler and monitor its VRAM usage in real-time
"""
import sys
import time
import traceback
from threading import Thread

def run_gpu_monitor():
    try:
        import nu_scaler_core
        print(f"Loaded nu_scaler_core module from: {nu_scaler_core.__file__}")
        
        # Create an advanced upscaler
        print("\nCreating advanced upscaler...")
        upscaler = nu_scaler_core.create_advanced_upscaler("quality")
        
        # Check if we have proper GPU detection
        if hasattr(upscaler, 'get_gpu_info'):
            gpu_info = upscaler.get_gpu_info()
            print("\nGPU Information:")
            for key, value in gpu_info.items():
                print(f"  {key}: {value}")
        else:
            print("\nGPU info not available")
        
        # Initialize with some reasonable dimensions
        input_width, input_height = 1920, 1080
        scale = 2.0
        output_width, output_height = int(input_width * scale), int(input_height * scale)
        
        print(f"\nInitializing upscaler: {input_width}x{input_height} -> {output_width}x{output_height}")
        upscaler.initialize(input_width, input_height, output_width, output_height)
        
        # Create a test pattern
        print("Creating test image...")
        import numpy as np
        test_img = np.zeros((input_height, input_width, 4), dtype=np.uint8)
        for y in range(input_height):
            for x in range(input_width):
                test_img[y, x, 0] = int(x * 255 / input_width)  # R
                test_img[y, x, 1] = int(y * 255 / input_height)  # G
                test_img[y, x, 2] = int((x + y) * 255 / (input_width + input_height))  # B
                test_img[y, x, 3] = 255  # A
        
        # Launch monitoring thread
        stop_monitor = [False]
        monitor_thread = Thread(target=vram_monitor, args=(upscaler, stop_monitor))
        monitor_thread.daemon = True
        monitor_thread.start()
        
        # Run continuous upscaling
        print("\nStarting continuous upscaling (Press Ctrl+C to stop)...")
        try:
            frame_count = 0
            start_time = time.time()
            
            while True:
                # Upscale the test pattern
                output_bytes = upscaler.upscale(test_img.tobytes())
                
                # Update stats every 10 frames
                if frame_count % 10 == 0:
                    elapsed = time.time() - start_time
                    fps = frame_count / elapsed if elapsed > 0 else 0
                    print(f"\rFrame: {frame_count}, FPS: {fps:.2f}", end="", flush=True)
                    
                    # Force GPU stats update
                    if hasattr(upscaler, 'update_gpu_stats'):
                        upscaler.update_gpu_stats()
                        
                    # Force cleanup every 100 frames
                    if frame_count % 100 == 0 and hasattr(upscaler, 'force_cleanup'):
                        upscaler.force_cleanup()
                
                frame_count += 1
                
                # Sleep a tiny bit to prevent maxing out CPU
                time.sleep(0.001)
                
        except KeyboardInterrupt:
            print("\nStopping...")
        finally:
            stop_monitor[0] = True
            monitor_thread.join(1.0)
        
        return True
    except Exception as e:
        print(f"Error: {e}")
        traceback.print_exc()
        return False

def vram_monitor(upscaler, stop_flag):
    """Monitor thread to display VRAM usage"""
    try:
        while not stop_flag[0]:
            # Get VRAM stats
            if hasattr(upscaler, 'get_vram_stats'):
                stats = upscaler.get_vram_stats()
                print(f"\nVRAM: {stats.used_mb:.1f} MB / {stats.total_mb:.1f} MB ({stats.usage_percent:.1f}%)")
                
                # Get detailed GPU info
                if hasattr(upscaler, 'get_gpu_info'):
                    gpu_info = upscaler.get_gpu_info()
                    allocated_buffers = gpu_info.get('allocated_buffers', 0)
                    allocated_bytes = gpu_info.get('allocated_bytes', 0)
                    print(f"Buffers: {allocated_buffers}, Bytes: {allocated_bytes / (1024*1024):.1f} MB")
            
            # Sleep between updates
            time.sleep(2.0)
    except Exception as e:
        print(f"Monitor error: {e}")

if __name__ == "__main__":
    success = run_gpu_monitor()
    sys.exit(0 if success else 1) 