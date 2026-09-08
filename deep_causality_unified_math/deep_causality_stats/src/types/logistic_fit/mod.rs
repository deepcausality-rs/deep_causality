/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The result of an iteratively reweighted least-squares fit.

use alloc::vec::Vec;

/// Coefficients and iteration count from a logistic fit.
#[derive(Debug, Clone, PartialEq)]
pub struct LogisticFit<T> {
    /// The fitted coefficients, one per column of the design.
    pub beta: Vec<T>,
    /// How many iterations the fit took to meet its stopping test.
    ///
    /// Reported on success as well as failure, so a caller can see a fit that only just
    /// converged before its cap.
    pub iterations: usize,
}
