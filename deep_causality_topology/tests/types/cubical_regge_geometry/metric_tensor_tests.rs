/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `CubicalReggeGeometry::metric_tensor_at` — Phase R5.3.
//!
//! Covers the Euclidean and Lorentzian per-cell metric tensor shape on
//! axis-aligned cubical complexes for all four edge-length tiers.

use deep_causality_topology::utils_tests::{
    open_cube_3, open_square_3, per_axis_geometry, per_edge_uniform_per_axis, periodic_cube_3,
    unit_geometry,
};
use deep_causality_topology::{CubicalReggeGeometry, LatticeCell, LatticeComplex};

const TOL: f64 = 1e-12;

fn assert_diagonal_with_off_diagonals_zero(
    tensor: &deep_causality_tensor::CausalTensor<f64>,
    d: usize,
) {
    assert_eq!(tensor.shape(), &[d, d], "metric tensor must be D × D");
    for i in 0..d {
        for j in 0..d {
            if i != j {
                let entry = tensor.as_slice()[i * d + j];
                assert!(
                    entry.abs() < TOL,
                    "off-diagonal ({i}, {j}) must be zero, got {entry}"
                );
            }
        }
    }
}

// -- Euclidean --------------------------------------------------------------------

#[test]
fn euclidean_unit_metric_is_identity_at_every_vertex_2d() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    for cell in lattice.iter_cells(0) {
        let g = geom.metric_tensor_at(&lattice, &cell);
        assert_diagonal_with_off_diagonals_zero(&g, 2);
        // All diagonals = +1
        for axis in 0..2 {
            assert!((g.as_slice()[axis * 2 + axis] - 1.0).abs() < TOL);
        }
    }
}

#[test]
fn euclidean_per_axis_metric_diagonals_are_axis_squared_2d() {
    let a = 3.0_f64;
    let b = 5.0_f64;
    let lattice = open_square_3();
    let geom = per_axis_geometry::<2>([a, b]);
    let cell = LatticeCell::vertex([0, 0]);
    let g = geom.metric_tensor_at(&lattice, &cell);
    assert_diagonal_with_off_diagonals_zero(&g, 2);
    let s = g.as_slice();
    assert!((s[0] - a * a).abs() < TOL); // g_{0,0}
    assert!((s[3] - b * b).abs() < TOL); // g_{1,1} at (1, 1) → 1*2 + 1 = 3
}

#[test]
fn euclidean_per_axis_metric_diagonals_are_axis_squared_3d() {
    let a = 2.0_f64;
    let b = 3.0_f64;
    let c = 5.0_f64;
    let lattice = open_cube_3();
    let geom = per_axis_geometry::<3>([a, b, c]);
    let cell = LatticeCell::vertex([1, 1, 1]);
    let g = geom.metric_tensor_at(&lattice, &cell);
    assert_diagonal_with_off_diagonals_zero(&g, 3);
    assert!((g.as_slice()[0] - a * a).abs() < TOL);
    assert!((g.as_slice()[4] - b * b).abs() < TOL);
    assert!((g.as_slice()[8] - c * c).abs() < TOL);
}

#[test]
fn euclidean_per_edge_metric_reduces_to_per_axis_on_uniform_input() {
    // Per-edge with uniform-per-axis lengths produces the same diagonal as
    // PerAxis.
    let lengths = [2.0_f64, 3.0, 5.0];
    let lattice = periodic_cube_3();
    let per_edge = per_edge_uniform_per_axis::<3>(&lattice, lengths);
    let per_axis = per_axis_geometry::<3>(lengths);
    let cell = LatticeCell::vertex([0, 0, 0]);
    let g_pe = per_edge.metric_tensor_at(&lattice, &cell);
    let g_pa = per_axis.metric_tensor_at(&lattice, &cell);
    for axis in 0..3 {
        assert!((g_pe.as_slice()[axis * 3 + axis] - g_pa.as_slice()[axis * 3 + axis]).abs() < TOL);
    }
}

// -- Lorentzian -------------------------------------------------------------------

#[test]
fn lorentzian_metric_has_negative_diagonal_at_timelike_axis() {
    // Lorentzian 2D with axis 0 timelike, axis 1 spacelike.
    let lattice = open_square_3();
    let geom = unit_geometry::<2>()
        .with_timelike_axes([true, false])
        .expect("exactly one timelike axis ⇒ Lorentzian");
    let cell = LatticeCell::vertex([0, 0]);
    let g = geom.metric_tensor_at(&lattice, &cell);
    assert_diagonal_with_off_diagonals_zero(&g, 2);
    // g_00 = -L_0² = -1 (timelike), g_11 = +L_1² = +1 (spacelike)
    assert!(
        (g.as_slice()[0] - (-1.0)).abs() < TOL,
        "expected -1, got {}",
        g.as_slice()[0]
    );
    assert!((g.as_slice()[3] - 1.0).abs() < TOL);
}

#[test]
fn lorentzian_per_axis_metric_4d() {
    // 4D Lorentzian with axis 3 timelike (Minkowski-like): diag(L_0², L_1², L_2², -L_3²).
    let a = 1.0_f64;
    let b = 2.0_f64;
    let c = 3.0_f64;
    let t = 4.0_f64;
    let lattice: LatticeComplex<4, f64> = LatticeComplex::hypercubic_torus(2);
    let geom: CubicalReggeGeometry<4, f64> = CubicalReggeGeometry::per_axis([a, b, c, t]);
    let lor = geom
        .with_timelike_axes([false, false, false, true])
        .expect("exactly one timelike axis");
    let cell = LatticeCell::vertex([0, 0, 0, 0]);
    let g = lor.metric_tensor_at(&lattice, &cell);
    assert_diagonal_with_off_diagonals_zero(&g, 4);
    let s = g.as_slice();
    // g_{i,i} sits at index i*D + i with D = 4 → 0, 5, 10, 15.
    assert!((s[0] - a * a).abs() < TOL);
    assert!((s[5] - b * b).abs() < TOL);
    assert!((s[10] - c * c).abs() < TOL);
    assert!((s[15] - (-t * t)).abs() < TOL);
}
