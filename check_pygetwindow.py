import sys

print("--- PyGetWindow Test Script ---")
print(f"Python Executable: {sys.executable}")
print(f"Python Version: {sys.version}")
print(f"sys.path: {sys.path}")

print("\nAttempting to import pygetwindow...")
try:
    import pygetwindow
    print("Successfully imported pygetwindow.")
    print(f"  Version: {getattr(pygetwindow, '__version__', 'NOT FOUND')}")
    print(f"  File: {getattr(pygetwindow, '__file__', 'NOT FOUND')}")
    
    print("\nChecking for 'getWindowsWithPid' attribute...")
    if hasattr(pygetwindow, 'getWindowsWithPid'):
        print("  SUCCESS: 'getWindowsWithPid' was found in pygetwindow module.")
        print("  Attempting to call pygetwindow.getWindowsWithPid(0) as a basic test...")
        try:
            windows = pygetwindow.getWindowsWithPid(0) # Test with PID 0 (System Idle Process)
            print(f"  Call to getWindowsWithPid(0) returned: {len(windows)} windows (this is just a basic functionality test).")
        except Exception as e_call:
            print(f"  ERROR calling getWindowsWithPid(0): {e_call}")
    else:
        print("  FAILURE: 'getWindowsWithPid' was NOT found in pygetwindow module.")
        print("  This is the primary issue.")
        print("\n  Listing all attributes found in pygetwindow module:")
        for attr in dir(pygetwindow):
            print(f"    {attr}")

except ImportError as e:
    print(f"FAILURE: Could not import pygetwindow. Error: {e}")
except Exception as e_general:
    print(f"An unexpected error occurred: {e_general}")

print("\n--- End of PyGetWindow Test Script ---") 