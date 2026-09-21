/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The coefficient field, re-exported from `deep_causality_homology`.
//!
//! `HomologyField` and its `rank_of` moved with `ChainComplex`: choosing ℚ or 𝔽₂ is a statement
//! about a computation over a boundary matrix, and a boundary matrix needs no geometry. The
//! `widen_to_dense_i64` helper that lived beside it went further down, to
//! `deep_causality_linear::csr_i8_to_dense_i64`, next to the five conversions already there.
//!
//! # Error type
//!
//! The re-export keeps the path `deep_causality_topology::HomologyField` resolving. `rank_of`
//! returns `Result<usize, HomologyError>`; this crate re-exports neither `HomologyError` nor
//! `HomologyErrorEnum`, and `TopologyError` has no `From<HomologyError>`, so a caller propagating
//! the error with `?` depends on `deep_causality_homology`.

pub use deep_causality_homology::HomologyField;
