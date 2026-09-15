/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the crosstalk-attribution example over circuits.

/// Shots per experiment, and the run seed.
pub const SHOTS: u64 = 1024;
pub const SEED: u64 = 20260821;

/// The separation, in bits, at which an experiment resolves a pair of hypotheses. A configuration
/// literal, lifted once into `FloatType` where it is used; a `const` cannot hold a `Float106`.
pub const FLOOR_BITS: f64 = 5.0;

/// How many standard errors a world's prediction may sit from the observation and still hold.
/// A configuration literal, as above.
pub const AGREEMENT_SIGMAS: f64 = 3.0;

/// The node ids under the flat convention: one system per node.
pub const Q1: usize = 0;
pub const Q2: usize = 1;
pub const BATH: usize = 2;

/// The rotation angles of the two candidate circuits: the driven qubit's own `R_y`, and the
/// coupling `R_y` the driving qubit's box applies to the wire it hands on. Configuration
/// literals, lifted once where they are used.
pub const OWN_ANGLE: f64 = 0.4;
pub const COUPLING_ANGLE: f64 = 0.9;
