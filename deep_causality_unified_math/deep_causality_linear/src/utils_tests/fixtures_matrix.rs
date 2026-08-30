/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Matrices whose answers are known independently of this crate.

use alloc::vec;
use alloc::vec::Vec;

/// A 3×3 matrix of rank 2: the third row is the sum of the first two.
///
/// Rank is 2 by construction rather than by computation, so a test asserting it is checking the
/// implementation rather than agreeing with it.
pub fn rank_deficient_3x3() -> (Vec<f64>, usize, usize) {
    (vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 5.0, 7.0, 9.0], 3, 3)
}

/// A 3×3 matrix with determinant 1, in row-major order.
///
/// Upper unitriangular, so the determinant is the product of the diagonal.
pub fn unit_determinant_3x3() -> (Vec<f64>, usize, usize) {
    (vec![1.0, 2.0, 3.0, 0.0, 1.0, 4.0, 0.0, 0.0, 1.0], 3, 3)
}

/// A non-singular 3×3 matrix whose `(0, 0)` entry is zero.
///
/// The shape that separates an elimination which searches for its pivot from one that takes the
/// diagonal. Determinant is `-1`.
pub fn zero_leading_entry_3x3() -> (Vec<f64>, usize, usize) {
    (vec![0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0], 3, 3)
}

/// A 2×2 matrix whose first pivot candidate is near zero with a much larger one below it.
///
/// Eliminating on the `1e-18` entry loses the second row entirely to rounding; pivoting on magnitude
/// does not. Determinant is `1e-18 - 1.0`, which is `-1.0` in `f64`.
pub fn near_zero_pivot_2x2() -> (Vec<f64>, usize, usize) {
    (vec![1e-18, 1.0, 1.0, 1.0], 2, 2)
}

/// A singular 2×2 matrix: the second row is twice the first.
pub fn singular_2x2() -> (Vec<f64>, usize, usize) {
    (vec![1.0, 2.0, 2.0, 4.0], 2, 2)
}

/// An integer matrix of rank 2 whose entries are in `{-1, 0, 1}`.
///
/// The alphabet of `deep_causality_topology`'s boundary operators.
pub fn boundary_alphabet_3x3() -> (Vec<i64>, usize, usize) {
    (vec![1, -1, 0, 0, 1, -1, 1, 0, -1], 3, 3)
}

/// An integer matrix whose rank over ℚ is 3 and whose rank over 𝔽₂ is 2.
///
/// The cyclic `{-1, 0, 1}` incidence pattern. Its determinant is `2`: non-zero over ℚ, so full rank
/// there, and zero mod 2, so the rank drops to 2 over 𝔽₂. The dependency that appears is
/// `r₁ + r₂ + r₃ ≡ 0 (mod 2)` — an even-weight dependency, invisible over ℚ.
///
/// This is the divergence `qcl-gaps.md` G-02 records: a complex carrying one of these reports a
/// wrong `k` when the ℝ-rank stands in for the 𝔽₂-rank. Verified against exact rational and exact
/// mod-2 elimination rather than against this crate.
pub fn ranks_disagree_3x3() -> (Vec<i64>, usize, usize) {
    (vec![1, 1, 0, 0, 1, 1, 1, 0, 1], 3, 3)
}

/// The rank of [`ranks_disagree_3x3`] over ℚ, and equally over ℤ.
pub const RANKS_DISAGREE_RATIONAL_RANK: usize = 3;

/// The rank of [`ranks_disagree_3x3`] over 𝔽₂.
pub const RANKS_DISAGREE_GF2_RANK: usize = 2;

/// The determinant of [`ranks_disagree_3x3`], which is `2` and therefore `0` mod 2.
pub const RANKS_DISAGREE_DETERMINANT: i64 = 2;

/// A 4×4 integer matrix with determinant **5**, whose Gaussian elimination leaves ℤ immediately.
///
/// The tridiagonal matrix with `2` on the diagonal and `1` beside it. Its determinant follows
/// `Dₙ = 2Dₙ₋₁ − Dₙ₋₂` from `D₁ = 2`, giving `n + 1`, so `D₄ = 5`. Dividing by the first pivot
/// produces `3/2` immediately; fraction-free elimination keeps every intermediate an integer and
/// reaches the same answer.
pub fn integer_determinant_4x4() -> (Vec<i64>, usize, usize) {
    (vec![2, 1, 0, 0, 1, 2, 1, 0, 0, 1, 2, 1, 0, 0, 1, 2], 4, 4)
}

/// The determinant of [`integer_determinant_4x4`].
pub const INTEGER_DETERMINANT_4X4: i64 = 5;

/// The determinant of [`unit_determinant_3x3`].
pub const UNIT_DETERMINANT_3X3: f64 = 1.0;

/// The determinant of [`zero_leading_entry_3x3`], which is where an unpivoted elimination fails.
pub const ZERO_LEADING_ENTRY_DETERMINANT: f64 = -1.0;

/// The rank of [`rank_deficient_3x3`].
pub const RANK_DEFICIENT_3X3_RANK: usize = 2;

/// The rank of [`boundary_alphabet_3x3`].
pub const BOUNDARY_ALPHABET_3X3_RANK: usize = 2;
