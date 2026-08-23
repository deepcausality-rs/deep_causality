/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the ℚ laws proved in Lean.
//!
//! Lean source of truth: `lean/DeepCausalityFormal/Rational/Rational.lean` — this directory
//! mirrors it one-to-one. Lean proves the laws over Mathlib's `ℚ`; each test pins the crate's
//! `Rational<i64>` to the same statement at representative inputs. The `THEOREM_MAP` ids match
//! `lean/THEOREM_MAP.md`.

#[cfg(test)]
mod rational_tests;
