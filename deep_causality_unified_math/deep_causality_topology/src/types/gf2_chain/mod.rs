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
//! The re-export keeps the path `deep_causality_topology::Gf2Chain` resolving. It does not keep the
//! signatures. `from_support`, `from_row`, `add`, `intersect` and `inner` return
//! `Result<_, HomologyError>` where they returned `Result<_, TopologyError>`, and the mismatch a
//! binary operation raises is now `HomologyError::ChainGroupMismatch` rather than
//! `TopologyError::DimensionMismatch`. That guard also widened: it compares length as well as
//! degree, so two same-degree chains of unequal length are rejected here instead of one layer down.
//! This crate re-exports neither `HomologyError` nor `HomologyErrorEnum`, and `TopologyError` has
//! no `From<HomologyError>`, so a caller that propagated the old error with `?` has to depend on
//! `deep_causality_homology`. Released as 0.8.0 for that reason.
//!
//! `from_column` is new and additive.

pub use deep_causality_homology::Gf2Chain;
