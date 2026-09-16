/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the code-switching example.

use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lift_i128};

/// The depolarising probability of the noisy gadget, as an exact fraction: one tenth, applied to
/// the first logical wire between the decoder of code A and the encoder of code B.
///
/// It is written as a numerator and a denominator and divided at the precision in force, rather
/// than as an `f64` literal that is widened afterwards. `lift::<Float106>(0.1_f64)` is the `f64`
/// approximation of a tenth carried into a wider type, which is a different number from the tenth
/// that type can represent, so the cross-precision rows would be comparing arithmetic on three
/// slightly different probabilities.
pub const GADGET_NOISE_NUMERATOR: i128 = 1;
pub const GADGET_NOISE_DENOMINATOR: i128 = 10;

/// How the probability is written when it is printed.
pub const GADGET_NOISE_LABEL: &str = "1/10";

/// The side of the square torus giving code B, the `[[8,2,2]]` toric code.
pub const TORUS_SIDE: usize = 2;

/// How many machine epsilons a residual may carry and still be read as exact.
///
/// A noiseless switch has residual zero in exact arithmetic, so what the run measures is
/// accumulated rounding. The room it needs scales with the scalar, which is why the threshold is
/// counted in epsilons rather than fixed at a number that would suit one precision.
pub const EXACTNESS_EPSILONS: i128 = 4096;

/// The depolarising probability at the working precision.
pub fn gadget_noise<S>() -> S
where
    S: RealField + FromPrimitive,
{
    lift_i128::<S>(GADGET_NOISE_NUMERATOR) / lift_i128::<S>(GADGET_NOISE_DENOMINATOR)
}

/// The residual below which a square is read as exact, at the precision in force.
pub fn exactness_threshold<S>() -> S
where
    S: RealField + FromPrimitive,
{
    lift_i128::<S>(EXACTNESS_EPSILONS) * deep_causality_algebra::Real::epsilon()
}
