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
#[cfg(fsr3_bindings_generated)]
pub use bindings::*;

// If bindings are NOT generated, provide placeholder types/functions.
#[cfg(not(fsr3_bindings_generated))]
pub mod ffi {
    // This block will be compiled if fsr3_bindings_generated is not set.
    // Add minimal stubs or type aliases that nu_scaler_core might expect to exist
    // when the `fsr3` feature is enabled but bindings couldn't be generated.
    // This prevents compile errors in nu_scaler_core itself.
    // For example:
    // pub type FfxFsr3Context = *mut ::std::os::raw::c_void;
    // pub const FFX_FSR3_STATUS_SUCCESS: u32 = 0;
    // pub unsafe fn ffxFsr3ContextCreate(_context: *mut FfxFsr3Context, _params: *const ::std::os::raw::c_void) -> u32 {
    //     unimplemented!("FSR3 bindings not generated");
    // }
}

// Optionally, you can add a function that nu_scaler_core can call to check if bindings are available.
// pub fn are_bindings_available() -> bool {
//     cfg!(fsr3_bindings_generated)
// }

// You can add helper functions or safer wrappers around the raw FFI bindings here if needed.
