/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Area, Density, Force, JARVINEN_ADAMS_CA0_M2, JARVINEN_ADAMS_TRANSITION_CT_M2, Pressure, Speed,
    jarvinen_adams_baseline_axial_coefficient_kernel, momentum_flux_ratio_kernel,
    srp_flow_regime_margin_kernel, srp_preserved_drag_fraction_kernel,
    srp_thrust_coefficient_kernel, srp_total_axial_force_coefficient_kernel,
};

#[test]
fn test_thrust_coefficient_definition() {
    // C_T = T/(q_inf * S_ref): 1000 N, q = 500 Pa, S = 0.5 m^2 -> C_T = 4.0.
    let c_t = srp_thrust_coefficient_kernel(
        Force::new(1000.0_f64).unwrap(),
        Pressure::new(500.0).unwrap(),
        Area::new(0.5).unwrap(),
    )
    .unwrap();
    assert!((c_t - 4.0).abs() < 1e-12);
}

#[test]
fn test_thrust_coefficient_rejects_degenerate() {
    assert!(
        srp_thrust_coefficient_kernel(
            Force::new(1000.0_f64).unwrap(),
            Pressure::new(0.0).unwrap(),
            Area::new(0.5).unwrap()
        )
        .is_err()
    );
    assert!(
        srp_thrust_coefficient_kernel(
            Force::new(-1.0_f64).unwrap(),
            Pressure::new(500.0).unwrap(),
            Area::new(0.5).unwrap()
        )
        .is_err()
    );
    // Zero reference area is a singularity.
    assert!(
        srp_thrust_coefficient_kernel(
            Force::new(1000.0_f64).unwrap(),
            Pressure::new(500.0).unwrap(),
            Area::new(0.0).unwrap()
        )
        .is_err()
    );
}

#[test]
fn test_preserved_drag_interpolates_between_knots_and_rejects_nan() {
    // A C_T strictly between two digitized abscissae exercises the
    // interpolation arm (not an exact knot): between 0.46 (0.22) and 0.72
    // (0.17) the value lies strictly inside (0.17, 0.22).
    let mid = srp_preserved_drag_fraction_kernel(0.6_f64).unwrap();
    assert!(mid < 0.22 && mid > 0.17, "interpolated fraction {mid}");
    // A non-finite C_T reaches the interpolator's finiteness guard (it passes
    // the c_t < 0 check because NaN comparisons are false).
    assert!(srp_preserved_drag_fraction_kernel(f64::NAN).is_err());
}

#[test]
fn test_momentum_flux_ratio_definition() {
    // J = (rho_j u_j^2)/(rho_inf u_inf^2): equal states -> 1.0.
    let j = momentum_flux_ratio_kernel(
        Density::new(2.0_f64).unwrap(),
        Speed::new(100.0).unwrap(),
        Density::new(0.5).unwrap(),
        Speed::new(200.0).unwrap(),
    )
    .unwrap();
    // (2 * 100^2) / (0.5 * 200^2) = 20000 / 20000 = 1.0
    assert!((j - 1.0).abs() < 1e-12);
}

#[test]
fn test_momentum_flux_ratio_rejects_zero_freestream() {
    assert!(
        momentum_flux_ratio_kernel(
            Density::new(2.0_f64).unwrap(),
            Speed::new(100.0).unwrap(),
            Density::new(0.0).unwrap(),
            Speed::new(200.0).unwrap()
        )
        .is_err()
    );
}

#[test]
fn test_preserved_drag_reproduces_digitized_points() {
    // Every digitized (C_T, fraction) abscissa from JARVINEN_ADAMS_PRESERVED_DRAG_M2
    // must return its ordinate exactly (interpolation lands on the knot).
    for &(c_t, frac) in &[
        (0.00_f64, 1.00_f64),
        (0.46, 0.22),
        (1.05, 0.02),
        (1.98, -0.03),
        (8.80, -0.12),
    ] {
        let got = srp_preserved_drag_fraction_kernel(c_t).unwrap();
        assert!(
            (got - frac).abs() < 1e-9,
            "C_T {c_t}: got {got}, expected {frac}"
        );
    }
}

