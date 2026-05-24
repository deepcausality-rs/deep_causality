/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the flight-envelope fault analysis chain.

#![allow(dead_code)] // Domain fields kept for narrative clarity even if not all are read.

use deep_causality_core::PropagatingProcess;

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this crate would need wrapping
/// in `FloatType::from(…)` to switch away from `f64`.
pub type FloatType = f64;

#[derive(Debug, Default, Clone)]
pub struct SensorReading {
    pub airspeed_kn: FloatType,
    pub altitude_ft: FloatType,
    pub attitude_deg: FloatType,
}

#[derive(Debug, Default, Clone)]
pub struct FlightState {
    pub estimate_airspeed_kn: FloatType,
    pub estimate_altitude_ft: FloatType,
    pub risk: FloatType,
}

#[derive(Debug, Clone)]
pub struct AircraftConfig {
    pub stall_kn: FloatType,
    pub overspeed_kn: FloatType,
    pub service_ceiling_ft: FloatType,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Verdict {
    #[default]
    Nominal,
    Caution,
    Warning,
    Failure,
}

impl Verdict {
    pub fn from_risk(risk: FloatType) -> Self {
        if risk < 0.10 {
            Verdict::Nominal
        } else if risk < 0.50 {
            Verdict::Caution
        } else if risk < 1.00 {
            Verdict::Warning
        } else {
            Verdict::Failure
        }
    }
}

/// Process alias for the chain. Mirrors the avionics convention.
pub type FlightProcess<T> = PropagatingProcess<T, FlightState, AircraftConfig>;
