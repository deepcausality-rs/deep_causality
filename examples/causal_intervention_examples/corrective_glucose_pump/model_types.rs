/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the closed-loop insulin-pump example.

#![allow(dead_code)] // Domain fields kept for narrative clarity even if not all are read.

use deep_causality_core::PropagatingProcess;

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this crate would need wrapping
/// in `FloatType::from(…)` to switch away from `f64`.
pub type FloatType = f64;

/// One tick is 15 minutes. The 24-tick run covers six hours of monitoring,
/// long enough for both the hyperglycemic and the ketoacidotic threshold to
/// be crossed in the open-loop trajectory.
pub const N_TICKS: u32 = 24;

/// Deterministic perturbation schedule. Positive values raise blood
/// glucose. Three meal bumps spaced every six ticks (roughly 90 min) plus
/// a slow baseline drift mimic a fasting baseline punctuated by snacks.
pub fn perturbation_at(tick: u32) -> FloatType {
    let baseline = 6.0; // mg/dL per 15-min tick of hepatic glucose production
    let meal = match tick {
        2 => 70.0,  // small meal
        8 => 90.0,  // larger meal
        14 => 60.0, // snack
        _ => 0.0,
    };
    baseline + meal
}

/// Accumulated trajectory and clinical statistics.
#[derive(Debug, Default, Clone)]
pub struct PatientState {
    pub tick: u32,
    pub trajectory: Vec<FloatType>,
    pub bolus_count: u32,
    pub total_insulin_units: FloatType,
    pub max_glucose_observed: FloatType,
    pub ketoacidosis_at: Option<u32>,
}

/// Read-only thresholds, calibration, and patient parameters.
#[derive(Debug, Clone)]
pub struct PumpConfig {
    /// Normal fasting glucose target (mg/dL).
    pub target_glucose: FloatType,
    /// Hyperglycemic alarm. The monitor fires a corrective bolus when
    /// glucose climbs above this level.
    pub hyperglycemic_threshold: FloatType,
    /// Catastrophic threshold. Crossing this enters diabetic ketoacidosis
    /// territory and is recorded against the trajectory.
    pub ketoacidosis_threshold: FloatType,
    /// Insulin sensitivity factor: mg/dL of glucose reduction per unit of
    /// fast-acting insulin.
    pub units_per_mg_dl: FloatType,
}

pub fn nominal_pump_config() -> PumpConfig {
    PumpConfig {
        target_glucose: 100.0,
        hyperglycemic_threshold: 180.0,
        ketoacidosis_threshold: 300.0,
        units_per_mg_dl: 50.0,
    }
}

pub type PumpProcess<T> = PropagatingProcess<T, PatientState, PumpConfig>;
