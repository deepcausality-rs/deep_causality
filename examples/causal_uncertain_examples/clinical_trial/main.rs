/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Aspirin Clinical Trial as a `CausalFlow` Chain over `MaybeUncertain`
//!
//! Reworks the original straight-line analysis into a five-stage stateless
//! chain over `CausalFlow`. The carrier type is the per-arm `TrialCohort`,
//! which holds the patient-level `MaybeUncertain<f64>` values.
//!
//! Pipeline:
//!
//! 1. `cohort_stage`       — assemble per-patient `MaybeUncertain` pain-reduction values
//! 2. `presence_stage`     — report Bernoulli-style data-presence probabilities
//! 3. `lift_stage`         — apply `lift_to_uncertain` per patient; skip those that fail the gate
//! 4. `aggregate_stage`    — average within each arm (Aspirin vs Control)
//! 5. `verdict_stage`      — compare means and emit an evidence-based recommendation
//!
//! `MaybeUncertain`'s `None` propagation maps onto the flow's short-circuit at
//! the `lift_to_uncertain` boundary: if no patient clears the data-presence
//! gate, `lift_stage` returns `Err`, and `try_step` halts the remaining stages
//! exactly the way `None` propagates inside `MaybeUncertain` arithmetic.

mod model;

use deep_causality_core::CausalFlow;
use model::{aggregate_stage, cohort_stage, lift_stage, presence_stage, verdict_stage};

fn main() {
    println!("Aspirin Headache Trial (CausalFlow chain over MaybeUncertain)");
    println!("=======================================================================\n");

    CausalFlow::effect()
        .map(|_| cohort_stage())
        .map(presence_stage)
        .try_step(lift_stage)
        .map(aggregate_stage)
        .map(verdict_stage)
        .run(
            |_| println!("\n✅ Trial analysis complete."),
            |_| println!("\n⚠️  Trial analysis short-circuited (insufficient reliable data)."),
        );
}
