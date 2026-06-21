/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Memoization tables shared by the clique-picking counting recursion.
//!
//! Ported from the authoritative `cliquepicking_rs::memoization`, with one
//! deliberate deviation: the reference uses `BigUint::ZERO` as an
//! "uncomputed" sentinel inside plain `Vec<BigUint>`. Because this port is
//! generic over the count type `T` (which may legitimately take the value zero),
//! we instead use `Option<T>` — `None` means "not yet computed". This is both
//! cleaner and generic-safe.
//!
//! Three tables are kept:
//! * `count[subproblem]` — the per-subproblem AMO count (clique-tree flowers),
//! * `factorial[k]` — factorials `0! ..= n!`,
//! * `rho` — the rho recurrence keyed by its forbidden-size argument vector.

use deep_causality_num::{FromPrimitive, RealField};
use std::collections::HashMap;

/// Memoization state for one `count_amos` invocation, parameterized by the count
/// type `T`.
#[derive(Debug)]
pub(crate) struct Memoization<T> {
    /// Per-subproblem count cache; `None` = not yet computed.
    pub(crate) count: Vec<Option<T>>,
    /// Cache of the `rho` recurrence keyed by its argument vector.
    pub(crate) rho: HashMap<Vec<usize>, T>,
    /// Factorial cache; `factorial[k] = k!`, `None` = not yet computed.
    pub(crate) factorial: Vec<Option<T>>,
}

impl<T: RealField + FromPrimitive> Memoization<T> {
    /// Allocates the tables for a clique tree with `num_cliques` nodes over a
    /// graph with `n` vertices. The `count` table has `2 * num_cliques - 1`
    /// slots (one per directed clique-tree edge plus the whole-tree subproblem),
    /// and the factorial table covers `0 ..= n`.
    pub(crate) fn new(num_cliques: usize, n: usize) -> Memoization<T> {
        Memoization {
            count: vec![None; 2 * num_cliques - 1],
            rho: HashMap::new(),
            factorial: vec![None; n + 1],
        }
    }
}
