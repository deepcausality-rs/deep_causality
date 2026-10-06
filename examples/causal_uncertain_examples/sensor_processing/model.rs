/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage functions for the sensor-processing `PropagatingProcess` chain.
//!
//! Each stage takes the previous stage's value out of `CausalEffect::Value`,
//! reads the fleet context, mutates `FleetState`, appends an `EffectLog` entry,
//! and re-lifts the new value. A stage short-circuits into the error channel
//! when the value or the fleet context is missing, when a fleet-context read
//! fails, or when sampling an `Uncertain<f64>` fails.

use crate::model_types::{
    ANOMALY_DISAGREEMENT_C, BandNodes, CORRELATION_TOLERANCE, CRITICAL_BELOW_PCT,
    DEGRADED_UNCERTAINTY_FACTOR, DRIFT_UNCERTAINTY_FACTOR, DRIFT_UNCERTAINTY_OFFSET, FleetContext,
    FleetProcess, FleetState, HIGH_BELOW_PCT, HIGH_UNCERTAINTY_THRESHOLD, HISTORICAL_TEMP_MEAN,
    HISTORICAL_TEMP_SD, HUMIDITY_BANDS, MEDIUM_BELOW_PCT, OUT_OF_RANGE_SD,
    PRESSURE_2_CALIBRATION_OFFSET, PRESSURE_BANDS, ProcessedReadings, REFERENCE_PRESSURE,
    REFERENCE_TEMP, RawReadings, RiskLevel, SensorReading, SensorStatus, TEMP_BANDS,
    TEMP_CALIBRATION_BIAS, TEMP_CALIBRATION_GAIN, TEMP_PER_HPA, read,
};
use deep_causality_core::{CausalEffect, CausalityError, CausalityErrorEnum, EffectLog};
use deep_causality_haft::LogAddEntry;
use deep_causality_uncertain::{Uncertain, UncertainError};
use std::collections::HashMap;

const SAMPLES: usize = 1000;

fn process_error(
    state: FleetState,
    ctx: Option<FleetContext>,
    error: CausalityError,
) -> FleetProcess<ProcessedReadings> {
    FleetProcess::new(Err(error), state, ctx, EffectLog::new())
}

fn process_failure(
    state: FleetState,
    ctx: Option<FleetContext>,
    msg: &str,
) -> FleetProcess<ProcessedReadings> {
    process_error(
        state,
        ctx,
        CausalityError::new(CausalityErrorEnum::Custom(msg.into())),
    )
}

/// A sampling error, carried in the chain's error channel.
fn sampling_error(error: UncertainError) -> CausalityError {
    CausalityError::UncertainError(error.to_string())
}

/// Run the body of a stage that passes the processed readings through. The body reads the fleet
/// context, updates the state and returns its log entries; its error lands in the error channel
/// with the state and context kept.
fn run_stage<F>(
    stage: &str,
    value: CausalEffect<ProcessedReadings>,
    mut state: FleetState,
    ctx: Option<FleetContext>,
    body: F,
) -> FleetProcess<ProcessedReadings>
where
    F: FnOnce(
        &ProcessedReadings,
        &mut FleetState,
        &FleetContext,
    ) -> Result<EffectLog, CausalityError>,
{
    let Some(processed) = value.into_value() else {
        return process_failure(state, ctx, &format!("{stage}: value was None"));
    };
    let outcome = match ctx.as_ref() {
        Some(fleet) => body(&processed, &mut state, fleet),
        None => Err(CausalityError::MissingContext()),
    };
    match outcome {
        Ok(logs) => FleetProcess::new(Ok(CausalEffect::value(processed)), state, ctx, logs),
        Err(error) => process_error(state, ctx, error),
    }
}

/// The fleet-context nodes holding the bands of the sensor family `sensor_id` belongs to.
fn band_for(sensor_id: &str) -> Option<BandNodes> {
    if sensor_id.starts_with("temp") {
        Some(TEMP_BANDS)
    } else if sensor_id.starts_with("pressure") {
        Some(PRESSURE_BANDS)
    } else if sensor_id.starts_with("humidity") {
        Some(HUMIDITY_BANDS)
    } else {
        None
    }
}

