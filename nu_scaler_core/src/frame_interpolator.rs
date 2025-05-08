#![allow(dead_code)] // Allow dead code for now during initial development

use anyhow::{anyhow, Result};
use image::{GrayImage, Rgba, RgbaImage};
use imageproc::geometric_transformations::{warp_into, Interpolation, Projection};
use optical_flow_lk::{
    pyramid::build_pyramid, 
    flow::Motion, 
    lk::{calc_optical_flow, LKFlags, Accuracy, TermCriteria, TermCriteriaTypes}
};
use serde::Deserialize;

/// Configuration for the frame interpolation process.
#[derive(Debug, Deserialize, Clone)]
pub struct InterpolationConfig {
    /// Number of pyramid levels for optical flow calculation.
    pub pyramid_levels: u32,
    /// Window size for the Lucas-Kanade algorithm.
    pub window_size: i32,
    /// Maximum number of features to track (if explicit tracking is used).
    pub max_features_to_track: usize,
    /// Quality level for feature detection (e.g., Shi-Tomasi).
    pub feature_quality_level: f64,
    /// Minimum distance between detected features.
    pub feature_min_distance: f64,
    /// Blend mode for combining warped frames.
    pub blend_mode: BlendMode,
    // LK specific parameters
    /// Max iteration count for LK.
    pub lk_max_iterations: i32,
    /// Epsilon for LK convergence criteria.
    pub lk_epsilon: f64,
    /// Minimum Eigenvalue threshold for corner detection/tracking stability.
    pub lk_min_eigen_threshold: f64,
}

impl Default for InterpolationConfig {
    fn default() -> Self {
        Self {
            pyramid_levels: 3,
            window_size: 21, // Common default for LK
            max_features_to_track: 500,
            feature_quality_level: 0.01,
            feature_min_distance: 10.0,
            blend_mode: BlendMode::Linear,
            lk_max_iterations: 20,
            lk_epsilon: 0.03,
            lk_min_eigen_threshold: 0.0001,
        }
    }
}

/// Defines how warped frames are blended together.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// Simple linear interpolation: (1-t)*frame_a + t*frame_b
    Linear,
    // Future: Add more sophisticated blending if needed
    // Weighted,
}

/// Represents the dense flow field, mapping each pixel to its motion vector.
/// For now, this is a placeholder. In reality, it would be a 2D grid of vectors.
/// `optical_flow_lk` returns `Vec<Motion>` which is sparse.
pub type SparseFlowMap = Vec<Motion>;

// Placeholder for a dense flow representation if we convert from sparse
// pub struct DenseFlowField {
//     pub vectors: Vec<(f32, f32)>, // or a more structured type
//     pub width: u32,
//     pub height: u32,
// }

/// Estimates optical flow between two grayscale frames.
pub struct FlowEstimator {
    config: InterpolationConfig,
}

impl FlowEstimator {
    pub fn new(config: InterpolationConfig) -> Self {
        Self { config }
    }

    /// Computes sparse optical flow between two grayscale images.
    pub fn compute_sparse_flow(
        &self,
        prev_gray: &GrayImage,
        next_gray: &GrayImage,
    ) -> Result<SparseFlowMap> {
        if prev_gray.dimensions() != next_gray.dimensions() {
            return Err(anyhow!("Previous and next frames must have the same dimensions."));
        }

        let prev_pyramid = build_pyramid(prev_gray, self.config.pyramid_levels as usize);
        let next_pyramid = build_pyramid(next_gray, self.config.pyramid_levels as usize);

        // Define termination criteria for LK iteration
        let term_criteria = TermCriteria {
            criteria_type: TermCriteriaTypes::COUNT as i32 | TermCriteriaTypes::EPS as i32,
            max_count: self.config.lk_max_iterations,
            epsilon: self.config.lk_epsilon,
        };

        // Corrected call to calc_optical_flow
        // Assuming we let the function detect features internally by passing an empty slice for initial features.
        let initial_features: Vec<(f32, f32)> = Vec::new(); // Empty slice signals internal detection

        let flow_vectors = calc_optical_flow(
            &prev_pyramid,
            &next_pyramid,
            &initial_features, // Pass empty slice to detect features
            self.config.window_size.try_into().unwrap_or(21), // Convert window size to usize
            self.config.pyramid_levels as usize, // max_level
            term_criteria,
            LKFlags::LK_DEFAULT, // flags
            self.config.lk_min_eigen_threshold, // min_eigen_threshold
            // Accuracy seems not to be a direct parameter in 0.1.0, possibly implicit?
        );
        
        if flow_vectors.is_empty() {
            // log::warn!("No optical flow vectors were found between the frames.");
        }

        Ok(flow_vectors)
    }
}

/// Synthesizes intermediate frames using optical flow.
pub struct FrameSynthesizer {
    config: InterpolationConfig,
}

impl FrameSynthesizer {
    pub fn new(config: InterpolationConfig) -> Self {
        Self { config }
    }

