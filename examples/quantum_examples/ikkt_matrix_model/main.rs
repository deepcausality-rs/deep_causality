/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The IKKT matrix model: spacetime as a property of matrices
//!
//! The IKKT model is a candidate non-perturbative formulation of type IIB superstring theory. It
//! has no spacetime in it. What it has is a set of matrices `X_μ` and an action
//!
//! ```text
//! S = Σ_{μ<ν} ‖[X_μ, X_ν]‖²
//! ```
//!
//! which is zero exactly when every pair of them commutes. Commuting matrices can be
//! simultaneously diagonalised, and their joint eigenvalues are then a set of points. That set is
//! the emergent spacetime: it is a property the matrices acquire at the minimum of the action, not
//! a stage they were placed on.
//!
//! # Relaxing along the equation of motion
//!
//! Varying the action gives
//!
//! ```text
//! Σ_ν [X_ν, [X_μ, X_ν]] = 0
//! ```
//!
//! so that double commutator is zero exactly at a solution, and moving against it drives the
//! configuration toward one. Every run prints the action, because a step that raised it would mean
//! the step length was too long, and that is worth seeing rather than hiding.
//!
//! # The norm has to be held fixed
//!
//! The action is quartic in the coordinates, so multiplying every matrix by `1 − η` multiplies the
//! action by `(1 − η)⁴` no matter what the matrices are doing. A run that shrinks everything toward
//! the origin therefore reports an action falling to zero while demonstrating nothing: the limit is
//! an empty vacuum, not a commuting configuration, and no spacetime emerges from it.
//!
//! Each step here restores the norm the configuration started with, so the only way left for the
//! action to fall is for the matrices to genuinely commute. The run prints the norm alongside the
//! action so that the constraint is visible rather than asserted.
//!
//! # What the run does
//!
//! ```text
//! fold   pairs (μ, ν) → the action                    over every commutator in the configuration
//! fold   ν → the double commutator at μ               the equation of motion, one coordinate
//! fold   coefficients → a norm                        the constraint each step restores
//! ```
//!
//! Each of the three is a reduction over a structure the model already has, so the step itself
//! stays one call and the loop below carries nothing but the configuration and what to report.

mod model;
mod utils_print;

use deep_causality_algebra::Real;
use deep_causality_num::Float106;
use deep_causality_quantum::QuantumError;
use model::{
    MAX_STEPS, action, configuration_norm, convergence_threshold, initial_configuration,
    largest_commutator, relax, step_size,
};
use utils_print::{Step, print_header, print_outcome, print_start, print_trajectory};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the
/// commutators, the action and the relaxation all recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

fn main() -> Result<(), QuantumError> {
    print_header();

    let start = initial_configuration()?;
    print_start(action(&start)?, configuration_norm(&start));

    // The relaxation. Each pass replaces the configuration and records what the step was worth
    // reporting; the algebra all lives in `relax`, which folds the commutators behind one call.
    let mut configuration = start;
    let mut trajectory: Vec<Step> = Vec::with_capacity(MAX_STEPS);

    for step in 1..=MAX_STEPS {
        configuration = relax(&configuration, step_size())?;

        let s = action(&configuration)?;
        trajectory.push(Step {
            index: step,
            action: s,
            norm: configuration_norm(&configuration),
            largest_commutator: largest_commutator(&configuration)?,
        });

        if s < convergence_threshold() {
            break;
        }
    }

    print_trajectory(&trajectory);

    // Whether the action fell on every step. The relaxation moves against the equation of motion,
    // so it should, and a run that says otherwise is reporting a step length rather than physics.
    let monotone = trajectory
        .windows(2)
        .all(|pair| pair[1].action <= pair[0].action);

    let eigenvalue_spread = Real::sqrt(
        configuration
            .iter()
            .map(model::squared_norm)
            .fold(model::ZERO, |a, b| a + b),
    );

    print_outcome(
        trajectory.last(),
        monotone,
        eigenvalue_spread,
        convergence_threshold(),
    );

    Ok(())
}
