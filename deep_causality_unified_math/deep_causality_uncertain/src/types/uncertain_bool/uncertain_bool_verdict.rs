/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Boolean [`Verdict`] instance — the other half of the collection-aggregation carrier bound
//! (`core.verdict.closure`); the MV half is on [`Uncertain`](crate::Uncertain).
//!
//! `UncertainBool<R>` is the **Boolean** class lifted pointwise: `meet = &`, `join = |`,
//! `complement = !`, bounds = the point masses at `false`/`true`. The instance is lazy — each
//! operation extends the graph, and the laws hold in distribution.

use crate::UncertainBool;
use deep_causality_algebra::Verdict;
use deep_causality_rand::RandScalar;

impl<R: RandScalar> Verdict for UncertainBool<R> {
    #[inline]
    fn bottom() -> Self {
        Self::point(false)
    }
    #[inline]
    fn top() -> Self {
        Self::point(true)
    }
    #[inline]
    fn meet(self, other: Self) -> Self {
        self & other
    }
    #[inline]
    fn join(self, other: Self) -> Self {
        self | other
    }
    #[inline]
    fn complement(self) -> Self {
        !self
    }
}
