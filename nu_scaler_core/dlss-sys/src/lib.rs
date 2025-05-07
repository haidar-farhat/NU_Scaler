#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)] // To silence warnings from unused generated bindings

// The line including bindgen-generated bindings has been removed.
// Manual FFI definitions should be directly in this file or its modules.

// You can add `pub use` statements here if you want to re-export specific items
// from the generated bindings for easier access by other crates.
// For example:
// pub use self::SlStatus; // Assuming SlStatus is generated in bindings.rs
// pub use self::slInitializeSDK; // Assuming slInitializeSDK is generated

// Any custom Rust helper functions or structs that operate on these FFI types
// can also be defined here if needed.

use std::ffi::c_void;
use std::fmt;
use std::sync::OnceLock;

use libloading::{Library, Symbol};

// Based on sl_consts.h, sl_dlss.h, and common patterns

pub type SlBool = u32;
pub const SL_TRUE: SlBool = 1;
pub const SL_FALSE: SlBool = 0;

pub type SlFeature = i32;
pub const SL_FEATURE_DLSS: SlFeature = 2; // Example value, confirm from sl.h or sl_consts.h

pub type SlViewportHandle = u32; // Placeholder, often an opaque pointer or handle

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SlStatus {
    Success = 0,
    ErrorNotInitialized = -1, // Example, confirm actual values
    ErrorNotSupported = -2,
    ErrorInvalidParameter = -3,
    ErrorMissingParameter = -4,
    ErrorFeatureNotSupported = -5,
    ErrorInternal = -6,
    ErrorDeviceRemoved = -7,
    ErrorResourceAllocationFailed = -8,
    ErrorVulkan = -100, // Generic, specific errors might exist
    ErrorDx11 = -200,
    ErrorDx12 = -300,
    // ... other error codes from sl.h or Streamline documentation
    ErrorLibraryLoadFailed = -1000, // Custom error for loading issues
    ErrorFunctionLoadFailed = -1001, // Custom error for symbol loading issues
}

