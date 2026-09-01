/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Homology and cohomology representatives over 𝔽₂.
//!
//! A basis of a quotient is not canonical: which vectors come back depends on the elimination
//! order, so there is nothing to diff against a stored answer. What is fixed, and what these tests
//! assert, are the invariants that make the vectors a basis at all:
//!
//! 1. **The count is `β_k` over 𝔽₂**, which the fixtures carry from Hatcher, *Algebraic Topology*,
//!    and which no code here produced.
//! 2. **Every representative is a cycle**: `∂ₖ γ = 0` over 𝔽₂.
//! 3. **No non-empty combination is a boundary**: adjoining the representatives to a basis of
//!    `im ∂ₖ₊₁` raises the rank by exactly their number, so each is independent modulo boundaries.
//!
//! Together those pin the basis without pinning the arbitrary part of it.

use deep_causality_homology::utils_tests::reference_spaces;
use deep_causality_homology::{ChainComplex, Gf2Chain, HomologyField};
use deep_causality_linear::{
    MatrixBuild, MatrixView, PackedGf2, csr_to_packed_gf2_mod2, image_basis_gf2, rank_gf2,
};
use deep_causality_num::Gf2;

type W = u64;

/// `∂ₖ γ` over 𝔽₂, as a bit vector.
fn boundary_of(complex: &impl ChainComplex, k: usize, gamma: &Gf2Chain<W>) -> Vec<bool> {
    let d = csr_to_packed_gf2_mod2::<W>(&complex.boundary_matrix(k));
    let support: Vec<usize> = gamma.support().collect();
    (0..d.rows())
        .map(|r| {
            support
                .iter()
                .filter(|&&c| d.get(r, c).unwrap().bit())
                .count()
                % 2
                == 1
        })
        .collect()
}

/// The rank over 𝔽₂ of `im ∂ₖ₊₁` with `extra` adjoined as further columns.
fn rank_with_boundaries(
    complex: &impl ChainComplex,
    k: usize,
    extra: &[Gf2Chain<W>],
) -> (usize, usize) {
    let n = complex.num_cells(k);
    let image = image_basis_gf2(&csr_to_packed_gf2_mod2::<W>(
        &complex.boundary_matrix(k + 1),
    ))
    .unwrap();
    let base_rank = rank_gf2(&image).unwrap();

    let mut joined = PackedGf2::<W>::zeros(n, image.cols() + extra.len());
    for c in 0..image.cols() {
        for r in 0..n {
            if image.get(r, c).unwrap().bit() {
                joined.set(r, c, Gf2::ONE).unwrap();
            }
        }
    }
    for (i, g) in extra.iter().enumerate() {
        for r in g.support() {
            joined.set(r, image.cols() + i, Gf2::ONE).unwrap();
        }
    }
    (base_rank, rank_gf2(&joined).unwrap())
}

#[test]
fn test_representative_count_is_the_gf2_betti_number_everywhere() {
    // The count is the one part of the answer that is canonical, and the fixtures carry it from
    // Hatcher rather than from this implementation.
    for (fixture, _rational, gf2) in reference_spaces() {
        for (k, &expected) in gf2.iter().enumerate() {
            let reps = fixture.homology_representatives::<W>(k).unwrap();
            assert_eq!(
                reps.len(),
                expected,
                "{} H_{} should have rank {}",
                fixture.name(),
                k,
                expected
            );
            // And it agrees with the rank-nullity path, which never builds a kernel.
            assert_eq!(
                reps.len(),
                fixture.betti_number_over(k, HomologyField::Gf2).unwrap(),
                "{} H_{} disagrees with betti_number_over",
                fixture.name(),
                k
            );
        }
    }
}

#[test]
fn test_every_representative_is_a_cycle() {
    for (fixture, _rational, gf2) in reference_spaces() {
        for k in 0..gf2.len() {
            for gamma in fixture.homology_representatives::<W>(k).unwrap() {
                assert!(
                    boundary_of(&fixture, k, &gamma).iter().all(|b| !b),
                    "{} H_{}: a representative is not a cycle",
                    fixture.name(),
                    k
                );
                assert_eq!(gamma.degree(), k);
                assert_eq!(gamma.len(), fixture.num_cells(k));
            }
        }
    }
}

#[test]
fn test_representatives_are_independent_modulo_boundaries() {
    // Adjoining r representatives to a basis of im ∂ₖ₊₁ must raise the rank by exactly r. Anything
    // less means one of them is a boundary, or two differ by one.
    for (fixture, _rational, gf2) in reference_spaces() {
        for k in 0..gf2.len() {
            let reps = fixture.homology_representatives::<W>(k).unwrap();
            if reps.is_empty() {
                continue;
            }
            let (base, joined) = rank_with_boundaries(&fixture, k, &reps);
            assert_eq!(
                joined - base,
                reps.len(),
                "{} H_{}: {} representatives raised the rank by {}",
                fixture.name(),
                k,
                reps.len(),
                joined - base
            );
        }
    }
}

