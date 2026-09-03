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

use crate::model::{diagonal, factors_on_shared_leg, sigma_x, sigma_z, two_node_graph};

/// The real working type. Every tolerance in the run derives from its `epsilon()`; switch it to
/// `f32` or `Float106` and the thresholds move with it. `f64` appears only at the display boundary.
pub type FloatType = Float106;

/// The count working type. ℕ, unsigned; widening it buys headroom and moves no threshold.
pub type NumberType = u64;

/// The complex scalar every Choi–Jamiołkowski factor carries.
pub type C = Complex<FloatType>;

fn main() {
    println!("=== QCL model path: the shipped freeze checks through the pipeline ===\n");
    commuting_model_screens();
    println!();
    non_commuting_model_fails_validate();
    println!();
    dynamic_graph_rolls_back();
}

/// Two diagonal factors on the shared leg commute: the model screens.
fn commuting_model_screens() {
    println!("[1] Commuting model: σz and diag(3, −1) on leg 0");
    let (factors, supports) = factors_on_shared_leg(sigma_z(), diagonal(3.0, -1.0));
    let tolerance = CommutatorTolerance::<FloatType>::default();

    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_model(two_node_graph(true), factors.clone(), supports.clone())
        .declare_systems(&[0], &[1])
        .build()
        .expect("a frozen, acyclic, validated model builds");

    let screened = QclBuilder::validate(&cfg)
        .check_markov(&tolerance)
        .check_decomposable()
        .finalize()
        .expect("a commuting model screens");

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
    let folded = screened.report().expect("current");
    println!(
        "    screen: {:?}, {} items examined across both checks",
        folded.verdict(),
        folded.examined()
    );

    // The same numbers the shipped freeze reports on the same model.
    let mut graph = two_node_graph(false);
    let shipped = freeze_quantum(&mut graph, &[], &factors, &supports, &tolerance, None)
        .expect("the shipped freeze agrees");
    println!(
        "    freeze_quantum on the same model: {} pair(s), worst margin {:.3e} — the same report",
        shipped.tested_pairs(),
        shipped.worst_margin().unwrap_or_default()
    );
    assert_eq!(screened.stages()[0].1.examined(), shipped.tested_pairs());
    assert_eq!(screened.stages()[1].1.verdict(), CheckVerdict::Vacuous);
}

/// σx and σz on the shared leg do not commute: validate fails with the structured cause, and
/// the frozen subject is left exactly as built.
fn non_commuting_model_fails_validate() {
    println!("[2] Non-commuting model: σx and σz on leg 0");
    let (factors, supports) = factors_on_shared_leg(sigma_x(), sigma_z());
    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_model(two_node_graph(true), factors, supports)
        .declare_systems(&[0], &[1])
        .build()
        .expect("builds; the pair fails at validate, not at build");

    let outcome = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::<FloatType>::default())
        .check_decomposable()
        .finalize();
    match outcome {
        Ok(_) => println!("    unexpected: a non-commuting model must not screen"),
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
            other => println!("    failed with an unexpected error: {other:?}"),
        },
    }
}

/// The shipped freeze on a dynamic graph: the same rejection, and the graph rolled back.
fn dynamic_graph_rolls_back() {
    println!("[3] The same non-commuting model on a dynamic graph, through the shipped freeze");
    let mut graph = two_node_graph(false);
    let (factors, supports) = factors_on_shared_leg(sigma_x(), sigma_z());
    let outcome = QclBuilder::freeze_model(
        &mut graph,
        &[],
        &factors,
        &supports,
        &CommutatorTolerance::<FloatType>::default(),
        Some((&[0], &[1])),
    );
    match outcome {
        Ok(_) => println!("    unexpected: a non-commuting model must not freeze"),
        Err(e) => match e.0 {
            QuantumErrorEnum::CommutatorNonZero { node_j, node_k, .. } => {
                println!(
                    "    ✓ freeze aborted: factors at nodes {node_j} and {node_k} do not commute"
                );
                println!("    is_frozen() = {} (rolled back)", graph.is_frozen());
            }
            other => println!("    aborted with an unexpected error: {other:?}"),
        },
    }
}
