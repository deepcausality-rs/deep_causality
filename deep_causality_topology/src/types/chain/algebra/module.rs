/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::chain::Chain;
use alloc::sync::Arc;
use deep_causality_num::{Module, Ring};

impl<T> Chain<T> {
    /// Scales the chain by a scalar.
    ///
    /// # Arguments
    /// * `scalar` - The scalar to multiply by.
    ///
    /// # Returns
    /// A new chain with all weights scaled.
    pub fn scale<S>(&self, scalar: S) -> Self
    where
        T: Module<S> + Copy,
        S: Ring + Copy,
    {
        let weights = self.weights.scale(scalar);
        Self {
            complex: Arc::clone(&self.complex),
            grade: self.grade,
            weights,
        }
    }
}
