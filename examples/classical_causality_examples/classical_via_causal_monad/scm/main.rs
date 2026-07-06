/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # SCM via the Causal Monad
//!
//! Pearl's Ladder of Causation on the smoking-tar-cancer chain,
//! implemented directly on `PropagatingProcess<f64, (), SmokingContext>`
//! using the `Alternatable` family. Each rung uses one operator from the
//! family:
//!
//! * **Rung 1 (Association)**: plain `bind` chain; no alternation.
//! * **Rung 2 (Intervention)**: `intervene` mid-chain to apply
//!   `do(Tar := 0.0)`; the Smoking -> Tar link is severed before stage 2
//!   reads the tar indicator.
//! * **Rung 3 (Counterfactual)**: `alternate_context` at the seed to
//!   switch to a world where nicotine was low but tar already accumulated;
//!   the same chain runs against the alternated world and the difference
//!   is the counterfactual quantity.
//!
//! The monad version is short enough to live in one file. The causaloid
//! version (`classical_via_causaloid/scm`) splits the rungs across files
//! because each rung carries non-trivial CausaloidGraph and Contextoid
//! scaffolding.

use deep_causality_core::{
    AlternatableContext, EffectValue, Intervenable, PropagatingEffect, PropagatingProcess,
};

fn main() {
    println!("\n=== SCM via the Causal Monad: Pearl's Ladder on smoking-tar-cancer ===\n");
    run_rung1_association();
    run_rung2_intervention();
    run_rung3_counterfactual();
}

// --- Rung 1: Association ---

fn run_rung1_association() {
    println!("--- Rung 1: Association ---");
    println!("Observation: a person has high nicotine consumption (0.8).");

    let world = SmokingContext {
        nicotine_level: 0.8,
        tar_level: 0.0,
    };

    let final_effect = start(world).bind(stage_has_tar).bind(stage_cancer_risk);

    let cancer_risk = cancer_risk_from(final_effect.value().unwrap());
    println!("Result: high nicotine is associated with cancer risk = {cancer_risk}.");
    assert!(cancer_risk, "Rung 1: expected high cancer risk");
    println!();
}

// --- Rung 2: Intervention ---

fn run_rung2_intervention() {
    println!("--- Rung 2: Intervention ---");
    println!("Operator: do(Tar := 0.0). High-nicotine world, tar forced absent mid-chain.");

    let world = SmokingContext {
        nicotine_level: 0.8,
        tar_level: 0.0,
    };

    let final_effect = start(world)
        .bind(stage_has_tar)
        .intervene(0.0_f64) // do(Tar := 0.0); alias for alternate_value(0.0)
        .bind(stage_cancer_risk);

    let cancer_risk = cancer_risk_from(final_effect.value().unwrap());
    println!("Result: under the intervention, cancer risk = {cancer_risk}.");
    assert!(
        !cancer_risk,
        "Rung 2: expected low cancer risk after do(Tar := 0.0)"
    );

    println!("\nAudit log (intervention run):");
    println!("{}", final_effect.logs());
    println!();
}

// --- Rung 3: Counterfactual ---

fn run_rung3_counterfactual() {
    println!("--- Rung 3: Counterfactual ---");
    println!("Query: given a smoker with high tar, what if they had not smoked?");

    let factual = SmokingContext {
        nicotine_level: 0.8,
        tar_level: 0.8,
    };
    let counterfactual = SmokingContext {
        nicotine_level: 0.1, // they did not smoke
        tar_level: 0.8,      // but tar is still in the lungs from earlier
    };

    let factual_final = start(factual.clone())
        .bind(stage_has_tar)
        .bind(stage_cancer_risk);

    let counterfactual_final = start(factual)
        .alternate_context(counterfactual)
        .bind(stage_has_tar)
        .bind(stage_cancer_risk);

    let f_risk = cancer_risk_from(factual_final.value().unwrap());
    let cf_risk = cancer_risk_from(counterfactual_final.value().unwrap());

    println!("Factual world (nicotine=0.8, tar=0.8):           cancer risk = {f_risk}");
    println!("Counterfactual (nicotine=0.1, tar=0.8 retained): cancer risk = {cf_risk}");
    println!(
        "Conclusion: in the counterfactual world, tar is still present (the body retains it),\n\
         so cancer risk stays high. Quitting now does not undo the accumulated damage."
    );

    assert!(f_risk, "Rung 3: factual cancer risk should be high");
    assert!(
        cf_risk,
        "Rung 3: counterfactual cancer risk should still be high (tar retained)"
    );

    println!("\nAudit log (counterfactual run):");
    println!("{}", counterfactual_final.logs());
    println!();
}

// --- Model: world state, chain seed, and bind stages ---

/// Smoking-tar-cancer world state, carried in the Context channel.
/// `nicotine_level` is current consumption; `tar_level` is pre-existing
/// tar already in the lungs from earlier smoking.
#[derive(Clone, Debug, PartialEq)]
struct SmokingContext {
    nicotine_level: f64,
    tar_level: f64,
}

/// Decision threshold for the binary qualifiers.
const THRESHOLD: f64 = 0.6;

/// Build the seed carrier with the given world state.
fn start(world: SmokingContext) -> PropagatingProcess<f64, (), SmokingContext> {
    let seed = PropagatingEffect::pure(0.0_f64);
    PropagatingProcess::with_state(seed, (), Some(world))
}

/// Stage 1 (`Smoking -> Tar`): tar is present when either nicotine is high
/// or tar has already accumulated. Emits a numeric tar indicator so the
/// value channel is alternable mid-chain.
fn stage_has_tar(
    _value: EffectValue<f64>,
    state: (),
    context: Option<SmokingContext>,
) -> PropagatingProcess<f64, (), SmokingContext> {
    let ctx = context.expect("SmokingContext must be set before stage 1");
    let high_nicotine = ctx.nicotine_level > THRESHOLD;
    let pre_existing_tar = ctx.tar_level > THRESHOLD;
    let has_tar = high_nicotine || pre_existing_tar;
    let next = PropagatingEffect::pure(if has_tar { 1.0 } else { 0.0 });
    PropagatingProcess::with_state(next, state, Some(ctx))
}

/// Stage 2 (`Tar -> Cancer`): cancer risk follows from the tar indicator
/// the upstream stage produced. Reading from the value (not the Context)
/// is what makes `intervene(...)` between the two stages a clean
/// `do(Tar := x)`.
fn stage_cancer_risk(
    value: EffectValue<f64>,
    state: (),
    context: Option<SmokingContext>,
) -> PropagatingProcess<f64, (), SmokingContext> {
    let tar_indicator = value
        .into_value()
        .expect("stage_has_tar must produce a numeric tar indicator");
    let cancer_risk = tar_indicator > 0.5;
    let next = PropagatingEffect::pure(if cancer_risk { 1.0 } else { 0.0 });
    PropagatingProcess::with_state(next, state, context)
}

fn cancer_risk_from(value: &f64) -> bool {
    *value > 0.5
}
