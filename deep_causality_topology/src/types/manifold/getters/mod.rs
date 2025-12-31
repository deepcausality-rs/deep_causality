/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Getter methods for Manifold.
//! Getter methods for Manifold fields.

use crate::{Manifold, ReggeGeometry, SimplicialComplex};
use deep_causality_tensor::CausalTensor;

impl<T> Manifold<T> {
    /// Returns a reference to the underlying simplicial complex.
    pub fn complex(&self) -> &SimplicialComplex {
        &self.complex
    }

    /// Returns a reference to the tensor data.
    pub fn data(&self) -> &CausalTensor<T> {
        &self.data
    }

    /// Returns an optional reference to the Regge geometry metric.
    pub fn metric(&self) -> Option<&ReggeGeometry> {
        self.metric.as_ref()
    }

    /// Returns the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }
}
