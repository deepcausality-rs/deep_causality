/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `ChainComplex` impl on `SimplicialComplex`.
//!
//! Exercises: `cells`, `num_cells`, `max_dim`, `boundary_matrix`, `coboundary_matrix`,
//! `betti_number`, plus the `SimplicialCellIter` and the `rank_of_csr` helper (covered
//! indirectly via `betti_number`).

use deep_causality_topology::utils_tests::create_triangle_complex;
use deep_causality_topology::{ChainComplex, Simplex, SimplicialComplex, SimplicialComplexBuilder};
use std::borrow::Cow;

fn triangle_with_coboundary() -> SimplicialComplex<f64> {
    // Builder auto-computes coboundary as transpose of boundary, so we exercise
    // the `Cow::Borrowed` path of `coboundary_matrix`.
    let mut b = SimplicialComplexBuilder::new(2);
    b.add_simplex(Simplex::new(vec![0, 1, 2])).unwrap();
    b.build::<f64>().unwrap()
}

#[test]
fn test_max_dim_triangle() {
    let c = create_triangle_complex();
    assert_eq!(c.max_dim(), 2);
}

#[test]
fn test_max_dim_empty_complex_via_direct_constructor() {
    // Direct constructor with no skeletons exercises the `unwrap_or(0)` branch of `max_dim`.
    let c: SimplicialComplex<f64> =
        SimplicialComplex::new(Vec::new(), Vec::new(), Vec::new(), Vec::new());
    assert_eq!(c.max_dim(), 0);
}

#[test]
fn test_num_cells_per_grade() {
    let c = create_triangle_complex();
    assert_eq!(c.num_cells(0), 3);
    assert_eq!(c.num_cells(1), 3);
    assert_eq!(c.num_cells(2), 1);
    // No skeleton at grade 3 -> 0.
    assert_eq!(c.num_cells(3), 0);
}

#[test]
fn test_cells_iter_yields_expected_simplices_at_each_grade() {
    let c = create_triangle_complex();

    let v: Vec<Simplex> = c.cells(0).collect();
    assert_eq!(v.len(), 3);
    assert!(v.contains(&Simplex::new(vec![0])));
    assert!(v.contains(&Simplex::new(vec![1])));
    assert!(v.contains(&Simplex::new(vec![2])));

    let e: Vec<Simplex> = c.cells(1).collect();
    assert_eq!(e.len(), 3);
    assert!(e.contains(&Simplex::new(vec![0, 1])));
    assert!(e.contains(&Simplex::new(vec![0, 2])));
    assert!(e.contains(&Simplex::new(vec![1, 2])));

    let f: Vec<Simplex> = c.cells(2).collect();
    assert_eq!(f, vec![Simplex::new(vec![0, 1, 2])]);
}

#[test]
fn test_cells_iter_missing_grade_is_empty() {
    let c = create_triangle_complex();
    // Grade 3 does not exist -> iterator is empty (covers the `None` arm of
    // `SimplicialCellIter::next`).
    let none: Vec<Simplex> = c.cells(3).collect();
    assert!(none.is_empty());
}

#[test]
fn test_boundary_matrix_k_zero_returns_empty_owned() {
    let c = create_triangle_complex();
    let m = c.boundary_matrix(0);
    // Must be the empty owned matrix for k == 0 by design.
    assert!(matches!(m, Cow::Owned(_)));
    assert_eq!(m.shape(), (0, 0));
}

#[test]
fn test_boundary_matrix_k_one_borrowed_and_shape() {
    let c = create_triangle_complex();
    let m = c.boundary_matrix(1);
    assert!(matches!(m, Cow::Borrowed(_)));
    // ∂_1: 3 vertices x 3 edges.
    assert_eq!(m.shape(), (3, 3));
}

#[test]
fn test_boundary_matrix_k_two_borrowed_and_shape() {
    let c = create_triangle_complex();
    let m = c.boundary_matrix(2);
    assert!(matches!(m, Cow::Borrowed(_)));
    // ∂_2: 3 edges x 1 face.
    assert_eq!(m.shape(), (3, 1));
}

#[test]
fn test_boundary_matrix_out_of_range_returns_empty_owned() {
    let c = create_triangle_complex();
    let m = c.boundary_matrix(99);
    assert!(matches!(m, Cow::Owned(_)));
    assert_eq!(m.shape(), (0, 0));
}

#[test]
fn test_coboundary_matrix_present_is_borrowed() {
    // Builder populates coboundary operators (transposes of ∂_k).
    let c = triangle_with_coboundary();
    let m = c.coboundary_matrix(0);
    assert!(matches!(m, Cow::Borrowed(_)));
    // δ_0 = transpose(∂_1) -> 3 edges x 3 vertices.
    assert_eq!(m.shape(), (3, 3));
}

#[test]
fn test_coboundary_matrix_missing_is_owned_empty() {
    // create_triangle_complex passes an empty coboundary_operators vec, so all
    // grades hit the `None` arm.
    let c = create_triangle_complex();
    let m = c.coboundary_matrix(0);
    assert!(matches!(m, Cow::Owned(_)));
    assert_eq!(m.shape(), (0, 0));

    let m_oor = c.coboundary_matrix(42);
    assert!(matches!(m_oor, Cow::Owned(_)));
    assert_eq!(m_oor.shape(), (0, 0));
}

#[test]
fn test_betti_numbers_triangle() {
    // Triangle (2-disk) is contractible: b_0 = 1, b_1 = 0, b_2 = 0.
    let c = create_triangle_complex();
    assert_eq!(c.betti_number(0), 1);
    assert_eq!(c.betti_number(1), 0);
    assert_eq!(c.betti_number(2), 0);
    // Out-of-range grades default to 0.
    assert_eq!(c.betti_number(5), 0);
}

#[test]
fn test_betti_numbers_line() {
    // Line (two vertices + one edge): connected, contractible.
    // b_0 = 1, b_1 = 0.
    let c = deep_causality_topology::utils_tests::create_line_complex();
    assert_eq!(c.betti_number(0), 1);
    assert_eq!(c.betti_number(1), 0);
}
