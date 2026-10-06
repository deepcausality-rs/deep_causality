/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the sensor-processing `PropagatingProcess` pipeline.

use deep_causality_context::{Context, ContextoidId, Data, NoSpace, NoSpaceTime, NoTime};
use deep_causality_core::{CausalityError, PropagatingProcess};
use deep_causality_uncertain::Uncertain;
use std::collections::HashMap;

/// Lifecycle status of a single physical sensor.
#[derive(Debug, Clone, PartialEq)]
pub enum SensorStatus {
    Healthy,
    Degraded,
    Failed,
    OutOfRange,
    CalibrationDrift,
    CommunicationError,
}

/// Raw reading lifted from the wire — value and uncertainty may be missing.
#[derive(Debug, Clone)]
pub struct SensorReading {
    pub id: String,
    pub value: Option<f64>,
    #[allow(dead_code)] // part of the sensor record contract; not consumed in this demo
    pub timestamp: u64,
    pub status: SensorStatus,
    pub uncertainty: Option<f64>,
}

/// The fleet facts the stages read, one `Data` contextoid per quantity, carried through the
/// `Context` channel. The context holds no position, clock or event, so its spatial, temporal and
/// spacetime slots are empty.
pub type FleetContext = Context<Data<f64>, NoSpace<f64>, NoTime, NoSpaceTime<f64>>;

/// Contextoid id: lowest physically plausible temperature in °C.
pub const TEMP_PLAUSIBLE_MIN: ContextoidId = 1;
/// Contextoid id: highest physically plausible temperature in °C.
pub const TEMP_PLAUSIBLE_MAX: ContextoidId = 2;
/// Contextoid id: lower bound of the nominal temperature band in °C.
pub const TEMP_NOMINAL_MIN: ContextoidId = 3;
/// Contextoid id: upper bound of the nominal temperature band in °C.
pub const TEMP_NOMINAL_MAX: ContextoidId = 4;
/// Contextoid id: lowest physically plausible pressure in hPa.
pub const PRESSURE_PLAUSIBLE_MIN: ContextoidId = 5;
/// Contextoid id: highest physically plausible pressure in hPa.
pub const PRESSURE_PLAUSIBLE_MAX: ContextoidId = 6;
/// Contextoid id: lower bound of the nominal pressure band in hPa.
pub const PRESSURE_NOMINAL_MIN: ContextoidId = 7;
/// Contextoid id: upper bound of the nominal pressure band in hPa.
pub const PRESSURE_NOMINAL_MAX: ContextoidId = 8;
/// Contextoid id: lowest physically plausible relative humidity in %.
pub const HUMIDITY_PLAUSIBLE_MIN: ContextoidId = 9;
/// Contextoid id: highest physically plausible relative humidity in %.
pub const HUMIDITY_PLAUSIBLE_MAX: ContextoidId = 10;
/// Contextoid id: lower bound of the nominal relative-humidity band in %.
pub const HUMIDITY_NOMINAL_MIN: ContextoidId = 11;
/// Contextoid id: upper bound of the nominal relative-humidity band in %.
pub const HUMIDITY_NOMINAL_MAX: ContextoidId = 12;
/// Contextoid id: calibration offset added to the `pressure_2` reading, in hPa.
pub const PRESSURE_2_CALIBRATION_OFFSET: ContextoidId = 13;
/// Contextoid id: calibration gain applied to temperature readings, dimensionless.
pub const TEMP_CALIBRATION_GAIN: ContextoidId = 14;
/// Contextoid id: calibration bias added to temperature readings after the gain, in °C.
pub const TEMP_CALIBRATION_BIAS: ContextoidId = 15;
/// Contextoid id: temperature spread between fused sensors that raises an anomaly, in °C.
pub const ANOMALY_DISAGREEMENT_C: ContextoidId = 16;
/// Contextoid id: standard deviation above which a sensor counts as high-uncertainty, in the
/// sensor's own unit.
pub const HIGH_UNCERTAINTY_THRESHOLD: ContextoidId = 17;
/// Contextoid id: factor on a degraded sensor's rated uncertainty.
pub const DEGRADED_UNCERTAINTY_FACTOR: ContextoidId = 18;
/// Contextoid id: standard deviation given to a plausible out-of-range reading, in the sensor's
/// own unit.
pub const OUT_OF_RANGE_SD: ContextoidId = 19;
/// Contextoid id: factor on a drifting sensor's rated uncertainty.
pub const DRIFT_UNCERTAINTY_FACTOR: ContextoidId = 20;
/// Contextoid id: uncertainty added to a drifting sensor after the factor, in the sensor's own
/// unit.
pub const DRIFT_UNCERTAINTY_OFFSET: ContextoidId = 21;
/// Contextoid id: mean of the historical temperature model, in °C.
pub const HISTORICAL_TEMP_MEAN: ContextoidId = 22;
/// Contextoid id: standard deviation of the historical temperature model, in °C.
pub const HISTORICAL_TEMP_SD: ContextoidId = 23;
/// Contextoid id: temperature the site correlation expects at `REFERENCE_PRESSURE`, in °C.
pub const REFERENCE_TEMP: ContextoidId = 24;
/// Contextoid id: pressure at which the site correlation expects `REFERENCE_TEMP`, in hPa.
pub const REFERENCE_PRESSURE: ContextoidId = 25;
/// Contextoid id: slope of the site's temperature–pressure correlation, in °C per hPa.
pub const TEMP_PER_HPA: ContextoidId = 26;
/// Contextoid id: largest gap between measured and expected temperature the correlation accepts,
/// in °C.
pub const CORRELATION_TOLERANCE: ContextoidId = 27;
/// Contextoid id: fleet health below which the verdict is `Critical`, in %.
pub const CRITICAL_BELOW_PCT: ContextoidId = 28;
/// Contextoid id: fleet health below which the verdict is `High`, in %.
pub const HIGH_BELOW_PCT: ContextoidId = 29;
/// Contextoid id: fleet health below which the verdict is `Medium`, in %.
pub const MEDIUM_BELOW_PCT: ContextoidId = 30;

