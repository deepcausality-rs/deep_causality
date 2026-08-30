/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The chain-complex trait, re-exported from `deep_causality_homology`.
//!
//! # Where it went, and why
//!
//! `ChainComplex` used to be defined here, carrying eleven items: six about homology and five about
//! geometry. The homology half mentions no cell, no metric and no coordinate — a quantum
//! error-correcting code is a chain complex with none of those — so it now lives in
//! `deep_causality_homology`, which this crate depends on.
//!
//! The geometry half stayed, on [`CellularComplex`](crate::CellularComplex), which has
//! `ChainComplex` as a supertrait. Every complex here implements both, so a call that resolved
//! through one trait before resolves through the supertrait now.
//!
//! This module re-exports the moved name so that `use deep_causality_topology::ChainComplex` keeps
//! working. No downstream crate needed an import change.

pub use deep_causality_homology::ChainComplex;
