"""
Nu_Scaler benchmarking module for testing upscaler performance.
"""

import time
from typing import Dict, List, Optional, Tuple, Union
import matplotlib.pyplot as plt
import numpy as np

try:
    import nu_scaler_core
except ImportError:
    print("WARNING: nu_scaler_core module not found. Benchmarking will not be available.")
    nu_scaler_core = None

class BenchmarkResult:
    """Wrapper class for benchmark results with visualization methods"""
    
    def __init__(self, py_result=None):
        """Initialize from a PyBenchmarkResult or manually set attributes"""
        if py_result:
            self.upscaler_name = py_result.upscaler_name
            self.technology = py_result.technology
            self.quality = py_result.quality
            self.input_width = py_result.input_width
            self.input_height = py_result.input_height
            self.output_width = py_result.output_width 
            self.output_height = py_result.output_height
            self.scale_factor = py_result.scale_factor
            self.avg_frame_time_ms = py_result.avg_frame_time_ms
            self.fps = py_result.fps
            self.frames_processed = py_result.frames_processed
            self.total_duration_ms = py_result.total_duration_ms
        else:
            self.upscaler_name = ""
            self.technology = ""
            self.quality = ""
            self.input_width = 0
            self.input_height = 0
            self.output_width = 0
            self.output_height = 0
            self.scale_factor = 0.0
            self.avg_frame_time_ms = 0.0
            self.fps = 0.0
            self.frames_processed = 0
            self.total_duration_ms = 0.0
    
    def __str__(self):
        """String representation of benchmark results"""
        return (
            f"Benchmark: {self.upscaler_name} ({self.technology}, {self.quality})\n"
            f"Resolution: {self.input_width}x{self.input_height} → "
            f"{self.output_width}x{self.output_height} ({self.scale_factor}x)\n"
            f"Performance: {self.fps:.2f} FPS ({self.avg_frame_time_ms:.2f} ms/frame)\n"
            f"Total: {self.frames_processed} frames in {self.total_duration_ms/1000:.2f} seconds"
        )

def run_benchmark(
    technology: str = "auto",
    quality: str = "balanced",
    input_width: int = 1920,
    input_height: int = 1080,
    scale_factor: float = 1.5,
    frame_count: int = 100
) -> Optional[BenchmarkResult]:
    """
    Run a benchmark for a specific upscaler configuration.
    
    Args:
        technology: Upscaling technology ("auto", "fsr", "dlss", "wgpu", "fallback")
        quality: Quality preset ("ultra", "quality", "balanced", "performance")
        input_width: Input frame width in pixels
        input_height: Input frame height in pixels
        scale_factor: Scale factor (1.0-4.0)
        frame_count: Number of frames to process
        
    Returns:
        BenchmarkResult object or None if benchmarking failed
    """
    if nu_scaler_core is None:
        print("ERROR: nu_scaler_core module not available. Cannot run benchmark.")
        return None
    
    try:
        # Map "auto" to detection logic
        if technology.lower() == "auto":
            # Create GPU detector to find best technology
            upscaler = nu_scaler_core.create_best_upscaler(quality)
            # Run benchmark with the auto-detected technology
            tech_name = "auto"
        else:
            tech_name = technology.lower()
        
        # Run the benchmark
        result = nu_scaler_core.py_benchmark_upscaler(
            tech_name, quality, input_width, input_height, scale_factor, frame_count
        )
        
        # Create our wrapper around the result
        return BenchmarkResult(result)
    
    except Exception as e:
        print(f"Benchmark error: {e}")
        return None

def run_comparison_benchmark(
    input_width: int = 1920,
    input_height: int = 1080,
    scale_factor: float = 1.5,
    frame_count: int = 50
) -> List[BenchmarkResult]:
    """
    Run a comprehensive benchmark comparing all available upscaling technologies.
    
    Args:
        input_width: Input frame width in pixels
        input_height: Input frame height in pixels
        scale_factor: Scale factor (1.0-4.0)
        frame_count: Number of frames to process per configuration
        
    Returns:
        List of BenchmarkResult objects
    """
    if nu_scaler_core is None:
        print("ERROR: nu_scaler_core module not available. Cannot run benchmark.")
        return []
    
    try:
        # Run comparison benchmark in Rust
        results = nu_scaler_core.py_run_comparison_benchmark(
            input_width, input_height, scale_factor, frame_count
        )
        
        # Convert to our wrapper objects
        return [BenchmarkResult(r) for r in results]
    
    except Exception as e:
        print(f"Comparison benchmark error: {e}")
        return []

