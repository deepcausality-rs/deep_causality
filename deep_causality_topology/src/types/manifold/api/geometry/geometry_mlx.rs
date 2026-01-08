/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Public geometry API for Manifold.
//!
//! Dispatches to CPU or MLX implementations based on feature flags and heuristics.

use crate::{Manifold, Simplex, TopologyError};
use deep_causality_num::Float;

/// Threshold for simplex dimension above which MLX is preferred.
#[allow(dead_code)]
const GPU_SIMPLEX_THRESHOLD: usize = 4;

use std::iter::Product;

impl<T, D> Manifold<T, D>
where
    T: Float + From<f64> + Into<f64> + Product,
{
    /// Computes the squared volume of a k-simplex using Cayley-Menger determinant.
    ///
    /// GPU-accelerated when `mlx` feature is enabled and k ≥ 4.
    ///
    /// # Arguments
    /// * `simplex` - The simplex to compute volume for
    ///
    /// # Returns
    /// * `Ok(f64)` - The squared volume
    /// * `Err(TopologyError)` - If metric is missing or edges not found
    ///
    /// # Example
    /// ```rust,ignore
    /// let volume_sq = manifold.simplex_volume_squared(&simplex)?;
    /// ```
    pub fn simplex_volume_squared(&self, simplex: &Simplex) -> Result<f64, TopologyError> {
        let k = simplex.vertices.len() - 1;
        if k >= GPU_SIMPLEX_THRESHOLD {
            return self.simplex_volume_squared_mlx(simplex);
        }

        self.simplex_volume_squared_cpu(simplex).map(|v| v.into())
    }
}
