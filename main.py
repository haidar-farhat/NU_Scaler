import sys
import os

# Add the nu_scaler_py directory to the Python path
# This is necessary because we are running from the root, and nu_scaler_py is a package
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), 'nu_scaler_py')))

# Check if the nu_scaler_core library is importable
try:
    import nu_scaler_core
    print("Successfully imported nu_scaler_core.")
except ImportError as e:
    print(f"Error importing nu_scaler_core: {e}", file=sys.stderr)
    print("Please ensure you have built the Rust core using 'maturin develop' in the 'nu_scaler_core' directory.", file=sys.stderr)
    sys.exit(1)

# Try importing the gui module directly first
try:
    from nu_scaler_py.nu_scaler import gui # Import from the subdirectory
    print("Successfully imported nu_scaler_py.nu_scaler.gui module.")
except Exception as e:
    print(f"Error importing nu_scaler_py.nu_scaler.gui module: {e}", file=sys.stderr)
    print("Check the structure and content of nu_scaler_py/nu_scaler/gui.py.", file=sys.stderr)
    sys.exit(1)


# Now import the GUI function from the nu_scaler_py package
try:
    # Access function via the imported module
    run_gui = gui.run_gui
except AttributeError as e:
    print(f"Error accessing run_gui function in nu_scaler_py.nu_scaler.gui: {e}", file=sys.stderr)
    print("Check the definition of run_gui in nu_scaler_py/nu_scaler/gui.py.", file=sys.stderr)
    sys.exit(1)

if __name__ == "__main__":
    print("[main.py] Starting NuScaler GUI...")
    print("[main.py] Calling run_gui()...")
    run_gui()
    print("[main.py] run_gui() finished.") 