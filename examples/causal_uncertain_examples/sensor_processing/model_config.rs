/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Nominal fleet context and sensor seed data for the sensor-processing pipeline.

use crate::model_types::{
    ANOMALY_DISAGREEMENT_C, CORRELATION_TOLERANCE, CRITICAL_BELOW_PCT, DEGRADED_UNCERTAINTY_FACTOR,
    DRIFT_UNCERTAINTY_FACTOR, DRIFT_UNCERTAINTY_OFFSET, FleetContext, HIGH_BELOW_PCT,
    HIGH_UNCERTAINTY_THRESHOLD, HISTORICAL_TEMP_MEAN, HISTORICAL_TEMP_SD, HUMIDITY_NOMINAL_MAX,
    HUMIDITY_NOMINAL_MIN, HUMIDITY_PLAUSIBLE_MAX, HUMIDITY_PLAUSIBLE_MIN, MEDIUM_BELOW_PCT,
    OUT_OF_RANGE_SD, PRESSURE_2_CALIBRATION_OFFSET, PRESSURE_NOMINAL_MAX, PRESSURE_NOMINAL_MIN,
    PRESSURE_PLAUSIBLE_MAX, PRESSURE_PLAUSIBLE_MIN, REFERENCE_PRESSURE, REFERENCE_TEMP,
    RawReadings, SensorReading, SensorStatus, TEMP_CALIBRATION_BIAS, TEMP_CALIBRATION_GAIN,
    TEMP_NOMINAL_MAX, TEMP_NOMINAL_MIN, TEMP_PER_HPA, TEMP_PLAUSIBLE_MAX, TEMP_PLAUSIBLE_MIN,
};
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data,
};
use std::collections::HashMap;

/// The nominal fleet: one `Data` contextoid per quantity, keyed by its contextoid id.
pub fn nominal_fleet_context() -> Result<FleetContext, ContextIndexError> {
    let facts = [
        (TEMP_PLAUSIBLE_MIN, -50.0),
        (TEMP_PLAUSIBLE_MAX, 100.0),
        (TEMP_NOMINAL_MIN, 15.0),
        (TEMP_NOMINAL_MAX, 35.0),
        (PRESSURE_PLAUSIBLE_MIN, 800.0),
        (PRESSURE_PLAUSIBLE_MAX, 1200.0),
        (PRESSURE_NOMINAL_MIN, 980.0),
        (PRESSURE_NOMINAL_MAX, 1050.0),
        (HUMIDITY_PLAUSIBLE_MIN, 0.0),
        (HUMIDITY_PLAUSIBLE_MAX, 100.0),
        (HUMIDITY_NOMINAL_MIN, 20.0),
        (HUMIDITY_NOMINAL_MAX, 80.0),
        (PRESSURE_2_CALIBRATION_OFFSET, -2.3),
        (TEMP_CALIBRATION_GAIN, 0.98),
        (TEMP_CALIBRATION_BIAS, 0.5),
        (ANOMALY_DISAGREEMENT_C, 5.0),
        (HIGH_UNCERTAINTY_THRESHOLD, 5.0),
        (DEGRADED_UNCERTAINTY_FACTOR, 2.0),
        (OUT_OF_RANGE_SD, 10.0),
        (DRIFT_UNCERTAINTY_FACTOR, 1.5),
        (DRIFT_UNCERTAINTY_OFFSET, 2.0),
        (HISTORICAL_TEMP_MEAN, 22.0),
        (HISTORICAL_TEMP_SD, 3.0),
        (REFERENCE_TEMP, 20.0),
        (REFERENCE_PRESSURE, 1013.25),
        (TEMP_PER_HPA, 0.02),
        (CORRELATION_TOLERANCE, 10.0),
        (CRITICAL_BELOW_PCT, 50.0),
        (HIGH_BELOW_PCT, 70.0),
        (MEDIUM_BELOW_PCT, 85.0),
    ];
    let mut context = Context::with_capacity(1, "fleet", facts.len());
    for (id, value) in facts {
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
