use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=wrapper.h"); // Rerun if wrapper.h changes

    let streamline_sdk_root_path = PathBuf::from(
        env::var("NVIDIA_STREAMLINE_SDK_PATH")
            .unwrap_or_else(|_| r"C:\\nvideasdk\\bckup\\Streamline".to_string()),
    );

    // Path and link for sl.interposer.lib
    let interposer_lib_path = streamline_sdk_root_path.join(r"lib\x64");
    if !interposer_lib_path.exists() {
        panic!(
            r"NVIDIA Streamline SDK interposer library path does not exist: {}. Please verify Streamline\lib\x64.",
            interposer_lib_path.display()
        );
    }
    println!("cargo:rustc-link-search=native={}", interposer_lib_path.display());
    println!("cargo:rustc-link-lib=static=sl.interposer");

    // Path and link for sl.common.lib
    let common_lib_artifacts_path = streamline_sdk_root_path.join(r"_artifacts\sl.common\Production_x64");
    if !common_lib_artifacts_path.exists() {
        panic!(
            r"NVIDIA Streamline SDK sl.common library path does not exist: {}. Please verify _artifacts\sl.common\Production_x64.",
            common_lib_artifacts_path.display()
        );
    }
    println!("cargo:rustc-link-search=native={}", common_lib_artifacts_path.display());
    println!("cargo:rustc-link-lib=static=sl.common");

    // Path and link for sl.dlss.lib
    let dlss_lib_artifacts_path = streamline_sdk_root_path.join(r"_artifacts\sl.dlss\Production_x64");
    if !dlss_lib_artifacts_path.exists() {
        panic!(
            r"NVIDIA Streamline SDK sl.dlss library path does not exist: {}. Please verify _artifacts\sl.dlss\Production_x64.",
            dlss_lib_artifacts_path.display()
        );
    }
    println!("cargo:rustc-link-search=native={}", dlss_lib_artifacts_path.display());
    println!("cargo:rustc-link-lib=static=sl.dlss");

    // Include path (assuming it's still relevant from the root of "bckup\Streamline")
    let sdk_include_path = streamline_sdk_root_path.join("include");
    if !sdk_include_path.exists() {
        panic!(
            "NVIDIA Streamline SDK include path does not exist: {}. Please verify Streamline\include.",
            sdk_include_path.display()
        );
    }
    // If bindgen were used, you'd add: println!("cargo:include={}", sdk_include_path.display());

    Ok(())
}
