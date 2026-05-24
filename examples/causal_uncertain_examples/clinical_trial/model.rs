/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage functions for the clinical-trial `PropagatingEffect` chain.

use deep_causality_core::{EffectValue, PropagatingEffect};
use deep_causality_uncertain::{MaybeUncertain, Uncertain};

const SAMPLES: usize = 1000;
const LIFT_CONFIDENCE: f64 = 0.95;
const LIFT_EPSILON: f64 = 0.05;

/// Per-arm cohort of patient-level pain reduction observations.
#[derive(Debug, Default, Clone)]
pub struct TrialCohort {
    pub aspirin: Vec<Patient>,
    pub control: Vec<Patient>,
}

#[derive(Debug, Clone)]
pub struct Patient {
    pub id: &'static str,
    /// Minimum data-presence probability the lift stage requires for this patient.
    pub min_presence: f64,
    pub reduction: MaybeUncertain<f64>,
}

/// Per-arm aggregate carried into the verdict stage.
#[derive(Debug, Default, Clone)]
pub struct LiftedCohort {
    pub aspirin_arm: Vec<Uncertain<f64>>,
    pub control_arm: Vec<Uncertain<f64>>,
}

#[derive(Debug, Default, Clone)]
pub struct ArmAverages {
    pub aspirin_aggregate: Option<Uncertain<f64>>,
    pub control_mean: Option<f64>,
}

/// Stage 1 — assemble per-patient `MaybeUncertain` values.
pub fn cohort_stage() -> PropagatingEffect<TrialCohort> {
    let cohort = TrialCohort {
        aspirin: vec![
            Patient {
                id: "B (strong responder)",
                min_presence: 0.9,
                reduction: MaybeUncertain::<f64>::from_uncertain(Uncertain::normal(6.0, 2.0)),
            },
            Patient {
                id: "C (weak / intermittent)",
                min_presence: 0.8,
                reduction: MaybeUncertain::<f64>::from_bernoulli_and_uncertain(
                    0.3,
                    Uncertain::normal(1.0, 0.5),
                ),
            },
            Patient {
                id: "D (moderate / reliable)",
                min_presence: 0.7,
                reduction: MaybeUncertain::<f64>::from_bernoulli_and_uncertain(
                    0.8,
                    Uncertain::normal(4.0, 2.5),
                ),
            },
        ],
        control: vec![
            Patient {
                id: "A (placebo, certain)",
                min_presence: 0.9,
                reduction: MaybeUncertain::<f64>::from_value(0.5),
            },
            Patient {
                id: "E (missing)",
                min_presence: 0.1,
                reduction: MaybeUncertain::<f64>::always_none(),
            },
        ],
    };

    println!(
        "[Stage 1] Cohort assembled: {} aspirin, {} control patients",
        cohort.aspirin.len(),
        cohort.control.len()
    );
    PropagatingEffect::pure(cohort)
}

/// Stage 2 — print per-patient data-presence probabilities (`is_some` over `MaybeUncertain`).
pub fn presence_stage(value: EffectValue<TrialCohort>) -> PropagatingEffect<TrialCohort> {
    let Some(cohort) = value.into_value() else {
        return PropagatingEffect::none();
    };

    println!("\n[Stage 2] Data-presence probabilities");
    for arm_name in ["Aspirin", "Control"] {
        let arm = if arm_name == "Aspirin" {
            &cohort.aspirin
        } else {
            &cohort.control
        };
        for p in arm {
            let p_present = p
                .reduction
                .is_some()
                .estimate_probability(SAMPLES)
                .unwrap_or(f64::NAN)
                * 100.0;
            println!(
                "   {arm_name:>7} | {:<25} P(present) = {p_present:.1}%",
                p.id
            );
        }
    }

    PropagatingEffect::pure(cohort)
}

