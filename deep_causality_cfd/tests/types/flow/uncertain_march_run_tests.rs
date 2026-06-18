/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `CfdFlow::uncertain_march` pipeline (`UncertainMarchPipeline` → `UncertainMarchRun`
//! → `Report`) and the per-step `UncertainStepView` hook.
//!
//! Fast, no-validation rungs mirroring the deterministic split: a small all-walls lattice driven by
//! a short all-present sensor stream. Covers:
//!
//! - `.on(&manifold).run()` → an owned `Report` carrying the final field and the `EffectLog` count;
//! - `.run_with(hook)` → every `UncertainStepView` accessor (`step`, `one_form`, `in_dropout`,
//!   `manifold`, `max_speed`, `divergence`, `kinetic_energy`);
//! - a dropout stream surfacing `in_dropout` and a non-zero log-entry count in the report;
//! - the `flow_axis >= D` guard inside `execute` returning a `DimensionMismatch`.
//!
//! Not covered (and not reachable through this public DSL): the per-step bind-failure arm in
//! `execute` (wrapping a monad error as `PhysicalInvariantBroken`). Every cause of a per-step
//! failure — a missing context, an exhausted stream, an invalid wall reconfiguration — is
//! prevalidated away before the bind loop runs: `build()` enforces `stream.len() >= steps`,
//! `execute` always supplies a context, and the initial `with_moving_wall` install validates the
//! prescribed-wall axes. The only residual triggers are a runtime sampling/solver failure that a
//! valid, finite stream cannot produce, so the arm is a defensive guard with no constructible input.

use deep_causality_cfd::{
    CfdConfigBuilder, CfdFlow, DecNsConfig, Seed, UncertainInflowZone, UncertainMarchConfig,
    UncertainStepView,
};
use deep_causality_physics::PhysicsErrorEnum;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};
use deep_causality_uncertain::MaybeUncertain;

const N: usize = 5;
const NU: f64 = 0.1;
const DT: f64 = 0.02;
const U_IN: f64 = 0.2;
const STEPS: usize = 4;

/// An all-walls `N×N` unit-square lattice (so the moving-wall boundary is valid).
fn wall_manifold() -> Manifold<LatticeComplex<2, f64>, f64> {
    let h = 1.0 / (N - 1) as f64;
    let lattice = LatticeComplex::<2, f64>::new([N, N], [false, false]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(h);
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn solver_config() -> DecNsConfig<f64> {
    CfdConfigBuilder::dec_ns()
        .viscosity(NU)
        .time_step(DT)
        .build()
        .unwrap()
}

fn zone(flow_axis: usize) -> UncertainInflowZone<f64> {
    UncertainInflowZone::new(1, true, flow_axis, U_IN)
        .with_presence_gate(0.5, 0.9, 0.1, 64)
        .with_collapse_samples(8)
}

/// A buildable config over an all-present `stream`, marching `STEPS` steps with `flow_axis = 0`.
fn present_config(name: &str) -> UncertainMarchConfig<f64> {
    CfdConfigBuilder::uncertain_march::<f64>(name)
        .solver(solver_config())
        .inflow_zone(zone(0))
        .sensor_stream(vec![MaybeUncertain::<f64>::from_value(U_IN); STEPS])
        .seed(Seed::Rest)
        .march_for(STEPS)
        .build()
        .unwrap()
}

#[test]
fn run_returns_a_report_with_a_final_field_and_a_log_count() {
    let manifold = wall_manifold();
    let config = present_config("present-run");
    let report = CfdFlow::uncertain_march(&config)
        .on(&manifold)
        .run()
        .unwrap();

    assert_eq!(report.name(), "present-run");
    let field = report
        .final_field()
        .expect("an uncertain march records its final edge cochain");
    assert!(!field.is_empty());
    assert!(field.iter().all(|v| v.is_finite()));

    // An all-present stream records no dropouts, but the run still carries an (empty) effect log.
    assert_eq!(
        report.log_entries(),
        Some(0),
        "a present stream logs nothing, but the count is reported"
    );
}

#[test]
fn run_with_hook_observes_every_step_view_accessor() {
    let manifold = wall_manifold();
    let config = present_config("hook-run");

    let mut steps_seen = Vec::new();
    let hook = |view: &UncertainStepView<'_, 2, f64>| {
        steps_seen.push(view.step());
        // The raw edge cochain is the plain velocity 1-form.
        assert!(!view.one_form().as_slice().is_empty());
        // An all-present stream never drops out.
        assert!(!view.in_dropout());
        // The lent manifold is exposed verbatim.
        assert_eq!(view.manifold().complex().num_cells(0), N * N);
        // The convenience diagnostics resolve through the DEC operators.
        assert!(view.max_speed().unwrap().is_finite());
        assert!(view.divergence().unwrap().is_finite());
        assert!(view.kinetic_energy().unwrap() >= 0.0);
    };

    let report = CfdFlow::uncertain_march(&config)
        .on(&manifold)
        .run_with(hook)
        .unwrap();

    // The hook fires once per step, 1-based and in order.
    assert_eq!(steps_seen, (1..=STEPS).collect::<Vec<_>>());
    // The hooked run produces the same report shape as `run`.
    assert!(report.final_field().is_some());
    assert_eq!(report.log_entries(), Some(0));
}

#[test]
fn run_with_a_dropout_stream_flags_dropout_and_logs_entries() {
    let manifold = wall_manifold();
    // present, absent, present, absent — two dropouts.
    let stream = vec![
        MaybeUncertain::<f64>::from_value(U_IN),
        MaybeUncertain::<f64>::always_none(),
        MaybeUncertain::<f64>::from_value(U_IN),
        MaybeUncertain::<f64>::always_none(),
    ];
    let config = CfdConfigBuilder::uncertain_march::<f64>("dropout-run")
        .solver(solver_config())
        .inflow_zone(zone(0))
        .sensor_stream(stream)
        .march_for(STEPS)
        .build()
        .unwrap();

    let mut dropout_steps = Vec::new();
    let report = CfdFlow::uncertain_march(&config)
        .on(&manifold)
        .run_with(|view: &UncertainStepView<'_, 2, f64>| {
            if view.in_dropout() {
                dropout_steps.push(view.step());
            }
        })
        .unwrap();

    // The final step's sample is absent, so its view is flagged in dropout.
    assert!(
        dropout_steps.contains(&STEPS),
        "the final (absent) step is flagged in dropout: {dropout_steps:?}"
    );
    // The dropouts are recorded in the report's effect-log count.
    assert!(
        report.log_entries().unwrap() > 0,
        "the dropouts record EffectLog entries"
    );
}

#[test]
fn run_rejects_a_flow_axis_out_of_range_for_the_geometry() {
    let manifold = wall_manifold();
    // flow_axis 5 ≥ D = 2: the materialized run rejects it before marching.
    let config = CfdConfigBuilder::uncertain_march::<f64>("bad-axis")
        .solver(solver_config())
        .inflow_zone(zone(5))
        .sensor_stream(vec![MaybeUncertain::<f64>::from_value(U_IN); STEPS])
        .march_for(STEPS)
        .build()
        .unwrap();

    let err = CfdFlow::uncertain_march(&config)
        .on(&manifold)
        .run()
        .unwrap_err();
    assert!(matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)));
}
