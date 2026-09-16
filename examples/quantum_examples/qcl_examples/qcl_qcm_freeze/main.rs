/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The QCL model path, expressed against the shipped `freeze_quantum` callers.
//!
//! `qcm_freeze_check` runs the Markov commutativity check at the freeze boundary through
//! `freeze_quantum`. This example runs the same two checks through the QCL pipeline:
//!
//!   * one configuration origin, `QclBuilder::config::<FloatType, NumberType>()`, over a frozen
//!     model with its declared input and output systems;
//!   * `validate` running `check_markov` and `check_decomposable`, the two level checks the
//!     shipped freeze runs inside `freeze_verified_with_check`, terminating in a `Screened`
//!     whose report says how many pairs were examined and how close the worst came to the edge;
//!   * the non-commuting model failing `validate` with the structured `CommutatorNonZero` naming
//!     the pair, and the frozen subject left exactly as built;
//!   * the same model on a dynamic graph through `QclBuilder::freeze_model`, which is the shipped
//!     freeze and rolls the graph back.

mod model;

use deep_causality::CausableGraph;
use deep_causality_num::Float106;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    CheckVerdict, CommutatorTolerance, QclBuilder, QuantumErrorEnum, freeze_quantum,
};

use crate::model::{
    SOURCE_NODE, TARGET_NODE, diagonal, factors_on_shared_leg, sigma_x, sigma_z, two_node_graph,
};

/// The real working type. Every tolerance in the run derives from its `epsilon()`; switch it to
/// `f32`, `f64`, or `Float106` and the thresholds move with it. `f64` appears only at the display boundary.
pub type FloatType = Float106;

/// The count working type. ℕ, unsigned; widening it buys headroom and moves no threshold.
pub type NumberType = u64;

/// The complex scalar every Choi–Jamiołkowski factor carries.
pub type C = Complex<FloatType>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== QCL model path: the shipped freeze checks through the pipeline ===\n");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());

    commuting_model_screens()?;
    println!();
    non_commuting_model_fails_validate()?;
    println!();
    dynamic_graph_rolls_back()?;

    Ok(())
}

/// Two diagonal factors on the shared leg commute: the model screens.
fn commuting_model_screens() -> Result<(), Box<dyn std::error::Error>> {
    println!("[1] Commuting model: σz and diag(3, −1) on leg 0");
    let (factors, supports) = factors_on_shared_leg(sigma_z()?, diagonal()?);
    let tolerance = CommutatorTolerance::<FloatType>::default();

    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_model(two_node_graph(true)?, factors.clone(), supports.clone())
        .declare_systems(&[SOURCE_NODE], &[TARGET_NODE])
        .build()?;

    let screened = QclBuilder::validate(&cfg)
        .check_markov(&tolerance)
        .check_decomposable()
        .finalize()?;

    for (name, report) in screened.stages() {
        println!(
            "    {name:<20} {:?}  examined {}  worst margin {}",
            report.verdict(),
            report.examined(),
            report
                .worst_margin()
                .map_or("—".to_string(), |m| format!("{m:.3e}"))
        );
    }
    let folded = screened
        .report()
        .ok_or("the screen should carry a current report")?;
    println!(
        "    screen: {:?}, {} items examined across both checks",
        folded.verdict(),
        folded.examined()
    );

    // The same numbers the shipped freeze reports on the same model, with the same declared
    // systems, so both of its checks run: Markov, and C₃-exclusion between input 0 and output 1.
    let mut graph = two_node_graph(false)?;
    let shipped = freeze_quantum(
        &mut graph,
        &[],
        &factors,
        &supports,
        &tolerance,
        Some((&[SOURCE_NODE], &[TARGET_NODE])),
    )?;
    println!(
        "    freeze_quantum on the same model: {} pair(s), worst margin {:.3e} — the same report",
        shipped.tested_pairs(),
        shipped.worst_margin().unwrap_or_default()
    );
    // The two routes agree, which is what makes either of them worth reading. `check_decomposable`
    // is vacuous here because a two-node chain has no C₃ sub-relation to exclude, and saying so is
    // more use than a bare "passed".
    let same_pairs = screened.stages()[0].1.examined() == shipped.tested_pairs();
    let decomposable_is_vacuous = screened.stages()[1].1.verdict() == CheckVerdict::Vacuous;

    println!(
        "    the pipeline and the shipped freeze examined the same pairs: {}",
        if same_pairs { "yes" } else { "NO" }
    );
    println!(
        "    check_decomposable is vacuous (no C₃ sub-relation on a two-node chain): {}",
        if decomposable_is_vacuous { "yes" } else { "NO" }
    );

    if !(same_pairs && decomposable_is_vacuous) {
        return Err("the pipeline and the shipped freeze disagreed".into());
    }

    Ok(())
}

/// σx and σz on the shared leg do not commute: validate fails with the structured cause, and
/// the frozen subject is left exactly as built.
fn non_commuting_model_fails_validate() -> Result<(), Box<dyn std::error::Error>> {
    println!("[2] Non-commuting model: σx and σz on leg 0");
    let (factors, supports) = factors_on_shared_leg(sigma_x()?, sigma_z()?);

    // The build succeeds: the pair is rejected at validate, not at build. Which stage catches it
    // is worth knowing, because a model that cannot be built and one that builds and then fails
    // its checks are different problems for a caller.
    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_model(two_node_graph(true)?, factors, supports)
        .declare_systems(&[SOURCE_NODE], &[TARGET_NODE])
        .build()?;

    let outcome = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::<FloatType>::default())
        .check_decomposable()
        .finalize();
    match outcome {
        Ok(_) => return Err("a non-commuting model screened; validate should reject it".into()),
        Err(e) => match e.0 {
            QuantumErrorEnum::CommutatorNonZero { node_j, node_k, .. } => {
                println!(
                    "    ✓ validate failed: factors at nodes {node_j} and {node_k} do not commute"
                );
                println!(
                    "    the subject is as built: is_frozen() = {}",
                    cfg.subject().graph().is_frozen()
                );
            }
            other => return Err(format!("validate failed for the wrong reason: {other:?}").into()),
        },
    }

    Ok(())
}

/// The shipped freeze on a dynamic graph: the same rejection, and the graph rolled back.
fn dynamic_graph_rolls_back() -> Result<(), Box<dyn std::error::Error>> {
    println!("[3] The same non-commuting model on a dynamic graph, through the shipped freeze");
    let mut graph = two_node_graph(false)?;
    let (factors, supports) = factors_on_shared_leg(sigma_x()?, sigma_z()?);
    let outcome = QclBuilder::freeze_model(
        &mut graph,
        &[],
        &factors,
        &supports,
        &CommutatorTolerance::<FloatType>::default(),
        Some((&[SOURCE_NODE], &[TARGET_NODE])),
    );
    match outcome {
        Ok(_) => return Err("a non-commuting model froze; the freeze should abort".into()),
        Err(e) => match e.0 {
            QuantumErrorEnum::CommutatorNonZero { node_j, node_k, .. } => {
                println!(
                    "    ✓ freeze aborted: factors at nodes {node_j} and {node_k} do not commute"
                );
                println!("    is_frozen() = {} (rolled back)", graph.is_frozen());
            }
            other => {
                return Err(format!("the freeze aborted for the wrong reason: {other:?}").into());
            }
        },
    }

    Ok(())
}
