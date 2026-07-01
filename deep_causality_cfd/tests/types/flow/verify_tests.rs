/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The MMS-verification workflow `CfdFlow::verify`: feed a `Manufactured` solution's exact pointwise
//! inputs to the incompressible kernel and confirm the residual against the exact `∂u/∂t`, plus the
//! optional kernel-in-the-loop amplitude march against the analytic Taylor–Green decay.

use deep_causality_cfd::{
    CfdConfigBuilder, CfdFlow, Manufactured, ManufacturedSample, TaylorGreen,
};
use deep_causality_physics::PhysicsErrorEnum;

const NU: f64 = 0.1;
const RHO: f64 = 1.0;

/// A degenerate manufactured spec: a finite (accepted) zero density that the pressure-gradient
/// kernel later rejects — so the `incompressible_ns_rhs(...)?` inside `VerifyRun::run` returns Err.
struct ZeroDensitySpec;
impl Manufactured<f64> for ZeroDensitySpec {
    fn sample(&self, _p: &[f64; 3], _t: f64) -> ManufacturedSample<f64> {
        ManufacturedSample {
            velocity: [1.0, 0.0, 0.0],
            velocity_jacobian: [[0.0; 3]; 3],
            velocity_laplacian: [0.0; 3],
            pressure_gradient: [1.0, 0.0, 0.0],
            exact_time_derivative: [0.0; 3],
        }
    }
    fn density(&self) -> f64 {
        0.0 // Density::new(0.0) is accepted, but the pressure-gradient kernel divides by ρ.
    }
    fn viscosity(&self) -> f64 {
        NU
    }
}

#[test]
fn test_verify_residual_is_machine_zero() {
    let config = CfdConfigBuilder::verify::<f64, _>("tgv-verify", TaylorGreen::new(NU, RHO))
        .sample_at([1.0, 0.5, 0.0], 0.0)
        .build()
        .expect("sample point set");
    let report = CfdFlow::verify(&config)
        .run()
        .expect("verify workflow runs");

    assert_eq!(report.name(), "tgv-verify");

    let err = report.series("mms_error").expect("mms_error series")[0];
    assert!(
        err < 1e-12,
        "kernel residual {err} exceeds machine tolerance"
    );

    // The kernel `∂u/∂t` reproduces the exact reference componentwise.
    let kernel = report.series("kernel_dudt").expect("kernel_dudt");
    let exact = report.series("exact_dudt").expect("exact_dudt");
    assert_eq!(kernel.len(), 3);
    for (k, e) in kernel.iter().zip(exact.iter()) {
        assert!((k - e).abs() < 1e-12);
    }

    // The exact reference is −2ν·u.
    let velocity = report.series("velocity").expect("velocity");
    for (e, u) in exact.iter().zip(velocity.iter()) {
        assert!((e - (-2.0 * NU) * u).abs() < 1e-12);
    }

    // No amplitude march was configured.
    assert!(report.series("amplitude_final").is_none());
    assert!(report.series("amplitude_exact").is_none());
    // No DEC march, so there is no final edge cochain.
    assert!(report.final_field().is_none());
}

#[test]
fn test_verify_propagates_a_kernel_error() {
    // The kernel evaluation inside `run` short-circuits with `?`: a zero density passes the
    // `Density::new` guard but the pressure-gradient kernel rejects it, surfacing the error.
    let config = CfdConfigBuilder::verify::<f64, _>("zero-density", ZeroDensitySpec)
        .sample_at([1.0, 0.5, 0.0], 0.0)
        .build()
        .expect("sample point set");
    let err = CfdFlow::verify(&config)
        .run()
        .expect_err("the pressure-gradient kernel rejects a zero density");
    assert!(matches!(
        err.0,
        PhysicsErrorEnum::PhysicalInvariantBroken(_)
    ));
}

#[test]
fn test_verify_amplitude_march_matches_analytic_decay() {
    let dt = 0.005_f64;
    let steps = 200usize;
    let config = CfdConfigBuilder::verify::<f64, _>("tgv-amp", TaylorGreen::new(NU, RHO))
        .sample_at([1.0, 0.5, 0.0], 0.0)
        .amplitude_march(dt, steps)
        .build()
        .expect("sample point set");
    let report = CfdFlow::verify(&config)
        .run()
        .expect("verify workflow runs");

    let a_final = report.series("amplitude_final").expect("amplitude_final")[0];
    let a_exact = report.series("amplitude_exact").expect("amplitude_exact")[0];

    // Analytic decay a(t) = exp(−2ν t) over t = dt·steps = 1.0.
    let t_final = dt * steps as f64;
    let analytic = (-2.0 * NU * t_final).exp();
    assert!((a_exact - analytic).abs() < 1e-12);
    // RK4 march reproduces the exponential decay closely.
    assert!(
        (a_final - a_exact).abs() < 1e-6,
        "march {a_final} vs exact {a_exact}"
    );
}
