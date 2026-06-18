/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `UncertainMarchConfigBuilder` → `UncertainMarchConfig` configuration layer (the
//! sensor-fed uncertain-inflow march). Exercises the happy path (all required fields + a non-default
//! seed) and every validation failure: a missing `solver`, `inflow_zone`, `sensor_stream`, or
//! `march_for`, and a sensor stream shorter than the horizon.

use deep_causality_cfd::{
    CfdConfigBuilder, DecNsConfig, Seed, UncertainInflowZone, UncertainMarchConfig,
};
use deep_causality_physics::PhysicsErrorEnum;
use deep_causality_uncertain::MaybeUncertain;

const NU: f64 = 0.1;
const DT: f64 = 0.02;
const U_IN: f64 = 0.2;
const STEPS: usize = 4;

fn solver_config() -> DecNsConfig<f64> {
    CfdConfigBuilder::dec_ns()
        .viscosity(NU)
        .time_step(DT)
        .build()
        .unwrap()
}

fn zone() -> UncertainInflowZone<f64> {
    UncertainInflowZone::new(1, true, 0, U_IN)
        .with_presence_gate(0.5, 0.9, 0.1, 64)
        .with_collapse_samples(8)
}

fn stream(len: usize) -> Vec<MaybeUncertain<f64>> {
    vec![MaybeUncertain::<f64>::from_value(U_IN); len]
}

/// Assert a builder `Result` is the dimension-mismatch validation failure. `UncertainMarchConfig`
/// is not `Debug`, so the `Ok` arm cannot go through `unwrap_err`.
fn assert_dim_mismatch(
    result: Result<UncertainMarchConfig<f64>, deep_causality_cfd::PhysicsError>,
) {
    match result {
        Ok(_) => panic!("expected a DimensionMismatch validation failure, got Ok"),
        Err(e) => assert!(matches!(e.0, PhysicsErrorEnum::DimensionMismatch(_))),
    }
}

#[test]
fn builds_with_all_required_fields_and_a_non_default_seed() {
    let config = CfdConfigBuilder::uncertain_march::<f64>("sensor-march")
        .solver(solver_config())
        .inflow_zone(zone())
        .sensor_stream(stream(STEPS))
        .seed(Seed::Rest)
        .march_for(STEPS)
        .build();
    assert!(config.is_ok(), "all required fields present → Ok");
}

#[test]
fn build_rejects_a_missing_solver() {
    assert_dim_mismatch(
        CfdConfigBuilder::uncertain_march::<f64>("no-solver")
            .inflow_zone(zone())
            .sensor_stream(stream(STEPS))
            .march_for(STEPS)
            .build(),
    );
}

#[test]
fn build_rejects_a_missing_inflow_zone() {
    assert_dim_mismatch(
        CfdConfigBuilder::uncertain_march::<f64>("no-zone")
            .solver(solver_config())
            .sensor_stream(stream(STEPS))
            .march_for(STEPS)
            .build(),
    );
}

#[test]
fn build_rejects_a_missing_sensor_stream() {
    assert_dim_mismatch(
        CfdConfigBuilder::uncertain_march::<f64>("no-stream")
            .solver(solver_config())
            .inflow_zone(zone())
            .march_for(STEPS)
            .build(),
    );
}

#[test]
fn build_rejects_a_missing_horizon() {
    assert_dim_mismatch(
        CfdConfigBuilder::uncertain_march::<f64>("no-steps")
            .solver(solver_config())
            .inflow_zone(zone())
            .sensor_stream(stream(STEPS))
            .build(),
    );
}

#[test]
fn build_rejects_a_stream_shorter_than_the_horizon() {
    assert_dim_mismatch(
        CfdConfigBuilder::uncertain_march::<f64>("short-stream")
            .solver(solver_config())
            .inflow_zone(zone())
            .sensor_stream(stream(STEPS - 1))
            .march_for(STEPS)
            .build(),
    );
}
