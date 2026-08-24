/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::CsrMatrix;
use deep_causality_algebra::{Module, Ring};

impl<T> CsrMatrix<T> {
    /// Scalar multiplication.
    ///
    /// # Arguments
    /// * `scalar` - The scalar to multiply by
    ///
    /// # Returns
    /// A new matrix where each element is multiplied by `scalar`.
    pub fn scale<S>(&self, scalar: S) -> Self
    where
        T: Module<S> + Copy,
        S: Ring + Copy,
    {
        // For CsrMatrix, scalar multiplication is element-wise
        // T: Module<S> implies T: Mul<S, Output = T>
        let new_values: Vec<T> = self.values.iter().map(|&v| v * scalar).collect();
        Self {
            row_indices: self.row_indices.clone(),
            col_indices: self.col_indices.clone(),
            values: new_values,
            shape: self.shape,
        }
    }
}

// `Module<S>` needs no impl here. `deep_causality_algebra::module.rs:65` blankets it over
// `AbelianGroup + Mul<R, Output = Self> + MulAssign<R>`, and `CsrMatrix` satisfies all three —
// the additive markers carry it to `AbelianGroup`, and the scalar multiplication is implemented
// at `arithmetic/mod.rs:283,321`. Writing one by hand is E0119.
//
// `Module` rather than a vector space is what admits ℤ: the general notion is over a ring, and a
// vector space is the special case where that ring is a field. `CsrMatrix<i64>` scaled by `i64` is
// a module and is not a vector space.
