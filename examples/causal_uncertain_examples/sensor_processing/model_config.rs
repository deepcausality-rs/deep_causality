/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Nominal configuration and sensor seed data for the sensor-processing pipeline.

use crate::model_types::{Bands, FleetConfig, RawReadings, SensorReading, SensorStatus};
use std::collections::HashMap;

pub fn nominal_fleet_config() -> FleetConfig {
    FleetConfig {
        temp: Bands {
            plausible: (-50.0, 100.0),
            nominal: (15.0, 35.0),
        },
        pressure: Bands {
            plausible: (800.0, 1200.0),
            nominal: (980.0, 1050.0),
        },
        humidity: Bands {
            plausible: (0.0, 100.0),
            nominal: (20.0, 80.0),
        },
        pressure_2_calibration_offset: -2.3,
        temp_calibration_gain: 0.98,
        temp_calibration_bias: 0.5,
        anomaly_disagreement_c: 5.0,
        high_uncertainty_threshold: 5.0,
    }
}

pub fn seed_readings() -> RawReadings {
    let mut sensors: HashMap<String, SensorReading> = HashMap::new();

    sensors.insert(
        "temp_1".into(),
        SensorReading {
            id: "temp_1".into(),
            value: Some(23.2),
            timestamp: 1000,
            status: SensorStatus::Healthy,
            uncertainty: Some(0.5),
        },
    );
    sensors.insert(
        "temp_2".into(),
        SensorReading {
            id: "temp_2".into(),
            value: Some(85.7), // implausibly high for room temp
            timestamp: 1002,
            status: SensorStatus::OutOfRange,
            uncertainty: Some(5.0),
        },
    );
    sensors.insert(
        "temp_3".into(),
        SensorReading {
            id: "temp_3".into(),
            value: None,
            timestamp: 995,
            status: SensorStatus::CommunicationError,
            uncertainty: None,
        },
    );
    sensors.insert(
        "pressure_1".into(),
        SensorReading {
            id: "pressure_1".into(),
            value: Some(1013.25),
            timestamp: 1001,
            status: SensorStatus::Healthy,
            uncertainty: Some(2.0),
        },
    );
    sensors.insert(
        "pressure_2".into(),
        SensorReading {
            id: "pressure_2".into(),
            value: Some(1015.8),
            timestamp: 1003,
            status: SensorStatus::CalibrationDrift,
            uncertainty: Some(8.0),
        },
    );
    sensors.insert(
        "humidity_1".into(),
        SensorReading {
            id: "humidity_1".into(),
            value: Some(45.2),
            timestamp: 999,
            status: SensorStatus::Degraded,
            uncertainty: Some(3.5),
        },
    );
    sensors.insert(
        "humidity_2".into(),
        SensorReading {
            id: "humidity_2".into(),
            value: None,
            timestamp: 980,
            status: SensorStatus::Failed,
            uncertainty: None,
        },
    );

    RawReadings(sensors)
}
