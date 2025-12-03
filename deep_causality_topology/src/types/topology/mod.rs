/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::SimplicialComplex;
use alloc::sync::Arc;
use deep_causality_tensor::CausalTensor;

mod clone;
mod cup_product;
mod display;
mod getters;

/// Represents a discrete field defined on the k-skeleton.
/// (e.g., Temperature on Vertices, Magnetic Flux on Faces).
#[derive(Clone, Debug)]
pub struct Topology<T> {
    /// Shared reference to the underlying mesh
    pub(crate) complex: Arc<SimplicialComplex>,
    /// The dimension of the simplices this data lives on
    pub(crate) grade: usize,
    /// The values (CausalTensor is essentially a dense vector here)
    pub(crate) data: CausalTensor<T>,
    /// The Focus (Cursor) for Comonadic extraction
    pub(crate) cursor: usize,
}

impl<T> Topology<T> {
    pub fn new(
        complex: Arc<SimplicialComplex>,
        grade: usize,
        data: CausalTensor<T>,
        cursor: usize,
    ) -> Self {
        Self {
            complex,
            grade,
            data,
            cursor,
        }
    }
}
