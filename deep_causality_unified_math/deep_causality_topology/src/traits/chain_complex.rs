/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The chain-complex trait, re-exported from `deep_causality_homology`.
//!
//! # Where the trait lives
//!
//! `ChainComplex` carries six homology items, none of which mentions a cell, a metric or a
//! coordinate — a quantum error-correcting code is a chain complex with none of those. It is
//! therefore defined in `deep_causality_homology` and re-exported here, so the path
//! `deep_causality_topology::ChainComplex` resolves.
//!
//! The geometric items — `CellType`, `CellIter`, `Metric`, `cells` and `uniform_lattice_layout` —
//! belong to [`CellularComplex`](crate::CellularComplex), which has `ChainComplex` as a
//! supertrait. Every complex in this crate implements both, so a bound naming `K::CellType` or
//! calling `k.cells(..)` is written against `CellularComplex`.
//!
//! # Error type
//!
//! `betti_number_over` returns `Result<usize, HomologyError>`. This crate re-exports neither
//! `HomologyError` nor a `From` impl for it on `TopologyError`, so a caller propagating it with
//! `?` depends on `deep_causality_homology` and handles `HomologyError` itself.

pub use deep_causality_homology::ChainComplex;
