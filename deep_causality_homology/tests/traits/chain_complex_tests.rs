/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The chain-complex trait, against published homology.
//!
//! # Where the expected values come from
//!
//! Hatcher, *Algebraic Topology*, by way of `utils_tests::reference_spaces`, which carries the
//! citation per space. `openspec/notes/homology/reference/reference.py` builds the same ten spaces
//! independently in Python and checks *its* output against the same source. No expectation here is
//! a reading of this crate.
//!
//! # Suite audit, run before the implementation was trusted
//!
//! **Not a tautology.** Six of the nine fixtures have a non-zero Betti number above grade 0, so an
//! implementation returning zero everywhere fails. Two — `ℝP²` and the Klein bottle — give
//! *different* answers over ℚ and over 𝔽₂, so an implementation ignoring the field argument fails
//! too. Every complex the workspace shipped before this crate is torsion-free, and a suite built
//! only on those would have passed either way.
//!
//! **Not circular.** The Euler characteristic is computed twice: once from cell counts, which never
//! reach the rank routine, and once from Betti numbers, which are nothing but ranks. The two agree
//! by a theorem, so the agreement is evidence rather than arithmetic.
//!
//! **A range, not a point.** Grades 0 through 3, dimensions 0 through 2, orientable and
//! non-orientable, with and without boundary, from 1 cell to 96.

use deep_causality_homology::utils_tests::{SimplicialFixture, reference_spaces};
use deep_causality_homology::{ChainComplex, HomologyField};
use deep_causality_linear::{CsrMatrix, MatrixView};

/// Betti numbers over ℚ match the published values, at every grade of every space.
#[test]
fn test_rational_betti_numbers_match_the_published_values() {
    for (cx, beta_q, _) in reference_spaces() {
        for (k, want) in beta_q.iter().enumerate() {
            let got = cx.betti_number_over(k, HomologyField::Rational).unwrap();
            assert_eq!(got, *want, "{}: β_{k} over ℚ", cx.name());
        }
    }
}

/// Betti numbers over 𝔽₂ match the published values, at every grade of every space.
#[test]
fn test_gf2_betti_numbers_match_the_published_values() {
    for (cx, _, beta_f2) in reference_spaces() {
        for (k, want) in beta_f2.iter().enumerate() {
            let got = cx.betti_number_over(k, HomologyField::Gf2).unwrap();
            assert_eq!(got, *want, "{}: β_{k} over 𝔽₂", cx.name());
        }
    }
}

