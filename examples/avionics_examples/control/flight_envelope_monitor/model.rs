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
//! `fn(CausalEffect<I>, FlightState, Option<AirframeContext>) -> FlightProcess<O>`
//! so that the whole pipeline can be expressed in `main.rs` as one
//! `PropagatingProcess::pure(...).bind(stage1).bind(stage2_1).bind(...)`
//! chain.

use crate::model_types::{
    AIRSPEED_BAND_STALL_MARGIN, AIRSPEED_MAX_KN, AIRSPEED_MIN_KN, ALTITUDE_MAX_FT, ALTITUDE_MIN_FT,
    ATTITUDE_MAX_DEG, ATTITUDE_MIN_DEG, AirframeContext, FUEL_FLOW_MAX_PPH, FUEL_FLOW_MIN_PPH,
    FlightProcess, FlightState, FlightStateEstimate, MASS_KG, MTOW_KG, SERVICE_CEILING_M,
    STALL_MARGIN, SensorReading, VERTICAL_SPEED_MAX_FPM, VERTICAL_SPEED_MIN_FPM, read,
};
use deep_causality::*;
use deep_causality_context::ContextoidId;
use deep_causality_core::CausalityErrorEnum;
use std::fmt::Display;

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

/// The error process a stage returns when the `Context` channel carries no
/// airframe: the stage cannot evaluate without the airframe it reads.
fn missing_airframe<T>(state: FlightState) -> FlightProcess<T> {
    PropagatingProcess::new(
        Err(CausalityError::MissingContext()),
        state,
        None,
        EffectLog::new(),
    )
}

// ---------------------------------------------------------------------------
// Stage 1 — sensor causaloids (per-sensor f64 health probability)
// ---------------------------------------------------------------------------
//
// Each per-sensor closure reads its healthy band from the airframe in the
// `Context` channel and passes the State channel through untouched: health
// depends only on the per-sensor reading and the band.

/// The airframe in the `Context` channel, or the error a stage returns when the channel is empty.
fn airframe(ctx: Option<&AirframeContext>) -> Result<&AirframeContext, CausalityError> {
    ctx.ok_or_else(CausalityError::MissingContext)
}

/// One sensor's health: `measure` picks the reading, `band` names the
/// contextoid ids of the band edges, and `tolerance` is the distance
/// outside the band at which health reaches zero.
fn sensor_health(
    value: CausalEffect<SensorReading>,
    state: FlightState,
    ctx: Option<AirframeContext>,
    label: &str,
    measure: fn(&SensorReading) -> f64,
    band: (ContextoidId, ContextoidId),
    tolerance: f64,
) -> FlightProcess<f64> {
    let health = value
        .into_value()
        .ok_or_else(CausalityError::ValueNotAvailable)
        .and_then(|reading| {
            let airframe = airframe(ctx.as_ref())?;
            let edges = (read(airframe, band.0)?, read(airframe, band.1)?);
            Ok(health_probability(measure(&reading), edges, tolerance))
        });
    match health {
        Ok(h) => {
            let mut log = EffectLog::new();
            log.add_entry(&format!("{label}: health={h:.3}"));
            PropagatingProcess::new(Ok(CausalEffect::value(h)), state, ctx, log)
        }
        Err(e) => PropagatingProcess::new(Err(e), state, ctx, EffectLog::new()),
    }
}

fn airspeed_health(
    value: CausalEffect<SensorReading>,
    state: FlightState,
    ctx: Option<AirframeContext>,
) -> FlightProcess<f64> {
    sensor_health(
        value,
        state,
        ctx,
        "sensor.airspeed",
        |r| r.airspeed_kn,
        (AIRSPEED_MIN_KN, AIRSPEED_MAX_KN),
        80.0,
    )
}

fn altitude_health(
    value: CausalEffect<SensorReading>,
    state: FlightState,
    ctx: Option<AirframeContext>,
) -> FlightProcess<f64> {
    sensor_health(
        value,
        state,
        ctx,
        "sensor.altitude",
        |r| r.altitude_ft,
        (ALTITUDE_MIN_FT, ALTITUDE_MAX_FT),
        10_000.0,
    )
}

