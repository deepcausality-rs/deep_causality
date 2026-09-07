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
///
/// Dispatches to `deep_causality_stats::mean`, keeping the `NaN` this module's callers rely on:
/// the crate refuses an empty sample with a typed error, which is the right contract for a
/// statistic and the wrong one for a print helper that has to render something.
pub fn mean(xs: &[f64]) -> f64 {
    deep_causality_stats::mean(xs).unwrap_or(f64::NAN)
}

/// Collect a `HashSet` into a sorted `Vec`. Useful for deterministic
/// display of an otherwise-unordered set.
pub fn sorted<T: Copy + Ord + Hash>(set: &HashSet<T>) -> Vec<T> {
    let mut v: Vec<T> = set.iter().copied().collect();
    v.sort_unstable();
    v
}
