/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Bayesian Root Cause Discovery (BRCD) — causal-graph operations.
//!
//! This module hosts the structural causal-discovery operations BRCD composes:
//! Meek orientation (PDAG → CPDAG completion), the unshielded-collider validity
//! check, and Markov-equivalence-class sizing. They operate on the typed-endpoint
//! [`deep_causality_topology::MixedGraph`] and carry no floating-point scalars —
//! they are pure structure over node indices and edges.
//!
//! The BRCD estimator itself (F-node augmentation, Gaussian scoring, posterior
//! ranking) builds on these and lands in a later change.

pub mod mec;
pub mod meek;
pub mod validity;
