/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Parametric conformance tests for the `ChainComplex` trait.
//!
//! Verifies algebraic invariants that every `ChainComplex` impl MUST satisfy:
//!   (a) `boundary_matrix(k).shape() == (num_cells(k - 1), num_cells(k))` for `k > 0`.
//!   (b) `coboundary_matrix(k)` equals the transpose of `boundary_matrix(k + 1)`.
//!   (c) For `SimplicialComplex` specifically, `boundary_matrix` and `coboundary_matrix`
//!       return `Cow::Borrowed` (zero copy from the pre-computed cache).
//!   (d) `betti_number(k)` is `betti_number_over(k, HomologyField::Rational)` — the same number,
//!       not merely a similar one — so which field a Betti number is over stays readable from the
//!       call.

use deep_causality_topology::{
    ChainComplex, HomologyField, LatticeComplex, Simplex, SimplicialComplex,
    SimplicialComplexBuilder,
};
use std::borrow::Cow;

fn make_triangle_complex() -> SimplicialComplex<f64> {
    let mut builder = SimplicialComplexBuilder::new(2);
    builder
        .add_simplex(Simplex::new(vec![0, 1, 2]))
        .expect("add triangle");
    builder.build::<f64>().expect("build complex")
}

fn assert_shape_invariant<K: ChainComplex>(complex: &K) {
    let max_d = complex.max_dim();
    for k in 1..=max_d {
        let mat = complex.boundary_matrix(k);
        let (rows, cols) = mat.shape();
        let expected_rows = complex.num_cells(k - 1);
        let expected_cols = complex.num_cells(k);
        assert_eq!(
            rows,
            expected_rows,
            "boundary_matrix({k}) row count must equal num_cells({}) ({expected_rows}), got {rows}",
            k - 1
        );
        assert_eq!(
            cols, expected_cols,
            "boundary_matrix({k}) col count must equal num_cells({k}) ({expected_cols}), got {cols}"
        );
    }
}

fn csr_eq(
    a: &deep_causality_linear::CsrMatrix<i8>,
    b: &deep_causality_linear::CsrMatrix<i8>,
) -> bool {
    let (ar, ac) = a.shape();
    let (br, bc) = b.shape();
    if ar != br || ac != bc {
        return false;
    }
    let aps = a.row_indices();
    let acs = a.col_indices();
    let avs = a.values();
    let bps = b.row_indices();
    let bcs = b.col_indices();
    let bvs = b.values();
    for r in 0..ar {
        let (a_start, a_end) = (aps[r], aps[r + 1]);
        let (b_start, b_end) = (bps[r], bps[r + 1]);
        let mut a_row: Vec<(usize, i8)> = (a_start..a_end).map(|i| (acs[i], avs[i])).collect();
        let mut b_row: Vec<(usize, i8)> = (b_start..b_end).map(|i| (bcs[i], bvs[i])).collect();
        a_row.sort_by_key(|t| t.0);
        b_row.sort_by_key(|t| t.0);
        if a_row != b_row {
            return false;
        }
    }
    true
}

fn assert_transpose_invariant<K: ChainComplex>(complex: &K) {
    let max_d = complex.max_dim();
    for k in 0..max_d {
        let cob = complex.coboundary_matrix(k).into_owned();
        let bnd_transposed = complex.boundary_matrix(k + 1).into_owned().transpose();
        assert!(
            csr_eq(&cob, &bnd_transposed),
            "coboundary_matrix({k}) must equal boundary_matrix({}).transpose()",
            k + 1
        );
    }
}

#[test]
fn shape_invariant_simplicial_complex() {
    let complex = make_triangle_complex();
    assert_shape_invariant(&complex);
}

#[test]
fn shape_invariant_lattice_2d_open() {
    let lattice = LatticeComplex::<2, f64>::open([3, 3]);
    assert_shape_invariant(&lattice);
}

#[test]
fn shape_invariant_lattice_3d_torus() {
    let lattice = LatticeComplex::<3, f64>::torus([2, 2, 2]);
    assert_shape_invariant(&lattice);
}

#[test]
fn transpose_invariant_simplicial_complex() {
    let complex = make_triangle_complex();
    assert_transpose_invariant(&complex);
}

#[test]
fn transpose_invariant_lattice_2d_open() {
    let lattice = LatticeComplex::<2, f64>::open([3, 3]);
    assert_transpose_invariant(&lattice);
}

#[test]
fn transpose_invariant_lattice_3d_torus() {
    let lattice = LatticeComplex::<3, f64>::torus([2, 2, 2]);
    assert_transpose_invariant(&lattice);
}

#[test]
fn simplicial_boundary_matrix_returns_borrowed() {
    let complex = make_triangle_complex();
    // Grade 1 (edges → vertices): cached operator must be borrowable.
    let bnd = complex.boundary_matrix(1);
    assert!(
        matches!(bnd, Cow::Borrowed(_)),
        "expected Cow::Borrowed for ∂_1"
    );
    // Grade 2 (face → edges): same.
    let bnd2 = complex.boundary_matrix(2);
    assert!(
        matches!(bnd2, Cow::Borrowed(_)),
        "expected Cow::Borrowed for ∂_2"
    );
}

#[test]
fn simplicial_coboundary_matrix_returns_borrowed() {
    let complex = make_triangle_complex();
    let cob = complex.coboundary_matrix(0);
    assert!(
        matches!(cob, Cow::Borrowed(_)),
        "expected Cow::Borrowed for δ_0"
    );
}

#[test]
fn lattice_coboundary_matrix_lazy_memo() {
    // Two consecutive calls must yield equal matrices (memo correctness).
    let lattice = LatticeComplex::<2, f64>::torus([3, 3]);
    let first = lattice.coboundary_matrix(0).into_owned();
    let second = lattice.coboundary_matrix(0).into_owned();
    assert!(csr_eq(&first, &second));
}

// ---- (d) the field a Betti number is over is the one the call names ----------------------------

#[test]
fn betti_number_is_the_rational_case_of_betti_number_over() {
    let c = make_triangle_complex();
    for k in 0..=3 {
        assert_eq!(
            c.betti_number(k),
            c.betti_number_over(k, HomologyField::Rational).unwrap(),
            "betti_number({k}) must be defined as the rational case, not merely agree with it"
        );
    }
}

#[test]
fn a_triangle_has_the_same_betti_numbers_over_both_fields() {
    // A filled triangle is contractible: β₀ = 1 and nothing above it, over any field. Recorded
    // here because the two fields agreeing is a property of this complex rather than of homology,
    // and the crate's `[[32,2,4]]` toric-code result depends on the same coincidence.
    let c = make_triangle_complex();
    for k in 0..=2 {
        assert_eq!(
            c.betti_number_over(k, HomologyField::Rational).unwrap(),
            c.betti_number_over(k, HomologyField::Gf2).unwrap(),
            "the two fields must agree at grade {k} for a contractible complex"
        );
    }
}

#[test]
fn betti_number_over_is_available_on_a_lattice_complex() {
    // The default body reads the boundary matrices. `LatticeComplex` overrides `betti_number`
    // with a closed form for the torus and does not override this, so the two are separate
    // answers — see G-03 in `openspec/notes/quantum/qcl-gaps.md`, which is open.
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(3);
    let computed = lattice
        .betti_number_over(0, HomologyField::Rational)
        .expect("rank over the rationals");
    assert_eq!(
        computed, 1,
        "an open 3x3 lattice is connected, so its zeroth Betti number is 1"
    );
}
