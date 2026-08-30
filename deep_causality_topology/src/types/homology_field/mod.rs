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
//! Re-exported so `use deep_causality_topology::HomologyField` keeps working.

pub use deep_causality_homology::HomologyField;
