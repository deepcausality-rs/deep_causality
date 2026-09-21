/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The mod-2 chain, re-exported from `deep_causality_homology`.
//!
//! `Gf2Chain<W>` moved because it needs no complex to be well formed: its chain group is `𝔽₂^{n_k}`,
//! identified by the pair `(degree, len)`, and every operation it offers belongs to that group. It
//! is `Chain<R, G>` — which holds an `Arc<SimplicialComplex<R>>` — that is simplicial, and that one
//! stayed here.
//!
//! # This is a breaking change
//!
//! The re-export keeps the path `deep_causality_topology::Gf2Chain` resolving.
//!
//! `from_support`, `from_row`, `from_column`, `add`, `intersect` and `inner` return
//! `Result<_, HomologyError>`, and a binary operation on mismatched operands raises
//! `HomologyError::ChainGroupMismatch`. That guard compares length as well as degree, so two
//! same-degree chains of unequal length are rejected here rather than one layer down.
//!
//! This crate re-exports neither `HomologyError` nor `HomologyErrorEnum`, and `TopologyError` has
//! no `From<HomologyError>`, so a caller propagating the error with `?` depends on
//! `deep_causality_homology`.

pub use deep_causality_homology::Gf2Chain;
