/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Getter methods for Manifold.
//! Getter methods for Manifold fields.

use crate::{Manifold, ReggeGeometry, SimplicialComplex};
use deep_causality_tensor::CausalTensor;

impl<C, D> Manifold<C, D> {
    /// Returns a reference to the underlying simplicial complex.
    pub fn complex(&self) -> &SimplicialComplex<C> {
        &self.complex
    }

    /// Returns a reference to the tensor data.
    pub fn data(&self) -> &CausalTensor<D> {
        &self.data
    }

    /// Returns an optional reference to the Regge geometry metric.
    pub fn metric(&self) -> Option<&ReggeGeometry<C>> {
        self.metric.as_ref()
    }

    /// Returns the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }
}
