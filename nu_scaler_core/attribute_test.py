#!/usr/bin/env python
"""
Test script to inspect the PyWgpuUpscaler class attributes and methods.
"""
import nu_scaler_core

def inspect_class(cls_name, obj):
    print(f"\nInspecting {cls_name}:")
    print("-" * 50)
    
    # Get all attributes and methods
    attrs = dir(obj)
    
    print("Attributes and Methods:")
    for attr in sorted(attrs):
        # Skip private/special methods
        if attr.startswith('__') and attr.endswith('__'):
            continue
        
        # Try to get the attribute type or value
        try:
            val = getattr(obj, attr)
            attr_type = type(val).__name__
            
            # Try to get more info if it's a method
            if callable(val):
                import inspect
                try:
                    sig = inspect.signature(val)
                    print(f"  {attr}{sig} -> method")
                except (ValueError, TypeError):
                    print(f"  {attr}() -> method")
            else:
                print(f"  {attr} -> {attr_type}")
        except Exception as e:
            print(f"  {attr} -> Error: {str(e)}")
    
if __name__ == "__main__":
    print("Creating PyWgpuUpscaler instance...")
    upscaler = nu_scaler_core.PyWgpuUpscaler("quality", "bilinear")
    
    # Inspect the class
    inspect_class("PyWgpuUpscaler", upscaler)
    
    # Also check available module attributes
    print("\nAvailable in nu_scaler_core module:")
    print("-" * 50)
    for item in sorted(dir(nu_scaler_core)):
        if not item.startswith('__'):
            print(f"  {item}")
    
    print("\nDone.") 