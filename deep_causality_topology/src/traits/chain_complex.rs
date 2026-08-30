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
//! # This is a breaking change
//!
//! The re-export keeps the path `deep_causality_topology::ChainComplex` resolving, and keeps the
//! six homology items reachable through it. It does not keep the trait that name used to mean.
//! Two things changed for code outside this crate:
//!
//! - `CellType`, `CellIter`, `Metric`, `cells` and `uniform_lattice_layout` are no longer members
//!   of `ChainComplex`. An external `impl ChainComplex for MyType` that defines any of them fails
//!   with E0437, and a bound `K: ChainComplex` that names `K::CellType` or calls `k.cells(..)` no
//!   longer resolves. Move the impl, or the bound, to [`CellularComplex`](crate::CellularComplex),
//!   which carries those five and has `ChainComplex` as its supertrait.
//! - `betti_number_over` returns `Result<usize, HomologyError>` where it returned
//!   `Result<usize, TopologyError>`. This crate does not re-export `HomologyError`, and
//!   `TopologyError` has no `From<HomologyError>`, so a caller that propagated the old error with
//!   `?` has to depend on `deep_causality_homology` and handle `HomologyError` itself.
//!
//! Released as 0.8.0 for that reason. Inside the workspace the move cost two import lines in
//! `deep_causality_cfd`, both switched from `ChainComplex` to `CellularComplex`.

pub use deep_causality_homology::ChainComplex;