fn attitude_health(
    value: CausalEffect<SensorReading>,
    state: FlightState,
    ctx: Option<AirframeContext>,
) -> FlightProcess<f64> {
    sensor_health(
        value,
        state,
        ctx,
        "sensor.attitude",
        |r| r.attitude_deg,
        (ATTITUDE_MIN_DEG, ATTITUDE_MAX_DEG),
        20.0,
    )
}

fn vertical_speed_health(
    value: CausalEffect<SensorReading>,
    state: FlightState,
    ctx: Option<AirframeContext>,
) -> FlightProcess<f64> {
    sensor_health(
        value,
        state,
        ctx,
        "sensor.vsi",
        |r| r.vertical_speed_fpm,
        (VERTICAL_SPEED_MIN_FPM, VERTICAL_SPEED_MAX_FPM),
        2_000.0,
    )
}

fn fuel_flow_health(
    value: CausalEffect<SensorReading>,
    state: FlightState,
    ctx: Option<AirframeContext>,
) -> FlightProcess<f64> {
    sensor_health(
        value,
        state,
        ctx,
        "sensor.fuel_flow",
        |r| r.fuel_flow_pph,
        (FUEL_FLOW_MIN_PPH, FUEL_FLOW_MAX_PPH),
        1_500.0,
    )
}

/// Failing-airspeed closure used by the failing-sensor scenario.
fn airspeed_failing(_reading: SensorReading) -> PropagatingEffect<f64> {
    PropagatingEffect::from_error(CausalityError::new(CausalityErrorEnum::Custom(
        "sensor.airspeed: hardware fault — sensor lost".into(),
    )))
}

/// Build the five per-sensor singleton causaloids. Each healthy-sensor
/// causaloid holds the airframe as its stored context; the stateful
/// evaluation hands it the airframe of the incoming process.
fn build_sensor_causaloids(
    airframe: AirframeContext,
    failing_airspeed: bool,
) -> Vec<Causaloid<SensorReading, f64, FlightState, AirframeContext>> {
    let airspeed = if failing_airspeed {
        Causaloid::new(0, airspeed_failing, "airspeed sensor (FAILING)")
    } else {
        Causaloid::new_with_context(0, airspeed_health, airframe.clone(), "airspeed sensor")
    };

    vec![
        airspeed,
        Causaloid::new_with_context(1, altitude_health, airframe.clone(), "altitude sensor"),
        Causaloid::new_with_context(2, attitude_health, airframe.clone(), "attitude sensor"),
        Causaloid::new_with_context(
            3,
            vertical_speed_health,
            airframe.clone(),
            "vertical-speed sensor",
        ),
        Causaloid::new_with_context(4, fuel_flow_health, airframe, "fuel-flow sensor"),
    ]
}

/// **Stage 1** — sensor collection evaluation.
///
/// Bind-callback shape: takes `(CausalEffect<SensorReading>, FlightState,
/// Option<AirframeContext>)`, reconstructs the incoming process, and evaluates
/// the per-sensor collection via
/// `StatefulMonadicCausableCollection::evaluate_collection_stateful` with
/// `AggregateLogic::All` (joint health probability via `∏ p_i`). Returns
/// `FlightProcess<f64>` whose value channel carries the joint health
/// probability.
pub fn run_sensor_collection(
    value: CausalEffect<SensorReading>,
    state: FlightState,
    ctx: Option<AirframeContext>,
    failing_airspeed: bool,
) -> FlightProcess<f64> {
    let Some(airframe) = ctx.clone() else {
        return missing_airframe(state);
    };
    let incoming: FlightProcess<SensorReading> =
        PropagatingProcess::new(Ok(value), state, ctx, EffectLog::new());
    let sensors = build_sensor_causaloids(airframe, failing_airspeed);
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
    value: CausalEffect<f64>,
    mut state: FlightState,
    ctx: Option<AirframeContext>,
    seed_estimate: FlightStateEstimate,
) -> FlightProcess<FlightStateEstimate> {
    let health = match value.into_value() {
        Some(h) => h,
        None => {
            return PropagatingProcess::new(
                Err(CausalityError::new(CausalityErrorEnum::Custom(
                    "stage2.health_fold: value was None".into(),
                ))),
                state,
                ctx,
                EffectLog::new(),
            );
        }
    };

    state.risk += (1.0 - health) * RISK_HEALTH_WEIGHT;

    let mut log = EffectLog::new();
    log.add_entry(&format!(
        "stage2.health_fold: risk += {:.3} (health={:.3})",
        (1.0 - health) * RISK_HEALTH_WEIGHT,
        health
    ));
    PropagatingProcess::new(Ok(CausalEffect::value(seed_estimate)), state, ctx, log)
}

