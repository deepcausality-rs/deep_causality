/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types and stage logic for the CATE chain.

use deep_causality_core::{CausalFlow, PropagatingEffect};

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this file would need wrapping
/// in `FloatType::from(…)` to switch to `Float106`.
pub type FloatType = f64;

/// A patient: covariates and a baseline blood pressure.
#[derive(Debug, Clone, Default)]
pub struct Patient {
    pub id: u32,
    pub age: FloatType,
    pub baseline_bp: FloatType,
}

/// Post-treatment blood pressure under `do(T = treatment)`.
///
/// The flow begins with a NaN treatment value; `.intervene(treatment)` fires
/// before the `map` that consumes it. Inside the `map` closure, `t` holds the
/// intervened value, and the body computes the resulting post-treatment blood
/// pressure. `into_effect` hands the underlying effect back so the caller can
/// read both its value and its audit log.
pub fn evaluate_under(patient: &Patient, treatment: FloatType) -> PropagatingEffect<FloatType> {
    let baseline = patient.baseline_bp;
    let age = patient.age;

    CausalFlow::value(FloatType::NAN)
        .intervene(treatment)
        .map(move |t| {
            // Stronger reduction for older patients. That heterogeneity is
            // what makes the CATE non-trivial. Pharmacology is deliberately
            // shallow; the example is about the causal structure.
            let drug_reduction = if t > 0.5 {
                5.0 + 0.3 * (age - 50.0).max(0.0)
            } else {
                0.0
            };

            // Natural variation independent of treatment.
            let natural_drift = 0.5;

            baseline - drug_reduction + natural_drift
        })
        .into_effect()
}

pub fn potential_outcomes(patient: &Patient) -> (FloatType, FloatType) {
    let y1 = evaluate_under(patient, 1.0)
        .value
        .into_value()
        .unwrap_or(FloatType::NAN);
    let y0 = evaluate_under(patient, 0.0)
        .value
        .into_value()
        .unwrap_or(FloatType::NAN);
    (y1, y0)
}

pub fn synthetic_cohort() -> Vec<Patient> {
    vec![
        Patient {
            id: 1,
            age: 55.0,
            baseline_bp: 140.0,
        },
        Patient {
            id: 2,
            age: 62.0,
            baseline_bp: 135.0,
        },
        Patient {
            id: 3,
            age: 68.0,
            baseline_bp: 150.0,
        },
        Patient {
            id: 4,
            age: 71.0,
            baseline_bp: 145.0,
        },
        Patient {
            id: 5,
            age: 58.0,
            baseline_bp: 138.0,
        },
        Patient {
            id: 6,
            age: 74.0,
            baseline_bp: 155.0,
        },
        Patient {
            id: 7,
            age: 80.0,
            baseline_bp: 160.0,
        },
        Patient {
            id: 8,
            age: 48.0,
            baseline_bp: 130.0,
        },
    ]
}
