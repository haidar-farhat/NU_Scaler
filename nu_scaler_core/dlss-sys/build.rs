use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=wrapper.h");

    // --- DLSS SDK Path --- 
    // Option 1: Set an environment variable DLSS_SDK_PATH
    let sdk_path_env = env::var("DLSS_SDK_PATH").ok();
    // Option 2: Hardcode the path (replace with your actual path)
    let sdk_path_hardcoded = PathBuf::from("C:/NVIDIA/DLSS_SDK"); // EXAMPLE! Update this!

    let dlss_sdk_path = match sdk_path_env {
        Some(p) => PathBuf::from(p),
        None => sdk_path_hardcoded,
    };

    if !dlss_sdk_path.exists() {
        panic!("DLSS SDK path does not exist: {}. Please set DLSS_SDK_PATH or update build.rs.", dlss_sdk_path.display());
    }

    let dlss_include_path = dlss_sdk_path.join("include");
    let dlss_lib_path = dlss_sdk_path.join("lib/x64"); // Assuming 64-bit

    if !dlss_include_path.exists() {
        panic!("DLSS SDK include path does not exist: {}", dlss_include_path.display());
    }
    if !dlss_lib_path.exists() {
        panic!("DLSS SDK library path does not exist: {}", dlss_lib_path.display());
    }

    println!("cargo:rustc-link-search=native={}", dlss_lib_path.display());
    // Determine the correct library to link against. This might vary based on the DLSS version 
    // and whether you're linking the debug or release version of the DLL.
    // Common names are nvngx_dlss.lib or similar.
    println!("cargo:rustc-link-lib=nvngx_dlss"); // EXAMPLE! Verify library name.

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // Tell bindgen the include path for nvsdk_ngx*.h files
        .clang_arg(format!("-I{}", dlss_include_path.display()))
        // Add any other necessary clang args
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("NVSDK_NGX_.*") // Adjust based on what you need
        .allowlist_type("NVSDK_NGX_.*")
        .allowlist_var("NVSDK_NGX_.*")
        .generate_comments(true)
        .derive_debug(true)
        .derive_default(true)
        .generate()
        .expect("Unable to generate DLSS bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("dlss_bindings.rs"))
        .expect("Couldn't write DLSS bindings!");

    Ok(())
} 