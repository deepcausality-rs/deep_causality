/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage tests for the degenerate / boundary-condition branches of
//! `LatticeComplex` — `D < 2`, `shape[d] == 0`, out-of-range `hinge_id`.
//! Lifts these one-line conditional arms above the `cargo llvm-cov` floor.

use deep_causality_topology::{ChainComplex, LatticeComplex};

// -- D < 2: hinge_top_cube_neighbors short-circuits to empty -----------------------

#[test]
fn hinge_top_cube_neighbors_returns_empty_for_d1() {
    let l: LatticeComplex<1, f64> = LatticeComplex::new([5], [false]);
    assert!(l.hinge_top_cube_neighbors(0).is_empty());
    assert!(l.hinge_top_cube_neighbors(999).is_empty());
}

// -- shape[d] == 0: valid_positions / num_cells / iter_cells branches --------------

#[test]
fn zero_shape_dimension_num_cells_is_zero() {
    // Non-periodic axis with extent 0 hits the `dim_len == 0` arm in `valid_positions`
    // and the matching arm in `num_cells`.
    let l: LatticeComplex<2, f64> = LatticeComplex::new([3, 0], [true, false]);
    assert_eq!(l.num_cells(2), 0);
}

#[test]
fn zero_shape_dimension_iterator_yields_no_cells() {
    // `LatticeCellIterator::new` filters out orientations whose product of valid
    // positions is zero (e.g. an active non-periodic axis with shape 0), so
    // `cells(k).count()` agrees with `num_cells(k)` even in this degenerate case.
    let l: LatticeComplex<2, f64> = LatticeComplex::new([3, 0], [true, false]);
    assert_eq!(l.cells(2).count(), 0);
    assert_eq!(l.cells(2).count(), l.num_cells(2));
}

#[test]
fn zero_shape_dimension_gives_zero_edges_along_that_axis() {
    // Non-periodic axis with extent 0 produces zero edges along itself.
    let l: LatticeComplex<2, f64> = LatticeComplex::new([3, 0], [false, false]);
    assert_eq!(l.num_cells(1), 0);
}
