#!/usr/bin/env python3
"""
Nu_Scaler Benchmark CLI Application

This script provides a command-line interface for running benchmarks on
different upscaling technologies and comparing their performance.
"""

import argparse
import os
import sys
import time
from pathlib import Path

# Add parent directory to path if running directly
if __name__ == "__main__":
    sys.path.append(str(Path(__file__).parent.parent))

from nu_scaler.benchmark import (
    run_benchmark,
    run_comparison_benchmark,
    plot_benchmark_results,
    plot_quality_comparison,
    save_benchmark_results
)

def parse_args():
    """Parse command line arguments"""
    parser = argparse.ArgumentParser(description="Nu_Scaler Benchmark Tool")
    
    # Main operation mode
    parser.add_argument(
        "--mode", "-m",
        choices=["single", "compare", "batch"],
        default="single",
        help="Benchmark mode: single technology, comparison, or batch test"
    )
    
    # Technology selection
    parser.add_argument(
        "--technology", "-t",
        choices=["auto", "fsr", "dlss", "wgpu", "fallback"],
        default="auto",
        help="Upscaling technology to benchmark (for single mode)"
    )
    
    # Quality setting
    parser.add_argument(
        "--quality", "-q",
        choices=["ultra", "quality", "balanced", "performance"],
        default="balanced",
        help="Quality preset to use"
    )
    
    # Input resolution
    parser.add_argument(
        "--input-resolution", "-i",
        default="1920x1080",
        help="Input resolution in WIDTHxHEIGHT format"
    )
    
    # Scale factor
    parser.add_argument(
        "--scale-factor", "-s",
        type=float,
        default=1.5,
        help="Scale factor (1.0-4.0)"
    )
    
    # Frame count
    parser.add_argument(
        "--frames", "-f",
        type=int,
        default=100,
        help="Number of frames to process"
    )
    
    # Output options
    parser.add_argument(
        "--plot", "-p",
        action="store_true",
        help="Show plots of benchmark results"
    )
    
    parser.add_argument(
        "--save", "-o",
        help="Save results to CSV file (specify filename)"
    )
    
    return parser.parse_args()

def main():
    """Main benchmark application entry point"""
    args = parse_args()
    
    # Parse input resolution
    try:
        width_str, height_str = args.input_resolution.split("x")
        input_width = int(width_str)
        input_height = int(height_str)
    except ValueError:
        print(f"Invalid input resolution format: {args.input_resolution}")
        print("Please use format WIDTHxHEIGHT (e.g., 1920x1080)")
        return 1
    
    # Check scale factor
    if args.scale_factor < 1.0 or args.scale_factor > 4.0:
        print(f"Invalid scale factor: {args.scale_factor}")
        print("Scale factor must be between 1.0 and 4.0")
        return 1
    
    # Header
    print("=" * 50)
    print(f"Nu_Scaler Benchmark Tool")
    print("=" * 50)
    print(f"Mode: {args.mode}")
    print(f"Input Resolution: {input_width}x{input_height}")
    print(f"Scale Factor: {args.scale_factor}")
    print(f"Frames: {args.frames}")
    if args.mode == "single":
        print(f"Technology: {args.technology}")
        print(f"Quality: {args.quality}")
    print("=" * 50)
    print("Starting benchmark...")
    print()
    
    # Run appropriate benchmark mode
    start_time = time.time()
    
    if args.mode == "single":
        # Run single benchmark
        result = run_benchmark(
            technology=args.technology,
            quality=args.quality,
            input_width=input_width,
            input_height=input_height,
            scale_factor=args.scale_factor,
            frame_count=args.frames
        )
        
        if result:
            print("\nBenchmark results:\n")
            print(result)
            
            # Save results if requested
            if args.save:
                save_benchmark_results([result], args.save)
        else:
            print("Benchmark failed.")
            return 1
    
    elif args.mode == "compare":
        # Run comparison benchmark
        print("Running technology and quality comparison...")
        print("This will test all combinations and may take some time...")
        
        results = run_comparison_benchmark(
            input_width=input_width,
            input_height=input_height,
            scale_factor=args.scale_factor,
            frame_count=args.frames
        )
        
        if results:
            print(f"\nCompleted {len(results)} benchmark configurations\n")
            
            # Sort results by FPS
            results.sort(key=lambda r: r.fps, reverse=True)
            
            # Print top 3 results
            print("Top performing configurations:")
            for i, result in enumerate(results[:3], 1):
                print(f"\n{i}. {result.technology} ({result.quality})")
                print(f"   {result.fps:.2f} FPS ({result.avg_frame_time_ms:.2f} ms/frame)")
            
            # Show plots if requested
            if args.plot:
                plot_benchmark_results(results)
                plot_quality_comparison(results)
            
            # Save results if requested
            if args.save:
                save_benchmark_results(results, args.save)
        else:
            print("Comparison benchmark failed.")
            return 1
    
    elif args.mode == "batch":
        # Run batch test with different resolutions and scale factors
        resolutions = [
            (1280, 720),    # 720p
            (1920, 1080),   # 1080p
            (2560, 1440),   # 1440p
            (3840, 2160),   # 4K
        ]
        
        scale_factors = [1.5, 2.0, 3.0]
        
        all_results = []
        
        print("Running batch tests across resolutions and scale factors...")
        
        for width, height in resolutions:
            for scale in scale_factors:
                print(f"\nTesting {width}x{height} @ {scale}x scale...")
                
                result = run_benchmark(
                    technology=args.technology,
                    quality=args.quality,
                    input_width=width,
                    input_height=height,
                    scale_factor=scale,
                    frame_count=args.frames
                )
                
                if result:
                    print(f"Result: {result.fps:.2f} FPS")
                    all_results.append(result)
                else:
                    print(f"Test for {width}x{height} @ {scale}x failed.")
        
        if all_results:
            print("\nBatch test complete. Results summary:")
            
            # Group results by resolution
            by_resolution = {}
            for result in all_results:
                res_key = f"{result.input_width}x{result.input_height}"
                if res_key not in by_resolution:
                    by_resolution[res_key] = []
                by_resolution[res_key].append(result)
            
            # Print summary
            for res, res_results in by_resolution.items():
                print(f"\nResolution: {res}")
                for result in res_results:
                    print(f"  Scale {result.scale_factor}x: {result.fps:.2f} FPS")
            
            # Save results if requested
            if args.save:
                save_benchmark_results(all_results, args.save)
        else:
            print("All batch tests failed.")
            return 1
    
    else:
        print(f"Unknown mode: {args.mode}")
        return 1
    
    # Print total time
    total_time = time.time() - start_time
    print(f"\nTotal benchmark time: {total_time:.2f} seconds")
    
    return 0

if __name__ == "__main__":
    sys.exit(main()) 