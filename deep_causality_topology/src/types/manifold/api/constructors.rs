/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Constructors for Manifold type.

use crate::{ReggeGeometry, SimplicialComplex, TopologyError};
use deep_causality_num::Zero;
use deep_causality_tensor::CausalTensor;

use super::super::Manifold;

impl<T> Manifold<T>
where
    T: Default + Copy + Clone + PartialEq + Zero,
{
    /// Attempts to create a new `Manifold` from a `SimplicialComplex` and data.
    ///
    /// This constructor performs rigorous checks to ensure the complex satisfies
    /// manifold criteria including orientation and link conditions.
    ///
    /// # Arguments
    /// * `complex` - The underlying simplicial complex
    /// * `data` - Tensor data associated with simplices
    /// * `cursor` - Initial cursor position for comonadic operations
    ///
    /// # Returns
    /// * `Ok(Manifold)` - A valid manifold
    /// * `Err(TopologyError)` - If validation fails
    ///
    /// # Example
    /// ```rust,ignore
    /// let manifold = Manifold::new(complex, data, 0)?;
    /// ```
    pub fn new(
        complex: SimplicialComplex,
        data: CausalTensor<T>,
        cursor: usize,
    ) -> Result<Self, TopologyError> {
        Self::new_cpu(complex, data, cursor)
    }

    /// Creates a Manifold with an associated Regge geometry metric.
    ///
    /// # Arguments
    /// * `complex` - The underlying simplicial complex
    /// * `data` - Tensor data associated with simplices
    /// * `metric` - Optional Regge geometry containing edge lengths
    /// * `cursor` - Initial cursor position
    ///
    /// # Returns
    /// * `Ok(Manifold)` - A valid manifold with metric
    /// * `Err(TopologyError)` - If validation fails
    pub fn with_metric(
        complex: SimplicialComplex,
        data: CausalTensor<T>,
        metric: Option<ReggeGeometry>,
        cursor: usize,
    ) -> Result<Self, TopologyError> {
        Self::with_metric_cpu(complex, data, metric, cursor)
    }
}

impl<T> Manifold<T>
where
    T: Clone,
{
    /// Creates a shallow clone of the Manifold with cursor reset to 0.
    pub fn clone_shallow(&self) -> Self {
        Self::clone_shallow_cpu(self)
    }
}
