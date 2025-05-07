// This allows the generated bindings to be quite large
#![allow(clippy::all)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

#[cfg(fsr3_bindings_generated)]
mod bindings {
    // This will only be included if build.rs generated the bindings and set the cfg flag.
    include!(concat!(env!("OUT_DIR"), "/fsr3_bindings.rs"));
}

// Publicly re-export items from the bindings module if they were generated.
// If not, these will be empty or minimal stubs.
#[cfg(fsr3_bindings_generated)]
pub use bindings::*;

// If bindings are NOT generated, provide placeholder types/functions 
// if other parts of the codebase try to use them when the fsr3 feature is on 
// but bindings failed (e.g. SDK not found).
// This helps avoid further compilation errors in nu_scaler_core if it tries to use fsr3_sys types.
#[cfg(not(fsr3_bindings_generated))]
pub mod ffi {
    // Example placeholder, expand as needed based on what nu_scaler_core might try to use.
    // pub type FfxFsr3Context = *mut std::ffi::c_void;
    // pub const FFX_FSR3_STATUS_SUCCESS: u32 = 0;
    // pub unsafe fn ffxFsr3ContextCreate(_context: *mut FfxFsr3Context, _params: *const std::ffi::c_void) -> u32 {
    //     FFX_FSR3_STATUS_SUCCESS 
    // }
    println!("Warning: FSR3 bindings were not generated. FSR3 functionality will be unavailable.");
}

// You can add helper functions or safer wrappers around the raw FFI bindings here if needed. 