/// The coefficient field changes the answer, and the fixture set contains cases where it does.
///
/// Without this, an implementation that ignored `HomologyField` entirely would pass every other
/// test in this file. `ℝP²` has `β₁ = 0` over ℚ and `β₁ = 1` over 𝔽₂; the Klein bottle differs at
/// two grades.
#[test]
fn test_the_coefficient_field_changes_the_answer() {
    let mut separating = 0;
    for (cx, beta_q, beta_f2) in reference_spaces() {
        if beta_q == beta_f2 {
            continue;
        }
        separating += 1;
        let over = |f| {
            (0..beta_q.len())
                .map(|k| cx.betti_number_over(k, f).unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            over(HomologyField::Rational),
            beta_q,
            "{}: over ℚ",
            cx.name()
        );
        assert_eq!(over(HomologyField::Gf2), beta_f2, "{}: over 𝔽₂", cx.name());
        assert_ne!(
            over(HomologyField::Rational),
            over(HomologyField::Gf2),
            "{}: the two fields must disagree here",
            cx.name()
        );
    }
    assert_eq!(
        separating, 2,
        "the fixture set must keep both torsion-carrying spaces, or the field is untested"
    );
}

/// `betti_number` is `betti_number_over` at ℚ, which is what its docstring claims.
#[test]
fn test_the_default_field_is_the_rationals() {
    for (cx, beta_q, _) in reference_spaces() {
        for (k, want) in beta_q.iter().enumerate() {
            assert_eq!(cx.betti_number(k), *want, "{}: β_{k}", cx.name());
        }
    }
}

/// `Σ(−1)ᵏ nₖ = Σ(−1)ᵏ βₖ` for every space.
///
/// The left side comes from cell counts and never touches an elimination; the right side is
/// nothing but ranks. Two independent computations, equal by the Euler–Poincaré theorem, so this
/// catches a rank that is wrong in a way the published Betti numbers would not — a sign error at
/// one grade compensated at another, for instance.
#[test]
fn test_the_euler_characteristic_agrees_from_cells_and_from_betti_numbers() {
    for (cx, _, _) in reference_spaces() {
        let from_cells = cx.euler_from_cells();
        let from_betti: i64 = (0..=cx.max_dim())
            .map(|k| {
                let b = cx.betti_number_over(k, HomologyField::Rational).unwrap() as i64;
                if k % 2 == 0 { b } else { -b }
            })
            .sum();
        assert_eq!(
            from_cells,
            from_betti,
            "{}: χ from cells is {from_cells}, from Betti numbers {from_betti}",
            cx.name()
        );
    }
}

/// `∂ₖ ∘ ∂ₖ₊₁ = 0`, at every grade of every fixture.
///
/// The law the trait states and cannot enforce. Coefficients are widened to `i64` before the
/// product: the entries are `i8`, and an intermediate sum that wrapped would compare equal to zero
/// in release without the widening.
#[test]
fn test_the_boundary_of_a_boundary_is_zero() {
    for (cx, _, _) in reference_spaces() {
        for k in 1..=cx.max_dim() {
            let outer = cx.boundary_matrix(k);
            let inner = cx.boundary_matrix(k + 1);
            assert_eq!(
                outer.shape().1,
                inner.shape().0,
                "{}: ∂_{k} and ∂_{} do not compose",
                cx.name(),
                k + 1
            );
            for j in 0..inner.shape().1 {
                for i in 0..outer.shape().0 {
                    let acc: i64 = (0..outer.shape().1)
                        .map(|t| outer.get(i, t).unwrap() as i64 * inner.get(t, j).unwrap() as i64)
                        .sum();
                    assert_eq!(acc, 0, "{}: (∂_{k} ∂_{})[{i}, {j}]", cx.name(), k + 1);
                }
            }
        }
    }
}

/// The harness above discriminates: a complex with one incidence sign flipped fails it.
///
/// A conformance check that passes on a malformed complex measures nothing. This builds one
/// deliberately and confirms the assertion catches it.
#[test]
fn test_the_boundary_law_harness_rejects_a_malformed_complex() {
    let cx = SimplicialFixture::new(
        "sphere_2",
        &[&[0, 1, 2], &[0, 1, 3], &[0, 2, 3], &[1, 2, 3]],
    );
    let d1 = cx.boundary_matrix(1);
    let d2 = cx.boundary_matrix(2);

    // The genuine complex satisfies the law.
    let composite = |a: &CsrMatrix<i8>, b: &CsrMatrix<i8>| {
        (0..b.shape().1)
            .flat_map(|j| (0..a.shape().0).map(move |i| (i, j)))
            .map(|(i, j)| {
                (0..a.shape().1)
                    .map(|t| a.get(i, t).unwrap() as i64 * b.get(t, j).unwrap() as i64)
                    .sum::<i64>()
            })
            .collect::<Vec<_>>()
    };
    assert!(
        composite(&d1, &d2).iter().all(|&x| x == 0),
        "the genuine complex obeys the law"
    );

    // Flip one sign of ∂₂. The composite must now be non-zero somewhere.
    let mut triplets: Vec<(usize, usize, i8)> = Vec::new();
    let mut flipped = false;
    for i in 0..d2.shape().0 {
        for j in 0..d2.shape().1 {
            let v = d2.get(i, j).unwrap();
            if v != 0 {
                let v = if !flipped {
                    flipped = true;
                    -v
                } else {
                    v
                };
                triplets.push((i, j, v));
            }
        }
    }
    assert!(flipped, "the fixture must have a non-zero entry to flip");
    let bad = CsrMatrix::from_triplets(d2.shape().0, d2.shape().1, &triplets).unwrap();
    assert!(
        composite(&d1, &bad).iter().any(|&x| x != 0),
        "flipping an incidence sign must break ∂∘∂ = 0, or the harness proves nothing"
    );
}

/// The degenerate grades carry the shape their dimension implies, so the composite is formable at
/// both ends.
#[test]
fn test_the_degenerate_grades_carry_a_shape() {
    for (cx, _, _) in reference_spaces() {
        let top = cx.max_dim();
        assert_eq!(
            cx.boundary_matrix(0).shape(),
            (0, cx.num_cells(0)),
            "{}: ∂₀ has no rows and one column per vertex",
            cx.name()
        );
        assert_eq!(
            cx.boundary_matrix(top + 1).shape(),
            (cx.num_cells(top), 0),
            "{}: ∂_{{max+1}} has one row per top cell and no columns",
            cx.name()
        );
        for k in 0..=top {
            assert_eq!(
                cx.boundary_matrix(k).shape().1,
                cx.boundary_matrix(k + 1).shape().0,
                "{}: cols(∂_{k}) must equal rows(∂_{})",
                cx.name(),
                k + 1
            );
        }
    }
}

/// The coboundary is the transpose of the next boundary, entry for entry.
#[test]
fn test_the_coboundary_is_the_transpose_of_the_next_boundary() {
    for (cx, _, _) in reference_spaces() {
        for k in 0..=cx.max_dim() {
            let delta = cx.coboundary_matrix(k);
            let d = cx.boundary_matrix(k + 1);
            assert_eq!(
                delta.shape(),
                (d.shape().1, d.shape().0),
                "{}: δ_{k} shape",
                cx.name()
            );
            for i in 0..d.shape().0 {
                for j in 0..d.shape().1 {
                    assert_eq!(
                        delta.get(j, i).unwrap(),
                        d.get(i, j).unwrap(),
                        "{}: δ_{k}[{j},{i}]",
                        cx.name()
                    );
                }
            }
        }
    }
}

/// Cell counts match the reference oracle's, which is an independent construction of the same
/// spaces.
#[test]
fn test_cell_counts_match_the_independent_construction() {
    // From `reference.py`, which builds these in Python from the same descriptions.
    let expected: &[(&str, &[usize])] = &[
        ("point", &[1]),
        ("interval", &[2, 1]),
        ("circle", &[3, 3]),
        ("sphere_2", &[4, 6, 4]),
        ("torus_2", &[9, 27, 18]),
        ("cylinder", &[6, 12, 6]),
        ("mobius_band", &[6, 12, 6]),
        ("real_projective_plane", &[6, 15, 10]),
        ("klein_bottle", &[16, 48, 32]),
    ];
    for (cx, _, _) in reference_spaces() {
        let want = expected
            .iter()
            .find(|(n, _)| *n == cx.name())
            .unwrap_or_else(|| panic!("{} is missing from the reference counts", cx.name()))
            .1;
        let got: Vec<usize> = (0..=cx.max_dim()).map(|k| cx.num_cells(k)).collect();
        assert_eq!(got, want, "{}: cell counts", cx.name());
    }
}
