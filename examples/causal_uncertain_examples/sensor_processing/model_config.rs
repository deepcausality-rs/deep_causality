/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Nominal fleet context and sensor seed data for the sensor-processing pipeline.

use crate::model_types::{FleetContext, RawReadings, SensorReading, SensorStatus};
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data,
};
use std::collections::HashMap;

/// The nominal fleet, added in node-index order: node `i` holds contextoid id `i + 1`.
pub fn nominal_fleet_context() -> Result<FleetContext, ContextIndexError> {
    let mut context = Context::with_capacity(1, "fleet", 30);
    for (id, value) in [
        (1, -50.0),    // TEMP_PLAUSIBLE_MIN
        (2, 100.0),    // TEMP_PLAUSIBLE_MAX
        (3, 15.0),     // TEMP_NOMINAL_MIN
        (4, 35.0),     // TEMP_NOMINAL_MAX
        (5, 800.0),    // PRESSURE_PLAUSIBLE_MIN
        (6, 1200.0),   // PRESSURE_PLAUSIBLE_MAX
        (7, 980.0),    // PRESSURE_NOMINAL_MIN
        (8, 1050.0),   // PRESSURE_NOMINAL_MAX
        (9, 0.0),      // HUMIDITY_PLAUSIBLE_MIN
        (10, 100.0),   // HUMIDITY_PLAUSIBLE_MAX
        (11, 20.0),    // HUMIDITY_NOMINAL_MIN
        (12, 80.0),    // HUMIDITY_NOMINAL_MAX
        (13, -2.3),    // PRESSURE_2_CALIBRATION_OFFSET
        (14, 0.98),    // TEMP_CALIBRATION_GAIN
        (15, 0.5),     // TEMP_CALIBRATION_BIAS
        (16, 5.0),     // ANOMALY_DISAGREEMENT_C
        (17, 5.0),     // HIGH_UNCERTAINTY_THRESHOLD
        (18, 2.0),     // DEGRADED_UNCERTAINTY_FACTOR
        (19, 10.0),    // OUT_OF_RANGE_SD
        (20, 1.5),     // DRIFT_UNCERTAINTY_FACTOR
        (21, 2.0),     // DRIFT_UNCERTAINTY_OFFSET
        (22, 22.0),    // HISTORICAL_TEMP_MEAN
        (23, 3.0),     // HISTORICAL_TEMP_SD
        (24, 20.0),    // REFERENCE_TEMP
        (25, 1013.25), // REFERENCE_PRESSURE
        (26, 0.02),    // TEMP_PER_HPA
        (27, 10.0),    // CORRELATION_TOLERANCE
        (28, 50.0),    // CRITICAL_BELOW_PCT
        (29, 70.0),    // HIGH_BELOW_PCT
        (30, 85.0),    // MEDIUM_BELOW_PCT
    ] {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
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
