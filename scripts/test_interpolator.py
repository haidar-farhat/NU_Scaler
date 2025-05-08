import sys
import os
import numpy as np
from PIL import Image

# Add the target/wheels directory to sys.path to find the wheel if not installed
# Adjust the path based on your actual project structure and where the wheel is built
wheel_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'target', 'wheels'))
if wheel_dir not in sys.path:
    print(f"Adding {wheel_dir} to sys.path")
    sys.path.insert(0, wheel_dir)

try:
    # Assuming the module name inside the wheel follows the crate name
    from nu_scaler_core import WgpuFrameInterpolator 
except ImportError as e:
    print(f"Error importing nu_scaler_core: {e}")
    print("Ensure the wheel is built and discoverable (e.g., installed or path added).")
    print(f"Looking in: {sys.path}")
    exit(1)

def run_test():
    print("Generating test images...")
    # Generate two simple patterns
    width, height = 64, 64
    img_a = np.zeros((height, width, 4), dtype=np.uint8)
    img_a[16:48, 16:48] = [255, 0, 0, 255]      # red square
    img_b = np.zeros_like(img_a)
    img_b[16:48, 16:48] = [0, 0, 255, 255]      # blue square

    print("Initializing WgpuFrameInterpolator...")
    try:
        # TODO: Update instantiation if WgpuFrameInterpolator::new needs args (like device/queue)
        # For now, assuming a default constructor exists for basic testing.
        # You might need to adapt this based on your lib.rs bindings.
        interp = WgpuFrameInterpolator()
    except Exception as e:
        print(f"Error initializing WgpuFrameInterpolator: {e}")
        print("Check if the constructor in Rust needs specific arguments.")
        exit(1)

    print("Calling interpolate_py (time_t=0.5)...")
    time_t = 0.5
    try:
        # TODO: Verify the expected input/output types for the Python binding
        # Assuming it takes numpy arrays [height, width, 4] RGBA uint8
        # and returns a similar numpy array.
        out = interp.interpolate_py(img_a, img_b, time_t=time_t)
    except Exception as e:
        print(f"Error calling interpolate_py: {e}")
        print("Check the method signature and expected arguments in your Python bindings.")
        exit(1)

    print("Saving output image interp_half.png...")
    try:
        # Ensure output is a valid numpy array for Image.fromarray
        if not isinstance(out, np.ndarray) or out.shape != (height, width, 4) or out.dtype != np.uint8:
            print(f"Warning: Output from interpolate_py is not the expected numpy array format.")
            print(f"Got type: {type(out)}, shape: {getattr(out, 'shape', 'N/A')}, dtype: {getattr(out, 'dtype', 'N/A')}")
            # Attempt conversion if possible, otherwise save raw representation?
            # For now, we'll just try to save it.
        
        Image.fromarray(out, 'RGBA').save('interp_half.png')
        print("--> Successfully wrote interp_half.png")
    except Exception as e:
        print(f"Error saving output image: {e}")
        print("Check if the output 'out' has the correct format/type.")
        exit(1)

if __name__ == "__main__":
    # Ensure necessary libraries are installed
    try:
        import numpy
        import PIL
    except ImportError:
        print("Error: numpy and Pillow (PIL) are required for this test script.")
        print("Please install them: pip install numpy Pillow")
        exit(1)
        
    run_test() 