fn is_plausible(reading: &SensorReading, fleet: &FleetContext) -> Result<bool, CausalityError> {
    match (reading.value, band_for(&reading.id)) {
        (Some(v), Some(b)) => {
            Ok(v >= read(fleet, b.plausible_min)? && v <= read(fleet, b.plausible_max)?)
        }
        _ => Ok(true),
    }
}

fn apply_calibration(
    reading: &SensorReading,
    fleet: &FleetContext,
) -> Result<Option<f64>, CausalityError> {
    let Some(v) = reading.value else {
        return Ok(None);
    };
    if reading.id == "pressure_2" {
        Ok(Some(v + read(fleet, PRESSURE_2_CALIBRATION_OFFSET)?))
    } else if reading.id.starts_with("temp") {
        Ok(Some(
            v * read(fleet, TEMP_CALIBRATION_GAIN)? + read(fleet, TEMP_CALIBRATION_BIAS)?,
        ))
    } else {
        Ok(Some(v))
    }
}

/// Triage one reading into an `Uncertain<f64>` or a per-sensor error tag. The outer error is a
/// failed fleet-context read, which stops the stage.
fn triage(
    id: &str,
    reading: &SensorReading,
    fleet: &FleetContext,
) -> Result<Result<Uncertain<f64>, String>, CausalityError> {
    Ok(
        match (&reading.status, reading.value, reading.uncertainty) {
            (SensorStatus::Healthy, Some(v), Some(u)) => Ok(Uncertain::normal(v, u)),
            (SensorStatus::Degraded, Some(v), Some(u)) => Ok(Uncertain::normal(
                v,
                u * read(fleet, DEGRADED_UNCERTAINTY_FACTOR)?,
            )),
            (SensorStatus::OutOfRange, Some(v), _) => {
                if is_plausible(reading, fleet)? {
                    Ok(Uncertain::normal(v, read(fleet, OUT_OF_RANGE_SD)?))
                } else {
                    Err(format!("sensor {id} reading {v} is physically implausible"))
                }
            }
            (SensorStatus::CalibrationDrift, Some(_), Some(u)) => {
                match apply_calibration(reading, fleet)? {
                    Some(corrected) => Ok(Uncertain::normal(
                        corrected,
                        u * read(fleet, DRIFT_UNCERTAINTY_FACTOR)?
                            + read(fleet, DRIFT_UNCERTAINTY_OFFSET)?,
                    )),
                    None => Err(format!("sensor {id} calibration failed (no value)")),
                }
            }
            (SensorStatus::Failed | SensorStatus::CommunicationError, _, _) => {
                Err(format!("sensor {id} unavailable"))
            }
            _ => Err(format!("sensor {id} has invalid data configuration")),
        },
    )
}

/// Stage 1 — robust per-sensor processing into `Uncertain<f64>` or an error tag.
pub fn process_stage(
    value: CausalEffect<RawReadings>,
    state: FleetState,
    ctx: Option<FleetContext>,
) -> FleetProcess<ProcessedReadings> {
    let Some(raw) = value.into_value() else {
        return process_failure(state, ctx, "stage1.process: value was None");
    };
    let processed: Result<HashMap<String, Result<Uncertain<f64>, String>>, CausalityError> =
        match ctx.as_ref() {
            Some(fleet) => raw
                .0
                .iter()
                .map(|(id, reading)| Ok((id.clone(), triage(id, reading, fleet)?)))
                .collect(),
            None => Err(CausalityError::MissingContext()),
        };
    let processed = match processed {
        Ok(processed) => processed,
        Err(error) => return process_error(state, ctx, error),
    };

    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "stage1.process: {} sensors triaged",
        processed.len()
    ));

    FleetProcess::new(
        Ok(CausalEffect::value(ProcessedReadings(processed))),
        state,
        ctx,
        logs,
    )
}

/// Stage 2 — accumulate per-sensor health counts and total uncertainty into state.
pub fn validate_stage(
    value: CausalEffect<ProcessedReadings>,
    state: FleetState,
    ctx: Option<FleetContext>,
) -> FleetProcess<ProcessedReadings> {
    run_stage("stage2.validate", value, state, ctx, validate)
}

