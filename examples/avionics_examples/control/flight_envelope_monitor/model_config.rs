/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Flight Envelope Monitor — Preset Configuration Values
//!
//! Runtime instances supplied to the pipeline at start-up. Lifted out of
//! `main.rs` so the composition there reads as pure orchestration without
//! the noise of literal numbers.
//!
//! * [`nominal_airframe`] — the sample airframe the `Context` channel
//!   carries: weights, stall margin, ceiling, and the normal operating bands.
//! * [`nominal_sensor_reading`] — a slightly-degraded sample reading that
//!   exercises the smooth-deterioration aggregation in Stage 1 (the
//!   airspeed is just below the healthy band).
//! * [`seed_estimate_for`] — derives a `FlightStateEstimate` seed from a
//!   `SensorReading`. Used by the Stage-2.1 bind callback because
//!   `SensorReading` no longer flows on the value channel by that point.

use crate::model_types::{AirframeContext, FlightStateEstimate, SensorReading};
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data,
};

/// The sample airframe for the `Context` channel, added in node-index order:
/// node `i` holds contextoid id `i + 1`. Mass, MTOW and ceiling load at
/// dispatch; the bands are the airframe's normal operating ranges, which the
/// sensor-health causaloids and the envelope nodes both read.
pub fn nominal_airframe() -> Result<AirframeContext, ContextIndexError> {
    let quantities = [
        70_000.0, // MASS_KG
        80_000.0, // MTOW_KG
        1.3,      // STALL_MARGIN
        12_800.0, // SERVICE_CEILING_M
        1.3,      // AIRSPEED_BAND_STALL_MARGIN
        180.0,    // AIRSPEED_MIN_KN
        320.0,    // AIRSPEED_MAX_KN
        5_000.0,  // ALTITUDE_MIN_FT
        35_000.0, // ALTITUDE_MAX_FT
        -10.0,    // ATTITUDE_MIN_DEG
        10.0,     // ATTITUDE_MAX_DEG
        -1_500.0, // VERTICAL_SPEED_MIN_FPM
        1_500.0,  // VERTICAL_SPEED_MAX_FPM
        1_500.0,  // FUEL_FLOW_MIN_PPH
        3_500.0,  // FUEL_FLOW_MAX_PPH
    ];
    let mut context = Context::with_capacity(1, "nominal airframe", quantities.len());
    for (id, value) in (1..).zip(quantities) {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// Sample per-cycle sensor reading. Airspeed `175 kn` sits just below the
/// healthy band `[180, 320]`, producing a non-1.0 joint-health signal that
/// seeds a non-zero `state.risk` in Stage 2.1.
pub fn nominal_sensor_reading() -> SensorReading {
    SensorReading {
        airspeed_kn: 175.0,
        altitude_ft: 12_500.0,
        attitude_deg: 2.0,
        vertical_speed_fpm: 0.0,
        fuel_flow_pph: 2_400.0,
    }
}

/// Build the seed `FlightStateEstimate` that Stage 2.1 places on the value
/// channel after the sensor collection has reduced the value channel to a
/// scalar joint-health probability.
pub fn seed_estimate_for(reading: &SensorReading) -> FlightStateEstimate {
    FlightStateEstimate {
        airspeed_kn: reading.airspeed_kn,
        altitude_ft: reading.altitude_ft,
        attitude_deg: reading.attitude_deg,
        vertical_speed_fpm: reading.vertical_speed_fpm,
    }
}
