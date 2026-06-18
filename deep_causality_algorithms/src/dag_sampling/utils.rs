/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Small index-permutation helpers for the clique-picking machinery.
//!
//! Ported verbatim (counting-only part) from the authoritative
//! `cliquepicking_rs::utils`.

/// Returns the inverse of a permutation given as a slice.
///
/// For a permutation `p` (where `p[i]` is the element placed at position `i`),
/// the inverse `q` satisfies `q[p[i]] == i`. The input is assumed to be a valid
/// permutation of `0..permutation.len()`.
pub(crate) fn inverse_permutation(permutation: &[usize]) -> Vec<usize> {
    let mut inverse = vec![0; permutation.len()];
    for (i, &el) in permutation.iter().enumerate() {
        inverse[el] = i;
    }
    inverse
}