/// **Stage 2.2** — one-iteration scalar Kalman update on each diagonal element
/// of `state.covariance`. Illustrative only.
pub fn kalman_step(
    value: CausalEffect<FlightStateEstimate>,
    mut state: FlightState,
    ctx: Option<AirframeContext>,
) -> FlightProcess<FlightStateEstimate> {
    let estimate = match value.into_value() {
        Some(v) => v,
        None => {
            return PropagatingProcess::new(
                Err(CausalityError::new(CausalityErrorEnum::Custom(
                    "stage2.kalman: value was None".into(),
                ))),
                state,
                ctx,
                EffectLog::new(),
            );
        }
    };

    if state.covariance == [0.0; 4] {
        state.covariance = [4.0, 4.0, 4.0, 4.0];
    }

    for cov_i in state.covariance.iter_mut() {
        let k_i = *cov_i / (*cov_i + KALMAN_MEAS_NOISE);
        *cov_i *= 1.0 - k_i;
    }

    let mut log = EffectLog::new();
    log.add_entry("stage2.kalman: covariance updated (one-iteration scalar)");
    PropagatingProcess::new(Ok(CausalEffect::value(estimate)), state, ctx, log)
}

/// **Stage 2.3** — write the four `FlightStateEstimate` fields into
/// `state.estimate`.
pub fn estimate_step(
    value: CausalEffect<FlightStateEstimate>,
    mut state: FlightState,
    ctx: Option<AirframeContext>,
) -> FlightProcess<FlightStateEstimate> {
    let estimate = match value.clone().into_value() {
        Some(v) => v,
        None => {
            return PropagatingProcess::new(
                Err(CausalityError::new(CausalityErrorEnum::Custom(
                    "stage2.estimate: value was None".into(),
                ))),
                state,
                ctx,
                EffectLog::new(),
            );
        }
    };

    state.estimate = [
        estimate.airspeed_kn,
        estimate.altitude_ft,
        estimate.attitude_deg,
        estimate.vertical_speed_fpm,
    ];

    let mut log = EffectLog::new();
    log.add_entry("stage2.estimate: state.estimate written");
    PropagatingProcess::new(Ok(value), state, ctx, log)
}

// ---------------------------------------------------------------------------
// Stage 3 — envelope causaloid hypergraph
// ---------------------------------------------------------------------------

/// One envelope node: `pressure` maps the estimate (and the airframe, for the
/// nodes that read it) to a pressure in `[0, 1]`; the node adds
/// `pressure * weight` to `state.risk` and logs the increment under `label`.
fn envelope_node(
    value: CausalEffect<FlightStateEstimate>,
    mut state: FlightState,
    ctx: Option<AirframeContext>,
    label: &str,
    weight: f64,
    pressure: impl FnOnce(&FlightStateEstimate, Option<&AirframeContext>) -> Result<f64, CausalityError>,
) -> FlightProcess<FlightStateEstimate> {
    let assessed = value
        .into_value()
        .ok_or_else(CausalityError::ValueNotAvailable)
        .and_then(|est| pressure(&est, ctx.as_ref()).map(|p| (est, p)));
    let (est, pressure) = match assessed {
        Ok(assessed) => assessed,
        Err(e) => return PropagatingProcess::new(Err(e), state, ctx, EffectLog::new()),
    };
    let increment = pressure * weight;
    state.risk += increment;
    let mut log = EffectLog::new();
    log.add_entry(&format!("{label}: risk += {increment:.3}"));
    PropagatingProcess::new(Ok(CausalEffect::value(est)), state, ctx, log)
}

