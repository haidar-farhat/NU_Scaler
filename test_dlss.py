import sys
print(f"Python version: {sys.version}")

try:
    import nu_scaler_core
    print(f"Module imported: {nu_scaler_core}")
    
    # Check for the create_dlss_upscaler function
    if hasattr(nu_scaler_core, 'create_dlss_upscaler'):
        print("create_dlss_upscaler function exists")
        
        try:
            # Try to create a DLSS upscaler
            upscaler = nu_scaler_core.create_dlss_upscaler("quality")
            print(f"DLSS upscaler created successfully: {upscaler}")
            print(f"Upscaler type: {type(upscaler)}")
            
            # If we got this far, let's try to initialize it
            try:
                upscaler.initialize(1920, 1080, 3840, 2160)
                print("Upscaler initialized successfully")
            except Exception as e:
                print(f"Error initializing upscaler: {e}")
                
        except Exception as e:
            print(f"Error creating DLSS upscaler: {e}")
    else:
        print("create_dlss_upscaler function not found")
        
        # Look for other DLSS-related functions or classes
        dlss_items = [item for item in dir(nu_scaler_core) if 'dlss' in item.lower()]
        if dlss_items:
            print(f"DLSS-related items found: {dlss_items}")
        else:
            print("No DLSS-related items found in the module")
            
except ImportError as e:
    print(f"ImportError: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
    import traceback
    traceback.print_exc()

print("Script completed.") 