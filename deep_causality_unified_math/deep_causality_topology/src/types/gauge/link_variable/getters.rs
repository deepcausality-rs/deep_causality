/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GaugeGroup, LinkVariable};

impl<G: GaugeGroup, M: Clone, R> LinkVariable<G, M, R> {
    /// Matrix elements.
    ///
    /// # Returns
    ///
    /// The row-major `N x N` elements.
    #[inline]
    pub fn matrix(&self) -> &[M] {
        self.as_slice()
    }

    /// Matrix elements, mutably.
    ///
    /// # Returns
    ///
    /// The row-major `N x N` elements, mutably.
    #[inline]
    pub fn matrix_mut(&mut self) -> &mut [M] {
        &mut self.data
    }

    /// Raw matrix data as slice.
    ///
    /// # Returns
    ///
    /// Flat slice of matrix elements (row-major).
    #[inline]
    pub fn as_slice(&self) -> &[M] {
        self.data.as_slice()
    }

    /// Lie algebra dimension (N² - 1 for SU(N)).
    ///
    /// # Returns
    ///
    /// The number of generators.
    #[inline]
    pub fn lie_dim() -> usize {
        G::LIE_ALGEBRA_DIM
    }

    /// Matrix dimension N for SU(N).
    ///
    /// # Returns
    ///
    /// The size N of the N×N matrices.
    #[inline]
    pub fn matrix_dim() -> usize {
        G::matrix_dim()
    }
}
