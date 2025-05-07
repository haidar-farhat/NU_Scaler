use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/lib.rs"); // Rerun if manual bindings change
                                                   // Potentially rerun if you copy SDK headers into the project for reference
                                                   // println!("cargo:rerun-if-changed=include/sl_dlss.h");

    let streamline_sdk_path = PathBuf::from(
        env::var("NVIDIA_STREAMLINE_SDK_PATH")
            .unwrap_or_else(|_| "C:\\nvideasdk\\bckup\\Streamline".to_string()),
    );

    if !streamline_sdk_path.exists() {
        panic!(
            "NVIDIA Streamline SDK path does not exist: {}. Please set NVIDIA_STREAMLINE_SDK_PATH or update build.rs.",
            streamline_sdk_path.display()
        );
    }

    let lib_path = streamline_sdk_path.join("lib\\x64");

    if !lib_path.exists() {
        panic!(
            "NVIDIA Streamline SDK library path does not exist: {}. Please verify e.g., Streamline\\lib\\x64 or Streamline\\lib\\x64\\Release",
            lib_path.display()
        );
    }

    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-lib=sl.interposer"); // User needs to verify this library name

    Ok(())
}
