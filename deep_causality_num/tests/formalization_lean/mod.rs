/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the numeric-core laws proved in Lean.
//!
//! Lean source of truth: `lean/DeepCausalityFormal/Num/*.lean` — this directory mirrors that
//! tree one-to-one (`Identity.lean` ↔ `identity_tests.rs`, `Integer.lean` ↔ `integer_tests.rs`,
//! `Cast.lean` ↔ `cast_tests.rs`, `Float106.lean` ↔ `float106_tests.rs`). Each test carries the
//! shared `THEOREM_MAP` id from `lean/THEOREM_MAP.md` and checks the law empirically on the
//! crate's real implementation at representative inputs (Lean proves ∀; these tests pin the Rust
//! code to the same statements).

#[cfg(test)]
mod cast_tests;
#[cfg(test)]
mod float106_tests;
#[cfg(test)]
mod identity_tests;
#[cfg(test)]
mod integer_tests;
