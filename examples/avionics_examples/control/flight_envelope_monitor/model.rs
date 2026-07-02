/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Flight Envelope Monitor — Pipeline Stages
//!
//! Closures, builders, and stage primitives. Domain types live in
//! [`super::model_types`].
//!
//! ## Daisy-chain composition
//!
//! Every public stage primitive in this module has the bind-callback
//! signature
//! `fn(EffectValue<I>, FlightState, Option<AircraftConfig>) -> FlightProcess<O>`
//! so that the whole pipeline can be expressed in `main.rs` as one
//! `PropagatingProcess::pure(...).bind(stage1).bind(stage2_1).bind(...)`
//! chain.

use crate::model_types::{
    AircraftConfig, FlightProcess, FlightState, FlightStateEstimate, NOMINAL_BANDS, SensorReading,
};
use deep_causality::*;
use deep_causality_core::CausalityErrorEnum;

// ---------------------------------------------------------------------------
// Tuning constants — tightly coupled to the closures below.
// ---------------------------------------------------------------------------

/// Per-iteration scalar Kalman measurement noise.
const KALMAN_MEAS_NOISE: f64 = 1.0;
/// Weight applied when folding `(1 - joint_health)` into `state.risk`.
const RISK_HEALTH_WEIGHT: f64 = 1.0;

const STALL_RISK_WEIGHT: f64 = 0.20;
const OVERSPEED_RISK_WEIGHT: f64 = 0.20;
const TERRAIN_RISK_WEIGHT: f64 = 0.15;
const TRAFFIC_RISK_WEIGHT: f64 = 0.15;
const ICING_RISK_WEIGHT: f64 = 0.10;
const CG_RISK_WEIGHT: f64 = 0.10;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a measurement and a healthy band into a per-sensor health probability
/// in `[0.0, 1.0]`. Inside the band → `1.0`. Linearly drops to `0.0` when
/// outside by `tolerance` units.
fn health_probability(value: f64, band: (f64, f64), tolerance: f64) -> f64 {
    let (lo, hi) = band;
    let deviation = if value < lo {
        lo - value
    } else if value > hi {
        value - hi
    } else {
        0.0
    };
    (1.0 - (deviation / tolerance).clamp(0.0, 1.0)).clamp(0.0, 1.0)
}

/// Build an `AircraftConfig` to fall back on when a node receives `None`.
/// Defensive shim only; the example always supplies a real config in main.rs.
fn fallback_config() -> AircraftConfig {
    AircraftConfig {
        mass_kg: 0.0,
        mtow_kg: 1.0,
        stall_margin: 1.0,
        service_ceiling_m: 1.0,
    }
}

// ---------------------------------------------------------------------------
// Stage 1 — sensor causaloids (per-sensor f64 health probability)
// ---------------------------------------------------------------------------
//
// Per-sensor closures use the stateless `Causaloid::new` form because health
// depends only on the per-sensor reading. State and Context are introduced at
// the collection level via `from_causal_collection_with_context`.

fn airspeed_health(reading: SensorReading) -> PropagatingEffect<f64> {
    let h = health_probability(reading.airspeed_kn, NOMINAL_BANDS.airspeed_kn, 80.0);
    let mut eff = PropagatingEffect::from_value(h);
    eff.logs
        .add_entry(&format!("sensor.airspeed: health={:.3}", h));
    eff
}

fn altitude_health(reading: SensorReading) -> PropagatingEffect<f64> {
    let h = health_probability(reading.altitude_ft, NOMINAL_BANDS.altitude_ft, 10_000.0);
    let mut eff = PropagatingEffect::from_value(h);
    eff.logs
        .add_entry(&format!("sensor.altitude: health={:.3}", h));
    eff
}

fn attitude_health(reading: SensorReading) -> PropagatingEffect<f64> {
    let h = health_probability(reading.attitude_deg, NOMINAL_BANDS.attitude_deg, 20.0);
    let mut eff = PropagatingEffect::from_value(h);
    eff.logs
        .add_entry(&format!("sensor.attitude: health={:.3}", h));
    eff
}

fn vertical_speed_health(reading: SensorReading) -> PropagatingEffect<f64> {
    let h = health_probability(
        reading.vertical_speed_fpm,
        NOMINAL_BANDS.vertical_speed_fpm,
        2_000.0,
    );
    let mut eff = PropagatingEffect::from_value(h);
    eff.logs.add_entry(&format!("sensor.vsi: health={:.3}", h));
    eff
}

fn fuel_flow_health(reading: SensorReading) -> PropagatingEffect<f64> {
    let h = health_probability(reading.fuel_flow_pph, NOMINAL_BANDS.fuel_flow_pph, 1_500.0);
    let mut eff = PropagatingEffect::from_value(h);
    eff.logs
        .add_entry(&format!("sensor.fuel_flow: health={:.3}", h));
    eff
}

