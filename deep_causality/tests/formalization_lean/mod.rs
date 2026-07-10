/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the causaloid-layer laws proved in Lean.
//!
//! Lean source of truth: `lean/DeepCausalityFormal/Core/*.lean`. This directory is the
//! **main-crate half** of the Core witness mirror: Core Lean files whose Rust realization lives in
//! `deep_causality` (the causaloid fixpoint, Verdict closure, graph algebra, and catamorphism
//! layers) witness here, against the real `Causaloid` — the files whose realization lives in
//! `deep_causality_core` witness in `deep_causality_core/tests/formalization_lean/`. Same
//! conventions: one `<mechanism>_tests.rs` per Lean file, each test carries the shared
//! `THEOREM_MAP` id from `lean/THEOREM_MAP.md` and checks the law empirically on the crate's real
//! implementation at representative inputs (Lean proves ∀; these tests pin the Rust code to the
//! same statements).

mod catamorphism_tests;
mod causaloid_tests;
mod graph_algebra_tests;
mod verdict_closure_tests;
