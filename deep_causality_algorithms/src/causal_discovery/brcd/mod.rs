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
//!   (`brcd_linalg`).
//!
//! The **algorithm** itself — F-node augmentation ([`brcd_augment`]), the family
//! key canonicalization ([`brcd_cache`]), and the driver that assembles the
//! posterior and ranks the candidates ([`brcd_algo::brcd_run`]) — composes those
//! layers. [`brcd_run`] is the recommended entry point.
//!
//! # Performance
//!
//! [`brcd_run`] runs in three phases, and every optimization below is
//! **outcome-preserving**: each is either bit-identical arithmetic or an
//! order-independent reorganization, and the verification examples
//! (`examples/verification/brcd`) confirm the candidate ranking is reproduced
//! exactly against the reference.
//!
//! 1. **Structural enumeration (sequential).** Per candidate, enumerate valid cut
//!    configurations, F-node-augment each, size its Markov-equivalence class, and
//!    sample one representative DAG. This threads the RNG, so it stays sequential
//!    and deterministic. On well-oriented (mostly-directed) CPDAGs it is a small
//!    fraction of the total; the cost is exponential only in the size of the
//!    undirected components (the cut-configuration count).
//! 2. **Family scoring (the dominant cost).** Each unique `(node, parents)` family
//!    is scored **once** to its per-row log-likelihood. Two optimizations apply
//!    here:
//!    * the normal log-density is evaluated inline rather than through a
//!      `CausalTensor` round-trip ([`brcd_gaussian`]); and
//!    * the ridge fit streams the normal equations `XᵀX + λI`, `Xᵀz` from each
//!      row's `[1, parents]` design through a single reused buffer instead of
//!      materializing the design matrix as a `Vec<Vec<_>>` — eliminating roughly
//!      one heap allocation per row per fit, which was the dominant cost.
//!
//!    Because the families are independent, this phase runs in parallel across CPU
//!    cores with `rayon` when the crate is built with the **`parallel`** feature
//!    (mirroring the SURD decomposition loop); the result is identical to the
//!    sequential pass.
//! 3. **Posterior assembly (sequential, cheap).** A candidate with a single
//!    sampled DAG has no DAG mixture, so its `Σ_row logsumexp` collapses to a plain
//!    sum of the per-family scalar totals — skipping the length-`n_total` per-row
//!    vector entirely. Candidates with multiple configurations take the full
//!    per-row mixture path.

pub mod brcd_algo;
pub mod brcd_augment;
pub mod brcd_boss_bootstrap;
pub mod brcd_boss_config;
pub mod brcd_boss_cpdag;
pub mod brcd_boss_gst;
pub mod brcd_boss_learn;
pub mod brcd_boss_score;
pub mod brcd_boss_search;
pub mod brcd_cache;
pub mod brcd_config;
pub mod brcd_dirichlet;
pub mod brcd_error;
pub mod brcd_gate;
pub mod brcd_gaussian;
pub mod brcd_mapconfig;
pub mod brcd_mec;
pub mod brcd_meek;
pub mod brcd_result;
pub mod brcd_validity;

pub(crate) mod brcd_linalg;

// Driver entry point and its public types (the recommended access path).
pub use brcd_algo::brcd_run;
pub use brcd_boss_bootstrap::{BootstrapConfig, brcd_run_bootstrap};
pub use brcd_boss_config::{BOSS_LAMBDA_DEFAULT, BOSS_RIDGE_DEFAULT, BossConfig};
pub use brcd_boss_cpdag::dag_to_cpdag;
pub use brcd_boss_gst::Gst;
pub use brcd_boss_learn::boss_learn;
pub use brcd_boss_score::{BicScorer, FamilyScorer, family_bic};
pub use brcd_boss_search::{OrderSearchResult, best_order_search};
pub use brcd_config::{BrcdConfig, ConfigStrategy, FamilyKind};
pub use brcd_error::{BrcdError, BrcdErrorEnum};
pub use brcd_result::BrcdResult;
