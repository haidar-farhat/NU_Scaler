"""
nu_scaler.core - Python API for NuScaler (to be backed by Rust FFI via PyO3/maturin)
"""

class Engine:
    """Orchestrates capture, upscaling, and rendering. Will wrap Rust Engine via FFI."""
    def __init__(self):
        pass

class Capture:
    """Configures and manages capture source. Will wrap Rust Capture via FFI."""
    def __init__(self):
        pass

class Upscaler:
    """Configures upscaling parameters. Will wrap Rust Upscaler via FFI."""
    def __init__(self):
        pass

class Renderer:
    """Handles frame presentation. Will wrap Rust Renderer via FFI."""
    def __init__(self):
        pass

# Simple test for instantiation
if __name__ == "__main__":
    e = Engine()
    c = Capture()
    u = Upscaler()
    r = Renderer()
    print("All core classes instantiated successfully.")
