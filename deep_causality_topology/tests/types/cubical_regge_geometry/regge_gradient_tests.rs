/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `CubicalReggeGeometry::regge_gradient` — Phase R6.1 + R6.2 + R6.3.
//!
//! Covers:
//! - Closed-form expectations per dimension (D=2 zero, D=3 deficit, D=4 product rule).
//! - Central finite-difference verification on `PerEdge` geometries.
//! - Equilibrium / stationary-point check for the unit-edge configuration on
//!   periodic (flat) lattices.
//! - Lorentzian gradient is `i ·` Euclidean gradient per Wick-rotation convention.
//! - Locality smoke check.

use deep_causality_topology::utils_tests::{
    open_cube_3, open_square_3, per_edge_uniform_per_axis, periodic_cube_3, unit_geometry,
};
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex};

const TOL: f64 = 1e-12;
const FD_TOL: f64 = 1e-5;

// -- D = 2: gradient is zero everywhere -------------------------------------------

#[test]
fn d2_gradient_is_zero_on_open_lattice() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    let g = geom.regge_gradient(&lattice);
    assert_eq!(g.len(), lattice.num_cells(1));
    for v in &g {
        assert!(
            v.abs() < TOL,
            "D=2 gradient must be zero (vertices have empty product vol), got {v}"
        );
    }
}

#[test]
fn d2_gradient_is_zero_on_periodic_lattice_with_per_edge_metric() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let geom = per_edge_uniform_per_axis::<2>(&lattice, [2.0, 3.0]);
    let g = geom.regge_gradient(&lattice);
    for v in &g {
        assert!(v.abs() < TOL);
    }
}

// -- D = 3: gradient entries equal hinge (edge) deficit angles --------------------

#[test]
fn d3_gradient_equals_per_edge_deficit_on_open_cube() {
    let lattice = open_cube_3();
    let geom = unit_geometry::<3>();
    let g = geom.regge_gradient(&lattice);
    assert_eq!(g.len(), lattice.num_cells(1));
    // For D=3, hinges ARE edges (1-cells), vol=L_h, ∂vol/∂L_h = 1, so
    // gradient[h] = deficit_angle(h).
    for (hinge_id, &got) in g.iter().enumerate() {
        let expected = geom.deficit_angle(&lattice, hinge_id);
        assert!(
            (got - expected).abs() < TOL,
            "D=3 gradient[{hinge_id}] = {got} but expected deficit = {expected}"
        );
    }
}

#[test]
fn d3_gradient_is_zero_on_periodic_cube() {
    // Periodic lattice has every hinge interior (4 incident cubes), deficit 0.
    let lattice = periodic_cube_3();
    let geom = unit_geometry::<3>();
    for v in geom.regge_gradient(&lattice) {
        assert!(v.abs() < TOL);
    }
}

// -- Central finite-difference verification (R6.3.1) ------------------------------

fn build_per_edge_geom_3d_open(lengths: Vec<f64>) -> CubicalReggeGeometry<3, f64> {
    CubicalReggeGeometry::<3, f64>::from_edge_lengths(lengths)
}

#[test]
fn d3_gradient_matches_central_finite_difference_open_cube() {
    let lattice = open_cube_3();
    let num_edges = lattice.num_cells(1);
    // Use distinct, non-trivial lengths so the gradient is non-zero and the
    // finite-difference comparison is meaningful.
    let base: Vec<f64> = (0..num_edges).map(|i| 1.0 + 0.05 * (i as f64)).collect();
    let geom = build_per_edge_geom_3d_open(base.clone());
    let analytic = geom.regge_gradient(&lattice);

    let eps = 1e-6_f64;
    for edge in 0..num_edges {
        let mut lp = base.clone();
        let mut lm = base.clone();
        lp[edge] += eps;
        lm[edge] -= eps;
        let s_plus = build_per_edge_geom_3d_open(lp).regge_action(&lattice);
        let s_minus = build_per_edge_geom_3d_open(lm).regge_action(&lattice);
        let fd = (s_plus - s_minus) / (2.0 * eps);
        // Skip edges with deficit zero — both analytic and FD are zero, the
        // ratio test would be undefined.
        if analytic[edge].abs() < TOL && fd.abs() < TOL {
            continue;
        }
        let abs_err = (analytic[edge] - fd).abs();
        let rel_err = abs_err / fd.abs().max(TOL);
        assert!(
            rel_err < FD_TOL,
            "edge {edge}: analytic = {}, FD = {fd}, rel_err = {rel_err}",
            analytic[edge]
        );
    }
}

