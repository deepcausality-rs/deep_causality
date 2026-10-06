/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Nominal sensor reading and envelope context seed data.

use crate::model_types::{
    CAUTION_RISK, EnvelopeContext, FAILURE_RISK, FloatType, OVERSPEED_KN, OVERSPEED_RISK_WEIGHT,
    STALL_KN, STALL_RISK_WEIGHT, SensorReading, WARNING_RISK,
};
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data,
};

/// Service ceiling of the aircraft in feet. The header prints it; no stage reads it, so it is not
/// a contextoid of the envelope context.
pub const SERVICE_CEILING_FT: FloatType = 41_000.0;

pub fn nominal_sensor_reading() -> SensorReading {
    SensorReading {
        airspeed_kn: 240.0,
        altitude_ft: 28_000.0,
        attitude_deg: 1.5,
    }
}

/// The nominal aircraft and risk scale: one `Data` contextoid per quantity, keyed by its
/// contextoid id.
pub fn nominal_envelope_context() -> Result<EnvelopeContext, ContextIndexError> {
    let facts = [
        (STALL_KN, 180.0),
        (OVERSPEED_KN, 320.0),
        (STALL_RISK_WEIGHT, 0.8),
        (OVERSPEED_RISK_WEIGHT, 0.5),
        (CAUTION_RISK, 0.10),
        (WARNING_RISK, 0.50),
        (FAILURE_RISK, 1.00),
    ];
    let mut context = Context::with_capacity(1, "envelope", facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}
