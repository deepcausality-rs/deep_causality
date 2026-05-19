/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Constructors for Manifold type.

use crate::{ReggeGeometry, SimplicialComplex, TopologyError};
use deep_causality_num::Zero;
use deep_causality_tensor::CausalTensor;

use super::super::Manifold;

impl<C, D> Manifold<SimplicialComplex<C>, D>
where
    C: Default + Copy + Clone + PartialEq + Zero,
    D: Default + Copy + Clone + PartialEq + Zero,
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
        complex: SimplicialComplex<C>,
        data: CausalTensor<D>,
        cursor: usize,
    ) -> Result<Self, TopologyError> {
        Self::new_impl(complex, data, cursor)
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
        complex: SimplicialComplex<C>,
        data: CausalTensor<D>,
        metric: Option<ReggeGeometry<C>>,
        cursor: usize,
    ) -> Result<Self, TopologyError> {
        Self::with_metric_impl(complex, data, metric, cursor)
    }
}

impl<C, D> Manifold<SimplicialComplex<C>, D>
where
    C: Clone,
    D: Clone,
{
    /// Creates a shallow clone of the Manifold with cursor reset to 0.
    pub fn clone_shallow(&self) -> Self {
        Self::clone_shallow_impl(self)
    }
}

// -- Cubical constructors (Stage C) ----------------------------------------

use crate::types::lattice_complex::LatticeComplex;

impl<const D: usize, F> Manifold<LatticeComplex<D>, F> {
    /// Construct a manifold over a cubical complex without a metric (raw assembly).
    ///
    /// Stage C ships this minimal cubical constructor so that `Manifold<LatticeComplex<D>, F>`
    /// can be assembled by examples and tests. It does not validate the cell count against
    /// `data.len()`; richer validation belongs in a follow-up that lifts the simplicial
    /// `new`/`with_metric` validation logic to a complex-agnostic trait method.
    pub fn from_cubical(complex: LatticeComplex<D>, data: CausalTensor<F>, cursor: usize) -> Self {
        Self {
            complex,
            data,
            metric: None,
            cursor,
        }
    }

    /// Construct a manifold over a cubical complex with a `CubicalReggeGeometry<D>` (unit-edge).
    pub fn from_cubical_with_metric(
        complex: LatticeComplex<D>,
        data: CausalTensor<F>,
        metric: crate::CubicalReggeGeometry<D>,
        cursor: usize,
    ) -> Self {
        Self {
            complex,
            data,
            metric: Some(metric),
            cursor,
        }
    }
}
