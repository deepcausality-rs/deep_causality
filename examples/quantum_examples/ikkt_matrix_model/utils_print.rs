/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the IKKT relaxation.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use crate::model::{ALGEBRA_DIMENSION, MATRIX_SIZE, N_MATRICES};
use deep_causality_num::lower;

/// What one relaxation step is worth reporting.
pub struct Step {
    pub index: usize,
    pub action: FloatType,
    pub norm: FloatType,
    pub largest_commutator: FloatType,
}

pub fn print_header() {
    println!("=== The IKKT matrix model: spacetime as a property of matrices ===\n");
    println!("Precision:   {}", core::any::type_name::<FloatType>());
    println!(
        "Coordinates: {N_MATRICES} matrices X_0 .. X_{}",
        N_MATRICES - 1
    );
    println!("Algebra:     Cl({ALGEBRA_DIMENSION}), so {MATRIX_SIZE} complex coefficients each");
    println!("Action:      S = sum over mu<nu of ||[X_mu, X_nu]||^2\n");
}

/// Where the configuration starts.
pub fn print_start(action: FloatType, norm: FloatType) {
    println!("Start");
    println!("  action S          {:>14.9}", lower(action));
    println!("  configuration norm{:>14.9}", lower(norm));
    println!();
    println!("  The norm is what every step below restores. Without it the action falls as the");
    println!("  fourth power of a uniform shrink, and reaching zero would say nothing at all.\n");
}

/// The relaxation, one row per step.
pub fn print_trajectory(trajectory: &[Step]) {
    println!("Relaxing against the equation of motion");
    println!("  step        action S    largest [X,X]    configuration norm");

    // Every fifth step, and the last one whatever its index, so the table shows the shape of the
    // descent and where it ended without printing every row of it.
    let last = trajectory.len();
    let shown = trajectory
        .iter()
        .filter(|step| step.index % 5 == 0 || step.index == 1 || step.index == last);

    for step in shown {
        println!(
            "  {:>4}   {:>13.9}   {:>13.9}   {:>13.9}",
            step.index,
            lower(step.action),
            lower(step.largest_commutator),
            lower(step.norm)
        );
    }
    println!();
}

/// What the relaxation came to.
pub fn print_outcome(
    last: Option<&Step>,
    monotone: bool,
    norm: FloatType,
    residual: FloatType,
    threshold: FloatType,
) {
    let Some(last) = last else {
        println!("The run took no steps, so there is nothing to report.");
        return;
    };

    println!("Outcome");
    println!("  steps taken             {:>14}", last.index);
    println!("  final action S          {:>14.9}", lower(last.action));
    println!(
        "  largest [X_mu, X_nu]    {:>14.9}",
        lower(last.largest_commutator)
    );
    println!("  configuration norm      {:>14.9}", lower(norm));
    println!("  EOM residual            {:>14.3e}", lower(residual));
    println!(
        "  action fell every step  {:>14}",
        if monotone { "yes" } else { "no" }
    );
    println!();

    // Three outcomes, told apart by two numbers. The action says whether the matrices commute; the
    // residual says whether the configuration solves the equation of motion at all.
    if last.action < threshold {
        println!("  The matrices commute to the tolerance asked for, and they did it at the norm");
        println!("  they started with. Commuting matrices are simultaneously diagonalisable, so");
        println!("  the configuration now has a joint spectrum, and that set of points is the");
        println!("  spacetime the model is said to emerge into.");
    } else if residual < threshold {
        println!("  The double commutator vanishes while the single one does not: the matrices");
        println!("  solve the equation of motion without commuting. That is a fuzzy-sphere");
        println!("  solution, and it describes a non-commutative geometry.");
    } else {
        println!("  The run reached its step limit with the action above the tolerance and the");
        println!("  equation of motion unsatisfied. The descent had not finished; at this");
        println!("  precision and step length it needs more steps or a line search before it");
        println!("  can say what it converges to.");
    }
}
