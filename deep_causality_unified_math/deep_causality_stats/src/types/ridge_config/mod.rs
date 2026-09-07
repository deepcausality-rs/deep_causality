/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::penalisation::Penalisation;

/// How a ridge fit is penalised: the strength, and which coefficients it reaches.
///
/// A struct rather than two more positional arguments, for the reason [`EntropyConfig`] is one:
/// the knobs are not ordered by importance, and a call site reading `fit_ridge(&x, &y, l, p)` says
/// nothing about which scalar is which. It also means a later knob is additive.
///
/// [`EntropyConfig`]: crate::EntropyConfig
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RidgeConfig<T> {
    /// The ridge strength `λ`. A negative penalty is accepted: it subtracts from the diagonal
    /// rather than adding to it, which is still solvable while the diagonal survives.
    pub penalty: T,
    /// Which coefficients `λ` reaches.
    pub penalisation: Penalisation,
}

impl<T> RidgeConfig<T> {
    /// A penalty over every column, which is the reading when nothing is said about an intercept.
    pub fn new(penalty: T) -> Self {
        Self {
            penalty,
            penalisation: Penalisation::AllColumns,
        }
    }

    /// Exempts a column, or restores the penalty to all of them.
    pub fn with_penalisation(mut self, penalisation: Penalisation) -> Self {
        self.penalisation = penalisation;
        self
    }
}
