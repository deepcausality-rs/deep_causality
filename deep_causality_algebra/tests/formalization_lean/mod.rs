/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the Algebra-layer laws proved in Lean.
//!
//! Lean source of truth: `lean/DeepCausalityFormal/Algebra/*.lean` — this directory mirrors that
//! tree one-to-one (`Monoid.lean` ↔ `monoid_tests.rs`, `Ring.lean` ↔ `ring_tests.rs`, …). Each
//! Lean file cites its Mathlib reference; each test carries the shared `THEOREM_MAP` id from
//! `lean/THEOREM_MAP.md` and checks the law empirically on the crate's real traits and impls at
//! representative inputs (Lean proves ∀; these tests pin the Rust code to the same statements).
//! `f64` is the representative carrier for the real field, group, ring, module, division-algebra,
//! and scalar laws; the boolean and aggregation carriers witness the monoid, semilattice, and
//! verdict laws.

#[cfg(test)]
mod commutative_monoid_tests;
#[cfg(test)]
mod division_algebra_tests;
#[cfg(test)]
mod field_tests;
#[cfg(test)]
mod group_tests;
#[cfg(test)]
mod module_tests;
#[cfg(test)]
mod monoid_generic_tests;
#[cfg(test)]
mod monoid_tests;
#[cfg(test)]
mod ring_tests;
#[cfg(test)]
mod scalar_tests;
#[cfg(test)]
mod verdict_tests;
