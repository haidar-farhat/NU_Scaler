"""
GPU Optimization Helper for Nu_Scaler
This module helps maximize GPU utilization and performance
"""
import time
import sys
import traceback
import threading

def force_gpu_activation(upscaler):
    """Force the GPU to activate fully for better performance"""
    try:
        print("[GPU Optimizer] Activating GPU...")
        
        # Use new force_gpu_activation method if available
        if hasattr(upscaler, 'force_gpu_activation'):
            upscaler.force_gpu_activation()
            print("[GPU Optimizer] GPU activated via direct method")
            return True
            
        # Use older methods if new one isn't available
        if hasattr(upscaler, 'get_vram_stats'):
            stats = upscaler.get_vram_stats()
            print(f"[GPU Optimizer] VRAM status: {stats.used_mb}MB / {stats.total_mb}MB ({stats.usage_percent}%)")
            
        # Pre-allocate buffers for common resolutions
        print("[GPU Optimizer] Pre-allocating GPU buffers...")
        resolutions = [
            (1920, 1080),   # Full HD
            (2560, 1440),   # 2K
            (3840, 2160),   # 4K
        ]
        
        for width, height in resolutions:
            output_width = width * 2
            output_height = height * 2
            
            # Initialize the upscaler temporarily to allocate buffers
            upscaler.initialize(width, height, output_width, output_height)
            
            # Force cleanup after each initialization
            if hasattr(upscaler, 'force_cleanup'):
                upscaler.force_cleanup()
        
        print("[GPU Optimizer] GPU primed for better performance")
        return True
    except Exception as e:
        print(f"[GPU Optimizer] Error activating GPU: {e}")
        traceback.print_exc()
        return False

def start_gpu_monitor(upscaler):
    """Start a background thread to monitor GPU usage"""
    def monitor_loop():
        try:
            while True:
                if hasattr(upscaler, 'get_vram_stats'):
                    stats = upscaler.get_vram_stats()
                    print(f"[GPU Monitor] VRAM: {stats.used_mb:.1f}MB / {stats.total_mb:.1f}MB ({stats.usage_percent:.1f}%)")
                
                # Update GPU stats
                if hasattr(upscaler, 'update_gpu_stats'):
                    upscaler.update_gpu_stats()
                
                time.sleep(5.0)  # Check every 5 seconds
        except Exception as e:
            print(f"[GPU Monitor] Error: {e}")
    
    # Create and start the monitor thread
    thread = threading.Thread(target=monitor_loop, daemon=True)
    thread.start()
    return thread

def optimize_upscaler(upscaler):
    """Fully optimize an upscaler for best performance"""
    print("[GPU Optimizer] Optimizing upscaler performance...")
    
    # Force GPU activation
    force_gpu_activation(upscaler)
    
    # Start the GPU monitor in the background
    monitor_thread = start_gpu_monitor(upscaler)
    
    # Set GPU-optimized options
    if hasattr(upscaler, 'set_memory_strategy'):
        upscaler.set_memory_strategy("aggressive")
        print("[GPU Optimizer] Set memory strategy to aggressive")
    
    if hasattr(upscaler, 'set_adaptive_quality'):
        upscaler.set_adaptive_quality(True)
        print("[GPU Optimizer] Enabled adaptive quality for improved performance")
        
    print("[GPU Optimizer] Optimization complete")
    return True 