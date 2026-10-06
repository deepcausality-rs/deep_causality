/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the code-switching example.

use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lift_i128};

/// The side of the square torus giving code B, the `[[8,2,2]]` toric code.
pub const TORUS_SIDE: usize = 2;

/// How many machine epsilons a residual may carry and still be read as exact.
///
/// A noiseless switch has residual zero in exact arithmetic, so what the run measures is
/// accumulated rounding. The room it needs scales with the scalar, which is why the threshold is
/// counted in epsilons rather than fixed at a number that would suit one precision.
pub const EXACTNESS_EPSILONS: i128 = 4096;

/// The residual below which a square is read as exact, at the precision in force.
pub fn exactness_threshold<S>() -> S
where
    S: RealField + FromPrimitive,
{
    lift_i128::<S>(EXACTNESS_EPSILONS) * deep_causality_algebra::Real::epsilon()
}