/// Failing-airspeed closure used by the failing-sensor scenario.
fn airspeed_failing(_reading: SensorReading) -> PropagatingEffect<f64> {
    PropagatingEffect::from_error(CausalityError::new(CausalityErrorEnum::Custom(
        "sensor.airspeed: hardware fault — sensor lost".into(),
    )))
}

/// Build the five per-sensor singleton causaloids.
fn build_sensor_causaloids(
    failing_airspeed: bool,
) -> Vec<Causaloid<SensorReading, f64, FlightState, AircraftConfig>> {
    let airspeed = if failing_airspeed {
        Causaloid::new(0, airspeed_failing, "airspeed sensor (FAILING)")
    } else {
        Causaloid::new(0, airspeed_health, "airspeed sensor")
    };

    vec![
        airspeed,
        Causaloid::new(1, altitude_health, "altitude sensor"),
        Causaloid::new(2, attitude_health, "attitude sensor"),
        Causaloid::new(3, vertical_speed_health, "vertical-speed sensor"),
        Causaloid::new(4, fuel_flow_health, "fuel-flow sensor"),
    ]
}

/// **Stage 1** — sensor collection evaluation.
///
/// Bind-callback shape: takes `(EffectValue<SensorReading>, FlightState,
/// Option<AircraftConfig>)`, reconstructs the incoming process, and evaluates
/// the per-sensor collection via
/// `StatefulMonadicCausableCollection::evaluate_collection_stateful` with
/// `AggregateLogic::All` (joint health probability via `∏ p_i`). Returns
/// `FlightProcess<f64>` whose value channel carries the joint health
/// probability.
pub fn run_sensor_collection(
    value: EffectValue<SensorReading>,
    state: FlightState,
    ctx: Option<AircraftConfig>,
    failing_airspeed: bool,
) -> FlightProcess<f64> {
    let incoming: FlightProcess<SensorReading> = PropagatingProcess {
        value,
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    let sensors = build_sensor_causaloids(failing_airspeed);
    sensors
        .as_slice()
        .evaluate_collection_stateful(&incoming, &AggregateLogic::All, Some(0.0))
}

// ---------------------------------------------------------------------------
// Stage 2 — CausalMonad bind chain (three steps)
// ---------------------------------------------------------------------------

/// **Stage 2.1** — fold joint health into `state.risk` and project the value
/// channel from `f64` to `FlightStateEstimate` populated from the captured
/// `seed_estimate` (because the original `SensorReading` is no longer on the
/// value channel — Stage 1 reduced it to a scalar health probability).
pub fn health_fold(
    value: EffectValue<f64>,
    mut state: FlightState,
    ctx: Option<AircraftConfig>,
    seed_estimate: FlightStateEstimate,
) -> FlightProcess<FlightStateEstimate> {
    let health = match value.into_value() {
        Some(h) => h,
        None => {
            return PropagatingProcess {
                value: EffectValue::None,
                state,
                context: ctx,
                error: Some(CausalityError::new(CausalityErrorEnum::Custom(
                    "stage2.health_fold: value was None".into(),
                ))),
                logs: EffectLog::new(),
            };
        }
    };

    state.risk += (1.0 - health) * RISK_HEALTH_WEIGHT;

    let mut process: FlightProcess<FlightStateEstimate> = PropagatingProcess {
        value: EffectValue::Value(seed_estimate),
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    process.logs.add_entry(&format!(
        "stage2.health_fold: risk += {:.3} (health={:.3})",
        (1.0 - health) * RISK_HEALTH_WEIGHT,
        health
    ));
    process
}

/// **Stage 2.2** — one-iteration scalar Kalman update on each diagonal element
/// of `state.covariance`. Illustrative only.
pub fn kalman_step(
    value: EffectValue<FlightStateEstimate>,
    mut state: FlightState,
    ctx: Option<AircraftConfig>,
) -> FlightProcess<FlightStateEstimate> {
    let estimate = match value.into_value() {
        Some(v) => v,
        None => {
            return PropagatingProcess {
                value: EffectValue::None,
                state,
                context: ctx,
                error: Some(CausalityError::new(CausalityErrorEnum::Custom(
                    "stage2.kalman: value was None".into(),
                ))),
                logs: EffectLog::new(),
            };
        }
    };

    if state.covariance == [0.0; 4] {
        state.covariance = [4.0, 4.0, 4.0, 4.0];
    }

    for cov_i in state.covariance.iter_mut() {
        let k_i = *cov_i / (*cov_i + KALMAN_MEAS_NOISE);
        *cov_i *= 1.0 - k_i;
    }

    let mut process: FlightProcess<FlightStateEstimate> = PropagatingProcess {
        value: EffectValue::Value(estimate),
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    process
        .logs
        .add_entry("stage2.kalman: covariance updated (one-iteration scalar)");
    process
}

/// **Stage 2.3** — write the four `FlightStateEstimate` fields into
/// `state.estimate`.
pub fn estimate_step(
    value: EffectValue<FlightStateEstimate>,
    mut state: FlightState,
    ctx: Option<AircraftConfig>,
) -> FlightProcess<FlightStateEstimate> {
    let estimate = match value.clone().into_value() {
        Some(v) => v,
        None => {
            return PropagatingProcess {
                value: EffectValue::None,
                state,
                context: ctx,
                error: Some(CausalityError::new(CausalityErrorEnum::Custom(
                    "stage2.estimate: value was None".into(),
                ))),
                logs: EffectLog::new(),
            };
        }
    };

    state.estimate = [
        estimate.airspeed_kn,
        estimate.altitude_ft,
        estimate.attitude_deg,
        estimate.vertical_speed_fpm,
    ];

    let mut process: FlightProcess<FlightStateEstimate> = PropagatingProcess {
        value,
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    process
        .logs
        .add_entry("stage2.estimate: state.estimate written");
    process
}

// ---------------------------------------------------------------------------
// Stage 3 — envelope causaloid hypergraph
// ---------------------------------------------------------------------------

fn stall_risk(
    value: EffectValue<FlightStateEstimate>,
    mut state: FlightState,
    ctx: Option<AircraftConfig>,
) -> FlightProcess<FlightStateEstimate> {
    let est = value.into_value().unwrap_or_default();
    let cfg = ctx.clone().unwrap_or_else(fallback_config);
    let lo = NOMINAL_BANDS.airspeed_kn.0 * cfg.stall_margin / 1.3;
    let pressure = ((lo - est.airspeed_kn).max(0.0) / lo).clamp(0.0, 1.0);
    let increment = pressure * STALL_RISK_WEIGHT;
    state.risk += increment;
    let mut p = PropagatingProcess {
        value: EffectValue::Value(est),
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    p.logs
        .add_entry(&format!("envelope.stall: risk += {:.3}", increment));
    p
}

fn overspeed_risk(
    value: EffectValue<FlightStateEstimate>,
    mut state: FlightState,
    ctx: Option<AircraftConfig>,
) -> FlightProcess<FlightStateEstimate> {
    let est = value.into_value().unwrap_or_default();
    let hi = NOMINAL_BANDS.airspeed_kn.1;
    let pressure = ((est.airspeed_kn - hi).max(0.0) / hi).clamp(0.0, 1.0);
    let increment = pressure * OVERSPEED_RISK_WEIGHT;
    state.risk += increment;
    let mut p = PropagatingProcess {
        value: EffectValue::Value(est),
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    p.logs
        .add_entry(&format!("envelope.overspeed: risk += {:.3}", increment));
    p
}

fn terrain_proximity(
    value: EffectValue<FlightStateEstimate>,
    mut state: FlightState,
    ctx: Option<AircraftConfig>,
) -> FlightProcess<FlightStateEstimate> {
    let est = value.into_value().unwrap_or_default();
    let pressure =
        ((NOMINAL_BANDS.altitude_ft.0 - est.altitude_ft).max(0.0) / 5_000.0).clamp(0.0, 1.0);
    let increment = pressure * TERRAIN_RISK_WEIGHT;
    state.risk += increment;
    let mut p = PropagatingProcess {
        value: EffectValue::Value(est),
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    p.logs
        .add_entry(&format!("envelope.terrain: risk += {:.3}", increment));
    p
}

fn traffic_conflict(
    value: EffectValue<FlightStateEstimate>,
    mut state: FlightState,
    ctx: Option<AircraftConfig>,
) -> FlightProcess<FlightStateEstimate> {
    let est = value.into_value().unwrap_or_default();
    let cfg = ctx.clone().unwrap_or_else(fallback_config);
    // Traffic density drops sharply near the service ceiling.
    let altitude_m = est.altitude_ft * 0.3048;
    let near_ceiling = altitude_m / cfg.service_ceiling_m;
    let pressure = if near_ceiling > 0.8 {
        0.02
    } else if est.altitude_ft > 25_000.0 {
        0.05
    } else {
        0.20
    };
    let increment = pressure * TRAFFIC_RISK_WEIGHT;
    state.risk += increment;
    let mut p = PropagatingProcess {
        value: EffectValue::Value(est),
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    p.logs
        .add_entry(&format!("envelope.traffic: risk += {:.3}", increment));
    p
}

fn icing_risk(
    value: EffectValue<FlightStateEstimate>,
    mut state: FlightState,
    ctx: Option<AircraftConfig>,
) -> FlightProcess<FlightStateEstimate> {
    let est = value.into_value().unwrap_or_default();
    let pressure = if (10_000.0..20_000.0).contains(&est.altitude_ft) {
        0.30
    } else {
        0.05
    };
    let increment = pressure * ICING_RISK_WEIGHT;
    state.risk += increment;
    let mut p = PropagatingProcess {
        value: EffectValue::Value(est),
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    p.logs
        .add_entry(&format!("envelope.icing: risk += {:.3}", increment));
    p
}

fn cg_out_of_limits(
    value: EffectValue<FlightStateEstimate>,
    mut state: FlightState,
    ctx: Option<AircraftConfig>,
) -> FlightProcess<FlightStateEstimate> {
    let est = value.into_value().unwrap_or_default();
    let cfg = ctx.clone().unwrap_or_else(fallback_config);
    let pressure = (cfg.mass_kg / cfg.mtow_kg).clamp(0.0, 1.0);
    let increment = pressure * CG_RISK_WEIGHT;
    state.risk += increment;
    let mut p = PropagatingProcess {
        value: EffectValue::Value(est),
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    p.logs
        .add_entry(&format!("envelope.cg: risk += {:.3}", increment));
    p
}

/// Build the envelope hypergraph.
///
/// Topology (six nodes; edges marked with cause → effect):
/// ```text
///   stall(0)   ─► terrain(2)
///   icing(4)   ─► stall(0)
///   stall(0)   ─► overspeed(1)
///   traffic(3) ─► overspeed(1)
///   terrain(2) ─► traffic(3)
///   cg(5)      ─► stall(0)
/// ```
fn build_envelope_graph(
    config: AircraftConfig,
) -> CausaloidGraph<Causaloid<FlightStateEstimate, FlightStateEstimate, FlightState, AircraftConfig>>
{
    let mut g: CausaloidGraph<
        Causaloid<FlightStateEstimate, FlightStateEstimate, FlightState, AircraftConfig>,
    > = CausaloidGraph::new(0u64);

    let n_stall = Causaloid::new_with_context(100, stall_risk, config.clone(), "stall risk");
    let n_overspeed =
        Causaloid::new_with_context(101, overspeed_risk, config.clone(), "overspeed risk");
    let n_terrain =
        Causaloid::new_with_context(102, terrain_proximity, config.clone(), "terrain proximity");
    let n_traffic =
        Causaloid::new_with_context(103, traffic_conflict, config.clone(), "traffic conflict");
    let n_icing = Causaloid::new_with_context(104, icing_risk, config.clone(), "icing risk");
    let n_cg = Causaloid::new_with_context(105, cg_out_of_limits, config, "CG out of limits");

    let i0 = g.add_root_causaloid(n_stall).expect("root stall");
    let i1 = g.add_causaloid(n_overspeed).expect("overspeed");
    let i2 = g.add_causaloid(n_terrain).expect("terrain");
    let i3 = g.add_causaloid(n_traffic).expect("traffic");
    let i4 = g.add_causaloid(n_icing).expect("icing");
    let i5 = g.add_causaloid(n_cg).expect("cg");

    g.add_edge(i0, i2).expect("stall -> terrain");
    g.add_edge(i4, i0).expect("icing -> stall");
    g.add_edge(i0, i1).expect("stall -> overspeed");
    g.add_edge(i3, i1).expect("traffic -> overspeed");
    g.add_edge(i2, i3).expect("terrain -> traffic");
    g.add_edge(i5, i0).expect("cg -> stall");

    g.freeze();
    g
}

/// **Stage 3** — envelope graph evaluation.
///
/// Bind-callback shape: rebuilds the incoming process from
/// `(value, state, ctx)`, builds the envelope graph using `ctx` (falling back
/// to a benign config if `None`), and evaluates from index 0 via
/// `StatefulMonadicCausableGraphReasoning::evaluate_subgraph_from_cause_stateful`.
pub fn run_envelope_graph(
    value: EffectValue<FlightStateEstimate>,
    state: FlightState,
    ctx: Option<AircraftConfig>,
) -> FlightProcess<FlightStateEstimate> {
    let cfg = ctx.clone().unwrap_or_else(fallback_config);
    let incoming: FlightProcess<FlightStateEstimate> = PropagatingProcess {
        value,
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    let graph = build_envelope_graph(cfg);
    graph.evaluate_subgraph_from_cause_stateful(0, &incoming)
}