    /// Synthesizes an intermediate frame between prev_rgba and next_rgba.
    ///
    /// # Arguments
    /// * `prev_rgba` - The previous frame (at time T).
    /// * `next_rgba` - The next frame (at time T+1).
    /// * `sparse_flow_map` - Sparse optical flow vectors from `prev_rgba` to `next_rgba`.
    /// * `t` - Interpolation factor (0.0 < t < 1.0). `t=0.5` is halfway between frames.
    pub fn synthesize_frame(
        &self,
        prev_rgba: &RgbaImage,
        next_rgba: &RgbaImage,
        sparse_flow_map: &SparseFlowMap,
        t: f32,
    ) -> Result<RgbaImage> {
        if prev_rgba.dimensions() != next_rgba.dimensions() {
            return Err(anyhow!("Previous and next frames must have the same dimensions for synthesis."));
        }
        if !(0.0..=1.0).contains(&t) {
            return Err(anyhow!("Interpolation factor t must be between 0.0 and 1.0. Found: {}", t));
        }

        let (width, height) = prev_rgba.dimensions();
        
        // --- CRITICAL SECTION: Sparse to Dense Flow & Warping ---
        // The `sparse_flow_map` contains motion vectors for specific points.
        // `imageproc::warp_into` requires a backward mapping for *every* pixel of the output frame.
        // This requires converting the sparse flow into a dense flow field or using a more
        // sophisticated warping technique.
        // The `create_projection_from_sparse_flow` function below is a MAJOR simplification
        // and will NOT produce good results. It's a placeholder for a complex algorithm.
        // Real implementations might use:
        //  - Delaunay triangulation of feature points and per-triangle affine warps.
        //  - Radial Basis Function (RBF) interpolation of flow vectors.
        //  - Other dense flow estimation techniques if performance allows.

        let proj_for_prev = create_projection_from_sparse_flow(sparse_flow_map, t, width, height, false);
        let mut warped_prev_rgba = RgbaImage::new(width, height);
        // Corrected warp_into call order: input, map, interpolation, fill_value, output
        warp_into(
            prev_rgba,
            &proj_for_prev,
            Interpolation::Bilinear,
            Rgba([0, 0, 0, 0]), // Fill value
            &mut warped_prev_rgba, // Output buffer
        );

        // For warping `next_rgba` to the intermediate time `t`:
        // The flow is from `prev` to `next`.
        // We want to find where a pixel in `intermediate` came from in `next`.
        // Motion from `next` to `intermediate` is -(1-t) * flow.
        let proj_for_next = create_projection_from_sparse_flow(sparse_flow_map, -(1.0 - t), width, height, false);
        let mut warped_next_rgba = RgbaImage::new(width, height);
        // Corrected warp_into call order
        warp_into(
            next_rgba,
            &proj_for_next,
            Interpolation::Bilinear,
            Rgba([0, 0, 0, 0]), // Fill value
            &mut warped_next_rgba, // Output buffer
        );
        
        // --- Blending ---
        let mut interpolated_frame = RgbaImage::new(width, height);
        match self.config.blend_mode {
            BlendMode::Linear => {
                for y in 0..height {
                    for x in 0..width {
                        let p_prev = warped_prev_rgba.get_pixel(x, y);
                        let p_next = warped_next_rgba.get_pixel(x, y);

                        let r = ((1.0 - t) * p_prev[0] as f32 + t * p_next[0] as f32).round() as u8;
                        let g = ((1.0 - t) * p_prev[1] as f32 + t * p_next[1] as f32).round() as u8;
                        let b = ((1.0 - t) * p_prev[2] as f32 + t * p_next[2] as f32).round() as u8;
                        let final_alpha = ((1.0-t) * p_prev[3] as f32 + t * p_next[3] as f32).round() as u8;

                        interpolated_frame.put_pixel(x, y, Rgba([r, g, b, final_alpha]));
                    }
                }
            }
        }
        Ok(interpolated_frame)
    }
}

/// PLACEHOLDER: Creates a simplified projection for `imageproc::warp_into` from sparse flow.
/// This is a highly simplified and likely incorrect way to handle sparse flow for dense warping.
/// A real implementation needs a robust sparse-to-dense flow interpolation.
///
/// # Arguments
/// * `sparse_flow_map`: Output from Lucas-Kanade (Vec<Motion>).
/// * `t_factor`: Time factor for flow (e.g., `t` for prev->interp, `-(1-t)` for next->interp).
/// * `_width`, `_height`: Dimensions of the frame.
/// * `_is_forward_flow`: Indicates if flow vectors point from source to target (true) or target to source (false).
///                      `imageproc::warp_into` expects a map from target coords to source coords.
fn create_projection_from_sparse_flow(
    sparse_flow_map: &SparseFlowMap,
    t_factor: f32,
    _width: u32,
    _height: u32,
    _is_forward_flow: bool, // True if flow is (src_x, src_y) -> (dst_x, dst_y)
) -> Projection {
    // Simplistic: Average all flow vectors to get one global motion vector.
    // This is a very naive placeholder.
    let mut avg_dx = 0.0;
    let mut avg_dy = 0.0;
    if !sparse_flow_map.is_empty() {
        for motion in sparse_flow_map {
            avg_dx += motion.get_dx();
            avg_dy += motion.get_dy();
        }
        avg_dx /= sparse_flow_map.len() as f32;
        avg_dy /= sparse_flow_map.len() as f32;
    }

    let transform_matrix = [
        1.0, 0.0, -t_factor * avg_dx,
        0.0, 1.0, -t_factor * avg_dy,
        0.0, 0.0, 1.0,
    ];
    Projection::from_matrix(transform_matrix).expect("Failed to create projection from matrix")
}

// TODO: Implement a more robust sparse-to-dense flow interpolation method.
// Possible approaches:
// 1. Iterate over sparse_flow_map. For each (x,y) in output, find N nearest flow vectors.
//    Interpolate these N vectors (e.g., Inverse Distance Weighting). This is slow.
// 2. Triangulate the feature points (e.g., Delaunay). For each output pixel, find which
//    triangle it's in, and use barycentric interpolation of the triangle's vertex flows.
//    Then apply an affine warp for that triangle. (Complex to implement correctly).
// 3. Use a library that can do scattered data interpolation to create a dense grid. 