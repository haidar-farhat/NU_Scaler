#!/usr/bin/env python
"""
Test script to check if the nu_scaler_core module can be imported
from the Nu_Scaler Python application.
"""
import sys
import os
import traceback

def test_import():
    print(f"Python executable: {sys.executable}")
    print(f"Python version: {sys.version}")
    print(f"Current working directory: {os.getcwd()}")
    print(f"Python path: {sys.path}")
    print("\nTrying to import nu_scaler_core...")
    
    try:
        import nu_scaler_core
        print(f"✓ Successfully imported nu_scaler_core")
        print(f"Module location: {nu_scaler_core.__file__}")
        
        # Try to create an upscaler
        print("\nTrying to create an upscaler...")
        upscaler = nu_scaler_core.PyWgpuUpscaler("quality", "bilinear")
        print(f"✓ Successfully created upscaler")
        
        # List available windows to test the function failing in the app
        print("\nTrying to list windows...")
        if hasattr(nu_scaler_core.PyScreenCapture, 'list_windows'):
            windows = nu_scaler_core.PyScreenCapture.list_windows()
            print(f"✓ Successfully listed windows: {windows}")
        else:
            print("✗ PyScreenCapture.list_windows method not found")
        
        return True
    except ImportError as e:
        print(f"✗ Failed to import nu_scaler_core: {e}")
        traceback.print_exc()
        return False
    except Exception as e:
        print(f"✗ Error during test: {e}")
        traceback.print_exc()
        return False

if __name__ == "__main__":
    sys.exit(0 if test_import() else 1) 