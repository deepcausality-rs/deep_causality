/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Column-to-row projection shared by the BRCD driver and the BOSS bootstrap.
//!
//! **Kept out of `deep_causality_linear`** (unified-math-next task 6.10). The names say transpose,
//! but these select a subset of columns and gather them into rows — an index projection over a
//! column store, with no arithmetic at all. There is no linear algebra here to move, so the
//! de-duplication lands *inside* `algorithms`: `brcd_algo.rs` and `brcd_boss_bootstrap.rs` each
//! carried a verbatim copy of both functions, and both now call these.

use deep_causality_algebra::RealField;

/// Builds `n` parent feature rows from the chosen continuous columns.
pub(super) fn transpose<T: RealField>(columns: &[Vec<T>], idxs: &[usize], n: usize) -> Vec<Vec<T>> {
    if idxs.is_empty() {
        return Vec::new();
    }
    (0..n)
        .map(|i| idxs.iter().map(|&c| columns[c][i]).collect())
        .collect()
}

/// Builds `n` parent configuration rows from the chosen integer columns.
pub(super) fn transpose_int(columns: &[Vec<usize>], idxs: &[usize], n: usize) -> Vec<Vec<usize>> {
    if idxs.is_empty() {
        return Vec::new();
    }
    (0..n)
        .map(|i| idxs.iter().map(|&c| columns[c][i]).collect())
        .collect()
}
