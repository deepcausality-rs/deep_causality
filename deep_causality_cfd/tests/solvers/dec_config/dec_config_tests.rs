/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `DecNs` type-state config builder: the optional knobs, the validated getters, and
//! every `build()` rejection branch. The marching cases only ever exercise the happy path
//! (`viscosity → time_step → build`), so the validation and the optional tuners are covered here.

use deep_causality_cfd::DecNs;
use deep_causality_physics::PhysicsErrorEnum;
use deep_causality_topology::HodgeDecomposeOptions;

#[test]
fn test_minimal_build_succeeds_and_getters_read_back() {
    let config = DecNs::config()
        .viscosity(0.1_f64)
        .time_step(0.01)
        .build()
        .expect("valid minimal config");
    assert_eq!(config.nu(), 0.1);
    assert_eq!(config.dt(), 0.01);
    // The CG options getter returns the default projection options.
    let _opts: &HodgeDecomposeOptions<f64> = config.cg_options();
}

#[test]
fn test_all_optional_knobs_build() {
    let config = DecNs::config()
        .viscosity(0.05_f64)
        .time_step(0.02)
        .cg_options(HodgeDecomposeOptions::default())
        .cfl_factors(0.8, 0.7)
        .warm_start()
        .staircase_noslip()
        .spectral_diffusion()
        .build()
        .expect("all optional knobs are valid");
    assert_eq!(config.nu(), 0.05);
    assert_eq!(config.dt(), 0.02);
    // The owned config is Clone + Debug.
    let cloned = config.clone();
    assert_eq!(cloned.dt(), 0.02);
    assert!(format!("{config:?}").contains("DecNsConfig"));
}

#[test]
fn test_non_finite_viscosity_is_numerical_instability() {
    let err = DecNs::config()
        .viscosity(f64::NAN)
        .time_step(0.01)
        .build()
        .expect_err("NaN viscosity rejected");
    assert!(matches!(err.0, PhysicsErrorEnum::NumericalInstability(_)));
}

#[test]
fn test_negative_viscosity_breaks_invariant() {
    let err = DecNs::config()
        .viscosity(-0.1_f64)
        .time_step(0.01)
        .build()
        .expect_err("negative viscosity rejected");
    assert!(matches!(
        err.0,
        PhysicsErrorEnum::PhysicalInvariantBroken(_)
    ));
}

#[test]
fn test_non_positive_time_step_breaks_invariant() {
    for bad_dt in [0.0_f64, -0.01, f64::INFINITY] {
        let err = DecNs::config()
            .viscosity(0.1_f64)
            .time_step(bad_dt)
            .build()
            .expect_err(&format!("dt {bad_dt} should be rejected"));
        assert!(matches!(
            err.0,
            PhysicsErrorEnum::PhysicalInvariantBroken(_)
        ));
    }
}

#[test]
fn test_non_positive_cfl_factor_breaks_invariant() {
    // Advective factor not positive.
    let err = DecNs::config()
        .viscosity(0.1_f64)
        .time_step(0.01)
        .cfl_factors(0.0, 0.9)
        .build()
        .expect_err("zero advective CFL rejected");
    assert!(matches!(
        err.0,
        PhysicsErrorEnum::PhysicalInvariantBroken(_)
    ));

    // Diffusive factor not finite.
    let err = DecNs::config()
        .viscosity(0.1_f64)
        .time_step(0.01)
        .cfl_factors(0.9, f64::NAN)
        .build()
        .expect_err("NaN diffusive CFL rejected");
    assert!(matches!(
        err.0,
        PhysicsErrorEnum::PhysicalInvariantBroken(_)
    ));
}