#[test]
fn test_preserved_drag_collapse_structure() {
    // The report's central-nozzle drag collapse: preserved fraction is a
    // small remnant of its unpowered (1.0) value by C_T ~ 1, corroborated by
    // the Korzun survey's "~10% of the no-jet value".
    let near_unity = srp_preserved_drag_fraction_kernel(1.05_f64).unwrap();
    assert!(
        near_unity < 0.10,
        "drag not collapsed at C_T ~ 1: {near_unity}"
    );
    // Monotone drop from C_T = 0 to the collapse.
    assert!(srp_preserved_drag_fraction_kernel(0.0_f64).unwrap() > 0.9);
    assert!(
        srp_preserved_drag_fraction_kernel(0.46_f64).unwrap()
            > srp_preserved_drag_fraction_kernel(1.03_f64).unwrap()
    );
}

#[test]
fn test_preserved_drag_rejects_out_of_domain() {
    assert!(srp_preserved_drag_fraction_kernel(-0.1_f64).is_err());
    assert!(srp_preserved_drag_fraction_kernel(9.0_f64).is_err()); // domain ends at 8.8
}

#[test]
fn test_baseline_axial_coefficient_digitized_points() {
    for &(m, ca0) in &[
        (0.60_f64, 0.60_f64),
        (1.05, 0.89),
        (1.50, 1.12),
        (2.00, 1.25),
    ] {
        let got = jarvinen_adams_baseline_axial_coefficient_kernel(m).unwrap();
        assert!((got - ca0).abs() < 1e-9, "M {m}: got {got}, expected {ca0}");
    }
    // Interpolation between knots stays inside the bracket.
    let mid = jarvinen_adams_baseline_axial_coefficient_kernel(0.70_f64).unwrap();
    assert!(mid > 0.60 && mid < 0.68);
}

#[test]
fn test_baseline_axial_coefficient_rejects_out_of_envelope() {
    assert!(jarvinen_adams_baseline_axial_coefficient_kernel(0.4_f64).is_err());
    assert!(jarvinen_adams_baseline_axial_coefficient_kernel(2.5_f64).is_err());
}

#[test]
fn test_total_axial_force_non_monotone_band() {
    // At M = 2.0, C_A,total = C_T + preserved(C_T)*C_A0 dips BELOW the
    // unpowered C_A0 = 1.25 in the low-C_T band (lighting the engine gently
    // buys less deceleration than coasting), then rises as thrust dominates.
    let unpowered = JARVINEN_ADAMS_CA0_M2;
    let at_046 = srp_total_axial_force_coefficient_kernel(0.46_f64, 2.0).unwrap();
    assert!(
        at_046 < unpowered,
        "low-C_T total axial force {at_046} should dip below unpowered {unpowered}"
    );
    // Marginal gain in the low band is well below 1 (thrust replacing lost drag).
    let at_00 = srp_total_axial_force_coefficient_kernel(0.0_f64, 2.0).unwrap();
    let marginal = (at_046 - at_00) / 0.46;
    assert!(
        marginal < 1.0,
        "marginal C_A per C_T in the collapse band should be < 1, got {marginal}"
    );
    // By high C_T thrust dominates and total axial force exceeds unpowered.
    let at_4 = srp_total_axial_force_coefficient_kernel(4.05_f64, 2.0).unwrap();
    assert!(at_4 > unpowered);
}

#[test]
fn test_flow_regime_margin_sign() {
    let transition = JARVINEN_ADAMS_TRANSITION_CT_M2; // 1.0 at M = 2
    assert!(srp_flow_regime_margin_kernel(2.0_f64, transition).unwrap() > 0.0);
    assert!(srp_flow_regime_margin_kernel(0.5_f64, transition).unwrap() < 0.0);
    assert!(
        srp_flow_regime_margin_kernel(1.0_f64, transition)
            .unwrap()
            .abs()
            < 1e-12
    );
}

#[test]
fn test_flow_regime_margin_rejects_bad_inputs() {
    assert!(srp_flow_regime_margin_kernel(-1.0_f64, 1.0).is_err());
    assert!(srp_flow_regime_margin_kernel(2.0_f64, 0.0).is_err());
}
