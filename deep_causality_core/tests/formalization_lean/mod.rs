/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the Core-layer laws proved in Lean.
//!
//! Lean source of truth: `lean/DeepCausalityFormal/Core/*.lean` — this directory mirrors that
//! tree one-to-one (`EffectLog.lean` ↔ `effect_log_tests.rs`, `CausalEffect.lean` ↔
//! `causal_effect_tests.rs`, …), matching the haft convention
//! (`deep_causality_haft/tests/formalization_lean/`). Each Lean file cites its literature
//! reference and the Rust source; each test carries the shared `THEOREM_MAP` id from
//! `lean/THEOREM_MAP.md` and checks the law empirically on the crate's real implementation at
//! representative inputs (Lean proves ∀; these tests pin the Rust code to the same statements).
//!
//! The pre-existing Kani harnesses (`deep_causality_core/tests/kani_proofs.rs`, bounded model
//! checks for the monad `left_id`/`left_zero`) are kept and their `THEOREM_MAP` `Kani` entries
//! preserved; new ids default to a witness here.

#[cfg(all(test, feature = "alloc"))]
mod alternatable_tests;
#[cfg(all(test, feature = "alloc"))]
mod causal_command_tests;
#[cfg(all(test, feature = "alloc"))]
mod causal_effect_tests;
#[cfg(all(test, feature = "alloc"))]
mod causal_flow_tests;
#[cfg(all(test, feature = "alloc"))]
mod causal_monad_tests;
#[cfg(all(test, feature = "alloc"))]
mod consistency_tests;
#[cfg(all(test, feature = "alloc"))]
mod csv_tests;
#[cfg(all(test, feature = "alloc"))]
mod effect_log_tests;
