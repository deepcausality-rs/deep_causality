/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the complex-algebra laws proved in Lean.
//!
//! Lean source of truth: `lean/DeepCausalityFormal/Complex/*.lean` — this directory mirrors that
//! tree one-to-one (`Complex.lean` ↔ `complex_tests.rs`, `Quaternion.lean` ↔
//! `quaternion_tests.rs`). Each test carries the shared `THEOREM_MAP` id from
//! `lean/THEOREM_MAP.md` and checks the law empirically on the crate's real
//! `Complex`/`Quaternion` types at representative inputs (Lean proves ∀; these tests pin the
//! Rust code to the same statements). Floating-point comparisons use an epsilon tolerance where
//! rounding applies.

#[cfg(test)]
mod complex_tests;
#[cfg(test)]
mod quaternion_tests;
