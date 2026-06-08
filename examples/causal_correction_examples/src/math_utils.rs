/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared calculation helpers used across the counterfactual and corrective
//! intervention examples. Each helper is generic over the smallest set of
//! constraints needed by its callers and does not depend on any per-example
//! domain type.

use std::collections::HashSet;
use std::hash::Hash;

/// Arithmetic mean of an `f64` slice. Returns `NaN` for the empty slice.
pub fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        f64::NAN
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

/// Collect a `HashSet` into a sorted `Vec`. Useful for deterministic
/// display of an otherwise-unordered set.
pub fn sorted<T: Copy + Ord + Hash>(set: &HashSet<T>) -> Vec<T> {
    let mut v: Vec<T> = set.iter().copied().collect();
    v.sort_unstable();
    v
}
