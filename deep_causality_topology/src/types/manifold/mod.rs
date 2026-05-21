/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Manifold type for smooth geometric structures.
//!
//! `Manifold<K, F>` wraps any `ChainComplex` (simplicial, cubical, or user-defined) and
//! carries an associated field tensor + an optional metric typed via `K::Metric`.
//! `SimplicialManifold<C, F>` is the textbook alias for the simplicial case.

use crate::SimplicialComplex;
use crate::traits::chain_complex::ChainComplex;
use deep_causality_tensor::CausalTensor;

mod api;
mod constructors;
mod covariance;
mod display;
mod geometry;
mod getters;
mod utils;

mod differential;
mod topology;

/// A newtype wrapper around any `ChainComplex` that represents a Manifold.
///
/// Its construction enforces geometric properties essential for physics simulations.
/// `K` is the underlying chain complex; `F` is the field data type living on cells.
/// `F: RealField` doubles as the precision parameter for the metric — the metric is typed
/// by the complex via the GAT `ChainComplex::Metric<F>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Manifold<K: ChainComplex, F: deep_causality_num::RealField> {
    /// The underlying chain complex, guaranteed to satisfy manifold properties when set.
    pub(crate) complex: K,
    /// The data associated with the manifold (e.g., scalar field values on cells).
    pub(crate) data: CausalTensor<F>,
    /// The metric information of the manifold (e.g. edge lengths for Regge geometry,
    /// unit-edge flag for cubical complexes). Typed via the GAT `K::Metric<F>` so the
    /// metric's precision matches the field data's precision.
    pub(crate) metric: Option<K::Metric<F>>,
    /// The Focus (Cursor) for Comonadic extraction.
    pub(crate) cursor: usize,
}

/// Textbook alias for the simplicial case: `Manifold<SimplicialComplex<C>, F>`.
///
/// `C` is the simplex coordinate type, `F` is the field data type on simplices.
/// Eases migration from the previous `Manifold<C, D>` signature.
pub type SimplicialManifold<C, F> = Manifold<SimplicialComplex<C>, F>;
