# Simple test script to validate that nu_scaler_core can be imported
import sys
print(f"Python version: {sys.version}")
print("Attempting to import nu_scaler_core...")

try:
    import nu_scaler_core
    print(f"Success! Module imported: {nu_scaler_core}")
    print(f"Available attributes and classes: {dir(nu_scaler_core)}")
except ImportError as e:
    print(f"ImportError: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
    import traceback
    traceback.print_exc() 