impl SlStatus {
    pub fn is_ok(self) -> bool {
        self == SlStatus::Success
    }
    pub fn is_err(self) -> bool {
        self != SlStatus::Success
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SlExtent {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SlFloat2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SlFloat4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SlFloat4x4 {
    pub m: [[f32; 4]; 4],
}

// --- DLSS Specific Types ---
pub type SlDlssFeature = SlViewportHandle; // Assuming feature handle is like a viewport handle

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SlDLSSMode {
    Off = 0,
    MaxPerformance = 1, // Value for SL_DLSS_MODE_PERFORMANCE or similar
    Balanced = 2,       // Value for SL_DLSS_MODE_BALANCED
    MaxQuality = 3,     // Value for SL_DLSS_MODE_QUALITY
    UltraPerformance = 4,
    UltraQuality = 5, // If it exists
    DLAA = 6,           // Added DLAA mode
    // These values are illustrative. Check sl_dlss.h for actual enum values.
    // Ensure these align with what you defined in UpscalingQuality if used for mapping.
}

// Placeholder, actual structure might be more complex or part of SlDLSSOptions
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SlDLSSOptimalSettings {
    pub optimal_width: u32,
    pub optimal_height: u32,
    // ... other fields like sharpness, presets
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SlDLSSOptions {
    pub mode: SlDLSSMode,
    pub output_width: u32,
    pub output_height: u32,
    // pub sharpness: f32, // Optional, may require checking if supported/needed
    // pub preset_quality: SlDLSSPreset, // If presets are used
    // ... other options like pre_exposure, indicator_type etc.
    // For simplicity, starting with minimal options. Refer to sl_dlss.h
    pub color_input_format: u32, // Placeholder for actual format enum/value
    pub motion_vector_format: u32, // Placeholder for actual format enum/value
    pub depth_input_format: u32, // Placeholder for actual format enum/value
    pub is_hdr: SlBool,
    pub pre_exposure: f32,
    pub enable_auto_exposure: SlBool,
    // These are examples; the actual SlDLSSOptions needs to match sl_dlss.h
}


// Placeholder - structure will depend on what `slDLSSGetState` returns
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SlDLSSState {
    pub width: u32,
    pub height: u32,
    // ... other state information
}

// Constants from sl_consts.h
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SlConstants {
    // pub some_constant: u32, // Example
    // Add actual constants defined in sl_consts.h if needed directly
    // For now, many constants are used directly as values in function calls (e.g. feature IDs)
    _unused: u32, // To make the struct non-empty if no direct constants are needed yet
}


// --- Define Function Pointer Types ---
type FnSlInitializeSDK = unsafe extern "C" fn() -> SlStatus;
type FnSlShutdownSDK = unsafe extern "C" fn() -> SlStatus;
type FnSlIsFeatureSupported = unsafe extern "C" fn(feature: SlFeature, adapter_info: *const c_void) -> SlBool;
type FnSlCreateDlssFeature = unsafe extern "C" fn(
    dlss_feature_handle: *mut SlDlssFeature,
    application_id: u32,
    quality_mode: SlDLSSMode,
    output_width: u32,
    output_height: u32,
    native_device: *mut c_void,
) -> SlStatus;
type FnSlEvaluateDlssFeature = unsafe extern "C" fn(
    dlss_feature_handle: SlDlssFeature,
    cmd_buffer: *mut c_void,
    input_resource: *mut c_void,
    output_resource: *mut c_void,
    motion_vectors: *mut c_void,
    depth: *mut c_void,
    jitter_x: f32,
    jitter_y: f32,
    render_width: u32,
    render_height: u32,
    params: *const SlDLSSOptions,
) -> SlStatus;
type FnSlDestroyDlssFeature = unsafe extern "C" fn(dlss_feature_handle: SlDlssFeature) -> SlStatus;
type FnSlDLSSSetOptions = unsafe extern "C" fn(
    dlss_feature_handle: SlDlssFeature,
    options: *const SlDLSSOptions,
) -> SlStatus;
// Add other function pointer types here if needed, e.g.:
// type FnSlDLSSGetOptimalSettings = unsafe extern "C" fn(...) -> SlStatus;
// type FnSlDLSSGetState = unsafe extern "C" fn(...) -> SlStatus;


// --- Dynamic Loading Implementation ---

// Use a more descriptive error type if desired
#[derive(Debug)]
struct LoadError(String);

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Streamline API Load Error: {}", self.0)
    }
}

impl From<libloading::Error> for LoadError {
    fn from(e: libloading::Error) -> Self {
        LoadError(e.to_string())
    }
}


// StreamlineApi struct now only holds the Library
struct StreamlineApi {
    _lib: Library,
}

// Global static variable to hold the initialized API
static SL_API: OnceLock<Result<StreamlineApi, LoadError>> = OnceLock::new();

// Renamed back: Loads library and creates the simple StreamlineApi
fn load_streamline_api() -> Result<StreamlineApi, LoadError> {
    unsafe {
        let dll_name = if cfg!(target_os = "windows") {
            "sl.interposer.dll"
        } else {
            "libsl.interposer.so"
        };
        let lib = Library::new(dll_name).map_err(|e| LoadError(format!("Failed to load library '{}': {}", dll_name, e)))?;
        Ok(StreamlineApi { _lib: lib })
    }
}

// get_sl_api remains mostly the same, but calls the simpler load_streamline_api
fn get_sl_api() -> Result<&'static StreamlineApi, &'static LoadError> {
    SL_API.get_or_init(load_streamline_api).as_ref()
}

// --- Public wrapper functions --- 
// Modified to load symbols on demand

// Helper macro for loading symbols within wrappers
macro_rules! get_sl_func {
    ($api:expr, $fn_type:ty, $name:expr) => {
        $api._lib.get::<$fn_type>($name).map_err(|e| {
            // Log error, return specific SlStatus
            eprintln!(
                "Streamline API Error: Failed to load symbol '{}': {}",
                String::from_utf8_lossy($name).trim_end_matches('\0'),
                e
            );
            SlStatus::ErrorFunctionLoadFailed // Use custom error code
        })
    };
}

pub unsafe fn slInitializeSDK() -> SlStatus {
    match get_sl_api() {
        Ok(api) => match get_sl_func!(api, FnSlInitializeSDK, b"slInitializeSDK\0") {
            Ok(func) => func(),
            Err(status) => status,
        },
        Err(e) => {
             eprintln!("{}", e); // Print LoadError if library failed loading
             SlStatus::ErrorLibraryLoadFailed
        }
    }
}

pub unsafe fn slShutdownSDK() -> SlStatus {
    match get_sl_api() {
        Ok(api) => match get_sl_func!(api, FnSlShutdownSDK, b"slShutdownSDK\0") {
            Ok(func) => func(),
            Err(status) => status,
        },
        Err(e) => {
             eprintln!("{}", e);
             SlStatus::ErrorLibraryLoadFailed
        }
    }
}

pub unsafe fn slIsFeatureSupported(feature: SlFeature, adapter_info: *const c_void) -> SlBool {
     match get_sl_api() {
        Ok(api) => match get_sl_func!(api, FnSlIsFeatureSupported, b"slIsFeatureSupported\0") {
            Ok(func) => func(feature, adapter_info),
            Err(_) => SL_FALSE, // Return false if function load failed
        },
        Err(e) => {
             eprintln!("{}", e);
             SL_FALSE // Return false if library failed loading
        }
    }
}

pub unsafe fn slCreateDlssFeature(
    dlss_feature_handle: *mut SlDlssFeature,
    application_id: u32,
    quality_mode: SlDLSSMode,
    output_width: u32,
    output_height: u32,
    native_device: *mut c_void,
) -> SlStatus {
     match get_sl_api() {
        Ok(api) => match get_sl_func!(api, FnSlCreateDlssFeature, b"slCreateDlssFeature\0") {
            Ok(func) => func(
                dlss_feature_handle,
                application_id,
                quality_mode,
                output_width,
                output_height,
                native_device,
            ),
            Err(status) => status,
        },
        Err(e) => {
             eprintln!("{}", e);
             SlStatus::ErrorLibraryLoadFailed
        }
    }
}

pub unsafe fn slEvaluateDlssFeature(
    dlss_feature_handle: SlDlssFeature,
    cmd_buffer: *mut c_void,
    input_resource: *mut c_void,
    output_resource: *mut c_void,
    motion_vectors: *mut c_void,
    depth: *mut c_void,
    jitter_x: f32,
    jitter_y: f32,
    render_width: u32,
    render_height: u32,
    params: *const SlDLSSOptions,
) -> SlStatus {
     match get_sl_api() {
        Ok(api) => match get_sl_func!(api, FnSlEvaluateDlssFeature, b"slEvaluateDlssFeature\0") {
            Ok(func) => func(
                dlss_feature_handle,
                cmd_buffer,
                input_resource,
                output_resource,
                motion_vectors,
                depth,
                jitter_x,
                jitter_y,
                render_width,
                render_height,
                params,
            ),
            Err(status) => status,
        },
        Err(e) => {
             eprintln!("{}", e);
             SlStatus::ErrorLibraryLoadFailed
        }
    }
}

pub unsafe fn slDestroyDlssFeature(dlss_feature_handle: SlDlssFeature) -> SlStatus {
     match get_sl_api() {
        Ok(api) => match get_sl_func!(api, FnSlDestroyDlssFeature, b"slDestroyDlssFeature\0") {
            Ok(func) => func(dlss_feature_handle),
            Err(status) => status,
        },
        Err(e) => {
             eprintln!("{}", e);
             SlStatus::ErrorLibraryLoadFailed
        }
    }
}

pub unsafe fn slDLSSSetOptions(
    dlss_feature_handle: SlDlssFeature,
    options: *const SlDLSSOptions,
) -> SlStatus {
     match get_sl_api() {
        Ok(api) => match get_sl_func!(api, FnSlDLSSSetOptions, b"slDLSSSetOptions\0") {
            Ok(func) => func(dlss_feature_handle, options),
            Err(status) => status,
        },
        Err(e) => {
             eprintln!("{}", e);
             SlStatus::ErrorLibraryLoadFailed
        }
    }
}

// Add wrappers for other functions here as needed, following the same pattern


// --- Ensure the old extern "C" block is removed ---
