# NuScaler Python Package

This is the Python frontend for the NuScaler hybrid Rust+Python real-time upscaling application.

## Usage

```
# Run core API test
python -m nu_scaler.core

# Run GUI (placeholder)
python -m nu_scaler.gui
```

## About
- The core logic and performance-critical code is implemented in Rust (see nu_scaler_core).
- The Python package will use FFI (PyO3/maturin) to call into the Rust core.
- The GUI will be implemented with DearPyGui or PyQt.
