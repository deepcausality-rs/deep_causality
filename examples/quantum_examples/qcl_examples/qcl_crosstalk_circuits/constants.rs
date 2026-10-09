/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the crosstalk-attribution example over circuits.
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

/// The rotation angles of the two candidate circuits: the driven qubit's own `R_y`, and the
/// coupling `R_y` the driving qubit's box applies to the wire it hands on.
pub const OWN_ANGLE: FloatType = const_scalar_from_float!(FloatType, 0.4);
pub const COUPLING_ANGLE: FloatType = const_scalar_from_float!(FloatType, 0.9);

// =============================================================================
// The conditional tables the factorizations are written with
// =============================================================================

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const ONE: FloatType = const_scalar_from_int!(FloatType, 1);

/// Each qubit's own excitation, in every candidate.
pub const EXCITED: FloatType = const_scalar_from_float!(FloatType, 0.1);

/// A driven qubit's excitation when its driver, a qubit or the bath, is excited or on.
pub const DRIVEN: FloatType = const_scalar_from_float!(FloatType, 0.4);

// =============================================================================
// The experiment family
// =============================================================================

/// What each experiment costs to run, in the same arbitrary unit throughout.
pub const COST_PASSIVE: FloatType = const_scalar_from_int!(FloatType, 1);
pub const COST_INTERVENTION: FloatType = const_scalar_from_int!(FloatType, 1);
pub const COST_ECHO: FloatType = const_scalar_from_int!(FloatType, 2);

/// How many acyclic candidates the screen should admit.
pub const CANDIDATE_COUNT: usize = 3;

/// What the minimum-cost cover should come to.
pub const EXPECTED_PLAN_COST: FloatType = const_scalar_from_int!(FloatType, 2);
