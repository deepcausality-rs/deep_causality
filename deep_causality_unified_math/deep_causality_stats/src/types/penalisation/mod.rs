/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Which coefficients a ridge penalty reaches.
///
/// Neither fit in this crate has an intercept of its own: `beta` is one coefficient per column of
/// the design, and a caller who wants an intercept supplies a column of ones. That leaves the
/// question of whether the penalty applies to that column, and the answer is not the same for every
/// caller — so it is a parameter rather than a convention baked into the fit.
///
/// # Why the choice matters
///
/// Shrinking an intercept toward zero shrinks the fitted odds toward even, so a penalised intercept
/// discards the base rate. On a six-row logistic gate with a base rate of ⅓ and a penalty of 5, the
/// fitted probabilities span `0.12 … 0.59` when the intercept is exempt and collapse to
/// `0.44 … 0.60` when it is not — a difference of up to 0.32 in absolute probability, centred on
/// ½ rather than on the base rate. The exempt fit is also *equivariant*: shifting every label, or
/// re-balancing the classes, moves the intercept and leaves the slopes alone. The penalised fit
/// couples the two.
///
/// Exempting the intercept is the default in `sklearn`, `statsmodels` and `glmnet`. It is not the
/// default here, because this crate does not know which column the caller meant as the intercept —
/// [`AllColumns`](Self::AllColumns) is the honest answer when nothing has been said.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Penalisation {
    /// Every column is penalised. Correct for a design that carries no intercept, and the only
    /// answer available when the fit is not told which column is one.
    #[default]
    AllColumns,
    /// Every column except the one at this index.
    ///
    /// Named by position rather than by a `fit_intercept` flag because the intercept's position is
    /// the caller's choice: nothing here requires it to be column 0.
    Excluding(usize),
}

impl Penalisation {
    /// Whether the penalty reaches the coefficient at `column`.
    pub fn reaches(&self, column: usize) -> bool {
        match self {
            Self::AllColumns => true,
            Self::Excluding(exempt) => column != *exempt,
        }
    }

    /// The exempt column, if there is one.
    pub fn exempt(&self) -> Option<usize> {
        match self {
            Self::AllColumns => None,
            Self::Excluding(exempt) => Some(*exempt),
        }
    }

    /// Whether the exemption names a column that a design of this width actually has.
    ///
    /// An index at or past the width is a caller error rather than a silent no-op: it means the
    /// design does not have the column the caller believes carries the intercept. The fits check
    /// this and refuse, rather than penalising everything and returning a plausible answer.
    pub fn fits_width(&self, columns: usize) -> bool {
        match self {
            Self::AllColumns => true,
            Self::Excluding(exempt) => *exempt < columns,
        }
    }
}
