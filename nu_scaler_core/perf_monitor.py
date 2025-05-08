#!/usr/bin/env python
"""
Performance monitor for Nu_Scaler Core
Monitors CPU and GPU usage during upscaling operations
"""
import sys
import time
import traceback
import numpy as np
import psutil
from threading import Thread

def run_performance_test():
    try:
        import nu_scaler_core
        print(f"Loaded nu_scaler_core module from: {nu_scaler_core.__file__}")
        
        # Create an advanced upscaler
        print("\nCreating advanced upscaler...")
        upscaler = nu_scaler_core.create_advanced_upscaler("quality")
        
        # Print GPU info
        if hasattr(upscaler, 'get_gpu_info'):
            gpu_info = upscaler.get_gpu_info()
            print("\nGPU Information:")
            for key, value in gpu_info.items():
                print(f"  {key}: {value}")
        
        # Initialize with 4K input dimensions to stress test
        input_width, input_height = 3840, 2160
        scale = 1.5
        output_width, output_height = int(input_width * scale), int(input_height * scale)
        
        print(f"\nInitializing upscaler for performance test: {input_width}x{input_height} -> {output_width}x{output_height}")
        upscaler.initialize(input_width, input_height, output_width, output_height)
        
        # Create a test pattern
        print("Creating large test image...")
        test_img = np.zeros((input_height, input_width, 4), dtype=np.uint8)
        for y in range(0, input_height, 8):
            for x in range(0, input_width, 8):
                # Create checkerboard pattern
                is_white = ((x // 64) % 2) == ((y // 64) % 2)
                color = 255 if is_white else 60
                test_img[y:y+8, x:x+8, 0] = color  # R
                test_img[y:y+8, x:x+8, 1] = color  # G
                test_img[y:y+8, x:x+8, 2] = color  # B
                test_img[y:y+8, x:x+8, 3] = 255    # A
        
        # Launch monitoring thread
        stop_monitor = [False]
        monitor_thread = Thread(target=performance_monitor, args=(upscaler, stop_monitor))
        monitor_thread.daemon = True
        monitor_thread.start()
        
        # Run upscaling performance test
        print("\nRunning upscaling performance test (Press Ctrl+C to stop)...")
        frame_times = []
        try:
            frame_count = 0
            start_time = time.time()
            
            while frame_count < 100:  # Run 100 frames
                # Measure frame time
                frame_start = time.time()
                
                # Upscale the test pattern
                output_bytes = upscaler.upscale(test_img.tobytes())
                
                # Calculate frame time
                frame_end = time.time()
                frame_time = (frame_end - frame_start) * 1000  # ms
                frame_times.append(frame_time)
                
                # Update stats every 5 frames
                if frame_count % 5 == 0:
                    elapsed = time.time() - start_time
                    fps = frame_count / elapsed if elapsed > 0 else 0
                    print(f"\rFrame: {frame_count}, Last frame time: {frame_time:.2f}ms, FPS: {fps:.2f}", end="", flush=True)
                    
                    # Force GPU stats update
                    if hasattr(upscaler, 'update_gpu_stats'):
                        upscaler.update_gpu_stats()
                
                frame_count += 1
                
                # Sleep a tiny bit to allow monitoring thread to run
                time.sleep(0.01)
            
            # Calculate and print summary statistics
            avg_frame_time = np.mean(frame_times)
            min_frame_time = np.min(frame_times)
            max_frame_time = np.max(frame_times)
            p95_frame_time = np.percentile(frame_times, 95)
            avg_fps = 1000 / avg_frame_time
            
            print("\n\nPerformance Summary:")
            print(f"  Total frames: {frame_count}")
            print(f"  Average frame time: {avg_frame_time:.2f}ms")
            print(f"  Min frame time: {min_frame_time:.2f}ms")
            print(f"  Max frame time: {max_frame_time:.2f}ms")
            print(f"  95th percentile frame time: {p95_frame_time:.2f}ms")
            print(f"  Average FPS: {avg_fps:.2f}")
            
        except KeyboardInterrupt:
            print("\nTest stopped by user")
        finally:
            stop_monitor[0] = True
            monitor_thread.join(1.0)
        
        return True
    except Exception as e:
        print(f"Error: {e}")
        traceback.print_exc()
        return False

def performance_monitor(upscaler, stop_flag):
    """Monitor thread to display CPU and VRAM usage"""
    start_time = time.time()
    try:
        while not stop_flag[0]:
            # Get CPU usage
            cpu_percent = psutil.cpu_percent(interval=None)
            memory_percent = psutil.virtual_memory().percent
            
            # Get VRAM stats
            vram_used = 0
            vram_total = 0
            vram_percent = 0
            
            if hasattr(upscaler, 'get_vram_stats'):
                try:
                    stats = upscaler.get_vram_stats()
                    vram_used = stats.used_mb
                    vram_total = stats.total_mb
                    vram_percent = stats.usage_percent
                except:
                    pass
                    
            elapsed = time.time() - start_time
            print(f"\n[{elapsed:.1f}s] CPU: {cpu_percent:.1f}%, RAM: {memory_percent:.1f}%, VRAM: {vram_used:.1f}MB/{vram_total:.1f}MB ({vram_percent:.1f}%)")
            
            # Sleep between updates
            time.sleep(1.0)
    except Exception as e:
        print(f"Monitor error: {e}")

if __name__ == "__main__":
    try:
        import psutil
    except ImportError:
        print("Installing required package: psutil")
        import subprocess
        subprocess.check_call([sys.executable, "-m", "pip", "install", "psutil"])
        import psutil
        
    success = run_performance_test()
    sys.exit(0 if success else 1) 