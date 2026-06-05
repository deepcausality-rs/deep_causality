/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the SCUBA decompression planner: types, Bühlmann ZH-L16C constants, and the
//! physics/physiology computations. `main.rs` drives the monadic dive simulation and presents it.
//!
//! All domain quantities are typed `FloatType`, so switching the alias actually re-runs the whole
//! model at a different precision.

use crate::FloatType;
use deep_causality_calculus::{DifferentiableArrow, Scalar};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// Constants: Bühlmann ZH-L16C Parameters
// =============================================================================
/// Tissue compartment half-times (minutes) for N2
pub const HALF_TIMES: [FloatType; 16] = [
    5.0, 8.0, 12.5, 18.5, 27.0, 38.3, 54.3, 77.0, 109.0, 146.0, 187.0, 239.0, 305.0, 390.0, 498.0,
    635.0,
];
/// M-value 'a' coefficients (bar)
const A_COEFFICIENTS: [FloatType; 16] = [
    1.1696, 1.0000, 0.8618, 0.7562, 0.6200, 0.5043, 0.4410, 0.4000, 0.3750, 0.3500, 0.3295, 0.3065,
    0.2835, 0.2610, 0.2480, 0.2327,
];
/// M-value 'b' coefficients (dimensionless)
const B_COEFFICIENTS: [FloatType; 16] = [
    0.5578, 0.6514, 0.7222, 0.7825, 0.8126, 0.8434, 0.8693, 0.8910, 0.9092, 0.9222, 0.9319, 0.9403,
    0.9477, 0.9544, 0.9602, 0.9653,
];
/// Surface nitrogen partial pressure (bar) - ~79% of 1 atm
const SURFACE_N2_PP: FloatType = 0.79;
/// Oxygen fraction in air
const F_O2: FloatType = 0.21;
/// Gradient factors (conservative recreational diving)
pub const GF_LOW: FloatType = 0.30;
pub const GF_HIGH: FloatType = 0.85;
/// Descent rate (m/min)
pub const DESCENT_RATE: FloatType = 18.0;
/// Ascent rate (m/min) - PADI standard
pub const ASCENT_RATE: FloatType = 9.0;
/// NOAA CNS oxygen toxicity limits: (ppO2 threshold, max_time_minutes)
const CNS_LIMITS: [(FloatType, FloatType); 7] = [
    (1.60, 45.0),
    (1.50, 120.0),
    (1.40, 150.0),
    (1.30, 180.0),
    (1.20, 210.0),
    (1.10, 240.0),
    (1.00, 300.0),
];

// =============================================================================
// Data Types
// =============================================================================

/// Represents a diver's physiological state
#[derive(Debug, Clone)]
pub struct DiverState {
    pub depth: FloatType,
    pub elapsed_time: FloatType,
    pub tissue_tensions: CausalTensor<FloatType>,
    pub cns_percent: FloatType,
    pub phase: String,
}

impl Default for DiverState {
    fn default() -> Self {
        Self {
            depth: 0.0,
            elapsed_time: 0.0,
            tissue_tensions: CausalTensor::new(vec![SURFACE_N2_PP; 16], vec![16]).unwrap(),
            cns_percent: 0.0,
            phase: "Surface".to_string(),
        }
    }
}

/// Dive profile result
#[derive(Debug)]
pub struct DiveProfile {
    pub cns_percent: FloatType,
    pub safety_stop: Option<(FloatType, FloatType)>, // (depth, duration)
    pub deco_stops: Vec<(FloatType, FloatType)>,     // [(depth, duration), ...]
}

// =============================================================================
// Schreiner gas-loading model (differentiable)
// =============================================================================

/// The Schreiner tissue gas-loading curve `p(t) = p_inspired + (p_initial − p_inspired)·e^{−kt}`
/// with `k = ln2 / half_time`, written once as a scalar-generic model. Its parameters are baked as
/// `f64` constants and lifted into the working scalar with `from_f64`, so the same definition
/// evaluates at `f64` (the tension) and at `Dual` (the gas-loading rate `dp/dt`, the clinical
/// driver): `model.derivative(t)` yields the rate exactly — equal to the analytic
/// `k·(p_inspired − p(t))`. `main` uses it for the autodiff rate demonstration.
pub struct SchreinerLoading {
    pub p_initial: FloatType,
    pub p_inspired: FloatType,
    pub half_time: FloatType,
}

impl DifferentiableArrow for SchreinerLoading {
    fn run<S: Scalar>(&self, t: S) -> S {
        let k =
            S::from_f64(2.0_f64.ln() / self.half_time).expect("k lifts into the working scalar");
        let p_initial =
            S::from_f64(self.p_initial).expect("constant lifts into the working scalar");
        let p_inspired =
            S::from_f64(self.p_inspired).expect("constant lifts into the working scalar");

        p_inspired + (p_initial - p_inspired) * (-(k * t)).exp()
    }
}

