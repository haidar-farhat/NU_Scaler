#!/usr/bin/env python
"""
Simple test script for testing the screen capture functionality without the full GUI.
"""
import sys
import time
import traceback
import os
from PIL import Image
import numpy as np

def test_capture():
    print("Testing Nu_Scaler Core screen capture...")
    try:
        import nu_scaler_core
        print(f"Imported nu_scaler_core from: {nu_scaler_core.__file__}")
        
        # First, try to list available windows
        print("\nListing available windows:")
        try:
            if hasattr(nu_scaler_core, 'PyScreenCapture') and hasattr(nu_scaler_core.PyScreenCapture, 'list_windows'):
                windows = nu_scaler_core.PyScreenCapture.list_windows()
                print(f"Available windows: {windows}")
            else:
                print("PyScreenCapture.list_windows method not available")
        except Exception as e:
            print(f"Error listing windows: {e}")
            traceback.print_exc()
        
        # Create upscaler first
        print("\nCreating upscaler...")
        upscaler = nu_scaler_core.PyWgpuUpscaler("quality", "bilinear")
        
        # Create capture
        print("\nCreating screen capture...")
        capture = nu_scaler_core.PyScreenCapture()
        
        # Try fullscreen capture
        print("\nStarting fullscreen capture...")
        target = nu_scaler_core.PyCaptureTarget.FullScreen
        capture.start(target, None, None)
        
        # Wait a bit for capture to initialize
        print("Waiting for capture to initialize...")
        time.sleep(1)
        
        # Capture frames
        max_frames = 10
        for i in range(max_frames):
            print(f"\nCapturing frame {i+1}/{max_frames}...")
            frame_result = capture.get_frame()
            
            if frame_result is None:
                print("  No frame received")
                continue
                
            frame_bytes, width, height = frame_result
            print(f"  Received frame: {width}x{height} pixels, {len(frame_bytes)} bytes")
            
            # Initialize upscaler for the first frame
            if i == 0:
                scale = 1.5
                out_width = int(width * scale)
                out_height = int(height * scale)
                print(f"  Initializing upscaler: {width}x{height} -> {out_width}x{out_height}")
                upscaler.initialize(width, height, out_width, out_height)
            
            # Upscale frame
            print("  Upscaling frame...")
            try:
                out_bytes = upscaler.upscale(frame_bytes)
                print(f"  Upscale successful: {len(out_bytes)} bytes")
                
                # Save the last frame for verification
                if i == max_frames - 1:
                    save_frame(frame_bytes, width, height, "capture_input.png")
                    save_frame(out_bytes, out_width, out_height, "capture_output.png")
            except Exception as e:
                print(f"  Error upscaling: {e}")
                traceback.print_exc()
            
            time.sleep(0.2)
        
        # Stop capture
        print("\nStopping capture...")
        capture.stop()
        print("Capture stopped")
        
        return True
    except Exception as e:
        print(f"Error in test_capture: {e}")
        traceback.print_exc()
        return False

def save_frame(frame_bytes, width, height, filename):
    """Save frame bytes as an image file."""
    try:
        # Convert bytes to numpy array
        frame_array = np.frombuffer(frame_bytes, dtype=np.uint8)
        frame_array = frame_array.reshape((height, width, 4))
        
        # Create PIL image and save
        img = Image.fromarray(frame_array)
        img.save(filename)
        print(f"Saved {filename}")
    except Exception as e:
        print(f"Error saving {filename}: {e}")
        traceback.print_exc()

if __name__ == "__main__":
    success = test_capture()
    sys.exit(0 if success else 1) 