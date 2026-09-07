/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The result of a penalised least-squares fit.

use alloc::vec::Vec;

/// Coefficients and residual variance from a ridge fit.
#[derive(Debug, Clone, PartialEq)]
pub struct RidgeFit<T> {
    /// The fitted coefficients, one per column of the design.
    pub beta: Vec<T>,
    /// The residual variance, on `max(n − p, 1)` degrees of freedom.
    pub sigma2: T,
}
