import sys
import os
import numpy as np
from PIL import Image
import time # Import time module

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

def generate_test_images(width, height):
    """Generates simple red and blue square test images."""
    img_a = np.zeros((height, width, 4), dtype=np.uint8)
    center_h, center_w = height // 2, width // 2
    half_box_h, half_box_w = height // 4, width // 4
    img_a[center_h - half_box_h : center_h + half_box_h, 
          center_w - half_box_w : center_w + half_box_w] = [255, 0, 0, 255] # Red
    
    img_b = np.zeros_like(img_a)
    img_b[center_h - half_box_h : center_h + half_box_h, 
          center_w - half_box_w : center_w + half_box_w] = [0, 0, 255, 255] # Blue
    return img_a, img_b

def run_benchmark(interp, width, height, time_t=0.5):
    """Generates images, runs interpolation, and measures time."""
    print(f"\n--- Running Benchmark: {width}x{height} ---")
    print("Generating test images...")
    img_a, img_b = generate_test_images(width, height)
    
    print(f"Calling interpolate_py (time_t={time_t}) with bytes...")
    
    start_time = time.time()
    try:
        out_bytes = interp.interpolate_py(
            img_a.tobytes(), 
            img_b.tobytes(), 
            width, 
            height, 
            time_t=time_t
        )
        # Ensure GPU commands are flushed and potentially waited for.
        # Note: The current Rust implementation includes blocking readback, 
        # so this time measurement includes GPU execution + readback.
    except Exception as e:
        print(f"Error during interpolation benchmark: {e}")
        return None, None # Indicate failure
    end_time = time.time()
    duration_ms = (end_time - start_time) * 1000
    print(f"--> interpolate_py took: {duration_ms:.2f} ms")

    # Get GPU-specific timing if available
    try:
        gpu_duration_ms = interp.get_last_gpu_duration_ms()
        if gpu_duration_ms is not None:
            print(f"--> GPU execution took: {gpu_duration_ms:.3f} ms")
        else:
            print("--> GPU execution time not available (feature unsupported or query failed).")
    except Exception as e:
        print(f"Error calling get_last_gpu_duration_ms: {e}")

    print("Rebuilding NumPy array from output bytes...")
    try:
        out_arr = np.frombuffer(out_bytes, dtype=np.uint8).reshape((height, width, 4))
    except Exception as e:
        print(f"Error rebuilding NumPy array from bytes: {e}")
        return None, duration_ms # Return time but indicate array failure
        
    return out_arr, duration_ms

def run_test():
    print("Initializing WgpuFrameInterpolator...")
    try:
        # Initialize with default workgroup preset (Wide32x8)
        interp = WgpuFrameInterpolator()
    except Exception as e:
        print(f"Error initializing WgpuFrameInterpolator: {e}")
        exit(1)

    results = {}

    # 64x64 Benchmark
    out_64, time_64 = run_benchmark(interp, 64, 64)
    if out_64 is not None:
        results["64x64"] = time_64
        # Save the 64x64 output for verification
        print("Saving 64x64 output image interp_64.png...")
        try:
            Image.fromarray(out_64, 'RGBA').save('interp_64.png')
            print("--> Successfully wrote interp_64.png")
        except Exception as e:
            print(f"Error saving 64x64 output image: {e}")
    
    # 720p Benchmark (1280x720)
    _out_720, time_720 = run_benchmark(interp, 1280, 720)
    if time_720 is not None:
       results["1280x720"] = time_720
       # Optional: Save 720p image if needed
       # print("Saving 720p output image interp_720.png...")
       # Image.fromarray(_out_720, 'RGBA').save('interp_720.png')

    # 1080p Benchmark (1920x1080)
    _out_1080, time_1080 = run_benchmark(interp, 1920, 1080)
    if time_1080 is not None:
       results["1920x1080"] = time_1080
       # Optional: Save 1080p image if needed
       # print("Saving 1080p output image interp_1080.png...")
       # Image.fromarray(_out_1080, 'RGBA').save('interp_1080.png')

    print("\n--- Benchmark Summary ---")
    for res, timing in results.items():
        # Note: Need to store/retrieve GPU timings separately if wanting to summarize them here
        print(f"{res}: {timing:.2f} ms (End-to-End)")
    print("-------------------------")

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