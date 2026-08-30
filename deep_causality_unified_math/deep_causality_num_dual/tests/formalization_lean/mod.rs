/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the dual-number laws proved in Lean.
//!
//! Lean source of truth: `lean/DeepCausalityFormal/Dual/Dual.lean` — this directory mirrors that
//! tree one-to-one (`Dual.lean` ↔ `dual_tests.rs`). Each test carries the shared `THEOREM_MAP`
//! id from `lean/THEOREM_MAP.md` and checks the law empirically on the crate's real `Dual`
//! implementation at representative inputs (Lean proves ∀; these tests pin the Rust code to the
//! same statements).

#[cfg(test)]
mod dual_tests;
