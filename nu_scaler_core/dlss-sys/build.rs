use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=wrapper.h"); // Rerun if wrapper.h changes

    let streamline_sdk_path = PathBuf::from(
        env::var("NVIDIA_STREAMLINE_SDK_PATH")
            .unwrap_or_else(|_| r"C:\\nvideasdk\\bckup\\Streamline".to_string()),
    );

    let sdk_include_path = streamline_sdk_path.join("include");

    if !sdk_include_path.exists() {
        panic!(
            "NVIDIA Streamline SDK include path does not exist: {}. Please set NVIDIA_STREAMLINE_SDK_PATH or update build.rs.",
            sdk_include_path.display()
        );
    }
    
    let lib_path = streamline_sdk_path.join(r"lib\x64");

    if !lib_path.exists() {
        panic!(
            r"NVIDIA Streamline SDK library path does not exist: {}. Please verify Streamline\lib\x64.",
            lib_path.display()
        );
    }

    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-lib=static=sl.interposer");

    // Bindgen related code has been removed as we are using manual FFI definitions.

    Ok(())
}
