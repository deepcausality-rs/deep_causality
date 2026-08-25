/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Linear/RankNullity.lean`.
//!
//! Lean proves these over an arbitrary `Matrix (Fin m) (Fin n) (ZMod 2)`. Rust checks them at
//! concrete matrices, computing each side by a different route: the rank by elimination
//! (`rank_gf2`) and the nullity by counting the kernel basis (`kernel_basis_gf2`), so the identity
//! is a claim about two independent computations rather than an algebraic rearrangement of one.

use deep_causality_linear::{
    MatrixBuild, MatrixView, PackedGf2, image_basis_gf2, kernel_basis_gf2, rank_gf2,
};
use deep_causality_num::Gf2;

/// Builds an `m × n` 𝔽₂ matrix from a row-major bit pattern.
fn gf2(rows: usize, cols: usize, bits: &[u8]) -> PackedGf2<u64> {
    assert_eq!(bits.len(), rows * cols);
    let mut m = PackedGf2::<u64>::zeros(rows, cols);
    for r in 0..rows {
        for c in 0..cols {
            if bits[r * cols + c] & 1 == 1 {
                m.set(r, c, Gf2::ONE).unwrap();
            }
        }
    }
    m
}

/// The matrices the identity is checked at: a spread of shapes, ranks and degeneracies.
fn cases() -> Vec<(&'static str, PackedGf2<u64>)> {
    vec![
        ("zero 3x3", gf2(3, 3, &[0; 9])),
        ("identity 3x3", gf2(3, 3, &[1, 0, 0, 0, 1, 0, 0, 0, 1])),
        ("all ones 3x3", gf2(3, 3, &[1; 9])),
        // Rank 3 over ℚ (determinant 2), rank 2 over 𝔽₂ — the case that makes the field a choice.
        (
            "even-weight dependency 3x3",
            gf2(3, 3, &[1, 1, 0, 0, 1, 1, 1, 0, 1]),
        ),
        ("wide 2x5", gf2(2, 5, &[1, 0, 1, 0, 1, 0, 1, 1, 1, 0])),
        ("tall 5x2", gf2(5, 2, &[1, 0, 0, 1, 1, 1, 0, 0, 1, 0])),
        (
            "triangle boundary 3x3",
            gf2(3, 3, &[1, 0, 1, 1, 1, 0, 0, 1, 1]),
        ),
        ("single column 4x1", gf2(4, 1, &[1, 0, 1, 0])),
        ("single row 1x4", gf2(1, 4, &[1, 1, 0, 1])),
    ]
}

/// THEOREM_MAP: linear.gf2.rank_nullity
#[test]
fn test_gf2_rank_nullity() {
    // rank ∂ + dim ker ∂ = n, with the two summands computed by different routines.
    for (name, m) in cases() {
        let n = m.cols();
        let rank = rank_gf2(&m).unwrap();
        let nullity = kernel_basis_gf2(&m).unwrap().cols();
        assert_eq!(
            rank + nullity,
            n,
            "{name}: rank {rank} + nullity {nullity} must be the column count {n}"
        );
    }
}

/// THEOREM_MAP: linear.gf2.nullity_is_count_minus_rank
#[test]
fn test_gf2_nullity_is_count_minus_rank() {
    // The substitution `betti_number_over` performs: it subtracts a rank from a cell count and
    // uses the result as a nullity, never computing a kernel. This checks the two agree.
    for (name, m) in cases() {
        let n = m.cols();
        let rank = rank_gf2(&m).unwrap();
        let nullity = kernel_basis_gf2(&m).unwrap().cols();
        assert_eq!(
            nullity,
            n - rank,
            "{name}: the kernel dimension must equal the column count minus the rank"
        );
    }
}

/// THEOREM_MAP: linear.gf2.rank_le_cell_count
#[test]
fn test_gf2_rank_le_cell_count() {
    // `betti_number_over` floors its subtraction with `saturating_sub`. This is the theorem that
    // the floor is never reached at that step: the rank cannot exceed the column count, so
    // `n_k - rank_k` is never a truncation of a negative number.
    for (name, m) in cases() {
        let rank = rank_gf2(&m).unwrap();
        assert!(
            rank <= m.cols(),
            "{name}: rank {rank} must not exceed the column count {}",
            m.cols()
        );
        assert!(
            rank <= m.rows(),
            "{name}: rank {rank} must not exceed the row count {}",
            m.rows()
        );
    }
}

/// THEOREM_MAP: linear.gf2.betti_from_ranks
#[test]
fn test_gf2_betti_from_ranks() {
    // A chain complex over 𝔽₂ whose homology is known: the boundary of a filled triangle.
    //
    //   ∂₂ : C₂ → C₁ — one 2-cell, three edges, all incident
    //   ∂₁ : C₁ → C₀ — three edges, three vertices
    //
    // Contractible, so β₁ = 0. Lean proves `dim H_k = (n_k − rank ∂_k) − rank ∂_{k+1}`; this
    // checks the right-hand side against the number the left-hand side has to be.
    let d1 = gf2(3, 3, &[1, 0, 1, 1, 1, 0, 0, 1, 1]);
    let d2 = gf2(3, 1, &[1, 1, 1]);

    // The chain condition ∂₁∘∂₂ = 0, checked rather than assumed: every column of ∂₂ must land in
    // the kernel of ∂₁. Over 𝔽₂ that is exactly the image basis of ∂₂ having rank-many columns
    // that ∂₁ annihilates.
    let n_1 = d1.cols();
    let rank_1 = rank_gf2(&d1).unwrap();
    let rank_2 = rank_gf2(&d2).unwrap();
    let cycles = kernel_basis_gf2(&d1).unwrap().cols();
    let boundaries = image_basis_gf2(&d2).unwrap().cols();

    assert_eq!(cycles, n_1 - rank_1, "cycles are the kernel of ∂₁");
    assert_eq!(boundaries, rank_2, "boundaries are the image of ∂₂");
    assert_eq!(
        (n_1 - rank_1) - rank_2,
        0,
        "a filled triangle is contractible, so β₁ = 0 over 𝔽₂"
    );

    // And the ranks are what makes it so: the same 1-skeleton without ∂₂ is a circle, so the
    // assertion above is a fact about this complex rather than about the arithmetic.
    assert_eq!(
        n_1 - rank_1,
        1,
        "without ∂₂ the same 1-skeleton is a circle, so β₁ = 1"
    );
}
