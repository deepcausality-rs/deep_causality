/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Fixtures, assertions and oracles shared by the test suites.
//!
//! This module lives in `src` rather than `tests` because Bazel test targets cannot reach the
//! `tests` tree, but can reach the whole of `src`. It is `#[doc(hidden)]` at the root: it is
//! reachable, and it is not API.
//!
//! The scalar crossing itself is not here. `deep_causality_num::lift` owns it, for every primitive
//! float and integer, and these fixtures call it. What is here is only what a crossing does not
//! provide: an array of literals, a distribution, the per-precision tolerance table, the shared
//! assertions and the independent oracles.

pub mod assertions;
pub mod oracles;
pub mod precision;
pub mod samples;

use alloc::vec::Vec;
use deep_causality_num::{FromPrimitive, lift};

/// An array of `f64` literals lifted into the working scalar.
///
/// The elementwise form of [`deep_causality_num::lift`], which crosses one value. It lives here
/// rather than beside it because it returns a `Vec` and `deep_causality_num` is `no_std` over
/// `core` alone.
///
/// Test expectations are written as decimal literals and the suites run at three precisions, so a
/// fixture crosses into `T` at one place rather than at each element.
pub fn lift_array<T: FromPrimitive>(xs: &[f64]) -> Vec<T> {
    xs.iter().map(|&x| lift(x)).collect()
}

/// A uniform distribution over `n` outcomes.
///
/// Not a crossing: a fixture. `1/n` is computed at `f64` and lifted, so the entries are the same
/// value at every precision rather than the nearest representable quotient in each.
pub fn uniform<T: FromPrimitive>(n: usize) -> Vec<T> {
    let p = 1.0 / n as f64;
    (0..n).map(|_| lift(p)).collect()
}
