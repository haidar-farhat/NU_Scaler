# Simple test script to validate that nu_scaler_core can be imported
import sys
import os
print(f"Python version: {sys.version}")
print("Attempting to import nu_scaler_core...")
print(f"Working directory: {os.getcwd()}")
print(f"Python path: {sys.path}")

try:
    print("Before import...")
    import nu_scaler_core
    print("After import...")
    print(f"Success! Module imported: {nu_scaler_core}")
    print(f"Module file: {getattr(nu_scaler_core, '__file__', 'No file attribute')}")
    print(f"Available attributes and classes:")
    for item in sorted(dir(nu_scaler_core)):
        print(f"  - {item}")
except ImportError as e:
    print(f"ImportError: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
    import traceback
    traceback.print_exc()

print("Script completed.") 