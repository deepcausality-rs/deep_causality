/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the flight-envelope fault analysis chain.

#![allow(dead_code)] // Domain fields kept for narrative clarity even if not all are read.

use deep_causality_context::{Context, ContextoidId, Data, NoSpace, NoSpaceTime, NoTime};
use deep_causality_core::{CausalityError, PropagatingProcess};

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this crate would need lifting
/// through `deep_causality_num::lift` to switch away from `f64`.
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

/// The aircraft's envelope limits and the risk scale the steps read, one `Data` contextoid per
/// quantity. The context holds no position, clock or event, so its spatial, temporal and spacetime
/// slots are empty.
pub type EnvelopeContext =
    Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: stall speed in knots.
pub const STALL_KN: ContextoidId = 1;
/// Contextoid id: never-exceed speed (Vne) in knots.
pub const OVERSPEED_KN: ContextoidId = 2;
/// Contextoid id: risk added per unit of the fractional shortfall below the stall speed.
pub const STALL_RISK_WEIGHT: ContextoidId = 3;
/// Contextoid id: risk added per unit of the fractional excess above the never-exceed speed.
pub const OVERSPEED_RISK_WEIGHT: ContextoidId = 4;
/// Contextoid id: risk at and above which the verdict is `Caution`.
pub const CAUTION_RISK: ContextoidId = 5;
/// Contextoid id: risk at and above which the verdict is `Warning`.
pub const WARNING_RISK: ContextoidId = 6;
/// Contextoid id: risk at and above which the verdict is `Failure`.
pub const FAILURE_RISK: ContextoidId = 7;

/// Read the payload of the `Data` contextoid `id` out of the envelope context.
pub fn read(context: &EnvelopeContext, id: ContextoidId) -> Result<FloatType, CausalityError> {
    context.get_data_by_id(id).ok_or_else(|| {
        CausalityError::MissingParameter(format!(
            "the envelope context holds no Datoid with contextoid id {id}"
        ))
    })
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
    /// Classify `risk` against the verdict thresholds of the envelope context.
    pub fn from_risk(risk: FloatType, envelope: &EnvelopeContext) -> Result<Self, CausalityError> {
        Ok(if risk < read(envelope, CAUTION_RISK)? {
            Verdict::Nominal
        } else if risk < read(envelope, WARNING_RISK)? {
            Verdict::Caution
        } else if risk < read(envelope, FAILURE_RISK)? {
            Verdict::Warning
        } else {
            Verdict::Failure
        })
    }
}

/// Process alias for the chain. Mirrors the avionics convention.
pub type FlightProcess<T> = PropagatingProcess<T, FlightState, EnvelopeContext>;
