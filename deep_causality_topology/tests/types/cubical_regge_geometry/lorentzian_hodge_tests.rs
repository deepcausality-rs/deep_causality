/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the Lorentzian sign factors on the cubical `HasHodgeStar<R>`
//! implementation — Phase R5.4.
//!
//! Verifies design.md Decision 3 reduction property: at every cell, the
//! Lorentzian Hodge ⋆ diagonal entry equals the Euclidean entry times
//! `(−1)^t`, where `t` is the count of timelike axes in the primal cell's
//! active dimensions.

use deep_causality_topology::utils_tests::{
    open_square_3, periodic_cube_3, periodic_square_3, unit_geometry,
};
use deep_causality_topology::{ChainComplex, HasHodgeStar};

const TOL: f64 = 1e-12;

#[test]
fn lorentzian_2d_with_axis_0_timelike_negates_axis_0_edges_and_positive_2cells() {
    // Cell dimension sign rules in 2D with axis 0 timelike:
    //   ⋆_0: vertex has empty orientation ⇒ t = 0 ⇒ sign = +1.
    //   ⋆_1: axis-0 edge (orientation 0b01) ⇒ t = 1 ⇒ sign = -1.
    //   ⋆_1: axis-1 edge (orientation 0b10) ⇒ t = 0 ⇒ sign = +1.
    //   ⋆_2: 2-cube (orientation 0b11) ⇒ t = 1 ⇒ sign = -1.
    let lattice = periodic_square_3();
    let euclidean = unit_geometry::<2>();
    let lorentzian = unit_geometry::<2>()
        .with_timelike_axes([true, false])
        .unwrap();

    let e_star_0 = euclidean.hodge_star_matrix(&lattice, 0);
    let l_star_0 = lorentzian.hodge_star_matrix(&lattice, 0);
    for (a, b) in e_star_0.values().iter().zip(l_star_0.values().iter()) {
        assert!((a - b).abs() < TOL, "⋆_0 must match (both +1): {a} vs {b}");
    }

    let e_star_1 = euclidean.hodge_star_matrix(&lattice, 1);
    let l_star_1 = lorentzian.hodge_star_matrix(&lattice, 1);
    for (i, cell) in lattice.cells(1).enumerate() {
        let expected_sign = match cell.orientation() {
            0b01 => -1.0,
            0b10 => 1.0,
            other => panic!("unexpected orientation {other:b}"),
        };
        assert!(
            (l_star_1.values()[i] - expected_sign * e_star_1.values()[i]).abs() < TOL,
            "⋆_1 sign mismatch at orientation {:b}",
            cell.orientation()
        );
    }

    let e_star_2 = euclidean.hodge_star_matrix(&lattice, 2);
    let l_star_2 = lorentzian.hodge_star_matrix(&lattice, 2);
    for (a, b) in e_star_2.values().iter().zip(l_star_2.values().iter()) {
        assert!(
            (b - (-a)).abs() < TOL,
            "⋆_2 entry must be negated (axis-0 timelike, 2-cell has 1 timelike axis)"
        );
    }
}

#[test]
fn lorentzian_with_axis_d_minus_1_timelike_matches_minkowski_convention_3d() {
    // 3D, axis 2 timelike: classical Minkowski-like layout (t-axis = z-axis).
    //   ⋆_0: vertex t=0 ⇒ +1
    //   ⋆_1: axis-0 (0b001) t=0 ⇒ +1; axis-1 (0b010) t=0 ⇒ +1; axis-2 (0b100) t=1 ⇒ -1
    //   ⋆_2: face {0,1} (0b011) t=0 ⇒ +1; face {0,2} (0b101) t=1 ⇒ -1; face {1,2} (0b110) t=1 ⇒ -1
    //   ⋆_3: 3-cube (0b111) t=1 ⇒ -1
    let lattice = periodic_cube_3();
    let euclidean = unit_geometry::<3>();
    let lorentzian = unit_geometry::<3>()
        .with_timelike_axes([false, false, true])
        .unwrap();

    type SignByOrientation = fn(u32) -> f64;
    type GradeCase = (usize, SignByOrientation);
    let cases: [GradeCase; 4] = [
        (0usize, |_| 1.0_f64),
        (1, |o| if o == 0b100 { -1.0 } else { 1.0 }),
        (2, |o| if (o & 0b100) != 0 { -1.0 } else { 1.0 }),
        (3, |_| -1.0_f64),
    ];
    for (k, expected_fn) in cases {
        let e = euclidean.hodge_star_matrix(&lattice, k);
        let l = lorentzian.hodge_star_matrix(&lattice, k);
        for (i, cell) in lattice.cells(k).enumerate() {
            let expected = expected_fn(cell.orientation()) * e.values()[i];
            assert!(
                (l.values()[i] - expected).abs() < TOL,
                "k = {k}, orientation = {:b}: got {}, expected {expected}",
                cell.orientation(),
                l.values()[i]
            );
        }
    }
}

#[test]
fn lorentzian_hodge_on_open_lattice_handles_boundary_without_panicking() {
    // Lorentzian path through the open-lattice / boundary case (PerEdge branch
    // is exercised by separate per-edge tests; this hits UnitEdge boundary).
    let lattice = open_square_3();
    let lor = unit_geometry::<2>()
        .with_timelike_axes([true, false])
        .unwrap();
    for k in 0..=2 {
        let star = lor.hodge_star_matrix(&lattice, k);
        for v in star.values() {
            assert!(v.is_finite() && !v.is_nan());
        }
    }
}
