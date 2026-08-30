/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The mod-2 chain, re-exported from `deep_causality_homology`.
//!
//! `Gf2Chain<W>` moved because it needs no complex to be well formed: its chain group is `𝔽₂^{n_k}`,
//! identified by the pair `(degree, len)`, and every operation it offers belongs to that group. It
//! is `Chain<T>` — which holds an `Arc<SimplicialComplex<T>>` — that is simplicial, and that one
//! stayed here.
//!
//! Re-exported so `use deep_causality_topology::Gf2Chain` keeps working.

pub use deep_causality_homology::Gf2Chain;