fn validate(
    processed: &ProcessedReadings,
    state: &mut FleetState,
    fleet: &FleetContext,
) -> Result<EffectLog, CausalityError> {
    let mut high_uncertainty_sensors: Vec<String> = Vec::new();
    let high_thr = read(fleet, HIGH_UNCERTAINTY_THRESHOLD)?;

    for (id, result) in processed.0.iter() {
        match result {
            Ok(u) => {
                state.healthy_count += 1;
                let std_dev = u
                    .standard_deviation_from_entropy(100)
                    .map_err(sampling_error)?;
                state.total_uncertainty += std_dev;
                if std_dev > high_thr {
                    high_uncertainty_sensors.push(id.clone());
                }
            }
            Err(_) => state.failed_count += 1,
        }
    }

    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "stage2.validate: healthy={} failed={} mean_uncertainty={:.2}",
        state.healthy_count,
        state.failed_count,
        if state.healthy_count > 0 {
            state.total_uncertainty / state.healthy_count as f64
        } else {
            0.0
        }
    ));
    if !high_uncertainty_sensors.is_empty() {
        logs.add_entry(&format!(
            "stage2.validate: high-uncertainty sensors: {high_uncertainty_sensors:?}"
        ));
    }

    Ok(logs)
}

/// Stage 3 — inverse-variance fuse the temperature sensors; write fused mean into state.
pub fn fusion_stage(
    value: CausalEffect<ProcessedReadings>,
    state: FleetState,
    ctx: Option<FleetContext>,
) -> FleetProcess<ProcessedReadings> {
    run_stage("stage3.fusion", value, state, ctx, fuse)
}

fn fuse(
    processed: &ProcessedReadings,
    state: &mut FleetState,
    fleet: &FleetContext,
) -> Result<EffectLog, CausalityError> {
    let temps: Vec<(&String, &Uncertain<f64>)> = processed
        .0
        .iter()
        .filter(|(id, _)| id.starts_with("temp"))
        .filter_map(|(id, r)| r.as_ref().ok().map(|u| (id, u)))
        .collect();

    let mut logs = EffectLog::new();
    if temps.is_empty() {
        logs.add_entry("stage3.fusion: no healthy temperature sensors");
    } else if temps.len() == 1 {
        let (id, u) = temps[0];
        let mean = u
            .expected_value_from_entropy(SAMPLES)
            .map_err(sampling_error)?;
        state.fused_temp = Some(mean);
        logs.add_entry(&format!(
            "stage3.fusion: single sensor {id} → {mean:.1}°C (no redundancy)"
        ));
    } else {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        let mut values: Vec<f64> = Vec::new();
        for (_, u) in &temps {
            let mean = u
                .expected_value_from_entropy(SAMPLES)
                .map_err(sampling_error)?;
            let std = u
                .standard_deviation_from_entropy(SAMPLES)
                .map_err(sampling_error)?;
            let weight = 1.0 / (std + 0.1);
            weighted_sum += mean * weight;
            total_weight += weight;
            values.push(mean);
        }
        let fused = weighted_sum / total_weight;
        state.fused_temp = Some(fused);
        logs.add_entry(&format!(
            "stage3.fusion: fused {} temp sensors → {fused:.1}°C (1σ ≈ {:.1})",
            temps.len(),
            1.0 / total_weight.sqrt()
        ));

        let disagreement_thr = read(fleet, ANOMALY_DISAGREEMENT_C)?;
        if let (Some(&hi), Some(&lo)) = (
            values.iter().max_by(|a, b| a.total_cmp(b)),
            values.iter().min_by(|a, b| a.total_cmp(b)),
        ) && hi - lo > disagreement_thr
        {
            let spread = hi - lo;
            state.anomalies.push(format!(
                "temperature disagreement {spread:.1}°C exceeds {disagreement_thr}°C"
            ));
            logs.add_entry(&format!(
                "stage3.fusion: large sensor disagreement {spread:.1}°C — possible failure"
            ));
        }
    }

    Ok(logs)
}

/// Stage 4 — detect per-sensor anomalies against nominal bands.
pub fn anomaly_stage(
    value: CausalEffect<ProcessedReadings>,
    state: FleetState,
    ctx: Option<FleetContext>,
) -> FleetProcess<ProcessedReadings> {
    run_stage("stage4.anomaly", value, state, ctx, detect_anomalies)
}

