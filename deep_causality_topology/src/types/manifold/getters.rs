/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Manifold, ReggeGeometry, SimplicialComplex};
use deep_causality_tensor::CausalTensor;

impl<T> Manifold<T> {
    pub fn complex(&self) -> &SimplicialComplex {
        &self.complex
    }

    pub fn data(&self) -> &CausalTensor<T> {
        &self.data
    }

    pub fn metric(&self) -> Option<&ReggeGeometry> {
        self.metric.as_ref()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }
}
