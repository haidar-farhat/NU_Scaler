#!/usr/bin/env python
"""
Basic test script for Nu_Scaler Core that upscales a static image.
"""
import sys
import time
import traceback
import os
import numpy as np
from PIL import Image

def create_test_image(width, height):
    """Create a test gradient image."""
    print(f"Creating test image ({width}x{height})...")
    # Create a gradient pattern
    x = np.linspace(0, 1, width)
    y = np.linspace(0, 1, height)
    X, Y = np.meshgrid(x, y)
    
    # Create RGB channels
    r = X * 255
    g = Y * 255
    b = ((X + Y) / 2) * 255
    
    # Create RGBA image (R, G, B, 255)
    img = np.zeros((height, width, 4), dtype=np.uint8)
    img[:, :, 0] = r
    img[:, :, 1] = g
    img[:, :, 2] = b
    img[:, :, 3] = 255
    
    return img

def test_upscaling():
    print("Testing Nu_Scaler Core basic upscaling...")
    try:
        import nu_scaler_core
        print(f"Imported nu_scaler_core from: {nu_scaler_core.__file__}")
        
        # Create test image
        input_width = 320
        input_height = 240
        scale_factor = 2.0
        output_width = int(input_width * scale_factor)
        output_height = int(input_height * scale_factor)
        
        input_img = create_test_image(input_width, input_height)
        input_bytes = input_img.tobytes()
        
        # Save input image
        input_pil = Image.fromarray(input_img)
        input_pil.save("test_basic_input.png")
        print("Saved input image to test_basic_input.png")
        
        # Create upscaler
        print("\nCreating upscaler...")
        upscaler = nu_scaler_core.PyWgpuUpscaler("quality", "bilinear")
        
        # Initialize upscaler
        print(f"Initializing upscaler: {input_width}x{input_height} -> {output_width}x{output_height}")
        upscaler.initialize(input_width, input_height, output_width, output_height)
        
        # Upscale image
        print("Upscaling image...")
        start_time = time.time()
        output_bytes = upscaler.upscale(input_bytes)
        elapsed = time.time() - start_time
        print(f"Upscaling completed in {elapsed:.3f} seconds")
        
        # Convert output bytes to numpy array
        output_img = np.frombuffer(output_bytes, dtype=np.uint8).reshape(output_height, output_width, 4)
        
        # Save output image
        output_pil = Image.fromarray(output_img)
        output_pil.save("test_basic_output.png")
        print("Saved output image to test_basic_output.png")
        
        print("\nTest completed successfully!")
        return True
    except Exception as e:
        print(f"Error in test_upscaling: {e}")
        traceback.print_exc()
        return False

if __name__ == "__main__":
    success = test_upscaling()
    sys.exit(0 if success else 1) 