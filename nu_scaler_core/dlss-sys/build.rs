use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/lib.rs");
    // No longer running bindgen, so wrapper.h isn't needed for rerun trigger
    // println!("cargo:rerun-if-changed=wrapper.h");

    // --- IMPORTANT: Set this path to your OFFICIAL Streamline SDK installation ---
    let streamline_sdk_root_path = PathBuf::from(
        env::var("NVIDIA_STREAMLINE_SDK_PATH")
            .unwrap_or_else(|_| r"C:\nvideasdk\bckup\Streamline".to_string()), // <-- UPDATE THIS DEFAULT PATH
    );

    // Path and link ONLY for sl.interposer.lib (assuming it's needed for loading)
    let interposer_lib_path = streamline_sdk_root_path.join(r"lib\x64");
    if !interposer_lib_path.exists() {
        panic!(
            r"NVIDIA Streamline SDK library path does not exist: {}. Please verify the path and that sl.interposer.lib is present.",
            interposer_lib_path.display()
        );
    }
    println!("cargo:rustc-link-search=native={}", interposer_lib_path.display());
    println!("cargo:rustc-link-lib=static=sl.interposer");

    // We are NOT linking sl.common, sl.dlss, nvsdk_ngx_d etc. directly anymore,
    // as they don't export the public API. We will load functions at runtime.

    // Check for include path, might still be needed for C header consistency checks later if desired
    let sdk_include_path = streamline_sdk_root_path.join("include");
    if !sdk_include_path.exists() {
        panic!(
            r"NVIDIA Streamline SDK include path does not exist: {}. Please verify the include path.",
            sdk_include_path.display()
        );
    }

    println!("cargo:warning=Streamline build script configured for dynamic loading via sl.interposer.lib. Ensure NVIDIA_STREAMLINE_SDK_PATH is set or the default path is correct.");

    Ok(())
}