def plot_benchmark_results(results: List[BenchmarkResult], title: str = "Upscaler Performance Comparison"):
    """
    Create a bar chart comparing FPS of different upscalers.
    
    Args:
        results: List of BenchmarkResult objects
        title: Chart title
    """
    if not results:
        print("No benchmark results to plot")
        return
    
    try:
        import matplotlib.pyplot as plt
        import numpy as np
    except ImportError:
        print("Matplotlib not installed. Cannot plot results.")
        return
    
    # Group by technology and quality
    tech_quality_fps = {}
    for result in results:
        key = f"{result.technology}\n{result.quality}"
        tech_quality_fps[key] = result.fps
    
    # Sort by FPS (descending)
    sorted_items = sorted(tech_quality_fps.items(), key=lambda x: x[1], reverse=True)
    labels = [item[0] for item in sorted_items]
    values = [item[1] for item in sorted_items]
    
    # Plot
    plt.figure(figsize=(12, 8))
    
    # For colorful bars based on FPS value
    colors = plt.cm.viridis(np.linspace(0, 1, len(values)))
    
    bars = plt.bar(labels, values, color=colors)
    plt.xlabel('Technology & Quality')
    plt.ylabel('FPS')
    plt.title(title)
    plt.xticks(rotation=45, ha='right')
    
    # Add FPS values on top of bars
    for bar in bars:
        height = bar.get_height()
        plt.text(bar.get_x() + bar.get_width()/2., height + 0.5,
                f'{height:.1f}',
                ha='center', va='bottom', rotation=0)
    
    plt.tight_layout()
    plt.show()

def plot_quality_comparison(results: List[BenchmarkResult], title: str = "Quality vs Performance"):
    """
    Create a grouped bar chart comparing performance across quality settings.
    
    Args:
        results: List of BenchmarkResult objects
        title: Chart title
    """
    if not results:
        print("No benchmark results to plot")
        return
    
    try:
        import matplotlib.pyplot as plt
        import numpy as np
    except ImportError:
        print("Matplotlib not installed. Cannot plot results.")
        return
    
    # Group results by technology and quality
    tech_quality_map = {}
    for result in results:
        tech = result.technology
        quality = result.quality
        if tech not in tech_quality_map:
            tech_quality_map[tech] = {}
        tech_quality_map[tech][quality] = result.fps
    
    # Set up the plot
    fig, ax = plt.subplots(figsize=(12, 8))
    
    # Define width of bars and positions
    width = 0.2
    qualities = ["Ultra", "Quality", "Balanced", "Performance"]
    x = np.arange(len(tech_quality_map))
    
    # Plot bars for each quality
    for i, quality in enumerate(qualities):
        values = []
        for tech in tech_quality_map:
            values.append(tech_quality_map[tech].get(quality, 0))
        
        ax.bar(x + i*width - (3*width/2), values, width, label=quality)
    
    # Set labels and title
    ax.set_xlabel('Technology')
    ax.set_ylabel('FPS')
    ax.set_title(title)
    ax.set_xticks(x)
    ax.set_xticklabels(list(tech_quality_map.keys()))
    ax.legend()
    
    plt.tight_layout()
    plt.show()

def save_benchmark_results(results: List[BenchmarkResult], filename: str):
    """
    Save benchmark results to a CSV file.
    
    Args:
        results: List of BenchmarkResult objects
        filename: Output filename
    """
    try:
        import csv
    except ImportError:
        print("CSV module not available. Cannot save results.")
        return
    
    try:
        with open(filename, 'w', newline='') as f:
            writer = csv.writer(f)
            # Write header
            writer.writerow([
                'Upscaler', 'Technology', 'Quality', 
                'Input Resolution', 'Output Resolution', 'Scale Factor',
                'FPS', 'Avg Frame Time (ms)', 'Frames Processed', 'Duration (s)'
            ])
            
            # Write data
            for result in results:
                writer.writerow([
                    result.upscaler_name,
                    result.technology,
                    result.quality,
                    f"{result.input_width}x{result.input_height}",
                    f"{result.output_width}x{result.output_height}",
                    result.scale_factor,
                    result.fps,
                    result.avg_frame_time_ms,
                    result.frames_processed,
                    result.total_duration_ms / 1000
                ])
        
        print(f"Benchmark results saved to {filename}")
    
    except Exception as e:
        print(f"Error saving benchmark results: {e}")

if __name__ == "__main__":
    # Example usage
    print("Running upscaler benchmark...")
    
    # Single benchmark
    result = run_benchmark(
        technology="auto",
        quality="balanced",
        input_width=1920,
        input_height=1080,
        scale_factor=1.5,
        frame_count=100
    )
    
    if result:
        print("\nBenchmark result:\n")
        print(result)
    
    # Run comparison benchmark (commented out as it can take some time)
    # results = run_comparison_benchmark(
    #     input_width=1280,
    #     input_height=720,
    #     scale_factor=1.5,
    #     frame_count=50
    # )
    # 
    # if results:
    #     print(f"\nCompleted {len(results)} benchmark configurations\n")
    #     
    #     # Plot results
    #     plot_benchmark_results(results)
    #     plot_quality_comparison(results)
    #     
    #     # Save results
    #     save_benchmark_results(results, "benchmark_results.csv") 