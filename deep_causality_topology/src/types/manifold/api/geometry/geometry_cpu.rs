/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Public geometry API for Manifold.
//!
//! Dispatches to CPU or MLX implementations based on feature flags and heuristics.

use crate::{Manifold, Simplex, TopologyError};

impl<T> Manifold<T> {
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
        self.simplex_volume_squared_cpu(simplex)
    }
}
