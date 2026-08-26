/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stirling numbers, the two triangles that change basis between powers and
//! products over a set.
//!
//! They belong here because both are counts: values in ℕ, reached by a
//! recurrence that only ever adds and multiplies. The signs that appear where
//! these are used belong to the identity being applied, not to the numbers, so
//! this module carries none.
//!
//! # The caller supplies the working row
//!
//! This crate allocates nowhere, and these functions do not change that. Both
//! recurrences need one row of `k + 1` values, and both take it as a `scratch`
//! slice rather than reaching for a `Vec`. A caller with an allocator can pass
//! a vector's slice; one without can pass a stack array. Its contents on entry
//! are ignored and on return are unspecified.
//!
//! A fixed internal array was the alternative and was rejected. Any cap would
//! be a real limit rather than a formality: `S(n, n−1) = C(n, 2)`, so
//! `S(200, 199) = 19900` is perfectly representable and needs a 200-wide row.
//!
//! # Where they are needed
//!
//! Haruna, *Note on Logical Gates by Gauge Field Formalism of Quantum Error
//! Correction* (arXiv:2511.15224), Appendix A. With `a(γ) = Σ_i γ^i z_i` a sum
//! of commuting diagonal operators, A.12 expands a power of it over the
//! elementary products:
//!
//! ```text
//! a(γ)^m = Σ_{r=1}^{m} r!·S(m,r) · Σ_{i₁<⋯<i_r} γ^{i₁}⋯γ^{i_r} z_{i₁}⋯z_{i_r}
//! ```
//!
//! and A.14 inverts it:
//!
//! ```text
//! Σ_{i₁<⋯<i_r} γ^{i₁}⋯γ^{i_r} z_{i₁}⋯z_{i_r} = (1/r!)·Σ_{m=1}^{r} (−1)^{m+r}·s(r,m)·a(γ)^m
//! ```
//!
//! The paper defines its `s(r,m)` as *the number of permutations of r elements
//! with exactly m disjoint cycles*, which is the **unsigned** first-kind
//! number, with the sign written separately as `(−1)^{m+r}`. So
//! [`stirling_first_unsigned`] is what A.14 asks for; passing it a signed
//! convention would count the sign twice.

use crate::{FromPrimitive, NaturalNumber};

/// Stirling numbers of the second kind, `S(n, k)`.
///
/// The number of ways to partition a set of `n` elements into exactly `k`
/// non-empty subsets. `S(0, 0) = 1`, `S(n, 0) = 0` for `n > 0`, and
/// `S(n, k) = 0` for `k > n`.
///
/// From `S(n, k) = k·S(n−1, k) + S(n−1, k−1)`, over one row: `O(n·k)`
/// operations, no allocation.
///
/// # Errors
///
/// `None` when `scratch` is shorter than `k + 1`, and `None` when a value on
/// the way to the answer leaves the range of `N`. The growth is steep —
/// `S(n, 2) = 2^{n−1} − 1` alone leaves `u64` by `n = 65` — and an intermediate
/// can overflow where the requested value would not, so a `None` is not a
/// statement that `S(n, k)` is unrepresentable.
pub fn stirling_second<N>(n: usize, k: usize, scratch: &mut [N]) -> Option<N>
where
    N: NaturalNumber + FromPrimitive + Copy,
{
    if k > n {
        return Some(N::zero());
    }
    if scratch.len() < k + 1 {
        return None;
    }
    let row = &mut scratch[..=k];
    // Row `n = 0` is `[1, 0, 0, …]`: the empty set has one partition into no
    // parts and none into any positive number of them.
    row[0] = N::one();
    for cell in row.iter_mut().skip(1) {
        *cell = N::zero();
    }
    for _ in 1..=n {
        // Descending, so each write still reads the previous row at `j − 1`.
        for j in (1..=k).rev() {
            let scale = N::from_usize(j)?;
            let stay = row[j].checked_mul(scale)?;
            row[j] = stay.checked_add(row[j - 1])?;
        }
        row[0] = N::zero();
    }
    Some(row[k])
}

/// Unsigned Stirling numbers of the first kind, `c(n, k)`.
///
/// The number of permutations of `n` elements with exactly `k` disjoint cycles,
/// equal to `|s(n, k)|` under the signed convention. This is what Haruna's A.14
/// writes as `s(r, m)`, that identity carrying its own `(−1)^{m+r}`.
///
/// `c(0, 0) = 1`, `c(n, 0) = 0` for `n > 0`, and `c(n, k) = 0` for `k > n`.
///
/// From `c(n, k) = (n−1)·c(n−1, k) + c(n−1, k−1)`, the same shape as the second
/// kind with the multiplier read from the row rather than the column.
///
/// # Errors
///
/// As [`stirling_second`]. `c(n, 1) = (n−1)!`, so this leaves `u64` by `n = 22`.
pub fn stirling_first_unsigned<N>(n: usize, k: usize, scratch: &mut [N]) -> Option<N>
where
    N: NaturalNumber + FromPrimitive + Copy,
{
    if k > n {
        return Some(N::zero());
    }
    if scratch.len() < k + 1 {
        return None;
    }
    let row = &mut scratch[..=k];
    row[0] = N::one();
    for cell in row.iter_mut().skip(1) {
        *cell = N::zero();
    }
    for i in 1..=n {
        let scale = N::from_usize(i - 1)?;
        for j in (1..=k).rev() {
            let stay = row[j].checked_mul(scale)?;
            row[j] = stay.checked_add(row[j - 1])?;
        }
        // `c(n, 0)` is zero for every `n > 0`, which the first step below sets
        // and every later step preserves by multiplying by `n − 1`.
        row[0] = row[0].checked_mul(scale)?;
    }
    Some(row[k])
}
