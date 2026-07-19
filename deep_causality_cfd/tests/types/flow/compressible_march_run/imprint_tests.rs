/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The plume re-imprint channel (M3): the carrier reads the geometry `PlumeObstruction` publishes
//! and refreshes its forcing region, so a stage never touches the marched layer directly.

use super::{GAMMA_EFF, imprint_field, reference};
use deep_causality_cfd::{
    BlackoutTrigger, CfdFlow, CompressibleMarchConfig, CompressibleMarchConfigBuilder, MarchStop,
    QttObserve,
};
use deep_causality_tensor::Truncation;

// ── Plume re-imprint: the carrier's field-reading reconfiguration channel (M3) ──

/// A nozzle inside the Cordell validity envelope, matching the retropulsion-stage tests.
fn imprint_nozzle() -> deep_causality_cfd::PlumeNozzle<f64> {
    deep_causality_cfd::PlumeNozzle {
        chamber_pressure_max: 2.0e6,
        chamber_temperature: 1_500.0,
        r_specific: 300.0,
        gamma_jet: 1.3,
        exit_mach: 3.0,
        nozzle_half_angle_rad: 15.0 * std::f64::consts::PI / 180.0,
        throat_diameter: 0.03,
        exit_radius: 0.03407,
        cone_length: 0.0712,
        p_inf: 1_000.0,
        mach_inf: 2.0,
        gamma_inf: 1.4,
    }
}

fn imprint_spec(tolerance: f64, max_refreshes: usize) -> deep_causality_cfd::PlumeImprint<f64> {
    deep_causality_cfd::PlumeImprint {
        throttle_tolerance: tolerance,
        max_refreshes,
        face_x: 0.72,
        axis_y: 0.5,
        smoothing_cells: 1.0,
        domain_m: 4.0,
        target: [1.0, -0.5, 0.0, 2.0],
        eta: 0.002,
    }
}

/// A world that publishes a throttle and opts into plume re-imprint.
fn imprint_world(
    name: &str,
    steps: usize,
    throttle: f64,
    spec: Option<deep_causality_cfd::PlumeImprint<f64>>,
) -> CompressibleMarchConfig<f64> {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let mut builder = CompressibleMarchConfigBuilder::<f64>::new()
        .name(name)
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, 3.0, GAMMA_EFF, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 1.0, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(steps))
        .observe(QttObserve::default())
        .reference(reference())
        .publish_constant("commanded_throttle", throttle);
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
    let stage = deep_causality_cfd::PlumeObstruction::new(2_000.0, 2_800.0, 0.785)
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
    let stage = deep_causality_cfd::PlumeObstruction::new(2_000.0, 2_800.0, 0.785)
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
    let stage = deep_causality_cfd::PlumeObstruction::new(2_000.0, 2_800.0, 0.785)
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
    let stage = deep_causality_cfd::PlumeObstruction::new(2_000.0, 2_800.0, 0.785)
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
