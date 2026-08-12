/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Smoke tests for the `CfdConfigBuilder` entry points — the single public door to every owned
//! configuration family: the solver config (`dec_ns`), the four marching-case containers (`march`,
//! `qtt_march`, `compressible_march`, `duct`), and the MMS-verification config (`verify`). Each
//! case-family entry takes the case name, and the built config carries it.

use deep_causality_cfd::{
    AtmosphereRow, CfdConfigBuilder, DescentSchedule, DuctAreaProfile, MarchStop, Mesh, TaylorGreen,
};
use deep_causality_tensor::Truncation;

#[test]
fn dec_ns_entry_builds_a_solver_config() {
    let config = CfdConfigBuilder::dec_ns()
        .viscosity(0.1_f64)
        .time_step(0.01)
        .build()
        .expect("valid solver config");
    assert_eq!(config.nu(), 0.1);
    assert_eq!(config.dt(), 0.01);
}

#[test]
fn march_entry_builds_a_marching_case() {
    let config = CfdConfigBuilder::march::<2, f64>("case")
        .mesh(Mesh::box_domain([4, 4]))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(0.1)
                .time_step(0.01)
                .build()
                .unwrap(),
        )
        .build();
    assert!(config.is_ok());
}

#[test]
fn verify_entry_builds_a_verification_config() {
    let config = CfdConfigBuilder::verify::<f64, _>("mms", TaylorGreen::new(0.1, 1.0))
        .sample_at([1.0, 0.5, 0.0], 0.0)
        .build();
    assert!(config.is_ok());
}

#[test]
fn qtt_march_entry_builds_a_named_case() {
    let trunc = Truncation::<f64>::by_tol(1e-9).unwrap();
    let config = CfdConfigBuilder::qtt_march::<f64>("qtt-case")
        .grid(2, 2, 0.1, 0.1)
        .solver(0.01, 0.05, trunc)
        .seed_fn(|_, _| (0.0, 0.0))
        .expect("the grid is set before the seed")
        .build()
        .expect("valid qtt config");
    assert_eq!(config.name(), "qtt-case");
    assert_eq!(config.modes(), (2, 2));
}

#[test]
fn qtt_march_entry_rejects_a_missing_section() {
    let trunc = Truncation::<f64>::by_tol(1e-9).unwrap();
    // Grid and solver set, no seed.
    let r = CfdConfigBuilder::qtt_march::<f64>("qtt-incomplete")
        .grid(2, 2, 0.1, 0.1)
        .solver(0.01, 0.05, trunc)
        .build();
    assert!(r.is_err(), "a missing seed must be rejected");
}

#[test]
fn compressible_march_entry_builds_a_named_case() {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let rows = vec![
        AtmosphereRow {
            altitude_m: 0.0,
            n_tot: 2.5e25,
            temperature: 288.0,
            sound_speed: 340.0,
        },
        AtmosphereRow {
            altitude_m: 60_000.0,
            n_tot: 8.0e21,
            temperature: 250.0,
            sound_speed: 317.0,
        },
    ];
    let config = CfdConfigBuilder::compressible_march::<f64>("descent-case")
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, 6.0, 1.2, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 1.0, 0.0, 1.0))
        .expect("the grid and solver are set before the seed")
        .stop(MarchStop::Fixed(1))
        .schedule(DescentSchedule::new(rows, 1.2).expect("valid schedule"))
        .reference(6000.0, 1.0e22, 6000.0)
        .build()
        .expect("valid compressible config");
    assert_eq!(config.name(), "descent-case");
    assert_eq!(config.modes(), (3, 3));
}

#[test]
fn compressible_march_entry_rejects_a_missing_section() {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    // Grid and solver set, no flight step, seed, stop, or reference.
    let r = CfdConfigBuilder::compressible_march::<f64>("compressible-incomplete")
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, 6.0, 1.2, trunc)
        .build();
    assert!(r.is_err(), "a missing flight step must be rejected");
}

#[test]
fn duct_entry_builds_a_named_case() {
    let config = CfdConfigBuilder::duct::<f64>("duct-case")
        .profile(DuctAreaProfile::ConvergingDiverging {
            inlet_area: 3.0,
            throat_area: 1.0,
            exit_area: 2.0,
            length: 1.0,
        })
        .inlet(100_000.0, 300.0)
        .gamma(1.4)
        .back_pressure(70_000.0)
        .cells(64)
        .stop(1_000, 1e-8)
        .build()
        .expect("valid duct config");
    assert_eq!(config.name(), "duct-case");
    assert_eq!(config.cells(), 64);
}

#[test]
fn duct_entry_rejects_a_missing_section() {
    // Everything but the stop condition.
    let r = CfdConfigBuilder::duct::<f64>("duct-incomplete")
        .profile(DuctAreaProfile::ConvergingDiverging {
            inlet_area: 3.0,
            throat_area: 1.0,
            exit_area: 2.0,
            length: 1.0,
        })
        .inlet(100_000.0, 300.0)
        .gamma(1.4)
        .back_pressure(70_000.0)
        .cells(64)
        .build();
    assert!(r.is_err(), "a missing stop condition must be rejected");
}
