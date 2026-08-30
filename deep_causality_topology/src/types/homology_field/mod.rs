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
//! # This is a breaking change
//!
//! The re-export keeps the path `deep_causality_topology::HomologyField` resolving, and the enum
//! itself is unchanged. Its one method is not: `rank_of` returns `Result<usize, HomologyError>`
//! where it returned `Result<usize, TopologyError>`. This crate re-exports neither `HomologyError`
//! nor `HomologyErrorEnum`, and `TopologyError` has no `From<HomologyError>`, so a caller that
//! propagated the old error with `?` has to depend on `deep_causality_homology`. Released as 0.8.0
//! for that reason.
//!
//! The `widen_to_dense_i64` helper named above was private, so its move to
//! `deep_causality_linear::csr_i8_to_dense_i64` costs no caller anything.

pub use deep_causality_homology::HomologyField;
