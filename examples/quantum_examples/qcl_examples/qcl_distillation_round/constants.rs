/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the distillation-round example.

use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lift_i128};

/// The depolarising probabilities swept on every physical qubit of the encoded magic-state
/// preparation, as exact fractions. Zero is the noiseless round.
///
/// They are written as a numerator and a denominator and divided at the precision in force, rather
/// than as `f64` literals that are widened afterwards. `lift::<Float106>(0.01_f64)` is the `f64`
/// approximation of a hundredth carried into a wider type, which is a different number from the
/// hundredth that type can represent. Dividing the integers keeps every precision's probability its
/// own nearest value, which is what makes a comparison across precisions mean anything.
pub const NOISE_NUMERATORS: [i128; 3] = [0, 1, 5];
pub const NOISE_DENOMINATOR: i128 = 100;

/// How each probability is written when it is printed.
pub const NOISE_LABELS: [&str; 3] = ["0", "1/100", "5/100"];

/// The index of the probability the cross-precision comparison uses: the largest, where the
/// residual is furthest from zero and a difference between scalars has room to show.
pub const COMPARISON_INDEX: usize = 2;

/// The depolarising probabilities at the working precision.
pub fn noise_sweep<S>() -> [S; 3]
where
    S: RealField + FromPrimitive,
{
    let denominator = lift_i128::<S>(NOISE_DENOMINATOR);

    NOISE_NUMERATORS.map(|numerator| lift_i128::<S>(numerator) / denominator)
}
