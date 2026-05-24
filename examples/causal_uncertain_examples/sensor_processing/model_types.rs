/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the sensor-processing `PropagatingProcess` pipeline.

use deep_causality_core::PropagatingProcess;
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

/// Per-sensor-family bands for plausibility and anomaly checks.
#[derive(Debug, Clone, Copy)]
pub struct Bands {
    pub plausible: (f64, f64),
    pub nominal: (f64, f64),
}

/// Read-only fleet-wide config carried through the `Context` channel.
#[derive(Debug, Clone)]
pub struct FleetConfig {
    pub temp: Bands,
    pub pressure: Bands,
    pub humidity: Bands,
    pub pressure_2_calibration_offset: f64,
    pub temp_calibration_gain: f64,
    pub temp_calibration_bias: f64,
    pub anomaly_disagreement_c: f64,
    pub high_uncertainty_threshold: f64,
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
pub type FleetProcess<T> = PropagatingProcess<T, FleetState, FleetConfig>;

// ---------------------------------------------------------------------------
// Value-channel types
// ---------------------------------------------------------------------------

/// Stage 1 input: raw readings keyed by sensor id.
#[derive(Debug, Default, Clone)]
pub struct RawReadings(pub HashMap<String, SensorReading>);

/// Stage 2 output: per-sensor processed `Uncertain<f64>` or an error string.
#[derive(Debug, Default, Clone)]
pub struct ProcessedReadings(pub HashMap<String, Result<Uncertain<f64>, String>>);
