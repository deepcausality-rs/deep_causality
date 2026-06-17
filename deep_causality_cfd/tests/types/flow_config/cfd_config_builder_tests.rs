/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Smoke tests for the `CfdConfigBuilder` entry points: each starts a dedicated, validated config
//! builder for one solver kind (`dec_ns`), the marching-case container (`march`), or the
//! MMS-verification config (`verify`).

use deep_causality_cfd::{CfdConfigBuilder, Mesh, TaylorGreen};

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
