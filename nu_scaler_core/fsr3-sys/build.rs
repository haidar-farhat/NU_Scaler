use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only proceed if the specific environment variable is set
    if env::var("NU_SCALER_BUILD_FSR3").is_err() {
        println!("cargo:warning=NU_SCALER_BUILD_FSR3 not set, skipping FSR3 SDK setup.");
        return Ok(()); // Exit early, do nothing
    }

    println!("cargo:rerun-if-changed=wrapper.h");

    // --- FSR3 SDK Path --- 
    // Option 1: Set an environment variable FSR3_SDK_PATH
    let sdk_path_env = env::var("FSR3_SDK_PATH").ok();
    // Option 2: Hardcode the path (replace with your actual path)
    let sdk_path_hardcoded = PathBuf::from("C:/AMD/FSR3_SDK"); // EXAMPLE! Update this!

    let fsr3_sdk_path = match sdk_path_env {
        Some(p) => PathBuf::from(p),
        None => sdk_path_hardcoded,
    };

    if !fsr3_sdk_path.exists() {
        panic!("FSR3 SDK path does not exist: {}. Please set FSR3_SDK_PATH or update build.rs.", fsr3_sdk_path.display());
    }

    // Adjust these paths based on the FSR3 SDK structure
    let fsr3_include_path = fsr3_sdk_path.join("include"); 
    let fsr3_lib_path = fsr3_sdk_path.join("lib/ffx_fsr3_x64"); // EXAMPLE! Verify this path

    if !fsr3_include_path.exists() {
        panic!("FSR3 SDK include path does not exist: {}", fsr3_include_path.display());
    }
    if !fsr3_lib_path.exists() {
        panic!("FSR3 SDK library path does not exist: {}", fsr3_lib_path.display());
    }

    println!("cargo:rustc-link-search=native={}", fsr3_lib_path.display());
    // Determine the correct FSR3 library to link. This could be something like:
    // ffx_fsr3_api_x64.lib or ffx_fsr3_api_dx12_x64.lib / ffx_fsr3_api_vk_x64.lib etc.
    // depending on the graphics API you are using (DX12, Vulkan) with WGPU.
    // For now, assuming a generic name, PLEASE VERIFY.
    println!("cargo:rustc-link-lib=ffx_fsr3_api_x64"); // EXAMPLE! Verify library name.
    
    // WGPU typically uses DX12 on Windows by default. If Vulkan is used, this might need adjustment.
    // The FSR3 SDK has different libraries for DX12 and Vulkan.

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", fsr3_include_path.display()))
        // Add FSR3 specific defines or include paths if necessary
        // e.g., .clang_arg("-DFFX_GCC") or .clang_arg("-DFFX_CPU")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("ffxFsr3.*") // Or more specific FSR3 API functions
        .allowlist_type("FfxFsr3.*")
        .allowlist_var("FFX_FSR3_.*") // Constants
        .generate_comments(true)
        .derive_debug(true)
        .derive_default(true)
        // FSR3 headers might use C++ features that bindgen needs to handle
        .enable_cxx_namespaces()
        .opaque_type("std::.*") // To avoid issues with std types if FSR3 uses C++
        .generate()
        .expect("Unable to generate FSR3 bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("fsr3_bindings.rs"))
        .expect("Couldn't write FSR3 bindings!");

    Ok(())
}