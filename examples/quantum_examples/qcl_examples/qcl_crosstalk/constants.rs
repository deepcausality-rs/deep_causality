/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the crosstalk-attribution example.
//!
//! Every constant is declared at the working type through `const_scalar_from_int!` and
//! `const_scalar_from_float!`, so the compiler resolves them against the alias in `main` and no
//! conversion runs at any call site.

use crate::FloatType;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int};

/// Shots per experiment, and the run seed.
pub const SHOTS: u64 = 1024;
pub const SEED: u64 = 20260821;

/// The separation, in bits, at which an experiment resolves a pair of hypotheses.
pub const FLOOR_BITS: FloatType = const_scalar_from_int!(FloatType, 5);

/// How many standard errors a world's prediction may sit from the observation and still hold.
pub const AGREEMENT_SIGMAS: FloatType = const_scalar_from_int!(FloatType, 3);

/// The node ids under the flat convention: one system per node.
pub const Q1: usize = 0;
pub const Q2: usize = 1;
pub const BATH: usize = 2;

// =============================================================================
// The small numbers the factors are written with
// =============================================================================

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const ONE: FloatType = const_scalar_from_int!(FloatType, 1);

/// A single-qubit factor's diagonal: mostly ground, a little excited.
pub const QUBIT_FACTOR: [FloatType; 2] = [
    const_scalar_from_float!(FloatType, 0.9),
    const_scalar_from_float!(FloatType, 0.1),
];

/// A factor on a qubit and one parent.
pub const TWO_LEG_FACTOR: [FloatType; 4] = [
    const_scalar_from_float!(FloatType, 0.85),
    const_scalar_from_float!(FloatType, 0.05),
    const_scalar_from_float!(FloatType, 0.05),
    const_scalar_from_float!(FloatType, 0.05),
];

// =============================================================================
// The experiment family
// =============================================================================

/// What each experiment costs to run, in the same arbitrary unit throughout.
///
/// Tomography at two hundred against a targeted intervention at one is the whole economic argument:
/// the plan below buys the same answer for two.
pub const COST_PASSIVE: FloatType = const_scalar_from_int!(FloatType, 1);
pub const COST_INTERVENTION: FloatType = const_scalar_from_int!(FloatType, 1);
pub const COST_ECHO: FloatType = const_scalar_from_int!(FloatType, 2);
pub const COST_TOMOGRAPHY: FloatType = const_scalar_from_int!(FloatType, 200);

/// The read-out each experiment predicts under H₁, H₂ and H₃ in that order. These are the note's
/// Table §4.
pub const PREDICT_PASSIVE: [FloatType; 3] = [
    const_scalar_from_float!(FloatType, 0.04),
    const_scalar_from_float!(FloatType, 0.04),
    const_scalar_from_float!(FloatType, 0.04),
];
pub const PREDICT_HOLD_Q1: [FloatType; 3] = [
    const_scalar_from_float!(FloatType, 0.40),
    const_scalar_from_float!(FloatType, 0.10),
    const_scalar_from_float!(FloatType, 0.10),
];
pub const PREDICT_HOLD_Q2: [FloatType; 3] = [
    const_scalar_from_float!(FloatType, 0.10),
    const_scalar_from_float!(FloatType, 0.40),
    const_scalar_from_float!(FloatType, 0.10),
];
pub const PREDICT_ECHO: [FloatType; 3] = [
    const_scalar_from_float!(FloatType, 0.01),
    const_scalar_from_float!(FloatType, 0.01),
    const_scalar_from_float!(FloatType, 0.04),
];
pub const PREDICT_TOMOGRAPHY: [FloatType; 3] = [
    const_scalar_from_float!(FloatType, 0.90),
    const_scalar_from_float!(FloatType, 0.50),
    const_scalar_from_float!(FloatType, 0.10),
];

/// How many acyclic candidates the screen should admit. The fourth is cyclic and is refused
/// earlier, at `build()`.
pub const CANDIDATE_COUNT: usize = 3;

/// What the minimum-cost cover should come to: the two targeted interventions.
pub const EXPECTED_PLAN_COST: FloatType = const_scalar_from_int!(FloatType, 2);
