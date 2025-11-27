/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Chain, SimplicialComplex};
use alloc::sync::Arc;
use deep_causality_sparse::CsrMatrix;

impl<T> Chain<T> {
    pub fn complex(&self) -> &Arc<SimplicialComplex> {
        &self.complex
    }

    pub fn grade(&self) -> usize {
        self.grade
    }

    pub fn weights(&self) -> &CsrMatrix<T> {
        &self.weights
    }
}
