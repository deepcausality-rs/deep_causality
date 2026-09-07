/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The log-sum-exp reduction, in its slice and two-term forms.

use deep_causality_algebra::Real;

/// `log Σ exp(xᵢ)`, computed through the maximum so the exponentials cannot overflow.
///
/// Total, with no error path: every input has an answer.
/// - The empty slice sums to zero and `log 0` is `−∞`, which is the identity for this reduction.
/// - A non-finite maximum saturates the result, because the shift `x − max` is undefined there.
pub fn log_sum_exp<T: Real>(values: &[T]) -> T {
    if values.is_empty() {
        // The empty sum is zero and log 0 is −∞, the identity of this reduction.
        return T::zero().ln();
    }

    let max = values
        .iter()
        .fold(values[0], |acc, &x| if x > acc { x } else { acc });

    // With a non-finite maximum the shift `x − max` is undefined, so the saturated maximum is the
    // only meaningful answer.
    if !max.is_finite() {
        return max;
    }

    let sum = values
        .iter()
        .fold(T::zero(), |acc, &x| acc + (x - max).exp());
    max + sum.ln()
}

/// `log(eᵃ + e^b)` for two terms, computed through the larger so neither exponential overflows.
///
/// The two-term case is separate because its caller has two scalars rather than a slice, and
/// routing it through the slice form would allocate.
pub fn log_add_exp<T: Real>(a: T, b: T) -> T {
    let (hi, lo) = if a > b { (a, b) } else { (b, a) };

    // Both −∞, or either +∞ or NaN: the shift below is undefined and the larger is the answer.
    if !hi.is_finite() {
        return hi;
    }

    // `lo − hi ≤ 0`, so the exponential is in `(0, 1]` and cannot overflow.
    hi + (T::one() + (lo - hi).exp()).ln()
}
