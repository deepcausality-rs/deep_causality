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
/// - A `NaN` anywhere propagates, whatever else the slice holds.
/// - A non-finite maximum saturates the result, because the shift `x − max` is undefined there.
///
/// # Why `NaN` is tested before the maximum
///
/// A sum over a set does not depend on the order of the set, and neither does its log. The
/// running maximum is not enough on its own to keep that: every comparison against a `NaN` is
/// false, so a `NaN` is dropped when it arrives after a larger value and *kept* when it arrives
/// first. With an infinity in the slice the two readings differ — `[NaN, +∞]` saturating on a
/// `NaN` maximum and `[+∞, NaN]` on an infinite one — and the reduction would answer `NaN` for one
/// ordering and `+∞` for the other. Testing for a `NaN` outright removes the dependence.
pub fn log_sum_exp<T: Real>(values: &[T]) -> T {
    if values.is_empty() {
        // The empty sum is zero and log 0 is −∞, the identity of this reduction.
        return T::zero().ln();
    }

    let mut saw_nan = false;
    let mut max = values[0];
    for &x in values {
        if x.is_nan() {
            saw_nan = true;
        } else if max.is_nan() || x > max {
            max = x;
        }
    }
    if saw_nan {
        return T::nan();
    }

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
///
/// `e^a + e^b = e^b + e^a`, so this is symmetric in its arguments, including at the non-finite
/// ones. The `NaN` test comes before the "pick the larger" step for the reason it does in
/// [`log_sum_exp`]: `a > b` is false when either is a `NaN`, so the larger of `(NaN, +∞)` is the
/// infinity and the larger of `(+∞, NaN)` is the `NaN`, and saturating on that would answer `+∞`
/// one way round and `NaN` the other.
pub fn log_add_exp<T: Real>(a: T, b: T) -> T {
    if a.is_nan() || b.is_nan() {
        return T::nan();
    }

    let (hi, lo) = if a > b { (a, b) } else { (b, a) };

    // Both −∞, or either +∞: the shift below is undefined and the larger is the answer.
    if !hi.is_finite() {
        return hi;
    }

    // `lo − hi ≤ 0`, so the exponential is in `(0, 1]` and cannot overflow.
    hi + (T::one() + (lo - hi).exp()).ln()
}