fn stall_risk(
    value: CausalEffect<FlightStateEstimate>,
    state: FlightState,
    ctx: Option<AirframeContext>,
) -> FlightProcess<FlightStateEstimate> {
    envelope_node(
        value,
        state,
        ctx,
        "envelope.stall",
        STALL_RISK_WEIGHT,
        |est, ctx| {
            let airframe = airframe(ctx)?;
            let lo = read(airframe, AIRSPEED_MIN_KN)? * read(airframe, STALL_MARGIN)?
                / read(airframe, AIRSPEED_BAND_STALL_MARGIN)?;
            Ok(((lo - est.airspeed_kn).max(0.0) / lo).clamp(0.0, 1.0))
        },
    )
}

fn overspeed_risk(
    value: CausalEffect<FlightStateEstimate>,
    state: FlightState,
    ctx: Option<AirframeContext>,
) -> FlightProcess<FlightStateEstimate> {
    envelope_node(
        value,
        state,
        ctx,
        "envelope.overspeed",
        OVERSPEED_RISK_WEIGHT,
        |est, ctx| {
            let hi = read(airframe(ctx)?, AIRSPEED_MAX_KN)?;
            Ok(((est.airspeed_kn - hi).max(0.0) / hi).clamp(0.0, 1.0))
        },
    )
}

fn terrain_proximity(
    value: CausalEffect<FlightStateEstimate>,
    state: FlightState,
    ctx: Option<AirframeContext>,
) -> FlightProcess<FlightStateEstimate> {
    envelope_node(
        value,
        state,
        ctx,
        "envelope.terrain",
        TERRAIN_RISK_WEIGHT,
        |est, ctx| {
            let floor = read(airframe(ctx)?, ALTITUDE_MIN_FT)?;
            Ok(((floor - est.altitude_ft).max(0.0) / 5_000.0).clamp(0.0, 1.0))
        },
    )
}

fn traffic_conflict(
    value: CausalEffect<FlightStateEstimate>,
    state: FlightState,
    ctx: Option<AirframeContext>,
) -> FlightProcess<FlightStateEstimate> {
    envelope_node(
        value,
        state,
        ctx,
        "envelope.traffic",
        TRAFFIC_RISK_WEIGHT,
        |est, ctx| {
            // Traffic density drops sharply near the service ceiling.
            let altitude_m = est.altitude_ft * 0.3048;
            let near_ceiling = altitude_m / read(airframe(ctx)?, SERVICE_CEILING_M)?;
            Ok(if near_ceiling > 0.8 {
                0.02
            } else if est.altitude_ft > 25_000.0 {
                0.05
            } else {
                0.20
            })
        },
    )
}

fn icing_risk(
    value: CausalEffect<FlightStateEstimate>,
    state: FlightState,
    ctx: Option<AirframeContext>,
) -> FlightProcess<FlightStateEstimate> {
    envelope_node(
        value,
        state,
        ctx,
        "envelope.icing",
        ICING_RISK_WEIGHT,
        |est, _| {
            Ok(if (10_000.0..20_000.0).contains(&est.altitude_ft) {
                0.30
            } else {
                0.05
            })
        },
    )
}

fn cg_out_of_limits(
    value: CausalEffect<FlightStateEstimate>,
    state: FlightState,
    ctx: Option<AirframeContext>,
) -> FlightProcess<FlightStateEstimate> {
    envelope_node(
        value,
        state,
        ctx,
        "envelope.cg",
        CG_RISK_WEIGHT,
        |_, ctx| {
            let airframe = airframe(ctx)?;
            Ok((read(airframe, MASS_KG)? / read(airframe, MTOW_KG)?).clamp(0.0, 1.0))
        },
    )
}

