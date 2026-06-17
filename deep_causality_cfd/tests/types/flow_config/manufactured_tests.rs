/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the Taylor–Green manufactured solution and the MMS-verification config builder.
//!
//! The autodiff-derived spatial derivatives (Jacobian, Laplacian, pressure gradient) are checked
//! against the closed-form analytic expressions for the Taylor–Green vortex
//! `u = sin x · cos y · F`, `v = −cos x · sin y · F`, `w = 0`, `p = (ρ/4)(cos 2x + cos 2y)·F²`,
//! with `F(t) = exp(−2 ν t)`.

use deep_causality_cfd::{CfdConfigBuilder, Manufactured, TaylorGreen};
use deep_causality_physics::PhysicsErrorEnum;

const NU: f64 = 0.1;
const RHO: f64 = 1.3;
const TOL: f64 = 1e-12;

fn decay(t: f64) -> f64 {
    (-2.0 * NU * t).exp()
}

#[test]
fn test_density_and_viscosity_getters() {
    let tg = TaylorGreen::new(NU, RHO);
    assert_eq!(Manufactured::<f64>::density(&tg), RHO);
    assert_eq!(Manufactured::<f64>::viscosity(&tg), NU);
}

#[test]
fn test_velocity_matches_analytic_field() {
    let tg = TaylorGreen::new(NU, RHO);
    let p = [0.5_f64, 0.7, 0.3];
    let t = 0.0;
    let s = tg.sample(&p, t);
    let f = decay(t);

    assert!((s.velocity[0] - p[0].sin() * p[1].cos() * f).abs() < TOL);
    assert!((s.velocity[1] - (-(p[0].cos() * p[1].sin() * f))).abs() < TOL);
    // w is identically zero (2D field embedded in 3D).
    assert_eq!(s.velocity[2], 0.0);
}

#[test]
fn test_decay_scales_velocity_in_time() {
    let tg = TaylorGreen::new(NU, RHO);
    let p = [0.9_f64, 0.4, 0.0];
    let s0 = tg.sample(&p, 0.0);
    let t = 0.75;
    let st = tg.sample(&p, t);
    // Each component scales by the single exponential factor F(t).
    let f = decay(t);
    assert!((st.velocity[0] - s0.velocity[0] * f).abs() < TOL);
    assert!((st.velocity[1] - s0.velocity[1] * f).abs() < TOL);
}

#[test]
fn test_velocity_jacobian_matches_analytic_gradient() {
    let tg = TaylorGreen::new(NU, RHO);
    let p = [0.5_f64, 0.7, 0.3];
    let t = 0.2;
    let f = decay(t);
    let (x, y) = (p[0], p[1]);
    let j = tg.sample(&p, t).velocity_jacobian;

    // Row 0: ∇u, u = sin x cos y F.
    assert!((j[0][0] - x.cos() * y.cos() * f).abs() < TOL);
    assert!((j[0][1] - (-(x.sin() * y.sin() * f))).abs() < TOL);
    assert!(j[0][2].abs() < TOL);
    // Row 1: ∇v, v = −cos x sin y F.
    assert!((j[1][0] - x.sin() * y.sin() * f).abs() < TOL);
    assert!((j[1][1] - (-(x.cos() * y.cos() * f))).abs() < TOL);
    assert!(j[1][2].abs() < TOL);
    // Row 2: ∇w, w ≡ 0.
    assert!(j[2].iter().all(|v| v.abs() < TOL));
}

#[test]
fn test_velocity_laplacian_is_minus_two_velocity() {
    // Each Taylor–Green component is a 2D Laplacian eigenfunction with eigenvalue −2,
    // and is z-independent, so ∇²u = −2u exactly.
    let tg = TaylorGreen::new(NU, RHO);
    let p = [0.5_f64, 0.7, 0.3];
    let t = 0.2;
    let s = tg.sample(&p, t);
    for i in 0..3 {
        assert!(
            (s.velocity_laplacian[i] - (-2.0) * s.velocity[i]).abs() < TOL,
            "component {i}: lap {} vs -2u {}",
            s.velocity_laplacian[i],
            -2.0 * s.velocity[i]
        );
    }
}

#[test]
fn test_pressure_gradient_matches_analytic() {
    let tg = TaylorGreen::new(NU, RHO);
    let p = [0.5_f64, 0.7, 0.3];
    let t = 0.2;
    let f = decay(t);
    let f2 = f * f;
    let (x, y) = (p[0], p[1]);
    let g = tg.sample(&p, t).pressure_gradient;

    // ∂p/∂x = −(ρ/2) sin(2x) F², ∂p/∂y = −(ρ/2) sin(2y) F², ∂p/∂z = 0.
    assert!((g[0] - (-(RHO / 2.0) * (2.0 * x).sin() * f2)).abs() < TOL);
    assert!((g[1] - (-(RHO / 2.0) * (2.0 * y).sin() * f2)).abs() < TOL);
    assert!(g[2].abs() < TOL);
}

#[test]
fn test_exact_time_derivative_is_minus_two_nu_velocity() {
    let tg = TaylorGreen::new(NU, RHO);
    let p = [0.5_f64, 0.7, 0.3];
    let t = 0.2;
    let s = tg.sample(&p, t);
    for i in 0..3 {
        assert!((s.exact_time_derivative[i] - (-2.0 * NU) * s.velocity[i]).abs() < TOL);
    }
}

#[test]
fn test_manufactured_sample_is_copy_debug() {
    let tg = TaylorGreen::new(NU, RHO);
    let s = tg.sample(&[0.3_f64, 0.4, 0.5], 0.1);
    let copied = s; // ManufacturedSample is Copy
    assert_eq!(copied.velocity, s.velocity);
    assert!(format!("{s:?}").contains("ManufacturedSample"));
    assert!(format!("{tg:?}").contains("TaylorGreen"));
}

#[test]
fn test_verify_builder_requires_sample_point() {
    // Building without `sample_at` is a configuration error. (`VerifyConfig` is not `Debug`, so
    // the error is destructured directly rather than via `unwrap_err`.)
    let result =
        CfdConfigBuilder::verify::<f64, _>("missing-point", TaylorGreen::new(NU, RHO)).build();
    let Err(err) = result else {
        panic!("a missing sample point must be an error");
    };
    assert!(matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)));
}

#[test]
fn test_verify_builder_build_succeeds_with_sample_point() {
    let config = CfdConfigBuilder::verify::<f64, _>("tgv", TaylorGreen::new(NU, RHO))
        .sample_at([1.0, 0.5, 0.0], 0.0)
        .amplitude_march(0.01, 10)
        .build();
    assert!(config.is_ok());
}
