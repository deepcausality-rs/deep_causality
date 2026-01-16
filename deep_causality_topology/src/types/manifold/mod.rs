/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Manifold type for smooth geometric structures.
//!
//! A `Manifold<T>` wraps a `SimplicialComplex` and provides geometric operations
//! including volume computation, differential operators, and covariance analysis.

use crate::{ReggeGeometry, SimplicialComplex};
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

/// A newtype wrapper around `SimplicialComplex` that represents a Manifold.
///
/// Its construction enforces geometric properties essential for physics simulations.
/// The type parameter T represents data living on the manifold's simplices.
#[derive(Debug, Clone, PartialEq)]
pub struct Manifold<C, D> {
    /// The underlying simplicial complex, guaranteed to satisfy manifold properties.
    pub(crate) complex: SimplicialComplex<C>,
    /// The data associated with the manifold (e.g., scalar field values on simplices).
    pub(crate) data: CausalTensor<D>,
    /// The metric information of the manifold, containing edge lengths.
    pub(crate) metric: Option<ReggeGeometry<C>>,
    /// The Focus (Cursor) for Comonadic extraction.
    pub(crate) cursor: usize,
}
