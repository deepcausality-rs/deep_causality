/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::penalisation::Penalisation;

/// How a logistic fit is penalised and when its Newton iteration stops.
///
/// Four knobs is past the point where positional arguments read: `fit_logistic(&x, &y, a, b, c)`
/// leaves the reader to remember which scalar is the penalty and which the tolerance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogisticConfig<T> {
    /// The ridge strength `λ` on the log-likelihood.
    pub penalty: T,
    /// Which coefficients `λ` reaches. Exempting the intercept column is the convention in
    /// `sklearn`, `statsmodels` and `glmnet`; see [`Penalisation`] for what the choice costs.
    pub penalisation: Penalisation,
    /// The Newton/IRLS iteration cap. Reaching it is `NotConverged`, not a returned iterate.
    pub max_iterations: usize,
    /// The step size below which the iteration is judged converged.
    pub tolerance: T,
}

impl<T> LogisticConfig<T> {
    /// A penalty over every column, which is the reading when nothing is said about an intercept.
    pub fn new(penalty: T, max_iterations: usize, tolerance: T) -> Self {
        Self {
            penalty,
            penalisation: Penalisation::AllColumns,
            max_iterations,
            tolerance,
        }
    }

    /// Exempts a column, or restores the penalty to all of them.
    pub fn with_penalisation(mut self, penalisation: Penalisation) -> Self {
        self.penalisation = penalisation;
        self
    }
}
