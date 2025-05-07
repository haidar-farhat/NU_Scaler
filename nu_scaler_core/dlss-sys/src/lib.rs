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


// --- Streamline SDK API Functions ---
extern "C" {
    // Core SDK functions (from sl.h or similar)
    pub fn slInitializeSDK() -> SlStatus;
    pub fn slShutdownSDK() -> SlStatus;

    pub fn slIsFeatureSupported(feature: SlFeature, adapter_info: *const c_void /* SlAdapterInfo* */) -> SlBool;
    // pub fn slGetFeatureRequirements(feature: SlFeature, requirements: *mut SlFeatureRequirements) -> SlStatus; // If needed
    
    // Feature management functions
    pub fn slCreateFeature(viewport: SlViewportHandle, feature_id: SlFeature) -> SlStatus; // Simplified
    pub fn slDestroyFeature(viewport: SlViewportHandle, feature_id: SlFeature) -> SlStatus; // Simplified
    
    // Generic evaluation - might be used internally or you might use feature-specific ones
    // pub fn slEvaluateFeature(viewport: SlViewportHandle, feature_id: SlFeature, frame_index: u32, constants: *const SlConstants) -> SlStatus;

    // DLSS specific functions (from sl_dlss.h)

    // Note: The Streamline API often uses a pattern where you "create" a feature on a viewport/command buffer,
    // then "set options" for it, and then "evaluate" it.
    // The exact function names and parameters can vary based on Streamline version and abstraction level.
    // These are based on a common understanding; verify against your sl_dlss.h.

    // Simplified concept of creating a DLSS feature context/handle
    // The actual API might involve passing a command buffer or device pointer.
    // For now, using SlDlssFeature as an opaque handle type.
    pub fn slCreateDlssFeature(
        dlss_feature_handle: *mut SlDlssFeature, // Output parameter for the handle
        application_id: u32, // Your app ID from NVIDIA
        quality_mode: SlDLSSMode, // Initial quality mode
        output_width: u32,
        output_height: u32,
        native_device: *mut c_void // e.g., ID3D12Device* or VkDevice
    ) -> SlStatus;

    pub fn slEvaluateDlssFeature(
        dlss_feature_handle: SlDlssFeature,
        cmd_buffer: *mut c_void, // e.g., ID3D12GraphicsCommandList* or VkCommandBuffer
        input_resource: *mut c_void, // e.g., ID3D12Resource* or VkImage
        output_resource: *mut c_void, // e.g., ID3D12Resource* or VkImage
        motion_vectors: *mut c_void, // Optional: native motion vector resource
        depth: *mut c_void, // Optional: native depth buffer resource
        jitter_x: f32,
        jitter_y: f32,
        render_width: u32, // Input/render resolution
        render_height: u32,
        // ... other parameters like exposure, callback functions etc. from sl_dlss.h for evaluation
        params: *const SlDLSSOptions // This is a guess, slEvaluate might take options directly or they are set via slDLSSSetOptions
    ) -> SlStatus;

    pub fn slDestroyDlssFeature(dlss_feature_handle: SlDlssFeature) -> SlStatus;

    pub fn slDLSSGetOptimalSettings(
        app_id: u32, // May not be needed if feature already created
        dlss_feature_handle_or_null: SlDlssFeature, // or perhaps just output width/height
        mode: SlDLSSMode,
        render_width: u32,
        render_height: u32,
        out_optimal_width: *mut u32,
        out_optimal_height: *mut u32,
        out_sharpness: *mut f32, // if applicable
        out_preset_quality: *mut /* SlDLSSPreset */ c_void // Placeholder type
    ) -> SlStatus;

    pub fn slDLSSSetOptions(
        dlss_feature_handle: SlDlssFeature,
        options: *const SlDLSSOptions // Pointer to your options struct
    ) -> SlStatus;

    // Might return more detailed info than just width/height
    pub fn slDLSSGetState(
        dlss_feature_handle: SlDlssFeature,
        state_buffer: *mut SlDLSSState, // Pointer to a struct to be filled
        buffer_size: usize // Size of the buffer
    ) -> SlStatus;
}
