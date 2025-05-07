//! Minimal DLSS FFI layer for SL 2.7.30
//! Links against sl.interposer.dll/.lib

use std::os::raw::{c_void, c_uint};

/// Status codes returned by SL functions.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SlStatus {
    Success = 0,
    // Add other SL_STATUS codes as needed...
    // Example: Error = -1, // Add actual values from sl_consts.h
}

/// Opaque handle to a DLSS feature instance.
pub type SlDlssFeature = *mut c_void;

#[link(name = "sl.interposer")]
extern "C" {
    /// Initialize the SL SDK (call once per process).
    /// Returns SlStatus::Success on success.
    pub fn slInitializeSDK() -> SlStatus;

    /// Create a DLSS feature.
    /// - `device`: your native GPU device pointer (e.g. ID3D11Device* or VkDevice).
    /// - `width`, `height`: dimensions of the render target.
    /// - `flags`: reserved, pass 0.
    /// - `out_feature`: receives the created SlDlssFeature.
    pub fn slCreateDlssFeature(
        device: *mut c_void,
        width: c_uint,
        height: c_uint,
        flags: c_uint,
        out_feature: *mut SlDlssFeature,
    ) -> SlStatus;

    /// Evaluate (run) the DLSS feature.
    /// - `feature`: handle from slCreateDlssFeature.
    /// - `input_color`, `input_depth`: pointers to your input resources.
    /// - `jitter_x`, `jitter_y`: sub-pixel jitter for TAA.
    /// - `output_color`: pointer to your output resource.
    pub fn slEvaluateDlssFeature(
        feature: SlDlssFeature,
        input_color: *const c_void,
        input_depth: *const c_void,
        jitter_x: f32,
        jitter_y: f32,
        output_color: *mut c_void,
    ) -> SlStatus;

    /// Destroy the DLSS feature when no longer needed.
    pub fn slDestroyDlssFeature(feature: SlDlssFeature) -> SlStatus;
}