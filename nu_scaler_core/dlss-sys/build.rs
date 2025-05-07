use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=wrapper.h"); // Rerun if wrapper.h changes

    let streamline_sdk_path = PathBuf::from(
        env::var("NVIDIA_STREAMLINE_SDK_PATH")
            .unwrap_or_else(|_| r"C:\nvideasdk\bckup\Streamline".to_string()),
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
    println!("cargo:rustc-link-lib=sl.interposer");

    // Generate bindings using bindgen
    // The actual sl.h is included by wrapper.h
    let bindings = bindgen::Builder::default()
        .header("wrapper.h") // Processes nu_scaler_core/dlss-sys/wrapper.h
        // Tell bindgen where to find headers included by wrapper.h (like sl.h)
        .clang_arg(format!("-I{}", sdk_include_path.display()))
        // Add other necessary clang args if needed, e.g., for defines:
        // .clang_arg("-DSL_DX12_ENABLED=1") // Check sl_config.h for relevant defines
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings for Streamline SDK");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    // This is typically included in src/lib.rs as: include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs")) // Standard name often "bindings.rs"
        .expect("Couldn't write Streamline bindings!");

    Ok(())
}
