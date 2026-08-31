/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Seeded generator shared by the HKT law suites in `tests/extensions/`.
//!
//! The functor, foldable and lax monoidal laws are universally quantified, so a fixture cannot
//! establish them. Each suite sweeps many inputs instead, and every one is derived from an
//! explicit seed so a counterexample is reproducible from its failure message.
//!
//! This lives in `src` rather than under `tests/` because the Bazel `rust_test_suite` compiles
//! each `*_tests.rs` file as its own crate, so test files cannot share a module with each other.

/// A seeded linear congruential generator.
///
/// Deterministic on purpose. A property test that cannot reproduce its own counterexample reports
/// a failure nobody can act on, so the seed is part of every case label.
#[derive(Debug, Clone)]
pub struct LawRng(u64);

impl LawRng {
    pub fn new(seed: u64) -> Self {
        Self(seed ^ 0x9E37_79B9_7F4A_7C15)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 11
    }

    /// A float in `[-1, 1]`, stepped in millionths.
    pub fn scalar(&mut self) -> f64 {
        (self.next_u64() % 2_000_001) as f64 / 1_000_000.0 - 1.0
    }

    /// A small integer in `[-2000, 2000]`.
    ///
    /// The monoidal laws are checked by exact equality, which floats cannot carry across a
    /// reassociation. Integer payloads leave no slop to explain away.
    pub fn small(&mut self) -> i64 {
        (self.next_u64() % 4001) as i64 - 2000
    }
}
