#!/usr/bin/env python
"""
Simple test script to verify that the Nu_Scaler Core module is properly installed
and can be imported.
"""
import sys
import traceback

def test_import():
    print("Testing Nu_Scaler Core import...")
    try:
        import nu_scaler_core
        print(f"✓ Successfully imported nu_scaler_core")
        print(f"Module location: {nu_scaler_core.__file__}")
        return True
    except ImportError as e:
        print(f"✗ Failed to import nu_scaler_core: {e}")
        traceback.print_exc()
        return False

def test_create_upscaler():
    print("\nTesting upscaler creation...")
    try:
        import nu_scaler_core
        upscaler = nu_scaler_core.PyWgpuUpscaler("quality", "bilinear")
        print(f"✓ Successfully created upscaler")
        return True
    except Exception as e:
        print(f"✗ Failed to create upscaler: {e}")
        traceback.print_exc()
        return False

if __name__ == "__main__":
    print(f"Python version: {sys.version}")
    print(f"Python executable: {sys.executable}")
    print()
    
    import_success = test_import()
    if import_success:
        upscaler_success = test_create_upscaler()
    
    if import_success and upscaler_success:
        print("\n✓ All tests passed! Nu_Scaler Core is working properly.")
        sys.exit(0)
    else:
        print("\n✗ Some tests failed. Please check the error messages above.")
        sys.exit(1) 