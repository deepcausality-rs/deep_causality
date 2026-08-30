/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
//! The blanket that makes [`ScalarEval`](crate::ScalarEval) a facade rather than a definition.
//!
//! # Decision: `ScalarEval` stays, and is not a duplicate
//!
//! `add-linear-algebra-crate` surveyed this crate for operations duplicating one
//! `deep_causality_linear` provides. `ScalarEval` has the same three members as
//! `deep_causality_algebra::Normed` and is **not** one of them: this file is a single blanket
//! delegating every member to `Normed`, added only to attach the `Sum` bound on the real type. A
//! facade over the tower is the arrangement that survey wanted everywhere else, so removing it
//! would be removing the shape rather than the duplication.
//!
//! # What changed underneath it
//!
//! `MultiVectorL2Norm`'s impl was the last bound in `src` that named `ScalarEval`. It now names
//! `NormedScalar` instead — not because `ScalarEval` was wrong, but because a `T: ScalarEval`
//! bound does not let the compiler conclude `T: Normed`, and the shared norm asks for `Normed`.
//! Blanket impls carry that way and not back.
//!
//! So `ScalarEval` is now a public facade with no remaining caller inside this crate. It is still
//! exported, still covered by `tests/traits/scalar_eval_tests.rs`, and left in place: it is a
//! working part of the public surface, and this change is not the occasion to decide its future.

use crate::ScalarEval;
use core::iter::Sum;
use deep_causality_algebra::Normed;

// `ScalarEval` is the multivector-side facade over `deep_causality_algebra::Normed`. Every scalar with
// a real modulus, every real float and `Complex<T>`, satisfies `Normed`, so this single blanket
// covers them all.
impl<T> ScalarEval for T
where
    T: Normed,
    T::Real: Sum,
{
    type Real = T::Real;

    #[inline]
    fn modulus_squared(&self) -> Self::Real {
        Normed::modulus_squared(self)
    }

    #[inline]
    fn scale_by_real(&self, s: Self::Real) -> Self {
        Normed::scale_by_real(self, s)
    }
}