/// Contextoid ids of one sensor family's plausibility and nominal bands in the fleet context.
#[derive(Debug, Clone, Copy)]
pub struct BandNodes {
    pub plausible_min: ContextoidId,
    pub plausible_max: ContextoidId,
    pub nominal_min: ContextoidId,
    pub nominal_max: ContextoidId,
}

pub const TEMP_BANDS: BandNodes = BandNodes {
    plausible_min: TEMP_PLAUSIBLE_MIN,
    plausible_max: TEMP_PLAUSIBLE_MAX,
    nominal_min: TEMP_NOMINAL_MIN,
    nominal_max: TEMP_NOMINAL_MAX,
};

pub const PRESSURE_BANDS: BandNodes = BandNodes {
    plausible_min: PRESSURE_PLAUSIBLE_MIN,
    plausible_max: PRESSURE_PLAUSIBLE_MAX,
    nominal_min: PRESSURE_NOMINAL_MIN,
    nominal_max: PRESSURE_NOMINAL_MAX,
};

pub const HUMIDITY_BANDS: BandNodes = BandNodes {
    plausible_min: HUMIDITY_PLAUSIBLE_MIN,
    plausible_max: HUMIDITY_PLAUSIBLE_MAX,
    nominal_min: HUMIDITY_NOMINAL_MIN,
    nominal_max: HUMIDITY_NOMINAL_MAX,
};

/// Read the payload of the `Data` contextoid with contextoid id `id` out of the fleet context.
pub fn read(context: &FleetContext, id: ContextoidId) -> Result<f64, CausalityError> {
    context.get_data_by_id(id).ok_or_else(|| {
        CausalityError::MissingParameter(format!(
            "the fleet context holds no Datoid with contextoid id {id}"
        ))
    })
}

/// Per-fleet outcome carried in the `State` channel and accumulated across stages.
#[derive(Debug, Default, Clone)]
pub struct FleetState {
    pub healthy_count: usize,
    pub degraded_count: usize,
    pub failed_count: usize,
    pub total_uncertainty: f64,
    pub fused_temp: Option<f64>,
    pub anomalies: Vec<String>,
    pub verdict: Option<RiskLevel>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Process alias for this example (mirrors the avionics pattern).
pub type FleetProcess<T> = PropagatingProcess<T, FleetState, FleetContext>;

// ---------------------------------------------------------------------------
// Value-channel types
// ---------------------------------------------------------------------------

/// Stage 1 input: raw readings keyed by sensor id.
#[derive(Debug, Default, Clone)]
pub struct RawReadings(pub HashMap<String, SensorReading>);

/// Stage 2 output: per-sensor processed `Uncertain<f64>` or an error string.
#[derive(Debug, Default, Clone)]
pub struct ProcessedReadings(pub HashMap<String, Result<Uncertain<f64>, String>>);
