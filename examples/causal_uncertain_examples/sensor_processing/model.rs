/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage functions for the sensor-processing `PropagatingProcess` chain.
//!
//! Each stage takes the previous stage's value out of `CausalEffect::Value`,
//! mutates `FleetState`, appends an `EffectLog` entry, and re-lifts the new
//! value. Stages short-circuit by returning `CausalEffect::none()` with an
//! attached `CausalityError` when an unrecoverable precondition fails.

use crate::model_types::{
    Bands, FleetConfig, FleetProcess, FleetState, ProcessedReadings, RawReadings, RiskLevel,
    SensorReading, SensorStatus,
};
use deep_causality_core::{CausalEffect, CausalityError, CausalityErrorEnum, EffectLog};
use deep_causality_haft::LogAddEntry;
use deep_causality_uncertain::Uncertain;
use std::collections::HashMap;

const SAMPLES: usize = 1000;

fn process_failure(
    state: FleetState,
    ctx: Option<FleetConfig>,
    msg: &str,
) -> FleetProcess<ProcessedReadings> {
    FleetProcess::new(
        Err(CausalityError::new(CausalityErrorEnum::Custom(msg.into()))),
        state,
        ctx,
        EffectLog::new(),
    )
}

fn band_for<'a>(sensor_id: &str, config: &'a FleetConfig) -> Option<&'a Bands> {
    if sensor_id.starts_with("temp") {
        Some(&config.temp)
    } else if sensor_id.starts_with("pressure") {
        Some(&config.pressure)
    } else if sensor_id.starts_with("humidity") {
        Some(&config.humidity)
    } else {
        None
    }
}

fn is_plausible(reading: &SensorReading, config: &FleetConfig) -> bool {
    match (reading.value, band_for(&reading.id, config)) {
        (Some(v), Some(b)) => v >= b.plausible.0 && v <= b.plausible.1,
        _ => true,
    }
}

fn apply_calibration(reading: &SensorReading, config: &FleetConfig) -> Option<f64> {
    let v = reading.value?;
    if reading.id == "pressure_2" {
        Some(v + config.pressure_2_calibration_offset)
    } else if reading.id.starts_with("temp") {
        Some(v * config.temp_calibration_gain + config.temp_calibration_bias)
    } else {
        Some(v)
    }
}

