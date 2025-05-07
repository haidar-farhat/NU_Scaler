import sys
import traceback

print("======= START OF TEST_DLSS.PY =======")
print(f"Python version: {sys.version}")

try:
    print("About to import nu_scaler_core...")
    import nu_scaler_core
    print(f"Module imported: {nu_scaler_core}")
    
    # Check for the create_dlss_upscaler function
    print("Checking for DLSS functionality...")
    if hasattr(nu_scaler_core, 'create_dlss_upscaler'):
        print("create_dlss_upscaler function exists")
        
        try:
            print("Attempting to create DLSS upscaler...")
            upscaler = nu_scaler_core.create_dlss_upscaler("quality")
            print(f"DLSS upscaler created successfully: {upscaler}")
            print(f"Upscaler type: {type(upscaler)}")
            
            # If we got this far, let's try to initialize it
            try:
                print("Attempting to initialize DLSS upscaler...")
                upscaler.initialize(1920, 1080, 3840, 2160)
                print("Upscaler initialized successfully")
            except Exception as e:
                print(f"Error initializing upscaler: {e}")
                traceback.print_exc()
                
        except Exception as e:
            print(f"Error creating DLSS upscaler: {e}")
            traceback.print_exc()
    else:
        print("create_dlss_upscaler function not found")
        
        # Look for other DLSS-related functions or classes
        print("Searching for DLSS-related items...")
        dlss_items = [item for item in dir(nu_scaler_core) if 'dlss' in item.lower()]
        if dlss_items:
            print(f"DLSS-related items found: {dlss_items}")
        else:
            print("No DLSS-related items found in the module")
    
    # Also check other upscaler creation functions
    print("Checking other upscaler functions...")
    if hasattr(nu_scaler_core, 'create_best_upscaler'):
        print("create_best_upscaler function exists")
        try:
            print("Attempting to create best upscaler...")
            best = nu_scaler_core.create_best_upscaler("quality") 
            print(f"Best upscaler created: {best}, type: {type(best)}")
        except Exception as e:
            print(f"Error creating best upscaler: {e}")
            traceback.print_exc()
            
except ImportError as e:
    print(f"ImportError: {e}")
    traceback.print_exc()
except Exception as e:
    print(f"Unexpected error: {e}")
    traceback.print_exc()

print("======= END OF TEST_DLSS.PY =======") 