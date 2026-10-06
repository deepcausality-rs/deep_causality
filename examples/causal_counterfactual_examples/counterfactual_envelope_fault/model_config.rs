/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Nominal sensor reading and envelope context seed data.

use crate::model_types::{EnvelopeContext, FloatType, SensorReading};
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data,
};

/// Service ceiling of the aircraft in feet. The header prints it; no stage reads it, so it is not
/// a node of the envelope context.
pub const SERVICE_CEILING_FT: FloatType = 41_000.0;

pub fn nominal_sensor_reading() -> SensorReading {
    SensorReading {
        airspeed_kn: 240.0,
        altitude_ft: 28_000.0,
        attitude_deg: 1.5,
    }
}

/// The nominal aircraft and risk scale, added in node-index order: node `i` holds contextoid id
/// `i + 1`.
pub fn nominal_envelope_context() -> Result<EnvelopeContext, ContextIndexError> {
    let mut context = Context::with_capacity(1, "envelope", 7);
    for (id, value) in [
        (1, 180.0), // STALL_KN
        (2, 320.0), // OVERSPEED_KN
        (3, 0.8),   // STALL_RISK_WEIGHT
        (4, 0.5),   // OVERSPEED_RISK_WEIGHT
        (5, 0.10),  // CAUTION_RISK
        (6, 0.50),  // WARNING_RISK
        (7, 1.00),  // FAILURE_RISK
    ] {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}
