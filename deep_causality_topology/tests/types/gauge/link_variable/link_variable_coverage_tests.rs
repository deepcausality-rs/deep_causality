/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Additional coverage for LinkVariable phase construction and SU(N) projection
//! paths that require matrix dimensions other than 1, 2 or 3.

use deep_causality_num::Complex;
use deep_causality_topology::{LinkVariable, LinkVariableError, SE3, SO3_1};

// ============================================================================
// try_from_phase: general SU(n) arm for n >= 4 (matrix_dim() == 4 here).
// ============================================================================

#[test]
fn test_try_from_phase_general_arm_dim_four() {
    // SO3_1::matrix_dim() == 4, so this drives the `_ =>` general SU(n) arm:
    //   diag(exp(iφ), exp(-iφ/(n-1)), exp(-iφ/(n-1)), ...).
    let phase = 0.5_f64;
    let link: LinkVariable<SO3_1, Complex<f64>, f64> =
        LinkVariable::try_from_phase(phase).expect("phase link for 4x4 group");

    let s = link.as_slice();
    assert_eq!(s.len(), 16);

    // First diagonal entry is exp(iφ).
    let expected0 = Complex::new(phase.cos(), phase.sin());
    assert!((s[0].re - expected0.re).abs() < 1e-12);
    assert!((s[0].im - expected0.im).abs() < 1e-12);

    // Remaining diagonal entries are exp(-iφ/(n-1)) with n = 4.
    let comp_angle = -phase / 3.0;
    let comp = Complex::new(comp_angle.cos(), comp_angle.sin());
    for i in 1..4 {
        let d = s[i * 4 + i];
        assert!((d.re - comp.re).abs() < 1e-12);
        assert!((d.im - comp.im).abs() < 1e-12);
    }
}

#[test]
fn test_try_from_phase_general_arm_se3() {
    // SE3::matrix_dim() == 4 as well: independent confirmation of the general arm.
    let link: LinkVariable<SE3, Complex<f64>, f64> =
        LinkVariable::try_from_phase(0.25).expect("phase link for SE3");
    assert_eq!(link.as_slice().len(), 16);
}

// ============================================================================
// project_sun -> try_determinant: dimension other than 2/3 returns an error.
// ============================================================================

#[test]
fn test_project_sun_unsupported_determinant_dimension_errors() {
    // SO3_1 has n == 4. project_sun runs the Newton-Schulz iteration, then (since
    // n >= 2) calls try_determinant, whose only supported sizes are 2 and 3.
    // The 4x4 case hits the `_ => Err(InvalidDimension)` arm.
    let id: LinkVariable<SO3_1, Complex<f64>, f64> = LinkVariable::identity();
    let err = id
        .project_sun()
        .expect_err("4x4 determinant is unsupported");
    match err {
        LinkVariableError::InvalidDimension(n) => assert_eq!(n, 4),
        other => panic!("expected InvalidDimension(4), got {:?}", other),
    }
}
