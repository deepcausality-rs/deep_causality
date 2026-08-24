/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::dense_matrix::DenseMatrix;
use deep_causality_haft::{HKT, NoConstraint};

/// The higher-kinded witness for [`DenseMatrix`].
///
/// A witness is a zero-sized stand-in for the type constructor `DenseMatrix<_>`, which Rust cannot
/// name directly. Every `deep_causality_haft` trait is implemented on the witness rather than on the
/// container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DenseMatrixWitness;

impl HKT for DenseMatrixWitness {
    type Constraint = NoConstraint;
    type Type<T> = DenseMatrix<T>;
}

// `Functor`, `Foldable`, `Pure`, `Applicative`, `Monad`, `CoMonad` and `Adjunction` follow, matching
// `CsrMatrixWitness` member for member.
//
// Two of them carry a shape decision that the laws do not settle, and which the tests pin:
//
//   `pure` has to choose a shape for a single value. `CsrMatrixWitness` makes a 1x1. This does the
//   same, so that a value round-tripping through `pure` then `extract` is unchanged and the monad
//   identities hold at a shape both sides agree on.
//
//   `extract` has to choose which entry of a matrix is "the" one. It is the (0, 0) entry, and it
//   panics on an empty matrix — a comonad has no counit for an empty container, and returning a
//   fabricated zero would break `extend(extract) == id`.
