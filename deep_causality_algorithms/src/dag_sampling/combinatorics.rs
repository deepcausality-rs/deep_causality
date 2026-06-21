/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Factorials and the `rho` inclusion-exclusion recurrence.
//!
//! Ported from the authoritative `cliquepicking_rs::combinatorics`, made generic
//! over the count type `T`. The reference counts in `num_bigint::BigUint`; this
//! crate is dependency-free, so the count type is instead a generic
//! `T: RealField + FromPrimitive` (e.g. `f64` or `deep_causality_num::Float106`).
//!
//! Two functions live here:
//! * [`factorial`] — `k! = ∏_{i=1}^{k} i`, memoized.
//! * [`rho`] — the recurrence from the Clique-Picking paper that counts orderings
//!   of a clique avoiding the forbidden separator-extension prefixes. It is an
//!   inclusion-exclusion that **subtracts** terms; for that reason the count must
//!   stay in linear (not log) space, so this port keeps the subtraction exactly
//!   as in the reference.

use crate::dag_sampling::memoization::Memoization;
use deep_causality_num::{FromPrimitive, RealField};

/// Converts a `usize` into the count type `T`. Counts arising here (small
/// integers and factorials) are representable in every supported `RealField`.
#[inline]
fn from_usize<T: FromPrimitive>(n: usize) -> T {
    <T as FromPrimitive>::from_usize(n).expect("count is representable in every RealField")
}

/// Returns `n!` as a value of `T`, memoized in `memoization` (`memoization[k]`
/// holds `k!` once computed; `None` means not yet computed).
pub(crate) fn factorial<T: RealField + FromPrimitive>(
    n: usize,
    memoization: &mut [Option<T>],
) -> T {
    if let Some(res) = memoization[n] {
        return res;
    }
    let mut result = T::one();
    for i in 1..n + 1 {
        result *= from_usize::<T>(i);
    }
    memoization[n] = Some(result);
    result
}

/// Evaluates the `rho` recurrence for the forbidden-size vector `x`, memoized by
/// `x` in `memoization.rho`.
///
/// Mirrors the reference exactly, including the **subtraction**:
/// `rho(x) = x[0]! - Σ_{i≥1} (x[0] - x[i])! * rho(x[i..])`.
/// The first element `x[0]` is assumed present (`x` is non-empty in every call
/// site).
pub(crate) fn rho<T: RealField + FromPrimitive>(
    x: &[usize],
    memoization: &mut Memoization<T>,
) -> T {
    let x_vec = x.to_vec();
    if let Some(res) = memoization.rho.get(&x_vec) {
        return *res;
    }
    let mut result = factorial(x[0], &mut memoization.factorial);
    for i in 1..x.len() {
        let lead = factorial(x[0] - x[i], &mut memoization.factorial);
        let tail = rho(&x[i..], memoization);
        result -= lead * tail;
    }
    memoization.rho.insert(x_vec, result);
    result
}
