/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The plume re-imprint channel (M3): the carrier reads the geometry `PlumeObstruction` publishes
//! and refreshes its forcing region, so a stage never touches the marched layer directly.

use super::{GAMMA_EFF, REFERENCE, imprint_field};
use deep_causality_cfd::{
    BlackoutTrigger, CfdConfigBuilder, CfdFlow, CompressibleMarchConfig, MarchStop, QttObserve,
};
use deep_causality_tensor::Truncation;

// ── Plume re-imprint: the carrier's field-reading reconfiguration channel (M3) ──

/// A nozzle inside the Cordell validity envelope, matching the retropulsion-stage tests.
fn imprint_nozzle() -> deep_causality_cfd::PlumeNozzle<f64> {
    deep_causality_cfd::PlumeNozzle::new(
        2.0e6,
        1_500.0,
        300.0,
        1.3,
        3.0,
        15.0 * std::f64::consts::PI / 180.0,
        0.03,
        0.03407,
        0.0712,
        1.4,
    )
    .expect("a nozzle inside the Cordell envelope")
}

fn imprint_spec(tolerance: f64, max_refreshes: usize) -> deep_causality_cfd::PlumeImprint<f64> {
    deep_causality_cfd::PlumeImprint::new(
        tolerance,
        max_refreshes,
        0.72,
        0.5,
        1.0,
        4.0,
        [1.0, -0.5, 0.0, 2.0],
        0.002,
    )
    .expect("a valid imprint spec")
}

/// A world that publishes a throttle and opts into plume re-imprint.
fn imprint_world(
    name: &str,
    steps: usize,
    throttle: f64,
    spec: Option<deep_causality_cfd::PlumeImprint<f64>>,
) -> CompressibleMarchConfig<f64> {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let mut builder = CfdConfigBuilder::compressible_march::<f64>(name)
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, 3.0, GAMMA_EFF, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 1.0, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(steps))
        .observe(QttObserve::default())
        .reference(REFERENCE.0, REFERENCE.1, REFERENCE.2)
        .publish_constant("commanded_throttle", throttle)
        // The plume stage senses its freestream each step rather than carrying a constant. This
        // world composes no `FlightSensors`, so it publishes the sensed values directly.
        .publish_constant("q_inf", 2_800.0)
        .publish_constant("p_inf", 1_000.0)
        // No descent schedule here, so the carrier publishes no flight Mach of its own.
        .publish_constant("flight_mach", 2.0);
    if let Some(s) = spec {
        builder = builder.plume_imprint(s);
    }
    builder.build().unwrap()
}

#[test]
fn the_plume_imprint_follows_the_throttle_through_the_carrier() {
    // End-to-end: PlumeObstruction publishes the geometry into the coupled field; the carrier's
    // pre_step reads it and refreshes the forcing region — the same channel that already carries
    // "truth_state" into the inflow strip. A PhysicsStage never touches the marched layer.
    let cfg = imprint_world("imprint_on", 4, 0.5, Some(imprint_spec(0.01, 8)));
    let stage = deep_causality_cfd::PlumeObstruction::new(2_000.0, 0.785)
        .with_plume_geometry(imprint_nozzle());
    let report = CfdFlow::march(&cfg)
        .run_coupled(stage, imprint_field(), BlackoutTrigger::new(1.0e9), 0.0)
        .unwrap();
    let logged = report
        .effect_log()
        .expect("provenance log")
        .messages()
        .any(|m| m.contains("plume re-imprint"));
    assert!(
        logged,
        "the carrier refreshed the forcing region from the published geometry"
    );
}

#[test]
fn without_the_opt_in_the_carrier_never_re_imprints() {
    // No plume_imprint spec: the forcing region stays exactly as configured at world build, so the
    // march path is untouched and no re-imprint provenance appears.
    let cfg = imprint_world("imprint_off", 4, 0.5, None);
    let stage = deep_causality_cfd::PlumeObstruction::new(2_000.0, 0.785)
        .with_plume_geometry(imprint_nozzle());
    let report = CfdFlow::march(&cfg)
        .run_coupled(stage, imprint_field(), BlackoutTrigger::new(1.0e9), 0.0)
        .unwrap();
    assert!(
        report
            .effect_log()
            .is_none_or(|l| !l.messages().any(|m| m.contains("plume re-imprint"))),
        "no opt-in ⇒ no re-imprint"
    );
}

#[test]
fn a_steady_throttle_re_imprints_once_not_every_step() {
    // The solver-rebuild discipline: with a constant throttle the drift gate fires once and then
    // stays quiet, so a mask rebuild does not happen every step.
    let cfg = imprint_world("imprint_steady", 6, 0.5, Some(imprint_spec(0.01, 8)));
    let stage = deep_causality_cfd::PlumeObstruction::new(2_000.0, 0.785)
        .with_plume_geometry(imprint_nozzle());
    let report = CfdFlow::march(&cfg)
        .run_coupled(stage, imprint_field(), BlackoutTrigger::new(1.0e9), 0.0)
        .unwrap();
    let count = report
        .effect_log()
        .expect("provenance log")
        .messages()
        .filter(|m| m.contains("plume re-imprint"))
        .count();
    assert_eq!(count, 1, "a steady throttle re-imprints exactly once");
}

#[test]
fn the_refresh_cap_bounds_re_imprints() {
    // max_refreshes = 0 forbids any refresh, even with a live throttle and published geometry.
    let cfg = imprint_world("imprint_capped", 4, 0.5, Some(imprint_spec(0.01, 0)));
    let stage = deep_causality_cfd::PlumeObstruction::new(2_000.0, 0.785)
        .with_plume_geometry(imprint_nozzle());
    let report = CfdFlow::march(&cfg)
        .run_coupled(stage, imprint_field(), BlackoutTrigger::new(1.0e9), 0.0)
        .unwrap();
    assert!(
        report
            .effect_log()
            .is_none_or(|l| !l.messages().any(|m| m.contains("plume re-imprint"))),
        "the cap bounds refreshes"
    );
}
