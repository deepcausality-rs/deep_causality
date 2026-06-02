/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt::Debug;

/// A structured result for a BRCD root-cause discovery run.
///
/// Holds the candidate root-cause sets ranked by descending posterior (mirroring
/// the reference's `result["ranks"]`) alongside the aligned posterior weights.
/// Each candidate is a set of variable indices; for the common single-root case
/// (`k = 1`) every set has one element.
#[derive(Debug, Clone, PartialEq)]
pub struct BrcdResult<T> {
    /// Candidate root-cause sets, best first.
    ranks: Vec<Vec<usize>>,
    /// The (max-shifted, unnormalized) posterior aligned with `ranks`.
    posterior: Vec<T>,
}

impl<T> BrcdResult<T> {
    /// Creates a result from the ranked candidate sets and their aligned
    /// posterior weights (both in descending-posterior order).
    pub fn new(ranks: Vec<Vec<usize>>, posterior: Vec<T>) -> Self {
        Self { ranks, posterior }
    }
}

// --- Getters ----------------------------------------------------------------
impl<T> BrcdResult<T> {
    /// The candidate root-cause sets, ranked best first.
    pub fn ranks(&self) -> &[Vec<usize>] {
        &self.ranks
    }

    /// The posterior weights aligned with [`BrcdResult::ranks`].
    pub fn posterior(&self) -> &[T] {
        &self.posterior
    }

    /// The single top-ranked candidate set, or `None` if the result is empty.
    pub fn top(&self) -> Option<&[usize]> {
        self.ranks.first().map(Vec::as_slice)
    }
}

impl<T> std::fmt::Display for BrcdResult<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--- BRCD Root-Cause Ranking ---")?;
        for (rank, (candidate, weight)) in self.ranks.iter().zip(self.posterior.iter()).enumerate()
        {
            writeln!(f, "{}. {:?}  posterior={:?}", rank + 1, candidate, weight)?;
        }
        Ok(())
    }
}
