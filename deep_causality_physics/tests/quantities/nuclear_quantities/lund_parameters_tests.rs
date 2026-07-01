/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::LundParameters;

#[test]
fn test_lund_parameters_default() {
    let params = LundParameters::<f64>::default();
    assert!((params.kappa() - 1.0).abs() < 1e-10);
    assert!(params.strange_suppression() > 0.0);
    assert!(params.strange_suppression() < 1.0);
}

#[test]
fn test_lund_parameters_custom() {
    let params = LundParameters::<f64>::new(
        2.0, // kappa
        0.5, // a
        0.8, // b
        0.4, // sigma_pt
        0.2, // strange
        0.1, // diquark
        0.6, // vector
        0.3, // min mass
    );

    assert_eq!(params.kappa(), 2.0);
    assert_eq!(params.lund_a(), 0.5);
    assert_eq!(params.min_invariant_mass(), 0.3);
}

#[test]
fn test_lund_parameters_remaining_getters() {
    let params = LundParameters::<f64>::new(2.0, 0.5, 0.8, 0.4, 0.2, 0.1, 0.6, 0.3);
    assert!((params.lund_b() - 0.8).abs() < 1e-10);
    assert!((params.sigma_pt() - 0.4).abs() < 1e-10);
    assert!((params.diquark_suppression() - 0.1).abs() < 1e-10);
    assert!((params.vector_meson_fraction() - 0.6).abs() < 1e-10);
}
