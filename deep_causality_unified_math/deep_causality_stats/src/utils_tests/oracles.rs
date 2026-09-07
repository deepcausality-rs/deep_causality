/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Independent oracles: a second algorithm for a quantity a suite also computes the
//! ordinary way, so an assertion pins the mathematics rather than the implementation.

use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lift};

/// The corrected variance computed as `Σ_{i<j}(xᵢ − xⱼ)² / (n(n − 1))`.
///
/// This is the "demonstrably different algorithm" of the anti-circularity allow-list, not the
/// definition reordered. It never forms the mean, never forms a deviation from the mean, and is
/// quadratic where the definition is linear. Lagrange's identity supplies the equality:
///
/// ```text
/// Σ_{i<j} (xᵢ − xⱼ)²  =  n·Σᵢ xᵢ²  −  (Σᵢ xᵢ)²  =  n · Σᵢ (xᵢ − x̄)²
/// ```
///
/// so `Σᵢ(xᵢ − x̄)² / (n − 1) = Σ_{i<j}(xᵢ − xⱼ)² / (n(n − 1))`.
///
/// Checked by hand on `[1, 2, 3]`: the pair differences are 1, 2 and 1, whose squares sum to
/// `1 + 4 + 1 = 6`, over `n(n − 1) = 6`, giving 1. The definition agrees — mean 2, squared
/// deviations `1 + 0 + 1 = 2`, over `n − 1 = 2`, giving 1.
pub fn pairwise_variance_oracle<T: RealField + FromPrimitive>(xs: &[T]) -> T {
    let n = xs.len();
    let mut acc = lift::<T>(0.0);
    for (i, &xi) in xs.iter().enumerate() {
        for &xj in xs.iter().skip(i + 1) {
            let d = xi - xj;
            acc += d * d;
        }
    }
    acc / lift::<T>((n * (n - 1)) as f64)
}