// =============================================================================
// Physics Functions
// =============================================================================

/// Calculates ambient pressure at depth (bar)
pub fn ambient_pressure(depth: FloatType) -> FloatType {
    1.0 + depth / 10.0
}

/// Calculates inspired nitrogen partial pressure at depth
pub fn inspired_n2_pp(depth: FloatType) -> FloatType {
    let p_amb = ambient_pressure(depth);
    let p_water_vapor = 0.0627; // bar at 37°C
    let f_n2 = 0.79;
    (p_amb - p_water_vapor) * f_n2
}

/// Calculates oxygen partial pressure at depth
pub fn oxygen_pp(depth: FloatType) -> FloatType {
    ambient_pressure(depth) * F_O2
}

/// Schreiner equation: tissue gas loading over time, evaluated at `FloatType` for the simulation.
/// This is the value path; [`SchreinerLoading`] is the differentiable counterpart used for `dp/dt`.
pub fn tissue_loading(
    p_initial: FloatType,
    p_inspired: FloatType,
    time_minutes: FloatType,
    half_time: FloatType,
) -> FloatType {
    let two: FloatType = 2.0;
    let k = two.ln() / half_time;
    p_inspired + (p_initial - p_inspired) * (-(k * time_minutes)).exp()
}

/// Calculates ascent ceiling (minimum safe depth) for a tissue
pub fn tissue_ceiling(
    tissue_tension: FloatType,
    a: FloatType,
    b: FloatType,
    gf: FloatType,
) -> FloatType {
    let m_value = tissue_tension / b + a;
    let allowed_gradient = gf * (m_value - tissue_tension / b);
    let ceiling_pressure = tissue_tension - allowed_gradient;
    ((ceiling_pressure - 1.0) * 10.0).max(0.0)
}

/// Finds maximum exposure time for a given ppO2 (linear interpolation)
pub fn max_cns_time(pp_o2: FloatType) -> FloatType {
    if pp_o2 < 1.0 {
        return FloatType::INFINITY;
    }

    for i in 0..CNS_LIMITS.len() - 1 {
        let (pp_high, time_high) = CNS_LIMITS[i];
        let (pp_low, time_low) = CNS_LIMITS[i + 1];

        if pp_o2 >= pp_low && pp_o2 <= pp_high {
            let ratio = (pp_o2 - pp_low) / (pp_high - pp_low);
            return time_low + ratio * (time_high - time_low);
        }
    }

    if pp_o2 > 1.6 {
        return 45.0 * (1.6 / pp_o2);
    }

    300.0
}

/// Calculates CNS% accumulation for time spent at depth
pub fn cns_accumulation(depth: FloatType, time_minutes: FloatType) -> FloatType {
    let pp_o2 = oxygen_pp(depth);
    let max_time = max_cns_time(pp_o2);

    if max_time.is_infinite() {
        return 0.0;
    }

    (time_minutes / max_time) * 100.0
}

/// Updates tissue tensions for all 16 compartments
pub fn update_tissues(
    tensions: &CausalTensor<FloatType>,
    depth: FloatType,
    time: FloatType,
) -> CausalTensor<FloatType> {
    let p_inspired = inspired_n2_pp(depth);
    let new_tensions: Vec<FloatType> = tensions
        .as_slice()
        .iter()
        .enumerate()
        .map(|(i, &p_initial)| tissue_loading(p_initial, p_inspired, time, HALF_TIMES[i]))
        .collect();

    CausalTensor::new(new_tensions, vec![16]).unwrap()
}

/// Finds the controlling compartment (highest ceiling)
pub fn find_ceiling(tensions: &CausalTensor<FloatType>, gf: FloatType) -> (usize, FloatType) {
    let mut max_ceiling = 0.0;
    let mut controlling = 0;

    for (i, &tension) in tensions.as_slice().iter().enumerate() {
        let ceiling = tissue_ceiling(tension, A_COEFFICIENTS[i], B_COEFFICIENTS[i], gf);
        if ceiling > max_ceiling {
            max_ceiling = ceiling;
            controlling = i;
        }
    }

    (controlling, max_ceiling)
}

/// Estimates NDL (No Decompression Limit) for a depth
pub fn estimate_ndl(depth: FloatType) -> FloatType {
    // Conservative NDL estimates based on gradient factors
    match depth as i32 {
        0..=12 => 200.0,
        13..=18 => 80.0,
        19..=24 => 45.0,
        25..=30 => 25.0,
        31..=36 => 15.0,
        37..=42 => 10.0,
        43..=48 => 8.0,
        _ => 6.0,
    }
}
