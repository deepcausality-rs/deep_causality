/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `LatticeComplex::betti_number` is a closed form. This is what licenses it.
//!
//! The override returns a binomial coefficient without reading a boundary matrix: `C(D, k)` when
//! every dimension is periodic, and `C(p, k)` over the `p` periodic ones otherwise. That is the
//! right answer for a torus, and for a product of a `p`-torus with contractible factors, but it is
//! an assertion about the lattice rather than a computation over it.
//!
//! Before these tests, every assertion of Betti `[1, 4, 6, 4, 1]` on `T⁴` was testing the binomial
//! formula. The general path `betti_number_over(k, field)` reads the boundary matrices and computes
//! `dim ker ∂ₖ − rank ∂ₖ₊₁` exactly, over ℚ or over 𝔽₂. Asserting that the two agree turns the
//! closed form from a lookup into a fast path with a licence.

use deep_causality_topology::{ChainComplex, HomologyField, LatticeComplex};

/// Every grade of a 2D lattice, closed form against the boundary matrices, over both fields.
fn check_2d(name: &str, shape: [usize; 2], periodic: [bool; 2]) {
    let lc: LatticeComplex<2, f64> = LatticeComplex::new(shape, periodic);
    for k in 0..=3 {
        let closed = lc.betti_number(k);
        for field in [HomologyField::Rational, HomologyField::Gf2] {
            let computed = lc
                .betti_number_over(k, field)
                .unwrap_or_else(|e| panic!("{name} k={k} {field:?}: {e}"));
            assert_eq!(
                closed, computed,
                "{name} at k={k} over {field:?}: closed form says {closed}, the boundary matrices say {computed}"
            );
        }
    }
}

/// The same for 3D.
fn check_3d(name: &str, shape: [usize; 3], periodic: [bool; 3]) {
    let lc: LatticeComplex<3, f64> = LatticeComplex::new(shape, periodic);
    for k in 0..=4 {
        let closed = lc.betti_number(k);
        for field in [HomologyField::Rational, HomologyField::Gf2] {
            let computed = lc
                .betti_number_over(k, field)
                .unwrap_or_else(|e| panic!("{name} k={k} {field:?}: {e}"));
            assert_eq!(
                closed, computed,
                "{name} at k={k} over {field:?}: closed form says {closed}, the boundary matrices say {computed}"
            );
        }
    }
}

/// The fully periodic branch: `C(D, k)` against the complex.
///
/// `T²` has Betti `[1, 2, 1]` and `T³` has `[1, 3, 3, 1]`. Two sizes at `D = 2` because the closed
/// form does not depend on the extent and the computed one does.
#[test]
fn test_the_closed_form_agrees_with_the_boundary_matrices_on_a_torus() {
    check_2d("T² 3x3", [3, 3], [true, true]);
    check_2d("T² 4x4", [4, 4], [true, true]);
    check_3d("T³ 3x3x3", [3, 3, 3], [true, true, true]);
}

/// The partially periodic branch, which counts only the periodic dimensions.
///
/// A cylinder is `S¹ ×` an interval, homotopy equivalent to `S¹`, so `[1, 1, 0]`. The 3D case is
/// `T² ×` an interval, so `[1, 2, 1, 0]`. This is the arm the torus tests never reach.
#[test]
fn test_the_closed_form_agrees_when_only_some_dimensions_are_periodic() {
    check_2d("cylinder 3x3", [3, 3], [true, false]);
    check_3d("T² × I", [3, 3, 3], [true, true, false]);
}

/// The fully non-periodic branch: a contractible block, Betti `[1, 0, 0, …]`.
#[test]
fn test_the_closed_form_agrees_on_a_contractible_block() {
    check_2d("disk 3x3", [3, 3], [false, false]);
    check_3d("block 3x3x3", [3, 3, 3], [false, false, false]);
}

/// Above the lattice dimension there is no grade, and both paths say zero.
#[test]
fn test_both_paths_report_zero_above_the_lattice_dimension() {
    let lc: LatticeComplex<2, f64> = LatticeComplex::new([3, 3], [true, true]);
    for k in 3..=5 {
        assert_eq!(lc.betti_number(k), 0, "closed form at k={k}");
        assert_eq!(
            lc.betti_number_over(k, HomologyField::Rational).unwrap(),
            0,
            "computed at k={k}"
        );
    }
}
