/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Flight Envelope Monitor — Domain Types
//!
//! Pure data definitions for the flight-envelope-monitor pipeline. Kept
//! separate from `model.rs` (the closures and builders) so a reader can
//! grasp the value-channel and State-channel shapes before encountering the
//! reasoning logic.
//!
//! The airframe the pipeline reads lives in the `Context` channel as an
//! [`AirframeContext`]; the contextoid ids below key its quantities. Runtime
//! values (the nominal airframe, `SensorReading`, and `FlightStateEstimate`
//! instances) live in [`super::model_config`].

use deep_causality::{CausalityError, PropagatingProcess};
use deep_causality_context::{Context, ContextoidId, Data, NoSpace, NoSpaceTime, NoTime};

// ---------------------------------------------------------------------------
// Process channels
// ---------------------------------------------------------------------------

/// Markovian process state. Accumulates across all three stages.
///
/// * `estimate` — four-element state vector
///   (airspeed, altitude, attitude, vertical-speed).
/// * `covariance` — diagonal covariance, evolved by the bind-chain's
///   Kalman step.
/// * `risk` — cumulative scalar risk; receives contributions from Stage 2's
///   health-fold step and from each Stage 3 envelope node.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FlightState {
    pub estimate: [f64; 4],
    pub covariance: [f64; 4],
    pub risk: f64,
}

/// The airframe the `Context` channel carries: one `Data` contextoid per
/// quantity. The context holds no position, clock or event, so its spatial,
/// temporal and spacetime slots are empty.
pub type AirframeContext = Context<Data<f64>, NoSpace<f64>, NoTime, NoSpaceTime<f64>>;

/// Contextoid id: current aircraft mass, kg.
pub const MASS_KG: ContextoidId = 1;
/// Contextoid id: maximum takeoff weight, kg.
pub const MTOW_KG: ContextoidId = 2;
/// Contextoid id: stall-margin multiplier applied to the stall speed (dimensionless).
pub const STALL_MARGIN: ContextoidId = 3;
/// Contextoid id: service ceiling, m.
pub const SERVICE_CEILING_M: ContextoidId = 4;
/// Contextoid id: the stall-margin multiplier at which the airspeed band's lower
/// edge is stated (dimensionless). The stall node rescales that edge from this
/// margin to [`STALL_MARGIN`].
pub const AIRSPEED_BAND_STALL_MARGIN: ContextoidId = 5;
/// Contextoid id: lower edge of the normal airspeed band, kn.
pub const AIRSPEED_MIN_KN: ContextoidId = 6;
/// Contextoid id: upper edge of the normal airspeed band, kn.
pub const AIRSPEED_MAX_KN: ContextoidId = 7;
/// Contextoid id: lower edge of the normal altitude band, ft.
pub const ALTITUDE_MIN_FT: ContextoidId = 8;
/// Contextoid id: upper edge of the normal altitude band, ft.
pub const ALTITUDE_MAX_FT: ContextoidId = 9;
/// Contextoid id: lower edge of the normal attitude band, deg.
pub const ATTITUDE_MIN_DEG: ContextoidId = 10;
/// Contextoid id: upper edge of the normal attitude band, deg.
pub const ATTITUDE_MAX_DEG: ContextoidId = 11;
/// Contextoid id: lower edge of the normal vertical-speed band, ft/min.
pub const VERTICAL_SPEED_MIN_FPM: ContextoidId = 12;
/// Contextoid id: upper edge of the normal vertical-speed band, ft/min.
pub const VERTICAL_SPEED_MAX_FPM: ContextoidId = 13;
/// Contextoid id: lower edge of the normal fuel-flow band, lb/h.
pub const FUEL_FLOW_MIN_PPH: ContextoidId = 14;
/// Contextoid id: upper edge of the normal fuel-flow band, lb/h.
pub const FUEL_FLOW_MAX_PPH: ContextoidId = 15;

/// Read the `Data` contextoid carrying `id` out of the airframe context. An id the context does
/// not hold as a Datoid is an error.
pub fn read(context: &AirframeContext, id: ContextoidId) -> Result<f64, CausalityError> {
    context.get_data_by_id(id).ok_or_else(|| {
        CausalityError::MissingParameter(format!(
            "airframe context holds no Datoid with contextoid id {id}"
        ))
    })
}

// ---------------------------------------------------------------------------
// Value-channel types
// ---------------------------------------------------------------------------

/// Per-cycle sensor readings fed into Stage 1.
#[derive(Debug, Default, Clone)]
pub struct SensorReading {
    pub airspeed_kn: f64,
    pub altitude_ft: f64,
    pub attitude_deg: f64,
    pub vertical_speed_fpm: f64,
    pub fuel_flow_pph: f64,
}

/// Value-channel payload through the bind chain and the envelope graph.
///
/// `V == V` for the graph reasoning trait, so this is the type carried
/// end-to-end through Stage 3.
#[derive(Debug, Default, Clone)]
pub struct FlightStateEstimate {
    pub airspeed_kn: f64,
    pub altitude_ft: f64,
    pub attitude_deg: f64,
    pub vertical_speed_fpm: f64,
}

// ---------------------------------------------------------------------------
// Final classification
// ---------------------------------------------------------------------------

/// Final classification derived from `final_state.risk` in `main.rs`.
///
/// Risk thresholds:
/// * `risk < 0.10` → `Nominal`
/// * `risk < 0.50` → `Caution`
/// * `risk < 1.00` → `Warning`
/// * otherwise     → `Failure`
#[derive(Debug, Clone)]
pub enum SafetyVerdict {
    Nominal,
    Caution,
    Warning,
    Failure,
}

impl SafetyVerdict {
    pub fn from_risk(risk: f64) -> Self {
        if risk < 0.10 {
            SafetyVerdict::Nominal
        } else if risk < 0.50 {
            SafetyVerdict::Caution
        } else if risk < 1.00 {
            SafetyVerdict::Warning
        } else {
            SafetyVerdict::Failure
        }
    }
}

// ---------------------------------------------------------------------------
// Aliases
// ---------------------------------------------------------------------------

/// Local short alias for the long-form process type.
pub type FlightProcess<T> = PropagatingProcess<T, FlightState, AirframeContext>;
