#!/usr/bin/env python
"""
Test script for Nu_Scaler Core upscaling functionality.
This creates a simple test image and upscales it using the WGPU upscaler.
"""
import sys
import time
import os
from PIL import Image
import numpy as np
import nu_scaler_core

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

def save_image(img_array, filename):
    """Save numpy array as image."""
    img = Image.fromarray(img_array)
    img.save(filename)
    print(f"Saved image to {filename}")

def test_upscaling(input_width=320, input_height=240, scale_factor=2.0):
    """Test the upscaling functionality."""
    # Create upscaler
    print("\nInitializing upscaler...")
    upscaler = nu_scaler_core.PyWgpuUpscaler("quality", "bilinear")
    
    # Create test image
    input_img = create_test_image(input_width, input_height)
    
    # Calculate output dimensions
    output_width = int(input_width * scale_factor)
    output_height = int(input_height * scale_factor)
    
    # Initialize upscaler with dimensions
    print(f"Configuring upscaler: {input_width}x{input_height} → {output_width}x{output_height}")
    upscaler.initialize(input_width, input_height, output_width, output_height)
    # Access the upscale_scale property directly
    print(f"Current upscale scale: {upscaler.upscale_scale}")
    
    # Convert numpy array to bytes
    input_bytes = input_img.tobytes()
    
    # Save input image for reference
    save_image(input_img, "test_input.png")
    
    # Time the upscaling operation
    print("\nUpscaling image...")
    start_time = time.time()
    
    # Perform upscaling
    output_bytes = upscaler.upscale(input_bytes)
    
    # Calculate elapsed time
    elapsed = time.time() - start_time
    print(f"Upscaling completed in {elapsed:.3f} seconds")
    
    # Convert output bytes back to numpy array
    output_img = np.frombuffer(output_bytes, dtype=np.uint8).reshape(output_height, output_width, 4)
    
    # Save the output image
    save_image(output_img, "test_output.png")
    
    print("\nTest completed successfully!")
    print(f"Input image: {input_width}x{input_height}")
    print(f"Output image: {output_width}x{output_height}")
    
    return True

if __name__ == "__main__":
    # Parse command line arguments
    input_width = 320
    input_height = 240
    scale_factor = 2.0
    
    # Allow overriding dimensions from command line
    if len(sys.argv) > 2:
        input_width = int(sys.argv[1])
        input_height = int(sys.argv[2])
    
    if len(sys.argv) > 3:
        scale_factor = float(sys.argv[3])
    
    # Run the test
    try:
        test_upscaling(input_width, input_height, scale_factor)
    except Exception as e:
        print(f"Error during upscaling test: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1) 