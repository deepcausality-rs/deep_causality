/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Bayesian Root Cause Discovery (BRCD) — causal-graph operations.
//!
//! This module roots all BRCD code. Two layers live here:
//!
//! * **Structural operations** over the typed-endpoint
//!   [`deep_causality_topology::MixedGraph`], carrying no floating-point scalars
//!   — Meek orientation (PDAG → CPDAG completion, [`meek`]), the
//!   unshielded-collider validity check ([`validity`]), and Markov-
//!   equivalence-class sizing + uniform DAG sampling ([`mec`]).
//! * **Numeric estimator primitives**, generic over `T: RealField` — the
//!   logistic-regression gate for mixture-of-experts F-integration ([`gate`])
//!   and the small dense SPD solver it shares with the ridge-Gaussian fit
//!   ([`linalg`]).
//!
//! The remaining estimator pieces (F-node augmentation, ridge-Gaussian / Dirichlet
//! scoring, posterior ranking) compose these and land in later stages.

pub mod gate;
pub mod mec;
pub mod meek;
pub mod validity;

pub(crate) mod linalg;
