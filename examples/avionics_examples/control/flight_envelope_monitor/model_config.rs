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
//! * [`nominal_aircraft_config`] — sample airframe parameters used as the
//!   `Context` channel.
//! * [`nominal_sensor_reading`] — a slightly-degraded sample reading that
//!   exercises the smooth-deterioration aggregation in Stage 1 (the
//!   airspeed is just below the healthy band).
//! * [`seed_estimate_for`] — derives a `FlightStateEstimate` seed from a
//!   `SensorReading`. Used by the Stage-2.1 bind callback because
//!   `SensorReading` no longer flows on the value channel by that point.

use crate::model_types::{AircraftConfig, FlightStateEstimate, SensorReading};

/// Sample airframe configuration for the `Context` channel.
pub fn nominal_aircraft_config() -> AircraftConfig {
    AircraftConfig {
        mass_kg: 70_000.0,
        mtow_kg: 80_000.0,
        stall_margin: 1.3,
        service_ceiling_m: 12_800.0,
    }
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
