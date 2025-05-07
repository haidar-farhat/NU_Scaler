use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Read environment variables
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    
    // Look for FSR 3.0 SDK in several standard locations
    let fsr3_sdk_path = find_fsr3_sdk()?;
    println!("cargo:rustc-link-search=native={}", fsr3_sdk_path.display());
    
    // Link against FSR 3.0 SDK
    println!("cargo:rustc-link-lib=dylib=ffx_fsr3");
    
    // Make sure our output responds to changes in headers
    println!("cargo:rerun-if-changed=wrapper.h");
    
    // The bindgen::Builder is the main entry point to bindgen
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", fsr3_sdk_path.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .map_err(|_| anyhow!("Unable to generate bindings to FSR 3.0 SDK"))?;

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .map_err(|_| anyhow!("Could not write FSR 3.0 bindings"))?;

    Ok(())
}

fn find_fsr3_sdk() -> Result<PathBuf> {
    // 1. Try environment variable
    if let Ok(path) = env::var("FSR3_SDK_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
    }

    // 2. Try standard locations
    let standard_paths = [
        // Windows standard paths
        r"C:\Program Files\AMD\FSR3",
        r"C:\AMD\FSR3 SDK",
        r"C:\AMD_FSR3_SDK",
        // Project-relative paths
        "./vendor/FSR3",
        "../vendor/FSR3",
        "../../vendor/FSR3",
    ];

    for path_str in standard_paths.iter() {
        let path = PathBuf::from(path_str);
        if path.exists() {
            return Ok(path);
        }
    }

    // If not found, provide instructions
    println!("cargo:warning=AMD FSR 3.0 SDK not found. Using stub implementation.");
    println!("cargo:warning=To use the real FSR 3.0 SDK:");
    println!("cargo:warning=1. Download it from AMD developer site");
    println!("cargo:warning=2. Set FSR3_SDK_PATH environment variable");
    
    // Use internal stub headers as fallback
    Ok(PathBuf::from("./stub"))
}