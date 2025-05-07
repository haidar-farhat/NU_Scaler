use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=wrapper.h");

    let streamline_sdk_path = PathBuf::from(
        env::var("NVIDIA_STREAMLINE_SDK_PATH")
            .unwrap_or_else(|_| "C:/nvideasdk/bckup/Streamline".to_string()),
    );

    if !streamline_sdk_path.exists() {
        panic!(
            "NVIDIA Streamline SDK path does not exist: {}. Please set NVIDIA_STREAMLINE_SDK_PATH or update build.rs.",
            streamline_sdk_path.display()
        );
    }

    let include_path = streamline_sdk_path.join("include");
    let lib_path = streamline_sdk_path.join("lib/x64"); // User needs to verify this path

    if !include_path.exists() {
        panic!("NVIDIA Streamline SDK include path does not exist: {}", include_path.display());
    }
    if !lib_path.exists() {
        panic!(
            "NVIDIA Streamline SDK library path does not exist: {}. Please verify e.g., Streamline/lib/x64 or Streamline/lib/x64/Release",
            lib_path.display()
        );
    }

    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-lib=sl.interposer"); // User needs to verify this library name

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_path.display()))
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++17") // Or newer if Streamline requires
        // --- MSVC Integration Args ---
        .clang_arg("-fms-compatibility") // Enable MSVC compatibility
        .clang_arg("-fms-extensions")    // Allow MSVC-specific extensions
        // Try to force targeting the msvc ABI
        .clang_arg("--target=x86_64-pc-windows-msvc");
        
    // Attempt to add MSVC include paths. This is highly environment-dependent.
    // You might need to find these paths from your Visual Studio installation.
    // These are common locations for VS 2022 Community. ADJUST AS NEEDED.
    if let Ok(vc_tools_dir) = env::var("VCToolsInstallDir") {
        let msvc_include_path = PathBuf::from(vc_tools_dir).join("include");
        if msvc_include_path.exists() {
            builder = builder.clang_arg(format!("-I{}", msvc_include_path.display()));
            println!("cargo:warning=Added MSVC include path: {}", msvc_include_path.display());
        }
    }
    // Potentially add Windows Kits includes if problems persist (e.g. for <tuple>)
    // if let Ok(sdk_dir) = env::var("WindowsSdkDir") { ... sdk_dir.join("Include/<version>/ucrt") ... }

    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("sl[A-Z].*")
        .allowlist_type("sl[A-Z].*")
        .allowlist_var("sl[A-Z_].*")
        .allowlist_type("SL_.*")
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