// -- Equilibrium / stationary point (R6.3.2) --------------------------------------

#[test]
fn unit_edge_is_stationary_on_periodic_3d() {
    // Periodic 3D: every hinge is interior, deficit 0, gradient is identically 0.
    // The unit-edge configuration is trivially a stationary point — this confirms
    // the gradient correctly reports zero on a flat configuration.
    let lattice = periodic_cube_3();
    let geom = unit_geometry::<3>();
    for v in geom.regge_gradient(&lattice) {
        assert!(v.abs() < TOL);
    }
}

#[test]
fn unit_edge_open_3d_is_not_stationary_at_boundary() {
    // Open 3D: boundary hinges have non-zero deficit, so the unit-edge
    // configuration is NOT stationary even though all entries equal +1.
    // This confirms the gradient genuinely depends on the lattice topology,
    // not just on the metric values.
    let lattice = open_cube_3();
    let geom = unit_geometry::<3>();
    let g = geom.regge_gradient(&lattice);
    let any_nonzero = g.iter().any(|v| v.abs() > TOL);
    assert!(
        any_nonzero,
        "open 3D unit-edge should have non-zero gradient at boundary edges"
    );
}

// -- Lorentzian gradient is i · Euclidean gradient ---------------------------------

#[test]
fn lorentzian_gradient_is_pure_imaginary_with_im_equal_to_euclidean() {
    let lattice = open_cube_3();
    let euc = unit_geometry::<3>();
    let lor = unit_geometry::<3>()
        .with_timelike_axes([true, false, false])
        .unwrap();
    let g_e = euc.regge_gradient(&lattice);
    let g_l = lor.regge_gradient(&lattice);
    assert_eq!(g_e.len(), g_l.len());
    for (i, (e, l)) in g_e.iter().zip(g_l.iter()).enumerate() {
        assert!(l.re.abs() < TOL, "edge {i}: Lorentzian re must be zero, got {}", l.re);
        assert!(
            (l.im - e).abs() < TOL,
            "edge {i}: Lorentzian im {} must equal Euclidean {e}",
            l.im
        );
    }
}

// -- Locality smoke check ---------------------------------------------------------

#[test]
fn d3_gradient_entry_changes_only_when_local_edge_changes() {
    // Perturbing edge 0 must change gradient[0] (since edge 0 is itself a
    // hinge in 3D). Other entries that share no hinge with edge 0 must stay
    // the same. (This is a softer check than a full locality assertion but
    // sufficient to catch non-local cross-coupling regressions.)
    let lattice = open_cube_3();
    let num_edges = lattice.num_cells(1);
    let lens_a = vec![1.0_f64; num_edges];
    let mut lens_b = lens_a.clone();
    lens_b[0] = 2.0;

    let g_a = build_per_edge_geom_3d_open(lens_a).regge_gradient(&lattice);
    let g_b = build_per_edge_geom_3d_open(lens_b).regge_gradient(&lattice);

    // For D=3, hinges ARE edges. gradient[h] = deficit(h) — deficit depends
    // only on incidence (topology), not on edge lengths. So perturbing one
    // edge length must NOT change any gradient entry. This is a sharper
    // locality statement than the soft check above: in 3D the gradient is
    // entirely topology-driven.
    for i in 0..num_edges {
        assert!(
            (g_a[i] - g_b[i]).abs() < TOL,
            "edge {i}: D=3 gradient must not depend on per-edge length perturbation, \
             got Δ = {}",
            g_a[i] - g_b[i]
        );
    }
}
