use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=wrapper.h"); // Rerun if wrapper.h changes

    let streamline_sdk_root_path = PathBuf::from(
        env::var("NVIDIA_STREAMLINE_SDK_PATH")
            .unwrap_or_else(|_| r"C:\\nvideasdk\\Streamline".to_string()),
    );

    // Path for sl.interposer.lib
    let interposer_lib_path = streamline_sdk_root_path.join(r"lib\x64");
    if !interposer_lib_path.exists() {
        panic!(
            r"NVIDIA Streamline SDK interposer library path does not exist: {}. Please verify Streamline\lib\x64.",
            interposer_lib_path.display()
        );
    }
    println!("cargo:rustc-link-search=native={}", interposer_lib_path.display());
    println!("cargo:rustc-link-lib=static=sl.interposer");

    // Path for ngx (DLSS) libraries
    let ngx_lib_path = streamline_sdk_root_path.join(r"external\ngx-sdk\lib\Windows_x86_64");
    if !ngx_lib_path.exists() {
        panic!(
            r"NVIDIA NGX SDK library path does not exist: {}. Please verify Streamline\external\ngx-sdk\lib\Windows_x86_64.",
            ngx_lib_path.display()
        );
    }
    println!("cargo:rustc-link-search=native={}", ngx_lib_path.display());
    println!("cargo:rustc-link-lib=static=nvsdk_ngx_d"); // Link the release NGX library

    // Include path (assuming it's still relevant from the root)
    let sdk_include_path = streamline_sdk_root_path.join("include");
    if !sdk_include_path.exists() {
        panic!(
            "NVIDIA Streamline SDK include path does not exist: {}. Please set NVIDIA_STREAMLINE_SDK_PATH or update build.rs.",
            sdk_include_path.display()
        );
    }
    // If bindgen were used, you'd add: println!("cargo:include={}", sdk_include_path.display());

    Ok(())
}
