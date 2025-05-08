#!/usr/bin/env python
"""
Test script to verify GPU detection and VRAM usage in Nu_Scaler Core.
"""
import sys
import time
import traceback

def test_gpu_detection():
    try:
        import nu_scaler_core
        print(f"Loaded nu_scaler_core module from: {nu_scaler_core.__file__}")
        
        # Create an advanced upscaler to test GPU detection
        print("\nCreating advanced upscaler to test GPU detection...")
        upscaler = nu_scaler_core.create_advanced_upscaler("quality")
        
        # Check if VRAM stats are available
        print("\nChecking VRAM stats...")
        if hasattr(upscaler, 'get_vram_stats'):
            stats = upscaler.get_vram_stats()
            print(f"  Total VRAM: {stats.total_mb:.2f} MB")
            print(f"  Used VRAM: {stats.used_mb:.2f} MB")
            print(f"  Free VRAM: {stats.free_mb:.2f} MB")
            print(f"  App allocated: {stats.app_allocated_mb:.2f} MB")
            print(f"  Usage: {stats.usage_percent:.2f}%")
        else:
            print("VRAM stats not available")
        
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
        
        # Test upscaling
        print("Upscaling test image...")
        start_time = time.time()
        output_bytes = upscaler.upscale(test_img.tobytes())
        elapsed = time.time() - start_time
        
        print(f"Upscaling completed in {elapsed:.3f} seconds")
        print(f"Output size: {len(output_bytes)} bytes")
        
        # Check VRAM stats after upscaling
        print("\nChecking VRAM stats after upscaling...")
        if hasattr(upscaler, 'get_vram_stats'):
            stats = upscaler.get_vram_stats()
            print(f"  Total VRAM: {stats.total_mb:.2f} MB")
            print(f"  Used VRAM: {stats.used_mb:.2f} MB")
            print(f"  Free VRAM: {stats.free_mb:.2f} MB")
            print(f"  App allocated: {stats.app_allocated_mb:.2f} MB")
            print(f"  Usage: {stats.usage_percent:.2f}%")
        
        # Save output if PIL is available
        try:
            from PIL import Image
            output_array = np.frombuffer(output_bytes, dtype=np.uint8).reshape(output_height, output_width, 4)
            output_img = Image.fromarray(output_array)
            output_img.save("gpu_test_output.png")
            print("\nSaved output image to gpu_test_output.png")
        except ImportError:
            print("\nPIL not available, skipping image save")
        
        return True
    except Exception as e:
        print(f"Error: {e}")
        traceback.print_exc()
        return False

if __name__ == "__main__":
    success = test_gpu_detection()
    sys.exit(0 if success else 1) 