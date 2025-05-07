use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Read environment variables
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    
    // Look for DLSS SDK in several standard locations
    let dlss_sdk_path = find_dlss_sdk()?;
    println!("cargo:rustc-link-search=native={}", dlss_sdk_path.display());
    
    // Link against DLSS SDK
    println!("cargo:rustc-link-lib=dylib=nvsdk_ngx");
    
    // Make sure our output responds to changes in headers
    println!("cargo:rerun-if-changed=wrapper.h");
    
    // The bindgen::Builder is the main entry point to bindgen
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", dlss_sdk_path.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .map_err(|_| anyhow!("Unable to generate bindings to DLSS SDK"))?;

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .map_err(|_| anyhow!("Could not write DLSS bindings"))?;

    Ok(())
}

fn find_dlss_sdk() -> Result<PathBuf> {
    // 1. Try environment variable
    if let Ok(path) = env::var("DLSS_SDK_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
    }

    // 2. Try standard locations
    let standard_paths = [
        // Windows standard paths
        r"C:\Program Files\NVIDIA\DLSS",
        r"C:\NVIDIA\DLSS SDK",
        r"C:\NVIDIA_DLSS_SDK",
        // Project-relative paths
        "./vendor/DLSS",
        "../vendor/DLSS",
        "../../vendor/DLSS",
    ];

    for path_str in standard_paths.iter() {
        let path = PathBuf::from(path_str);
        if path.exists() {
            return Ok(path);
        }
    }

    // If not found, provide instructions
    println!("cargo:warning=NVIDIA DLSS SDK not found. Using stub implementation.");
    println!("cargo:warning=To use the real DLSS SDK:");
    println!("cargo:warning=1. Download it from NVIDIA Developer site");
    println!("cargo:warning=2. Set DLSS_SDK_PATH environment variable");
    
    // Use internal stub headers as fallback
    Ok(PathBuf::from("./stub"))
} 