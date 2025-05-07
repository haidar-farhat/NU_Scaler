try:
    # Attempt to import all from the .pyd file (nu_scaler.cpXXX.pyd)
    # This makes its contents available directly under the nu_scaler_py.nu_scaler package
    from .nu_scaler import *
except ImportError as e:
    print(f"CRITICAL: Failed to import symbols from .nu_scaler.pyd into nu_scaler_py.nu_scaler package: {e}")
    # Optionally re-raise or define a placeholder to indicate failure
    # raise e # Uncomment to make the package import fail loudly if .pyd is missing/broken 