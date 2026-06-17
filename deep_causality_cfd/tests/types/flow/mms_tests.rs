/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The MMS-verification CfdFlow solver kind, across every regime (the "all regimes" showcase):
//! each regime's pointwise kernel is checked against an exact manufactured reference through the
//! same `CfdFlow::verify_mms` DSL surface.

use deep_causality_cfd::{CfdFlow, Regime};

/// The regime showcase: incompressible, Euler, Stokes, and compressible are all reachable
/// through `CfdFlow::verify_mms`, and each kernel reproduces its manufactured reference to
/// machine precision.
#[test]
fn every_regime_verifies_against_its_manufactured_solution() {
    let nu = 0.1_f64;
    let rho = 1.0_f64;

    for regime in [
        Regime::Incompressible,
        Regime::Euler,
        Regime::Stokes,
        Regime::Compressible,
    ] {
        let report = CfdFlow::verify_mms::<f64>("regime-mms")
            .regime(regime)
            .viscosity(nu)
            .density(rho)
            .run()
            .unwrap_or_else(|e| panic!("{regime:?} MMS runs: {e}"));

        let err = report.series("mms_error").expect("mms_error series")[0];
        assert!(
            err < 1e-12,
            "{regime:?} kernel error {err} exceeds the manufactured tolerance"
        );
    }
}

/// The incompressible Taylor–Green decay is reproduced exactly: the kernel residual against
/// `∂u/∂t = −2ν u` is at machine zero.
#[test]
fn incompressible_taylor_green_decay_is_exact() {
    let report = CfdFlow::verify_mms::<f64>("tgv-mms")
        .regime(Regime::Incompressible)
        .viscosity(0.1)
        .density(1.0)
        .run()
        .expect("incompressible MMS runs");
    assert!(report.series("mms_error").expect("mms_error")[0] < 1e-13);
}

/// The compressible regime additionally pins the continuity residual at the divergence-free
/// manufactured state.
#[test]
fn compressible_regime_pins_continuity() {
    let report = CfdFlow::verify_mms::<f64>("compressible-mms")
        .regime(Regime::Compressible)
        .viscosity(0.05)
        .density(1.2)
        .run()
        .expect("compressible MMS runs");
    assert!(report.series("mms_error").expect("mms_error")[0] < 1e-12);
    assert!(
        report
            .series("continuity_error")
            .expect("continuity_error series")[0]
            < 1e-13,
        "the divergence-free manufactured state has zero continuity residual"
    );
}

/// Verification works at reduced precision too (the DSL is precision-generic): the f32 kernel
/// still reproduces the reference within the f32 tolerance.
#[test]
fn verification_is_precision_generic() {
    let report = CfdFlow::verify_mms::<f32>("tgv-mms-f32")
        .regime(Regime::Incompressible)
        .viscosity(0.1_f32)
        .density(1.0_f32)
        .run()
        .expect("f32 MMS runs");
    assert!(report.series("mms_error").expect("mms_error")[0] < 1e-6);
}
