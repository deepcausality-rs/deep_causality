/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GaugeGroup, LinkVariable};
use deep_causality_tensor::CausalTensor;

impl<G: GaugeGroup, T: Clone + Default> LinkVariable<G, T> {
    /// Matrix data as tensor reference.
    #[inline]
    pub fn matrix(&self) -> &CausalTensor<T> {
        &self.data
    }

    /// Matrix data as mutable tensor reference.
    #[inline]
    pub fn matrix_mut(&mut self) -> &mut CausalTensor<T> {
        &mut self.data
    }

    /// Raw matrix data as slice.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }

    /// Lie algebra dimension (N² - 1 for SU(N)).
    #[inline]
    pub fn lie_dim() -> usize {
        G::LIE_ALGEBRA_DIM
    }

    /// Matrix dimension N for SU(N).
    #[inline]
    pub fn matrix_dim() -> usize {
        G::matrix_dim()
    }
}
