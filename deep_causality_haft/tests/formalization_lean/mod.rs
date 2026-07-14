/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the Haft-layer laws proved in Lean.
//!
//! Lean source of truth: `lean/DeepCausalityFormal/Haft/*.lean` — this directory mirrors that
//! tree one-to-one (`Functor.lean` ↔ `functor_tests.rs`, …). Each Lean file cites its
//! reference (Mac Lane CWM; McBride–Paterson 2008; Hughes 2000; Atkey 2009; Uustalu–Vene
//! 2008; Moggi 1991); each test carries the shared `THEOREM_MAP` id from
//! `lean/THEOREM_MAP.md` and checks the law empirically on the crate's real implementation
//! at representative inputs (Lean proves ∀; these tests pin the Rust code to the same
//! statements). `Hkt.lean` is a definitional bridge with no theorems, hence no test file.

#[cfg(test)]
mod adjunction_tests;
#[cfg(test)]
mod applicative_tests;
#[cfg(test)]
mod arrow_choice_tests;
#[cfg(all(test, feature = "alloc"))]
mod arrow_term_tests;
#[cfg(test)]
mod arrow_tests;
#[cfg(test)]
mod bifunctor_tests;
#[cfg(test)]
mod category_tests;
// `Cofree` is alloc-only, like `Free`.
#[cfg(all(test, feature = "alloc"))]
mod cofree_tests;
#[cfg(test)]
mod comonad_tests;
#[cfg(test)]
mod effect_system_tests;
#[cfg(test)]
mod either_tests;
#[cfg(test)]
mod endomorphism_tests;
#[cfg(test)]
mod foldable_tests;
#[cfg(all(test, feature = "alloc"))]
mod free_monad_tests;
#[cfg(test)]
mod functor_tests;
#[cfg(all(test, feature = "alloc"))]
mod interpreter_tests;
#[cfg(test)]
mod io_tests;
#[cfg(test)]
mod kleisli_tests;
#[cfg(test)]
mod monad_tests;
#[cfg(test)]
mod monoidal_merge_tests;
#[cfg(test)]
mod monoidal_tests;
#[cfg(test)]
mod morphism_tests;
#[cfg(test)]
mod natural_iso_tests;
#[cfg(test)]
mod parametric_monad_tests;
#[cfg(test)]
mod profunctor_tests;
#[cfg(test)]
mod pure_tests;
#[cfg(test)]
mod signatures_tests;
#[cfg(test)]
mod traversable_tests;
