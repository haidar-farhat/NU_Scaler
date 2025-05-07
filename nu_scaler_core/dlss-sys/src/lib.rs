//! Minimal DLSS FFI layer for SL 2.7.30
//! Links against sl.interposer.dll/.lib

use std::os::raw::{c_uint, c_void};

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

// New FFI definitions based on sl_consts.h and sl_dlss.h

// --- From sl_consts.h ---

pub const SL_INVALID_FLOAT: f32 = 3.40282346638528859811704183484516925440e+38f32;
pub const SL_INVALID_UINT: u32 = 0xffffffff;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SlFloat2 {
    pub x: f32,
    pub y: f32,
}

impl Default for SlFloat2 {
    fn default() -> Self {
        Self {
            x: SL_INVALID_FLOAT,
            y: SL_INVALID_FLOAT,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SlFloat3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for SlFloat3 {
    fn default() -> Self {
        Self {
            x: SL_INVALID_FLOAT,
            y: SL_INVALID_FLOAT,
            z: SL_INVALID_FLOAT,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SlFloat4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for SlFloat4 {
    fn default() -> Self {
        Self {
            x: SL_INVALID_FLOAT,
            y: SL_INVALID_FLOAT,
            z: SL_INVALID_FLOAT,
            w: SL_INVALID_FLOAT,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SlFloat4x4 {
    pub row: [SlFloat4; 4],
}

impl Default for SlFloat4x4 {
    fn default() -> Self {
        Self {
            row: [SlFloat4::default(); 4],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct SlExtent {
    pub top: u32,
    pub left: u32,
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SlBoolean {
    False = 0,
    True = 1,
    Invalid = 2, // Assuming 'eInvalid' maps to 2, check original C++ char value if different
}

impl Default for SlBoolean {
    fn default() -> Self {
        SlBoolean::Invalid
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SlConstants {
    pub camera_view_to_clip: SlFloat4x4,
    pub clip_to_camera_view: SlFloat4x4,
    pub clip_to_lens_clip: SlFloat4x4,
    pub clip_to_prev_clip: SlFloat4x4,
    pub prev_clip_to_clip: SlFloat4x4,
    pub jitter_offset: SlFloat2,
    pub mvec_scale: SlFloat2,
    pub camera_pinhole_offset: SlFloat2,
    pub camera_pos: SlFloat3,
    pub camera_up: SlFloat3,
    pub camera_right: SlFloat3,
    pub camera_fwd: SlFloat3,
    pub camera_near: f32,
    pub camera_far: f32,
    pub camera_fov: f32,
    pub camera_aspect_ratio: f32,
    pub motion_vectors_invalid_value: f32,
    pub depth_inverted: SlBoolean,
    pub camera_motion_included: SlBoolean,
    pub motion_vectors_3d: SlBoolean,
    pub reset: SlBoolean,
    pub orthographic_projection: SlBoolean,
    pub motion_vectors_dilated: SlBoolean,
    pub motion_vectors_jittered: SlBoolean,
    // Version 2 member
    pub min_relative_linear_depth_object_separation: f32,
}

impl Default for SlConstants {
    fn default() -> Self {
        Self {
            camera_view_to_clip: SlFloat4x4::default(),
            clip_to_camera_view: SlFloat4x4::default(),
            clip_to_lens_clip: SlFloat4x4::default(),
            clip_to_prev_clip: SlFloat4x4::default(),
            prev_clip_to_clip: SlFloat4x4::default(),
            jitter_offset: SlFloat2::default(),
            mvec_scale: SlFloat2::default(),
            camera_pinhole_offset: SlFloat2::default(),
            camera_pos: SlFloat3::default(),
            camera_up: SlFloat3::default(),
            camera_right: SlFloat3::default(),
            camera_fwd: SlFloat3::default(),
            camera_near: SL_INVALID_FLOAT,
            camera_far: SL_INVALID_FLOAT,
            camera_fov: SL_INVALID_FLOAT,
            camera_aspect_ratio: SL_INVALID_FLOAT,
            motion_vectors_invalid_value: SL_INVALID_FLOAT,
            depth_inverted: SlBoolean::default(),
            camera_motion_included: SlBoolean::default(),
            motion_vectors_3d: SlBoolean::default(),
            reset: SlBoolean::default(),
            orthographic_projection: SlBoolean::False, // Default as per C++ header
            motion_vectors_dilated: SlBoolean::False,  // Default as per C++ header
            motion_vectors_jittered: SlBoolean::False, // Default as per C++ header
            min_relative_linear_depth_object_separation: 40.0, // Default as per C++ header
        }
    }
}

// --- From sl_dlss.h ---

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SlDLSSMode {
    Off = 0,
    MaxPerformance = 1,
    Balanced = 2,
    MaxQuality = 3,
    UltraPerformance = 4,
    UltraQuality = 5,
    DLAA = 6,
    Count, // Placeholder for the count, actual value might differ if used numerically
}

impl Default for SlDLSSMode {
    fn default() -> Self {
        SlDLSSMode::Off
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SlDLSSPreset {
    Default = 0,
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
    F = 6,
    G = 7,
    H = 8,
    I = 9,
    J = 10,
    K = 11,
    L = 12,
    M = 13,
    N = 14,
    O = 15,
    Count, // Placeholder
}

impl Default for SlDLSSPreset {
    fn default() -> Self {
        SlDLSSPreset::Default
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SlDLSSOptions {
    pub mode: SlDLSSMode,
    pub output_width: u32,
    pub output_height: u32,
    pub sharpness: f32,
    pub pre_exposure: f32,
    pub exposure_scale: f32,
    pub color_buffers_hdr: SlBoolean,
    pub indicator_invert_axis_x: SlBoolean,
    pub indicator_invert_axis_y: SlBoolean,
    pub dlaa_preset: SlDLSSPreset,
    pub quality_preset: SlDLSSPreset,
    pub balanced_preset: SlDLSSPreset,
    pub performance_preset: SlDLSSPreset,
    pub ultra_performance_preset: SlDLSSPreset,
    pub ultra_quality_preset: SlDLSSPreset,
    pub use_auto_exposure: SlBoolean,
    pub alpha_upscaling_enabled: SlBoolean,
}

impl Default for SlDLSSOptions {
    fn default() -> Self {
        Self {
            mode: SlDLSSMode::Off,
            output_width: SL_INVALID_UINT,
            output_height: SL_INVALID_UINT,
            sharpness: 0.0,
            pre_exposure: 1.0,
            exposure_scale: 1.0,
            color_buffers_hdr: SlBoolean::True,
            indicator_invert_axis_x: SlBoolean::False,
            indicator_invert_axis_y: SlBoolean::False,
            dlaa_preset: SlDLSSPreset::Default,
            quality_preset: SlDLSSPreset::Default,
            balanced_preset: SlDLSSPreset::Default,
            performance_preset: SlDLSSPreset::Default,
            ultra_performance_preset: SlDLSSPreset::Default,
            ultra_quality_preset: SlDLSSPreset::Default,
            use_auto_exposure: SlBoolean::False,
            alpha_upscaling_enabled: SlBoolean::False,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct SlDLSSOptimalSettings {
    pub optimal_render_width: u32,
    pub optimal_render_height: u32,
    pub optimal_sharpness: f32,
    pub render_width_min: u32,
    pub render_height_min: u32,
    pub render_width_max: u32,
    pub render_height_max: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct SlDLSSState {
    pub estimated_vram_usage_in_bytes: u64,
}

// Assuming ViewportHandle is a u32 or similar.
// This should be confirmed from sl_common.h or sl.h if possible.
pub type SlViewportHandle = u32;

// Add new function signatures to the extern "C" block
// Need to re-declare the block or ensure these are added to the existing one.
// For now, creating a new block for clarity, assuming the linker can handle it.
// Ideally, these would be added to the existing extern "C" block.
// For the edit tool, it's easier to append a new block.

#[link(name = "sl.interposer")] // Linker directive might be duplicated, but harmless
extern "C" {
    // Note: The original C++ uses function pointers like PFun_slDLSSGetOptimalSettings.
    // We declare the direct function names we expect to link against.
    // The C++ inline helpers suggest these are the names after import.

    pub fn slDLSSGetOptimalSettings(
        options: *const SlDLSSOptions,
        settings: *mut SlDLSSOptimalSettings,
    ) -> SlStatus;

    pub fn slDLSSGetState(viewport: SlViewportHandle, state: *mut SlDLSSState) -> SlStatus;

    pub fn slDLSSSetOptions(viewport: SlViewportHandle, options: *const SlDLSSOptions) -> SlStatus;

    // Placeholder for slCreateFeature if it's different from slCreateDlssFeature
    // For now, we assume slCreateDlssFeature is the specific function we need from earlier.
    // pub fn slCreateFeature(viewport: SlViewportHandle, featureId: u32, /* other params */) -> SlStatus;

    // Placeholder for slSetFeatureSpecifics - its parameters would depend on the feature.
    // For DLSS, this might be an alternative to slDLSSSetOptions or used for other setup.
    // pub fn slSetFeatureSpecifics(viewport: SlViewportHandle, featureId: u32, specifics: *const c_void) -> SlStatus;
}
