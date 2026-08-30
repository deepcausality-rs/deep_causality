/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `CubicalReggeGeometry::<D, R, Lorentzian>::regge_action_lorentzian`
//! — Phase R5.6.
//!
//! Returns `Complex<R>`. Real part is zero (purely-imaginary Wick rotation in
//! the chosen phase convention); imaginary part equals the Euclidean
//! `regge_action` evaluated on the same edge-length data — the design.md
//! Decision-3 reduction property.

use deep_causality_topology::utils_tests::{
    open_cube_3, open_square_3, per_axis_geometry, unit_geometry,
};

const TOL: f64 = 1e-12;

#[test]
fn lorentzian_action_real_part_is_zero_on_open_2d_unit_lattice() {
    let lattice = open_square_3();
    let lor = unit_geometry::<2>()
        .with_timelike_axes([true, false])
        .unwrap();
    let s = lor.regge_action_lorentzian(&lattice);
    assert!(s.re.abs() < TOL, "real part must be zero, got {}", s.re);
}

#[test]
fn lorentzian_action_imag_part_equals_euclidean_action_on_open_2d() {
    let lattice = open_square_3();
    let euc = unit_geometry::<2>();
    let lor = unit_geometry::<2>()
        .with_timelike_axes([true, false])
        .unwrap();
    let s_e = euc.regge_action(&lattice);
    let s_l = lor.regge_action_lorentzian(&lattice);
    assert!((s_l.im - s_e).abs() < TOL);
}

#[test]
fn lorentzian_action_imag_part_equals_euclidean_action_on_open_3d_per_axis() {
    let lattice = open_cube_3();
    let lengths = [2.0_f64, 3.0, 5.0];
    let euc = per_axis_geometry::<3>(lengths);
    let lor = per_axis_geometry::<3>(lengths)
        .with_timelike_axes([false, false, true])
        .unwrap();
    let s_e = euc.regge_action(&lattice);
    let s_l = lor.regge_action_lorentzian(&lattice);
    assert!(s_l.re.abs() < TOL);
    assert!((s_l.im - s_e).abs() < TOL);
}

#[test]
fn lorentzian_action_choice_of_timelike_axis_does_not_change_value() {
    // Under axis-aligned cubical assumption, the action depends only on hinge
    // volumes and incidence counts — neither sees the signature. So flipping
    // which axis is timelike must produce the same Lorentzian action value.
    let lattice = open_square_3();
    let a = unit_geometry::<2>()
        .with_timelike_axes([true, false])
        .unwrap();
    let b = unit_geometry::<2>()
        .with_timelike_axes([false, true])
        .unwrap();
    let s_a = a.regge_action_lorentzian(&lattice);
    let s_b = b.regge_action_lorentzian(&lattice);
    assert!((s_a.re - s_b.re).abs() < TOL);
    assert!((s_a.im - s_b.im).abs() < TOL);
}
