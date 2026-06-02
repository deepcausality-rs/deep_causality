/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Bayesian Root Cause Discovery (BRCD).
//!
//! This module roots all BRCD code. Every fallible operation returns the single
//! [`brcd_error::BrcdError`] type. Three layers live here:
//!
//! * **Structural operations** over the typed-endpoint
//!   [`deep_causality_topology::MixedGraph`], carrying no floating-point scalars
//!   — Meek orientation (PDAG → CPDAG completion, [`brcd_meek`]), the
//!   unshielded-collider validity check ([`brcd_validity`]), and Markov-
//!   equivalence-class sizing + uniform DAG sampling ([`brcd_mec`]).
//! * **Numeric estimator primitives**, generic over `T: RealField` — the
//!   logistic-regression gate ([`brcd_gate`]), the ridge-Gaussian / mixture
//!   continuous family ([`brcd_gaussian`]), the discrete Dirichlet family
//!   ([`brcd_dirichlet`]), and the small dense SPD solver they share
//!   ([`brcd_linalg`]).
//!
//! The remaining estimator pieces (F-node augmentation, posterior ranking)
//! compose these and land in later stages.

pub mod brcd_dirichlet;
pub mod brcd_error;
pub mod brcd_gate;
pub mod brcd_gaussian;
pub mod brcd_mec;
pub mod brcd_meek;
pub mod brcd_validity;

pub(crate) mod brcd_linalg;
