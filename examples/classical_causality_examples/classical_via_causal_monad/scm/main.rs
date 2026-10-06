/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # SCM via the Causal Monad
//!
//! Pearl's Ladder of Causation on the smoking-tar-cancer chain,
//! implemented directly on `PropagatingProcess<FloatType, (), SmokingContext>`
//! using the `Alternatable` family. Each rung uses one operator from the
//! family:
//!
//! * **Rung 1 (Association)**: plain `bind` chain; no alternation.
//! * **Rung 2 (Intervention)**: `alternate_value` mid-chain to apply
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

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{
    AlternatableContext, AlternatableValue, CausalEffect, CausalityError, PropagatingEffect,
    PropagatingProcess,
};
use std::error::Error;

/// The scalar this example works in. Declared here, per example, so changing the shared alias in
/// `deep_causality_core` cannot silently reconfigure every example that names one.
type FloatType = f64;

/// The world one run reasons about: numeric data only, no space and no time.
type SmokingContext = Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Node indices of the three `Data` contextoids each smoking world carries: current nicotine
/// consumption, pre-existing tar, and the level above which either counts as high.
const NICOTINE: usize = 0;
const TAR: usize = 1;
const THRESHOLD: usize = 2;

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n=== SCM via the Causal Monad: Pearl's Ladder on smoking-tar-cancer ===\n");
    run_rung1_association()?;
    run_rung2_intervention()?;
    run_rung3_counterfactual()
}

// --- Rung 1: Association ---

fn run_rung1_association() -> Result<(), Box<dyn Error>> {
    println!("--- Rung 1: Association ---");
    println!("Observation: a person has high nicotine consumption (0.8).");

    let world = smoking_world(0.8, 0.0)?;

    let final_effect = start(world).bind(stage_has_tar).bind(stage_cancer_risk);

    let cancer_risk = cancer_risk_from(&value_of(&final_effect)?);
    println!("Result: high nicotine is associated with cancer risk = {cancer_risk}.");
    assert!(cancer_risk, "Rung 1: expected high cancer risk");
    println!();
    Ok(())
}

// --- Rung 2: Intervention ---

fn run_rung2_intervention() -> Result<(), Box<dyn Error>> {
    println!("--- Rung 2: Intervention ---");
    println!("Operator: do(Tar := 0.0). High-nicotine world, tar forced absent mid-chain.");

    let world = smoking_world(0.8, 0.0)?;

    let final_effect = start(world)
        .bind(stage_has_tar)
        .alternate_value(0.0 as FloatType) // do(Tar := 0.0)
        .bind(stage_cancer_risk);

    let cancer_risk = cancer_risk_from(&value_of(&final_effect)?);
    println!("Result: under the intervention, cancer risk = {cancer_risk}.");
    assert!(
        !cancer_risk,
        "Rung 2: expected low cancer risk after do(Tar := 0.0)"
    );

    println!("\nAudit log (intervention run):");
    println!("{}", final_effect.logs());
    println!();
    Ok(())
}

// --- Rung 3: Counterfactual ---

fn run_rung3_counterfactual() -> Result<(), Box<dyn Error>> {
    println!("--- Rung 3: Counterfactual ---");
    println!("Query: given a smoker with high tar, what if they had not smoked?");

    let factual = smoking_world(0.8, 0.8)?;
    // They did not smoke (low nicotine), but tar is still in the lungs from earlier.
    let counterfactual = smoking_world(0.1, 0.8)?;

    let factual_final = start(factual.clone())
        .bind(stage_has_tar)
        .bind(stage_cancer_risk);

    let counterfactual_final = start(factual)
        .alternate_context(counterfactual)
        .bind(stage_has_tar)
        .bind(stage_cancer_risk);

    let f_risk = cancer_risk_from(&value_of(&factual_final)?);
    let cf_risk = cancer_risk_from(&value_of(&counterfactual_final)?);

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
    Ok(())
}

// --- Model: world state, chain seed, and bind stages ---

/// Build the world a run reasons against: current nicotine consumption, pre-existing tar and
/// the level at which either counts, as three `Data` contextoids in one [`SmokingContext`]. The
/// level is the same in every world.
fn smoking_world(
    nicotine_level: FloatType,
    tar_level: FloatType,
) -> Result<SmokingContext, ContextIndexError> {
    let mut context = Context::with_capacity(1, "smoking world", 3);
    for (id, value) in [(1, nicotine_level), (2, tar_level), (3, 0.6)] {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// Read one `Data` contextoid's payload out of a smoking world.
fn read(context: &SmokingContext, index: usize) -> Result<FloatType, CausalityError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(|data| data.get_data())
        .ok_or_else(|| CausalityError::ModelError(format!("context node {index} is not a Datoid")))
}

/// The value a finished chain carries, or the error that ended it.
fn value_of(
    process: &PropagatingProcess<FloatType, (), SmokingContext>,
) -> Result<FloatType, CausalityError> {
    match process.error() {
        Some(error) => Err(error.clone()),
        None => process
            .value_cloned()
            .ok_or(CausalityError::ValueNotAvailable()),
    }
}

/// Build the seed carrier with the given world state.
fn start(world: SmokingContext) -> PropagatingProcess<FloatType, (), SmokingContext> {
    let seed = PropagatingEffect::pure(0.0 as FloatType);
    PropagatingProcess::with_state(seed, (), Some(world))
}

/// Stage 1 (`Smoking -> Tar`): tar is present when either nicotine is high
/// or tar has already accumulated. Emits a numeric tar indicator so the
/// value channel is alternable mid-chain.
fn stage_has_tar(
    _value: CausalEffect<FloatType>,
    state: (),
    context: Option<SmokingContext>,
) -> PropagatingProcess<FloatType, (), SmokingContext> {
    let Some(ctx) = context else {
        return PropagatingProcess::from_error(CausalityError::MissingContext());
    };
    match has_tar(&ctx) {
        Ok(has_tar) => {
            let next = PropagatingEffect::pure(if has_tar { 1.0 } else { 0.0 });
            PropagatingProcess::with_state(next, state, Some(ctx))
        }
        Err(error) => PropagatingProcess::from_error(error),
    }
}

/// Whether nicotine or pre-existing tar exceeds the world's threshold.
fn has_tar(ctx: &SmokingContext) -> Result<bool, CausalityError> {
    let threshold = read(ctx, THRESHOLD)?;
    let high_nicotine = read(ctx, NICOTINE)? > threshold;
    let pre_existing_tar = read(ctx, TAR)? > threshold;
    Ok(high_nicotine || pre_existing_tar)
}

/// Stage 2 (`Tar -> Cancer`): cancer risk follows from the tar indicator
/// the upstream stage produced. Reading from the value (not the Context)
/// is what makes `alternate_value(...)` between the two stages a clean
/// `do(Tar := x)`.
fn stage_cancer_risk(
    value: CausalEffect<FloatType>,
    state: (),
    context: Option<SmokingContext>,
) -> PropagatingProcess<FloatType, (), SmokingContext> {
    let Some(tar_indicator) = value.into_value() else {
        return PropagatingProcess::from_error(CausalityError::ValueNotAvailable());
    };
    let cancer_risk = tar_indicator > 0.5;
    let next = PropagatingEffect::pure(if cancer_risk { 1.0 } else { 0.0 });
    PropagatingProcess::with_state(next, state, context)
}

fn cancer_risk_from(value: &FloatType) -> bool {
    *value > 0.5
}
