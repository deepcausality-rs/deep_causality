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
//! Runtime configuration values (preset `AircraftConfig`, `SensorReading`,
//! and `FlightStateEstimate` instances) live in [`super::model_config`].

use deep_causality::PropagatingProcess;

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

/// Read-only aircraft configuration carried in the `Context` channel.
#[derive(Debug, Clone)]
pub struct AircraftConfig {
    pub mass_kg: f64,
    pub mtow_kg: f64,
    pub stall_margin: f64,
    pub service_ceiling_m: f64,
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
// Aliases and per-sensor calibration
// ---------------------------------------------------------------------------

/// Local short alias for the long-form process type.
pub type FlightProcess<T> = PropagatingProcess<T, FlightState, AircraftConfig>;

/// Per-sensor healthy bands used to compute health probabilities.
#[derive(Debug, Clone, Copy)]
pub struct HealthyBands {
    pub airspeed_kn: (f64, f64),
    pub altitude_ft: (f64, f64),
    pub attitude_deg: (f64, f64),
    pub vertical_speed_fpm: (f64, f64),
    pub fuel_flow_pph: (f64, f64),
}

pub const NOMINAL_BANDS: HealthyBands = HealthyBands {
    airspeed_kn: (180.0, 320.0),
    altitude_ft: (5_000.0, 35_000.0),
    attitude_deg: (-10.0, 10.0),
    vertical_speed_fpm: (-1_500.0, 1_500.0),
    fuel_flow_pph: (1_500.0, 3_500.0),
};
