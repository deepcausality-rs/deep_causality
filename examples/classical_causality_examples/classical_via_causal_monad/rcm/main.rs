/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # RCM via the Causal Monad
//!
//! Rubin's potential-outcomes definition of a causal effect, implemented
//! directly on `PropagatingProcess<f64, (), TreatmentContext>` using the
//! `Alternatable` family.
//!
//! The estimand:
//!
//! ```text
//! ITE = Y(do(T = 1)) - Y(do(T = 0))
//! ```
//!
//! Mechanics:
//!
//! 1. Build the seed carrier once via `start(treatment_ctx)`.
//! 2. Run the chain factually: `start(treatment).bind(stage1).bind(stage2)`
//!    returns `Y(1)`.
//! 3. Counterfactual: `start(treatment).alternate_context(control)
//!    .bind(stage1).bind(stage2)` returns `Y(0)`. The alternation rewrites
//!    the Context *before* either bind reads it, so both stages see the
//!    control assignment.
//! 4. `ITE = Y(1) - Y(0)`.
//!
//! What `alternate_context` adds over building two separate pipelines:
//! one chain definition, one place to read for the dose model, one
//! audit log per run, and a `!!ContextAlternation!!` entry that records
//! exactly when and where the world was switched.

mod model;

use deep_causality_core::AlternatableContext;
use model::{PATIENT_INITIAL_BP, TreatmentContext, apply_drug_effect, compute_final_bp, start};

fn main() {
    println!("\n--- RCM via the Causal Monad: Drug Effect on Blood Pressure ---");
    println!("Patient baseline BP: {PATIENT_INITIAL_BP:.1}");

    let treatment_ctx = TreatmentContext {
        drug_administered: true,
        drug_effect_if_administered: -10.0,
    };
    let control_ctx = TreatmentContext {
        drug_administered: false,
        drug_effect_if_administered: -10.0,
    };

    // Factual run: chain executes against treatment_ctx.
    println!("\nSimulating treated outcome Y(1) under T=1...");
    let treated = start(treatment_ctx.clone())
        .bind(apply_drug_effect)
        .bind(compute_final_bp);
    let y1 = treated.value_cloned().unwrap();
    println!("Y(1) = {y1:.1}");

    // Counterfactual run: same seed, swap the Context before the binds
    // execute. Both stage closures will read control_ctx. The carrier's
    // value and state continue through the chain untouched; only the
    // Context channel was rewritten, and the audit log carries one
    // `!!ContextAlternation!!` entry recording the switch.
    println!("\nSimulating control outcome Y(0) via alternate_context(control_ctx)...");
    let control = start(treatment_ctx)
        .alternate_context(control_ctx)
        .bind(apply_drug_effect)
        .bind(compute_final_bp);
    let y0 = control.value_cloned().unwrap();
    println!("Y(0) = {y0:.1}");

    let ite = y1 - y0;

    println!("\n--- Individual Treatment Effect ---");
    println!("ITE = Y(1) - Y(0) = {y1:.1} - {y0:.1} = {ite:+.1}");
    println!(
        "The drug is predicted to {} this patient's BP by {:.1} points.",
        if ite < 0.0 { "lower" } else { "raise" },
        ite.abs()
    );

    println!("\n--- Audit log (counterfactual run) ---");
    println!("{}", control.logs());
}
