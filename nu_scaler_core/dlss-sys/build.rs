use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=wrapper.h");

    // --- NVIDIA Streamline SDK Path --- 
    let sdk_path_env = env::var("NVIDIA_STREAMLINE_SDK_PATH").ok();
    // User provided path:
    let sdk_path_hardcoded = PathBuf::from("C:/nvideasdk/bckup/Streamline");

    let streamline_sdk_path = match sdk_path_env {
        Some(p) => PathBuf::from(p),
        None => sdk_path_hardcoded, // Use the path you provided
    };

    if !streamline_sdk_path.exists() {
        panic!("NVIDIA Streamline SDK path does not exist: {}. Please set NVIDIA_STREAMLINE_SDK_PATH or ensure the hardcoded path is correct.", streamline_sdk_path.display());
    }

    let include_path = streamline_sdk_path.join("include");
    // IMPORTANT: Verify this subpath. It might be lib/x64/Release or just lib/x64
    let lib_path = streamline_sdk_path.join("lib/x64"); 

    if !include_path.exists() {
        panic!("NVIDIA Streamline SDK include path does not exist: {}", include_path.display());
    }
    if !lib_path.exists() {
        panic!("NVIDIA Streamline SDK library path does not exist: {}. Please verify the exact path to the .lib files (e.g., Streamline/lib/x64 or Streamline/lib/x64/Release)", lib_path.display());
    }

    println!("cargo:rustc-link-search=native={}", lib_path.display());
    // IMPORTANT: Verify this library name. For Streamline, it's often sl.interposer.lib
    // or you might need to link sl.dlss.lib directly if not using the full interposer.
    println!("cargo:rustc-link-lib=sl.interposer"); // EXAMPLE! Verify library name (without .lib extension)

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_path.display()))
        // Streamline headers are typically C++, so we might need to enable C++ support
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++17") // Or the version Streamline uses
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Allowlisting for Streamline (sl.* functions/types)
        .allowlist_function("sl[A-Z].*") 
        .allowlist_type("sl[A-Z].*")
        .allowlist_var("sl[A-Z_].*")
        .allowlist_type("SL_.*") // For SL_ enums and structs if any
        .allowlist_var("SL_.*")
        .generate_comments(true)
        .derive_debug(true)
        .derive_default(true)
        .enable_cxx_namespaces()
        .opaque_type("std::.*")
        .generate()
        .expect("Unable to generate Streamline/DLSS bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("dlss_bindings.rs"))
        .expect("Couldn't write Streamline/DLSS bindings!");

    Ok(())
} 