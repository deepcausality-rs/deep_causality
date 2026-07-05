/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{DuctAreaProfile, DuctConfig, DuctInlet, DuctStop};

fn nozzle() -> DuctAreaProfile<f64> {
    DuctAreaProfile::ConvergingDiverging {
        inlet_area: 3.0,
        throat_area: 1.0,
        exit_area: 2.0,
        length: 1.0,
    }
}

fn inlet() -> DuctInlet<f64> {
    DuctInlet {
        p0: 100_000.0,
        t0: 300.0,
    }
}

fn stop() -> DuctStop<f64> {
    DuctStop {
        max_steps: 10_000,
        residual_tol: 1e-8,
    }
}

#[test]
fn test_valid_analytic_config() {
    let cfg = DuctConfig::new(nozzle(), inlet(), 1.4, 70_000.0, 128, stop()).unwrap();
    assert_eq!(cfg.cells(), 128);
    assert_eq!(cfg.max_steps(), 10_000);
    assert_eq!(cfg.p0(), 100_000.0);
    assert_eq!(cfg.t0(), 300.0);
    assert_eq!(cfg.gamma(), 1.4);
    assert_eq!(cfg.back_pressure(), 70_000.0);
    assert_eq!(cfg.residual_tol(), 1e-8);
    // The analytic profile: throat is the minimum, ends match the spec.
    match cfg.profile() {
        DuctAreaProfile::ConvergingDiverging { throat_area, .. } => {
            assert_eq!(*throat_area, 1.0);
        }
        DuctAreaProfile::Table(_) => panic!("expected the analytic variant"),
    }
}

#[test]
fn test_valid_table_config() {
    let table = DuctAreaProfile::Table(vec![(0.0, 3.0), (0.5, 1.0), (1.0, 2.0)]);
    let cfg = DuctConfig::new(table, inlet(), 1.4, 70_000.0, 64, stop());
    assert!(cfg.is_ok());
}

#[test]
fn test_rejects_short_table() {
    let table = DuctAreaProfile::Table(vec![(0.0, 3.0)]);
    assert!(DuctConfig::new(table, inlet(), 1.4, 70_000.0, 64, stop()).is_err());
}

#[test]
fn test_rejects_unsorted_table() {
    let table = DuctAreaProfile::Table(vec![(0.0, 3.0), (0.5, 1.0), (0.5, 2.0)]);
    assert!(DuctConfig::new(table, inlet(), 1.4, 70_000.0, 64, stop()).is_err());
}

#[test]
fn test_rejects_nonpositive_table_area() {
    let table = DuctAreaProfile::Table(vec![(0.0, 3.0), (0.5, 0.0), (1.0, 2.0)]);
    assert!(DuctConfig::new(table, inlet(), 1.4, 70_000.0, 64, stop()).is_err());
    let table = DuctAreaProfile::Table(vec![(0.0, 3.0), (0.5, f64::NAN), (1.0, 2.0)]);
    assert!(DuctConfig::new(table, inlet(), 1.4, 70_000.0, 64, stop()).is_err());
}

#[test]
fn test_rejects_throat_not_strict_minimum() {
    let profile = DuctAreaProfile::ConvergingDiverging {
        inlet_area: 1.0,
        throat_area: 1.0,
        exit_area: 2.0,
        length: 1.0,
    };
    assert!(DuctConfig::new(profile, inlet(), 1.4, 70_000.0, 64, stop()).is_err());
}

#[test]
fn test_rejects_nonpositive_length() {
    let profile = DuctAreaProfile::ConvergingDiverging {
        inlet_area: 3.0,
        throat_area: 1.0,
        exit_area: 2.0,
        length: 0.0,
    };
    assert!(DuctConfig::new(profile, inlet(), 1.4, 70_000.0, 64, stop()).is_err());
}

#[test]
fn test_rejects_bad_stagnation_state() {
    let bad_p0 = DuctInlet { p0: 0.0, t0: 300.0 };
    assert!(DuctConfig::new(nozzle(), bad_p0, 1.4, 70_000.0, 64, stop()).is_err());
    let bad_t0 = DuctInlet {
        p0: 100_000.0,
        t0: f64::NAN,
    };
    assert!(DuctConfig::new(nozzle(), bad_t0, 1.4, 70_000.0, 64, stop()).is_err());
}

#[test]
fn test_rejects_gamma_not_above_one() {
    assert!(DuctConfig::new(nozzle(), inlet(), 1.0, 70_000.0, 64, stop()).is_err());
    assert!(DuctConfig::new(nozzle(), inlet(), f64::NAN, 70_000.0, 64, stop()).is_err());
}

#[test]
fn test_rejects_back_pressure_at_or_above_p0() {
    assert!(DuctConfig::new(nozzle(), inlet(), 1.4, 100_000.0, 64, stop()).is_err());
    assert!(DuctConfig::new(nozzle(), inlet(), 1.4, 120_000.0, 64, stop()).is_err());
    assert!(DuctConfig::new(nozzle(), inlet(), 1.4, 0.0, 64, stop()).is_err());
}

#[test]
fn test_rejects_too_few_cells() {
    assert!(DuctConfig::new(nozzle(), inlet(), 1.4, 70_000.0, 7, stop()).is_err());
}

#[test]
fn test_rejects_bad_stop_condition() {
    let zero_budget = DuctStop {
        max_steps: 0,
        residual_tol: 1e-8,
    };
    assert!(DuctConfig::new(nozzle(), inlet(), 1.4, 70_000.0, 64, zero_budget).is_err());
    let bad_tol = DuctStop {
        max_steps: 100,
        residual_tol: f64::NAN,
    };
    assert!(DuctConfig::new(nozzle(), inlet(), 1.4, 70_000.0, 64, bad_tol).is_err());
    let zero_tol = DuctStop {
        max_steps: 100,
        residual_tol: 0.0,
    };
    assert!(DuctConfig::new(nozzle(), inlet(), 1.4, 70_000.0, 64, zero_tol).is_err());
}
