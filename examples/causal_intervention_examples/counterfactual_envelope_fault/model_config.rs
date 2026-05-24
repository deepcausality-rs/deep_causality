/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Nominal sensor reading and aircraft configuration seed data.

use crate::model_types::{AircraftConfig, SensorReading};

pub fn nominal_sensor_reading() -> SensorReading {
    SensorReading {
        airspeed_kn: 240.0,
        altitude_ft: 28_000.0,
        attitude_deg: 1.5,
    }
}

pub fn nominal_aircraft_config() -> AircraftConfig {
    AircraftConfig {
        stall_kn: 180.0,
        overspeed_kn: 320.0,
        service_ceiling_ft: 41_000.0,
    }
}
