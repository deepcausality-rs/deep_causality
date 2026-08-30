/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the 𝔽₂ laws proved in Lean.
//!
//! Lean source of truth: `lean/DeepCausalityFormal/Linear/RankNullity.lean` — this directory
//! mirrors it one-to-one. Lean proves the statements over Mathlib's `ZMod 2` and `Matrix`; each
//! test pins the crate's `PackedGf2<u64>` and `rank_gf2` to the same statement at concrete
//! matrices. The `THEOREM_MAP` ids match `lean/THEOREM_MAP.md`.

#[cfg(test)]
mod rank_nullity_tests;
