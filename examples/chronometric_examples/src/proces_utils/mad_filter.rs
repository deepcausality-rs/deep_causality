/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::gravity_types::GmDataPoint;
use deep_causality_algebra::RealField;

/// Apply MAD (Median Absolute Deviation) filtering to remove outliers.
///
/// Generic over any `Float` type to maintain precision (f64, DoubleFloat, etc.).
///
/// # Statistical Background
///
/// The constant 1.4826 ≈ 1/Φ⁻¹(3/4) where Φ⁻¹ is the inverse normal CDF.
/// This makes MAD a consistent estimator of σ for Gaussian-distributed data:
/// σ ≈ 1.4826 × MAD
pub fn apply_mad_filter<T>(data: &[T], outlier_sigma: T) -> Vec<T>
where
    T: RealField + From<f64> + Clone,
{
    if data.is_empty() {
        return Vec::new();
    }

    // Sort for median calculation (preserves original order in `data` for final filter)
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // True median: average of middle two elements for even-length arrays
    let median = if sorted.len() % 2 == 1 {
        sorted[sorted.len() / 2]
    } else {
        let mid = sorted.len() / 2;
        let two = T::from(2.0);
        (sorted[mid - 1] + sorted[mid]) / two
    };

    // Calculate MAD (Median Absolute Deviation)
    let residuals: Vec<T> = sorted.iter().map(|x| (*x - median).abs()).collect();
    let mut sorted_residuals = residuals;
    sorted_residuals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // True median of residuals
    let mad = if sorted_residuals.len() % 2 == 1 {
        sorted_residuals[sorted_residuals.len() / 2]
    } else {
        let mid = sorted_residuals.len() / 2;
        let two = T::from(2.0);
        (sorted_residuals[mid - 1] + sorted_residuals[mid]) / two
    };

    // Estimate sigma from MAD (σ ≈ 1.4826 × MAD)
    let scale_factor = T::from(1.4826);
    let sigma_est = scale_factor * mad;

    if sigma_est > T::zero() {
        data.iter()
            .copied()
            .filter(|&g| (g - median).abs() <= outlier_sigma * sigma_est)
            .collect()
    } else {
        data.to_vec()
    }
}

/// Apply MAD filter to GmDataPoint based on GM values
pub fn apply_mad_filter_points<T>(data: &[GmDataPoint<T>], outlier_sigma: T) -> Vec<GmDataPoint<T>>
where
    T: RealField + From<f64> + Clone,
{
    if data.is_empty() {
        return Vec::new();
    }

    // 1. Sort by GM
    let mut sorted_by_gm = data.to_vec();
    sorted_by_gm.sort_by(|a, b| a.gm.partial_cmp(&b.gm).unwrap_or(std::cmp::Ordering::Equal));

    let len = sorted_by_gm.len();
    let two = T::from(2.0);

    // 2. Calculate Median
    let median = if len % 2 == 1 {
        sorted_by_gm[len / 2].gm
    } else {
        let mid = len / 2;
        (sorted_by_gm[mid - 1].gm + sorted_by_gm[mid].gm) / two
    };

    // 3. Calculate MAD
    let residuals: Vec<T> = sorted_by_gm.iter().map(|p| (p.gm - median).abs()).collect();
    let mut sorted_res = residuals;
    sorted_res.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mad = if len % 2 == 1 {
        sorted_res[len / 2]
    } else {
        let mid = len / 2;
        (sorted_res[mid - 1] + sorted_res[mid]) / two
    };

    let scale_factor = T::from(1.4826);
    let sigma_est = scale_factor * mad;

    if sigma_est > T::zero() {
        data.iter()
            .copied()
            .filter(|p| (p.gm - median).abs() <= outlier_sigma * sigma_est)
            .collect()
    } else {
        data.to_vec()
    }
}