/// Stage 3 — apply `lift_to_uncertain` per patient; drop those that fail the presence gate.
pub fn lift_stage(value: EffectValue<TrialCohort>) -> PropagatingEffect<LiftedCohort> {
    let Some(cohort) = value.into_value() else {
        return PropagatingEffect::none();
    };

    fn lift_arm(arm: &[Patient], label: &str) -> Vec<Uncertain<f64>> {
        let mut lifted: Vec<Uncertain<f64>> = Vec::new();
        for p in arm {
            match p.reduction.clone().lift_to_uncertain(
                p.min_presence,
                LIFT_CONFIDENCE,
                LIFT_EPSILON,
                SAMPLES,
            ) {
                Ok(u) => {
                    println!("   {label:>7} | {:<25} lifted ✓", p.id);
                    lifted.push(u);
                }
                Err(e) => {
                    println!("   {label:>7} | {:<25} skipped ✗ ({e})", p.id);
                }
            }
        }
        lifted
    }

    println!("\n[Stage 3] Lift `MaybeUncertain` → `Uncertain` per patient");
    let aspirin_arm = lift_arm(&cohort.aspirin, "Aspirin");
    let control_arm = lift_arm(&cohort.control, "Control");

    if aspirin_arm.is_empty() && control_arm.is_empty() {
        return PropagatingEffect::none();
    }

    PropagatingEffect::pure(LiftedCohort {
        aspirin_arm,
        control_arm,
    })
}

/// Stage 4 — average within each arm.
pub fn aggregate_stage(value: EffectValue<LiftedCohort>) -> PropagatingEffect<ArmAverages> {
    let Some(lifted) = value.into_value() else {
        return PropagatingEffect::none();
    };

    let aspirin_aggregate = average_arm(&lifted.aspirin_arm);
    let control_aggregate = average_arm(&lifted.control_arm);

    let aspirin_mean = aspirin_aggregate
        .as_ref()
        .map(|u| u.expected_value(SAMPLES).unwrap_or(f64::NAN));
    let control_mean = control_aggregate
        .as_ref()
        .map(|u| u.expected_value(SAMPLES).unwrap_or(f64::NAN));

    println!("\n[Stage 4] Per-arm averages");
    println!(
        "   Aspirin mean reduction: {}",
        aspirin_mean
            .map(|m| format!("{m:.2}"))
            .unwrap_or_else(|| "n/a".into())
    );
    println!(
        "   Control mean reduction: {}",
        control_mean
            .map(|m| format!("{m:.2}"))
            .unwrap_or_else(|| "n/a".into())
    );

    PropagatingEffect::pure(ArmAverages {
        aspirin_aggregate,
        control_mean,
    })
}

fn average_arm(arm: &[Uncertain<f64>]) -> Option<Uncertain<f64>> {
    if arm.is_empty() {
        return None;
    }
    let n = arm.len() as f64;
    let sum = arm.iter().cloned().reduce(|a, b| a + b)?;
    Some(sum / Uncertain::<f64>::point(n))
}

/// Stage 5 — compare aspirin against control with `greater_than` + `probability_exceeds`.
pub fn verdict_stage(value: EffectValue<ArmAverages>) -> PropagatingEffect<ArmAverages> {
    let Some(avgs) = value.into_value() else {
        return PropagatingEffect::none();
    };

    println!("\n[Stage 5] Verdict");
    let (Some(aspirin), Some(control_mean)) = (&avgs.aspirin_aggregate, avgs.control_mean) else {
        println!("   ⚠️  Insufficient reliable data in one or both arms.");
        return PropagatingEffect::pure(avgs);
    };

    let aspirin_better = aspirin.greater_than(control_mean);
    let confidence = aspirin_better
        .estimate_probability(SAMPLES)
        .unwrap_or(f64::NAN)
        * 100.0;
    println!("   P(Aspirin > Control): {confidence:.1}%");

    match aspirin_better.probability_exceeds(0.9, LIFT_CONFIDENCE, LIFT_EPSILON, SAMPLES) {
        Ok(true) => {
            println!("   ✅ Aspirin reduces headache pain within uncertainty bounds.")
        }
        Ok(false) => {
            println!("   ❌ Evidence insufficient to confidently say Aspirin > Control.")
        }
        Err(e) => println!("   ⚠️  Probability-exceeds check failed: {e}"),
    }

    PropagatingEffect::pure(avgs)
}