/// Build the envelope hypergraph. Each node holds the airframe as its stored
/// context; the stateful evaluation hands it the airframe of the incoming
/// process.
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
    airframe: AirframeContext,
) -> Result<
    CausaloidGraph<
        Causaloid<FlightStateEstimate, FlightStateEstimate, FlightState, AirframeContext>,
    >,
    CausalityError,
> {
    let mut g: CausaloidGraph<
        Causaloid<FlightStateEstimate, FlightStateEstimate, FlightState, AirframeContext>,
    > = CausaloidGraph::new(0u64);

    let n_stall = Causaloid::new_with_context(100, stall_risk, airframe.clone(), "stall risk");
    let n_overspeed =
        Causaloid::new_with_context(101, overspeed_risk, airframe.clone(), "overspeed risk");
    let n_terrain = Causaloid::new_with_context(
        102,
        terrain_proximity,
        airframe.clone(),
        "terrain proximity",
    );
    let n_traffic =
        Causaloid::new_with_context(103, traffic_conflict, airframe.clone(), "traffic conflict");
    let n_icing = Causaloid::new_with_context(104, icing_risk, airframe.clone(), "icing risk");
    let n_cg = Causaloid::new_with_context(105, cg_out_of_limits, airframe, "CG out of limits");

    let i0 = g
        .add_root_causaloid(n_stall)
        .map_err(|e| graph_error("root stall", e))?;
    let i1 = g
        .add_causaloid(n_overspeed)
        .map_err(|e| graph_error("overspeed", e))?;
    let i2 = g
        .add_causaloid(n_terrain)
        .map_err(|e| graph_error("terrain", e))?;
    let i3 = g
        .add_causaloid(n_traffic)
        .map_err(|e| graph_error("traffic", e))?;
    let i4 = g
        .add_causaloid(n_icing)
        .map_err(|e| graph_error("icing", e))?;
    let i5 = g.add_causaloid(n_cg).map_err(|e| graph_error("cg", e))?;

    for (from, to, edge) in [
        (i0, i2, "stall -> terrain"),
        (i4, i0, "icing -> stall"),
        (i0, i1, "stall -> overspeed"),
        (i3, i1, "traffic -> overspeed"),
        (i2, i3, "terrain -> traffic"),
        (i5, i0, "cg -> stall"),
    ] {
        g.add_edge(from, to).map_err(|e| graph_error(edge, e))?;
    }

    g.freeze();
    Ok(g)
}

/// A failure to build the envelope graph, naming the node or edge being added.
fn graph_error(step: &str, e: impl Display) -> CausalityError {
    CausalityError::GraphError(format!("envelope graph, {step}: {e}"))
}

/// **Stage 3** — envelope graph evaluation.
///
/// Bind-callback shape: rebuilds the incoming process from
/// `(value, state, ctx)`, builds the envelope graph over the airframe in
/// `ctx` (a missing airframe or a graph-construction failure is an error on
/// the process), and evaluates from index 0 via
/// `StatefulMonadicCausableGraphReasoning::evaluate_subgraph_from_cause_stateful`.
pub fn run_envelope_graph(
    value: CausalEffect<FlightStateEstimate>,
    state: FlightState,
    ctx: Option<AirframeContext>,
) -> FlightProcess<FlightStateEstimate> {
    let Some(airframe) = ctx.clone() else {
        return missing_airframe(state);
    };
    let graph = match build_envelope_graph(airframe) {
        Ok(graph) => graph,
        Err(e) => return PropagatingProcess::new(Err(e), state, ctx, EffectLog::new()),
    };
    let incoming: FlightProcess<FlightStateEstimate> =
        PropagatingProcess::new(Ok(value), state, ctx, EffectLog::new());
    graph.evaluate_subgraph_from_cause_stateful(0, &incoming)
}
