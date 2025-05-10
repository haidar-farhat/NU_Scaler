#!/usr/bin/env python
"""
Launcher for the Nu_Scaler Modern UI
"""
import sys
import os
from pathlib import Path

# Add project directory to path if running directly
if __name__ == "__main__":
    sys.path.append(str(Path(__file__).parent.parent))

try:
    from nu_scaler_py.nu_scaler.modern_gui import run_gui
    print("Launching Nu_Scaler Modern UI...")
    sys.exit(run_gui())
except ImportError as e:
    print(f"Error importing the modern_gui module: {e}")
    print("Make sure the modern_gui.py file exists in nu_scaler_py/nu_scaler/")
    sys.exit(1) 