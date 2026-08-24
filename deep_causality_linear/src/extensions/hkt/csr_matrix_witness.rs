/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::csr_matrix::CsrMatrix;
use deep_causality_haft::{HKT, NoConstraint};

/// The higher-kinded witness for [`CsrMatrix`].
///
/// Moves here from `deep_causality_sparse` unchanged, so that its trait impls and their results are
/// identical to what they were before the move.
///
/// # `fmap` maps the stored entries
///
/// A sparse matrix stores only its non-zeros, so a function that does not fix zero changes which
/// entries are structurally present. This witness maps the **stored** entries and leaves the
/// structural zeros alone, which keeps the result sparse. A caller who wants a function applied to
/// the whole logical matrix densifies first, and that conversion is explicit.
///
/// The shape is preserved either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CsrMatrixWitness;

impl HKT for CsrMatrixWitness {
    type Constraint = NoConstraint;
    type Type<T> = CsrMatrix<T>;
}

// `Functor`, `Foldable`, `Pure`, `Applicative`, `Monad`, `CoMonad` and `Adjunction` move with it.
