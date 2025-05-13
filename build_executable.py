import os
import sys
import subprocess
import shutil
from pathlib import Path

def install_dependencies():
    """Install required packages for building the executable"""
    print("Installing required packages...")
    subprocess.check_call([sys.executable, "-m", "pip", "install", "pyinstaller", "pywin32", "setuptools"])
    
    # Install UPX for compression (optional)
    print("Checking for UPX...")
    try:
        # Check if UPX is in PATH
        subprocess.check_call(["upx", "--version"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        print("UPX is already installed")
    except (subprocess.SubprocessError, FileNotFoundError):
        print("UPX not found. Download it for additional compression (optional).")
        print("Visit https://github.com/upx/upx/releases and add it to your PATH")
    
    # Install the package in development mode
    print("Installing package in development mode...")
    subprocess.check_call([sys.executable, "-m", "pip", "install", "-e", "."])

def create_launcher():
    """Create a temporary launcher script"""
    print("Creating launcher script...")
    with open("app_launcher.py", "w") as f:
        f.write("""
from nu_scaler_py.nu_scaler.main import run_gui

if __name__ == "__main__":
    run_gui()
""")
    return "app_launcher.py"

def build_executable():
    """Build the executable using PyInstaller with the spec file"""
    print("Building executable with PyInstaller...")
    
    # Create launcher
    create_launcher()
    
    # Check if spec file exists, otherwise use default command
    if os.path.exists("nuscaler.spec"):
        cmd = [
            "pyinstaller",
            "--clean",
            "nuscaler.spec"
        ]
    else:
        # PyInstaller command with optimizations for size
        cmd = [
            "pyinstaller",
            "--name=NuScaler",
            "--onefile",  # Create a single executable
            "--windowed",  # Windows-based application (no console)
            "--clean",     # Clean PyInstaller cache
            "--strip",     # Strip symbols from binary (reduces size)
            "--noupx",     # Set to False if UPX is installed
            # Add hidden imports that might be needed
            "--hidden-import=PySide6.QtCore",
            "--hidden-import=PySide6.QtWidgets",
            "--hidden-import=PySide6.QtGui",
            "--hidden-import=win32gui",
            "--hidden-import=win32process",
            "--hidden-import=win32con",
            "--exclude-module=matplotlib",
            "--exclude-module=scipy",
            "--exclude-module=pandas",
            "--exclude-module=tkinter",
            "app_launcher.py"
        ]
    
    try:
        subprocess.check_call(cmd)
        print(f"Executable built successfully in {os.path.abspath(os.path.join('dist', 'NuScaler.exe'))}")
    except subprocess.CalledProcessError as e:
        print(f"Error building executable: {e}")
        sys.exit(1)
    finally:
        # Clean up temporary files
        if os.path.exists("app_launcher.py"):
            os.remove("app_launcher.py")

def optimize_executable():
    """Optimize the executable size after build (if UPX is available)"""
    exe_path = os.path.join("dist", "NuScaler.exe")
    
    if not os.path.exists(exe_path):
        print(f"Executable not found at {exe_path}")
        return
    
    # Get original size
    original_size = os.path.getsize(exe_path) / (1024 * 1024)  # Size in MB
    print(f"Original executable size: {original_size:.2f} MB")
    
    # Try to use UPX for additional compression if available
    try:
        print("Trying additional compression with UPX...")
        subprocess.check_call(["upx", "--best", exe_path])
        
        # Get new size
        new_size = os.path.getsize(exe_path) / (1024 * 1024)  # Size in MB
        print(f"Compressed executable size: {new_size:.2f} MB")
        print(f"Size reduction: {(original_size - new_size) / original_size * 100:.2f}%")
    except (subprocess.SubprocessError, FileNotFoundError):
        print("UPX not available or compression failed. Skipping additional compression.")

if __name__ == "__main__":
    install_dependencies()
    build_executable()
    optimize_executable() 