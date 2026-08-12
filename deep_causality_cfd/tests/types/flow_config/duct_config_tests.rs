/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the owned `DuctConfig` — its accessors and the name it carries into the run's report.
//! The rejection paths live in `duct_builder_tests`, alongside the validation that produces them.

use deep_causality_cfd::{CfdConfigBuilder, DuctAreaProfile};

fn nozzle() -> DuctAreaProfile<f64> {
    DuctAreaProfile::ConvergingDiverging {
        inlet_area: 3.0,
        throat_area: 1.0,
        exit_area: 2.0,
        length: 1.0,
    }
}

#[test]
fn test_valid_analytic_config() {
    let cfg = CfdConfigBuilder::duct::<f64>("analytic-nozzle")
        .profile(nozzle())
        .inlet(100_000.0, 300.0)
        .gamma(1.4)
        .back_pressure(70_000.0)
        .cells(128)
        .stop(10_000, 1e-8)
        .build()
        .unwrap();
    assert_eq!(cfg.name(), "analytic-nozzle");
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
    let cfg = CfdConfigBuilder::duct::<f64>("table-nozzle")
        .profile(table)
        .inlet(100_000.0, 300.0)
        .gamma(1.4)
        .back_pressure(70_000.0)
        .cells(64)
        .stop(10_000, 1e-8)
        .build();
    assert!(cfg.is_ok());
}

#[test]
fn test_each_case_carries_its_own_name() {
    // The name is taken at the entry and is required, so a swept duct study distinguishes its
    // cases rather than sharing one fixed report label.
    let build = |name: &str| {
        CfdConfigBuilder::duct::<f64>(name)
            .profile(nozzle())
            .inlet(100_000.0, 300.0)
            .gamma(1.4)
            .back_pressure(70_000.0)
            .cells(64)
            .stop(10_000, 1e-8)
            .build()
            .unwrap()
    };
    assert_eq!(build("pr-0.7").name(), "pr-0.7");
    assert_eq!(build("pr-0.9").name(), "pr-0.9");
}