fn detect_anomalies(
    processed: &ProcessedReadings,
    state: &mut FleetState,
    fleet: &FleetContext,
) -> Result<EffectLog, CausalityError> {
    let mut logs = EffectLog::new();
    for (id, result) in processed.0.iter() {
        let Ok(u) = result else { continue };
        let mean = u
            .expected_value_from_entropy(SAMPLES)
            .map_err(sampling_error)?;
        let Some(bands) = band_for(id) else {
            continue;
        };
        let nominal = (
            read(fleet, bands.nominal_min)?,
            read(fleet, bands.nominal_max)?,
        );
        if !(nominal.0..=nominal.1).contains(&mean) {
            let note = format!("{id} reading {mean:.1} outside nominal {nominal:?}");
            state.anomalies.push(note.clone());
            logs.add_entry(&format!("stage4.anomaly: {note}"));
        }
    }
    if state.anomalies.is_empty() {
        logs.add_entry("stage4.anomaly: no anomalies detected");
    }

    Ok(logs)
}

/// Stage 5 — cross-validate temperature against pressure (physics check).
pub fn fallback_stage(
    value: CausalEffect<ProcessedReadings>,
    state: FleetState,
    ctx: Option<FleetContext>,
) -> FleetProcess<ProcessedReadings> {
    run_stage("stage5.fallback", value, state, ctx, cross_check)
}

fn cross_check(
    processed: &ProcessedReadings,
    state: &mut FleetState,
    fleet: &FleetContext,
) -> Result<EffectLog, CausalityError> {
    let mut logs = EffectLog::new();
    if state.healthy_count == 0 {
        logs.add_entry("stage5.fallback: no healthy sensors — falling back to historical model");
        let historical = Uncertain::normal(
            read(fleet, HISTORICAL_TEMP_MEAN)?,
            read(fleet, HISTORICAL_TEMP_SD)?,
        );
        state.fused_temp = Some(
            historical
                .expected_value_from_entropy(SAMPLES)
                .map_err(sampling_error)?,
        );
    }

    if let (Some(Ok(temp)), Some(Ok(pressure))) = (
        processed.0.get("temp_1").map(|r| r.as_ref()),
        processed.0.get("pressure_1").map(|r| r.as_ref()),
    ) {
        let t = temp
            .expected_value_from_entropy(SAMPLES)
            .map_err(sampling_error)?;
        let p = pressure
            .expected_value_from_entropy(SAMPLES)
            .map_err(sampling_error)?;
        let expected_t = read(fleet, REFERENCE_TEMP)?
            + (p - read(fleet, REFERENCE_PRESSURE)?) * read(fleet, TEMP_PER_HPA)?;
        let diff = (t - expected_t).abs();
        if diff > read(fleet, CORRELATION_TOLERANCE)? {
            let note = format!(
                "temp-pressure correlation failed: measured {t:.1}°C vs expected {expected_t:.1}°C"
            );
            state.anomalies.push(note.clone());
            logs.add_entry(&format!("stage5.fallback: {note}"));
        } else {
            logs.add_entry("stage5.fallback: temp-pressure correlation validated");
        }
    }

    Ok(logs)
}

/// Stage 6 — derive a final risk verdict from accumulated state.
pub fn reliability_stage(
    value: CausalEffect<ProcessedReadings>,
    state: FleetState,
    ctx: Option<FleetContext>,
) -> FleetProcess<ProcessedReadings> {
    run_stage("stage6.reliability", value, state, ctx, judge_reliability)
}

fn judge_reliability(
    _processed: &ProcessedReadings,
    state: &mut FleetState,
    fleet: &FleetContext,
) -> Result<EffectLog, CausalityError> {
    let total = state.healthy_count + state.degraded_count + state.failed_count;
    let health_pct = if total > 0 {
        state.healthy_count as f64 / total as f64 * 100.0
    } else {
        0.0
    };

    let verdict = if health_pct < read(fleet, CRITICAL_BELOW_PCT)? {
        RiskLevel::Critical
    } else if health_pct < read(fleet, HIGH_BELOW_PCT)? {
        RiskLevel::High
    } else if health_pct < read(fleet, MEDIUM_BELOW_PCT)? {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    };
    state.verdict = Some(verdict.clone());

    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "stage6.reliability: health={health_pct:.1}% verdict={verdict:?}"
    ));

    Ok(logs)
}
