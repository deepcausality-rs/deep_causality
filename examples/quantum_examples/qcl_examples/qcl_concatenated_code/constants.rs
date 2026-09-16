/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the concatenated-code example.

use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lift_i128};

/// How many machine epsilons a residual may carry and still be read as exact.
///
/// A transversal Pauli has residual zero in exact arithmetic, so what the run measures is
/// accumulated rounding and nothing else. The room it needs therefore scales with the scalar: a
/// single fixed threshold is either loose enough to hide a real residual at `Float106` or tight
/// enough to reject honest rounding at `f32`, and naming it after one concrete type — as
/// `EXACT_AT_F64` did — states which of the two the reader is getting.
pub const EXACTNESS_EPSILONS: i128 = 4096;

/// The residual below which a square is read as exact, at the precision in force.
pub fn exactness_threshold<S>() -> S
where
    S: RealField + FromPrimitive,
{
    lift_i128::<S>(EXACTNESS_EPSILONS) * deep_causality_algebra::Real::epsilon()
}