#[test]
fn test_a_boundary_is_never_returned_as_a_representative() {
    // The sharpest single case: on the 2-sphere H_1 is trivial, so every 1-cycle is a boundary and
    // the basis must be empty even though ker ∂₁ is not.
    let (sphere, _q, _f) = reference_spaces()
        .into_iter()
        .find(|(f, _, _)| f.name() == "sphere_2")
        .expect("the fixture set carries sphere_2");
    assert!(sphere.homology_representatives::<W>(1).unwrap().is_empty());
    // And ker ∂₁ is genuinely non-trivial there, so the emptiness is a quotient result rather than
    // an empty kernel.
    let d1 = csr_to_packed_gf2_mod2::<W>(&sphere.boundary_matrix(1));
    assert!(d1.cols() > rank_gf2(&d1).unwrap());
}

// ---------------------------------------------------------------------------
// Cohomology, and the Poincaré-dual pairing.
// ---------------------------------------------------------------------------

#[test]
fn test_cohomology_ranks_match_homology_over_a_field() {
    // Over a field, H^k and H_k have the same dimension: the universal coefficient theorem's Ext
    // term vanishes. That is an external identity, not a reading of this code, and it cross-checks
    // the cohomology path against the homology one.
    for (fixture, _rational, gf2) in reference_spaces() {
        for (k, &expected) in gf2.iter().enumerate() {
            assert_eq!(
                fixture.cohomology_representatives::<W>(k).unwrap().len(),
                expected,
                "{} H^{} should have rank {}",
                fixture.name(),
                k,
                expected
            );
        }
    }
}

#[test]
fn test_dual_representative_pairs_to_one_where_it_exists() {
    for (fixture, _rational, gf2) in reference_spaces() {
        for k in 0..gf2.len() {
            for gamma in fixture.homology_representatives::<W>(k).unwrap() {
                if let Some(dual) = fixture.dual_representative::<W>(&gamma, k).unwrap() {
                    assert_eq!(
                        gamma.inner(&dual).unwrap(),
                        Gf2::ONE,
                        "{} grade {}: the returned dual does not pair to one",
                        fixture.name(),
                        k
                    );
                    assert_eq!(dual.degree(), k);
                }
            }
        }
    }
}

#[test]
fn test_a_boundary_has_no_dual() {
    // The pairing descends to homology, so a boundary pairs to zero with every cocycle. On the
    // torus the whole of C_2 is a cycle; take a 1-boundary by pushing a 2-cell down.
    let (torus, _q, _f) = reference_spaces()
        .into_iter()
        .find(|(f, _, _)| f.name() == "torus_2")
        .expect("the fixture set carries torus_2");
    let d2 = csr_to_packed_gf2_mod2::<W>(&torus.boundary_matrix(2));
    let boundary = Gf2Chain::<W>::from_column(&d2, 0, 1).unwrap();
    assert!(
        torus
            .dual_representative::<W>(&boundary, 1)
            .unwrap()
            .is_none()
    );
}

#[test]
fn test_the_torus_has_two_independent_one_classes_each_with_a_dual() {
    // β₁(T²) = 2 over 𝔽₂, and the intersection pairing on a closed orientable surface is
    // non-degenerate, so every class has a dual. Both facts are Hatcher's, not this code's.
    let (torus, _q, gf2) = reference_spaces()
        .into_iter()
        .find(|(f, _, _)| f.name() == "torus_2")
        .expect("the fixture set carries torus_2");
    assert_eq!(gf2[1], 2);
    let reps = torus.homology_representatives::<W>(1).unwrap();
    assert_eq!(reps.len(), 2);
    for gamma in &reps {
        assert!(
            torus.dual_representative::<W>(gamma, 1).unwrap().is_some(),
            "a torus 1-class has no dual, so the pairing came out degenerate"
        );
    }
}

#[test]
fn test_degenerate_grades_yield_no_representatives() {
    let (point, _q, _f) = reference_spaces()
        .into_iter()
        .find(|(f, _, _)| f.name() == "point")
        .expect("the fixture set carries point");
    // Above the top grade there are no cells, so no classes.
    assert!(point.homology_representatives::<W>(5).unwrap().is_empty());
    assert!(point.cohomology_representatives::<W>(5).unwrap().is_empty());
}
