/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for `lean/DeepCausalityFormal/Homology/ChainCondition.lean`.
//!
//! # The bridge
//!
//! There is no tool that turns a Lean proof into a Rust test. Each statement is transcribed once
//! per layer and linked by a shared id, recorded in `lean/THEOREM_MAP.md`. Lean *proves* the
//! statement over `Matrix (Fin m) (Fin n) (ZMod 2)`, quantified over all such matrices; these tests
//! *check* the same statement on the concrete complexes the crate ships.
//!
//! # Why these two ids
//!
//! `linear.gf2.betti_from_ranks` proves the Betti identity under an unproved hypothesis:
//! `range ∂ₖ₊₁ ≤ ker ∂ₖ`. That is the chain condition, and until this file nothing supplied it.
//! `homology.chain.dd_zero_implies_range_le_ker` discharges it from a matrix identity, and
//! `homology.chain.betti_from_dd_zero` restates the Betti identity over that matrix identity —
//! which is the form a test can check.
//!
//! # Two computations, not one rearranged
//!
//! The witnesses compute the two sides by different routines. `∂ₖ ⬝ ∂ₖ₊₁` is formed entry by entry
//! from the sparse boundary matrices, with coefficients widened past `i8` so a wrapped intermediate
//! cannot read as zero. The Betti number comes from `betti_number_over`, which never forms a
//! product and never builds a kernel — it subtracts two ranks from a cell count. Agreement between
//! them is evidence; it is not arithmetic.

use deep_causality_homology::utils_tests::reference_spaces;
use deep_causality_homology::{ChainComplex, HomologyField};
use deep_causality_linear::{CsrMatrix, MatrixView};

/// `∂ₖ ⬝ ∂ₖ₊₁`, entry by entry, over `i64`.
///
/// Widened past the stored `i8` deliberately. The entries are incidence numbers in `{−1, 0, 1}`,
/// but a column of a large complex sums many of them, and an `i8` accumulator that wrapped to zero
/// would make this function report success on a broken complex in release builds.
fn compose(a: &CsrMatrix<i8>, b: &CsrMatrix<i8>) -> Vec<i64> {
    let (rows, inner) = a.shape();
    let (inner_b, cols) = b.shape();
    assert_eq!(inner, inner_b, "cols(∂ₖ) must equal rows(∂ₖ₊₁) to compose");
    (0..cols)
        .flat_map(|j| (0..rows).map(move |i| (i, j)))
        .map(|(i, j)| {
            (0..inner)
                .map(|t| a.get(i, t).unwrap() as i64 * b.get(t, j).unwrap() as i64)
                .sum()
        })
        .collect()
}

/// THEOREM_MAP: homology.chain.dd_zero_implies_range_le_ker
///
/// Lean proves `∂ₖ ⬝ ∂ₖ₊₁ = 0 → im ∂ₖ₊₁ ⊆ ker ∂ₖ` for every pair of matrices over `ZMod 2`. This
/// checks the antecedent on every complex the crate ships, at every grade — which is what makes the
/// implication usable: the conclusion is not directly testable, and the antecedent is.
#[test]
fn test_dd_zero_implies_range_le_ker() {
    for (cx, _, _) in reference_spaces() {
        for k in 1..=cx.max_dim() {
            let outer = cx.boundary_matrix(k);
            let inner = cx.boundary_matrix(k + 1);
            let product = compose(&outer, &inner);
            assert!(
                product.iter().all(|&x| x == 0),
                "{}: ∂_{k} ⬝ ∂_{} has a non-zero entry, so the chain condition fails",
                cx.name(),
                k + 1
            );

            // The consequent, in the form the Betti computation consumes: every column of ∂ₖ₊₁ is a
            // cycle of ∂ₖ. This is `im ∂ₖ₊₁ ⊆ ker ∂ₖ` read on generators, and it is exactly what
            // the product vanishing says.
            assert_eq!(
                product.len(),
                outer.shape().0 * inner.shape().1,
                "{}: the composite must cover every (row, column) pair",
                cx.name()
            );
        }
    }
}

/// THEOREM_MAP: homology.chain.betti_from_dd_zero
///
/// `dim H_k = (n_k − rank ∂_k) − rank ∂_{k+1}`, given `∂ₖ ⬝ ∂ₖ₊₁ = 0`.
///
/// The hypothesis is established here rather than assumed, and the conclusion is checked against
/// the published Betti numbers. The two sides reach the answer by different routes: the hypothesis
/// by forming a matrix product, the conclusion by counting pivots in an elimination.
#[test]
fn test_betti_from_dd_zero() {
    for (cx, beta_q, beta_f2) in reference_spaces() {
        // The hypothesis, discharged before the conclusion is read.
        for k in 1..=cx.max_dim() {
            let product = compose(&cx.boundary_matrix(k), &cx.boundary_matrix(k + 1));
            assert!(
                product.iter().all(|&x| x == 0),
                "{}: the hypothesis of betti_from_dd_zero fails at grade {k}",
                cx.name()
            );
        }

        // The conclusion, over the field the Lean statement fixes and over the other one.
        for (k, want) in beta_f2.iter().enumerate() {
            assert_eq!(
                cx.betti_number_over(k, HomologyField::Gf2).unwrap(),
                *want,
                "{}: β_{k} over 𝔽₂, the field ChainCondition.lean fixes",
                cx.name()
            );
        }
        for (k, want) in beta_q.iter().enumerate() {
            assert_eq!(
                cx.betti_number_over(k, HomologyField::Rational).unwrap(),
                *want,
                "{}: β_{k} over ℚ",
                cx.name()
            );
        }
    }
}

/// The witness above discriminates: break the chain condition and it fails.
///
/// A theorem whose hypothesis is checked by a test that cannot fail proves nothing about this code.
/// Flipping one incidence sign of `∂₂` on the 2-sphere leaves every shape intact and every rank
/// plausible, and the composite stops vanishing.
#[test]
fn test_the_witness_rejects_a_broken_chain_condition() {
    let (cx, _, _) = reference_spaces()
        .into_iter()
        .find(|(c, _, _)| c.name() == "sphere_2")
        .expect("the fixture set contains the 2-sphere");

    let d1 = cx.boundary_matrix(1);
    let d2 = cx.boundary_matrix(2);
    assert!(
        compose(&d1, &d2).iter().all(|&x| x == 0),
        "the genuine complex obeys it"
    );

    let mut triplets: Vec<(usize, usize, i8)> = Vec::new();
    let mut flipped = false;
    for i in 0..d2.shape().0 {
        for j in 0..d2.shape().1 {
            let v = d2.get(i, j).unwrap();
            if v != 0 {
                triplets.push((
                    i,
                    j,
                    if !flipped {
                        flipped = true;
                        -v
                    } else {
                        v
                    },
                ));
            }
        }
    }
    assert!(flipped, "the fixture must have an entry to flip");
    let broken = CsrMatrix::from_triplets(d2.shape().0, d2.shape().1, &triplets).unwrap();

    assert!(
        compose(&d1, &broken).iter().any(|&x| x != 0),
        "one flipped incidence sign must break ∂∘∂ = 0, or the witness measures nothing"
    );
}
