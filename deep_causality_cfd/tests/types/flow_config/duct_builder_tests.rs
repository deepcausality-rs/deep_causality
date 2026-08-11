/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `DuctConfigBuilder` — the required-section reports and the value validation that
//! `build()` carries. Started by `CfdConfigBuilder::duct`, the only public entry to a duct case.

use deep_causality_cfd::{CfdConfigBuilder, DuctAreaProfile, DuctConfigBuilder};

fn nozzle() -> DuctAreaProfile<f64> {
    DuctAreaProfile::ConvergingDiverging {
        inlet_area: 3.0,
        throat_area: 1.0,
        exit_area: 2.0,
        length: 1.0,
    }
}

/// Every section set to a valid value; each test overrides exactly the one it is about.
fn valid() -> DuctConfigBuilder<f64> {
    CfdConfigBuilder::duct::<f64>("duct-case")
        .profile(nozzle())
        .inlet(100_000.0, 300.0)
        .gamma(1.4)
        .back_pressure(70_000.0)
        .cells(64)
        .stop(10_000, 1e-8)
}

#[test]
fn test_builds_from_a_complete_specification() {
    assert!(valid().build().is_ok());
}

// --- required sections -------------------------------------------------------------------------

#[test]
fn test_rejects_missing_profile() {
    let r = CfdConfigBuilder::duct::<f64>("no-profile")
        .inlet(100_000.0, 300.0)
        .gamma(1.4)
        .back_pressure(70_000.0)
        .cells(64)
        .stop(10_000, 1e-8)
        .build();
    assert!(r.is_err(), "a missing profile must be rejected");
}

#[test]
fn test_rejects_missing_inlet() {
    let r = CfdConfigBuilder::duct::<f64>("no-inlet")
        .profile(nozzle())
        .gamma(1.4)
        .back_pressure(70_000.0)
        .cells(64)
        .stop(10_000, 1e-8)
        .build();
    assert!(r.is_err(), "a missing inlet state must be rejected");
}

#[test]
fn test_rejects_missing_gamma() {
    let r = CfdConfigBuilder::duct::<f64>("no-gamma")
        .profile(nozzle())
        .inlet(100_000.0, 300.0)
        .back_pressure(70_000.0)
        .cells(64)
        .stop(10_000, 1e-8)
        .build();
    assert!(r.is_err(), "a missing gamma must be rejected");
}

#[test]
fn test_rejects_missing_back_pressure() {
    let r = CfdConfigBuilder::duct::<f64>("no-back-pressure")
        .profile(nozzle())
        .inlet(100_000.0, 300.0)
        .gamma(1.4)
        .cells(64)
        .stop(10_000, 1e-8)
        .build();
    assert!(r.is_err(), "a missing back pressure must be rejected");
}

#[test]
fn test_rejects_missing_cells() {
    let r = CfdConfigBuilder::duct::<f64>("no-cells")
        .profile(nozzle())
        .inlet(100_000.0, 300.0)
        .gamma(1.4)
        .back_pressure(70_000.0)
        .stop(10_000, 1e-8)
        .build();
    assert!(r.is_err(), "a missing cell count must be rejected");
}

#[test]
fn test_rejects_missing_stop() {
    let r = CfdConfigBuilder::duct::<f64>("no-stop")
        .profile(nozzle())
        .inlet(100_000.0, 300.0)
        .gamma(1.4)
        .back_pressure(70_000.0)
        .cells(64)
        .build();
    assert!(r.is_err(), "a missing stop condition must be rejected");
}

// --- geometry ----------------------------------------------------------------------------------

#[test]
fn test_rejects_short_table() {
    let table = DuctAreaProfile::Table(vec![(0.0, 3.0)]);
    assert!(valid().profile(table).build().is_err());
}

#[test]
fn test_rejects_unsorted_table() {
    let table = DuctAreaProfile::Table(vec![(0.0, 3.0), (0.5, 1.0), (0.5, 2.0)]);
    assert!(valid().profile(table).build().is_err());
}

#[test]
fn test_rejects_nonpositive_table_area() {
    let table = DuctAreaProfile::Table(vec![(0.0, 3.0), (0.5, 0.0), (1.0, 2.0)]);
    assert!(valid().profile(table).build().is_err());
    let table = DuctAreaProfile::Table(vec![(0.0, 3.0), (0.5, f64::NAN), (1.0, 2.0)]);
    assert!(valid().profile(table).build().is_err());
}

#[test]
fn test_rejects_nonpositive_analytic_area() {
    let profile = DuctAreaProfile::ConvergingDiverging {
        inlet_area: 3.0,
        throat_area: 0.0,
        exit_area: 2.0,
        length: 1.0,
    };
    assert!(valid().profile(profile).build().is_err());
}

#[test]
fn test_rejects_throat_not_strict_minimum() {
    let profile = DuctAreaProfile::ConvergingDiverging {
        inlet_area: 1.0,
        throat_area: 1.0,
        exit_area: 2.0,
        length: 1.0,
    };
    assert!(valid().profile(profile).build().is_err());
}

#[test]
fn test_rejects_nonpositive_length() {
    let profile = DuctAreaProfile::ConvergingDiverging {
        inlet_area: 3.0,
        throat_area: 1.0,
        exit_area: 2.0,
        length: 0.0,
    };
    assert!(valid().profile(profile).build().is_err());
}

// --- physical state ----------------------------------------------------------------------------

#[test]
fn test_rejects_bad_stagnation_state() {
    assert!(valid().inlet(0.0, 300.0).build().is_err());
    assert!(valid().inlet(100_000.0, f64::NAN).build().is_err());
}

#[test]
fn test_rejects_gamma_not_above_one() {
    assert!(valid().gamma(1.0).build().is_err());
    assert!(valid().gamma(f64::NAN).build().is_err());
}

#[test]
fn test_rejects_back_pressure_at_or_above_p0() {
    assert!(valid().back_pressure(100_000.0).build().is_err());
    assert!(valid().back_pressure(120_000.0).build().is_err());
    assert!(valid().back_pressure(0.0).build().is_err());
}

#[test]
fn test_rejects_too_few_cells() {
    assert!(valid().cells(7).build().is_err());
}

#[test]
fn test_rejects_bad_stop_condition() {
    assert!(valid().stop(0, 1e-8).build().is_err());
    assert!(valid().stop(100, f64::NAN).build().is_err());
    assert!(valid().stop(100, 0.0).build().is_err());
}
