/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Topology type for discrete fields on simplicial complexes.

use crate::SimplicialComplex;
use deep_causality_tensor::CausalTensor;
use std::sync::Arc;

// Submodule declarations (folder-based)
mod api;
mod clone;
mod constructors;
mod display;
mod getters;

mod ops;

// Re-export public API

/// Represents a discrete field defined on the k-skeleton.
///
/// (e.g., Temperature on Vertices, Magnetic Flux on Faces).
#[derive(Clone, Debug)]
/// # The two parameters
///
/// `R` is the metric precision of the complex; `G` is the coefficient group the data lives in. They
/// are independent, and separating them is what lets `fmap` carry the complex across instead of
/// rebuilding it, which is what makes the functor identity law hold. See [`crate::Chain`].
///
/// Use [`UniformTopology`] where both are the same type.
pub struct Topology<R, G> {
    /// Shared reference to the underlying mesh.
    pub(crate) complex: Arc<SimplicialComplex<R>>,
    /// The dimension of the simplices this data lives on.
    pub(crate) grade: usize,
    /// The values (CausalTensor is essentially a dense vector here).
    pub(crate) data: CausalTensor<G>,
    /// The Focus (Cursor) for Comonadic extraction.
    pub(crate) cursor: usize,
}

/// A topology whose data shares the complex's precision type, the common case.
pub type UniformTopology<T> = Topology<T, T>;