/// Stage 1 — robust per-sensor processing into `Uncertain<f64>` or an error tag.
pub fn process_stage(
    value: CausalEffect<RawReadings>,
    state: FleetState,
    ctx: Option<FleetConfig>,
) -> FleetProcess<ProcessedReadings> {
    let Some(raw) = value.into_value() else {
        return process_failure(state, ctx, "stage1.process: value was None");
    };
    let Some(config) = ctx.clone() else {
        return process_failure(state, ctx, "stage1.process: missing FleetConfig");
    };

    let mut processed: HashMap<String, Result<Uncertain<f64>, String>> = HashMap::new();
    for (id, reading) in raw.0.iter() {
        let result = match (&reading.status, reading.value, reading.uncertainty) {
            (SensorStatus::Healthy, Some(v), Some(u)) => Ok(Uncertain::normal(v, u)),
            (SensorStatus::Degraded, Some(v), Some(u)) => Ok(Uncertain::normal(v, u * 2.0)),
            (SensorStatus::OutOfRange, Some(v), _) => {
                if is_plausible(reading, &config) {
                    Ok(Uncertain::normal(v, 10.0))
                } else {
                    Err(format!("sensor {id} reading {v} is physically implausible"))
                }
            }
            (SensorStatus::CalibrationDrift, Some(_), Some(u)) => {
                match apply_calibration(reading, &config) {
                    Some(corrected) => Ok(Uncertain::normal(corrected, u * 1.5 + 2.0)),
                    None => Err(format!("sensor {id} calibration failed (no value)")),
                }
            }
            (SensorStatus::Failed | SensorStatus::CommunicationError, _, _) => {
                Err(format!("sensor {id} unavailable"))
            }
            _ => Err(format!("sensor {id} has invalid data configuration")),
        };
        processed.insert(id.clone(), result);
    }

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
    mut state: FleetState,
    ctx: Option<FleetConfig>,
) -> FleetProcess<ProcessedReadings> {
    let Some(processed) = value.into_value() else {
        return process_failure(state, ctx, "stage2.validate: value was None");
    };

    let mut high_uncertainty_sensors: Vec<String> = Vec::new();
    let high_thr = ctx
        .as_ref()
        .map(|c| c.high_uncertainty_threshold)
        .unwrap_or(5.0);

    for (id, result) in processed.0.iter() {
        match result {
            Ok(u) => {
                state.healthy_count += 1;
                let std_dev = u.standard_deviation(100).unwrap_or(0.0);
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

    FleetProcess::new(Ok(CausalEffect::value(processed)), state, ctx, logs)
}

/// Stage 3 — inverse-variance fuse the temperature sensors; write fused mean into state.
pub fn fusion_stage(
    value: CausalEffect<ProcessedReadings>,
    mut state: FleetState,
    ctx: Option<FleetConfig>,
) -> FleetProcess<ProcessedReadings> {
    let Some(processed) = value.into_value() else {
        return process_failure(state, ctx, "stage3.fusion: value was None");
    };

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
        let mean = u.expected_value(SAMPLES).unwrap_or(f64::NAN);
        state.fused_temp = Some(mean);
        logs.add_entry(&format!(
            "stage3.fusion: single sensor {id} → {mean:.1}°C (no redundancy)"
        ));
    } else {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        let mut values: Vec<f64> = Vec::new();
        for (_, u) in &temps {
            let mean = u.expected_value(SAMPLES).unwrap_or(f64::NAN);
            let std = u.standard_deviation(SAMPLES).unwrap_or(f64::NAN);
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

        let disagreement_thr = ctx
            .as_ref()
            .map(|c| c.anomaly_disagreement_c)
            .unwrap_or(5.0);
        if let (Some(&hi), Some(&lo)) = (
            values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()),
            values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()),
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

    FleetProcess::new(Ok(CausalEffect::value(processed)), state, ctx, logs)
}

/// Stage 4 — detect per-sensor anomalies against nominal bands.
pub fn anomaly_stage(
    value: CausalEffect<ProcessedReadings>,
    mut state: FleetState,
    ctx: Option<FleetConfig>,
) -> FleetProcess<ProcessedReadings> {
    let Some(processed) = value.into_value() else {
        return process_failure(state, ctx, "stage4.anomaly: value was None");
    };
    let Some(config) = ctx.clone() else {
        return process_failure(state, ctx, "stage4.anomaly: missing FleetConfig");
    };

    let mut logs = EffectLog::new();
    for (id, result) in processed.0.iter() {
        let Ok(u) = result else { continue };
        let mean = u.expected_value(SAMPLES).unwrap_or(f64::NAN);
        let Some(bands) = band_for(id, &config) else {
            continue;
        };
        if !(bands.nominal.0..=bands.nominal.1).contains(&mean) {
            let note = format!("{id} reading {mean:.1} outside nominal {:?}", bands.nominal);
            state.anomalies.push(note.clone());
            logs.add_entry(&format!("stage4.anomaly: {note}"));
        }
    }
    if state.anomalies.is_empty() {
        logs.add_entry("stage4.anomaly: no anomalies detected");
    }

    FleetProcess::new(Ok(CausalEffect::value(processed)), state, ctx, logs)
}

/// Stage 5 — cross-validate temperature against pressure (physics check).
pub fn fallback_stage(
    value: CausalEffect<ProcessedReadings>,
    mut state: FleetState,
    ctx: Option<FleetConfig>,
) -> FleetProcess<ProcessedReadings> {
    let Some(processed) = value.into_value() else {
        return process_failure(state, ctx, "stage5.fallback: value was None");
    };

    let mut logs = EffectLog::new();
    if state.healthy_count == 0 {
        logs.add_entry("stage5.fallback: no healthy sensors — falling back to historical model");
        let historical = Uncertain::normal(22.0, 3.0);
        state.fused_temp = Some(historical.expected_value(SAMPLES).unwrap_or(f64::NAN));
    }

    if let (Some(Ok(temp)), Some(Ok(pressure))) = (
        processed.0.get("temp_1").map(|r| r.as_ref()),
        processed.0.get("pressure_1").map(|r| r.as_ref()),
    ) {
        let t = temp.expected_value(SAMPLES).unwrap_or(f64::NAN);
        let p = pressure.expected_value(SAMPLES).unwrap_or(f64::NAN);
        let expected_t = 20.0 + (p - 1013.25) * 0.02;
        let diff = (t - expected_t).abs();
        if diff > 10.0 {
            let note = format!(
                "temp-pressure correlation failed: measured {t:.1}°C vs expected {expected_t:.1}°C"
            );
            state.anomalies.push(note.clone());
            logs.add_entry(&format!("stage5.fallback: {note}"));
        } else {
            logs.add_entry("stage5.fallback: temp-pressure correlation validated");
        }
    }

    FleetProcess::new(Ok(CausalEffect::value(processed)), state, ctx, logs)
}

/// Stage 6 — derive a final risk verdict from accumulated state.
pub fn reliability_stage(
    value: CausalEffect<ProcessedReadings>,
    mut state: FleetState,
    ctx: Option<FleetConfig>,
) -> FleetProcess<ProcessedReadings> {
    let Some(processed) = value.into_value() else {
        return process_failure(state, ctx, "stage6.reliability: value was None");
    };

    let total = state.healthy_count + state.degraded_count + state.failed_count;
    let health_pct = if total > 0 {
        state.healthy_count as f64 / total as f64 * 100.0
    } else {
        0.0
    };

    let verdict = if health_pct < 50.0 {
        RiskLevel::Critical
    } else if health_pct < 70.0 {
        RiskLevel::High
    } else if health_pct < 85.0 {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    };
    state.verdict = Some(verdict.clone());

    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "stage6.reliability: health={health_pct:.1}% verdict={verdict:?}"
    ));

    FleetProcess::new(Ok(CausalEffect::value(processed)), state, ctx, logs)
}
