/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Aspirin Clinical Trial as a `PropagatingEffect` Chain over `MaybeUncertain`
//!
//! Reworks the original straight-line analysis into a five-stage stateless
//! monadic chain over `PropagatingEffect`. The carrier type is the per-arm
//! `TrialCohort`, which holds the patient-level `MaybeUncertain<f64>` values.
//!
//! Pipeline:
//!
//! 1. `cohort_stage`       — assemble per-patient `MaybeUncertain` pain-reduction values
//! 2. `presence_stage`     — report Bernoulli-style data-presence probabilities
//! 3. `lift_stage`         — apply `lift_to_uncertain` per patient; skip those that fail the gate
//! 4. `aggregate_stage`    — average within each arm (Aspirin vs Control)
//! 5. `verdict_stage`      — compare means and emit an evidence-based recommendation
//!
//! `MaybeUncertain`'s `None` propagation maps onto `EffectValue::None` at the
//! `lift_to_uncertain` boundary: a failed lift becomes `EffectValue::None`,
//! short-circuiting downstream stages exactly the way `None` propagates inside
//! `MaybeUncertain` arithmetic.

mod model;

use deep_causality_core::{EffectValue, PropagatingEffect};
use model::{aggregate_stage, cohort_stage, lift_stage, presence_stage, verdict_stage};

fn main() {
    println!("💊 Aspirin Headache Trial (PropagatingEffect chain over MaybeUncertain)");
    println!("=======================================================================\n");

    let pipeline = PropagatingEffect::pure(())
        .bind(|_, _, _| cohort_stage())
        .bind(|value, _, _| presence_stage(value))
        .bind(|value, _, _| lift_stage(value))
        .bind(|value, _, _| aggregate_stage(value))
        .bind(|value, _, _| verdict_stage(value));

    match pipeline.value {
        EffectValue::Value(_) => println!("\n✅ Trial analysis complete."),
        EffectValue::None => {
            println!("\n⚠️  Trial analysis short-circuited (insufficient reliable data).")
        }
        _ => println!("\n⚠️  Pipeline returned an unexpected EffectValue variant."),
    }

    if let Some(err) = pipeline.error {
        println!("   error: {err:?}");
    }
}
