/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The MV [`Verdict`] instance for the real carrier — one half of the collection-aggregation
//! carrier bound (`core.verdict.closure`). The Boolean half is on [`UncertainBool`], in that
//! type's own module.
//!
//! The instance is **lazy**: each operation extends the computation graph; the algebra laws hold
//! in distribution (per sample, the underlying operation is the corresponding pointwise `Verdict`
//! operation).
//!
//! `Uncertain<R>` is the **MV** class on `[0, 1]` lifted pointwise (`meet = min`, `join = max`,
//! `complement = 1 − p`), mirroring `Prob`/`f64`; the caller keeps sampled values in `[0, 1]`, as
//! with the scalar carriers.
//!
//! # Two instances, two types
//!
//! These are two different algebras, and that is the constraint that decides the carrier split.
//! One type cannot carry both a Boolean lattice and an MV lattice, so `Uncertain<bool>` collapsing
//! into `Uncertain<R>` would have left the Boolean instance with nowhere to live and broken
//! `Aggregatable: Verdict` downstream. As two blanket instances over two distinct local types they
//! are coherent — and every scalar gains the MV instance, where before only `f64` had it.

use crate::types::uncertain::uncertain_op_arithmetic::binary;
use crate::{ArithmeticOperator, Node, Uncertain};
use deep_causality_algebra::Verdict;
use deep_causality_rand::RandScalar;

impl<R: RandScalar> Verdict for Uncertain<R> {
    #[inline]
    fn bottom() -> Self {
        Uncertain::point(R::zero())
    }
    #[inline]
    fn top() -> Self {
        Uncertain::point(R::one())
    }
    #[inline]
    fn meet(self, other: Self) -> Self {
        binary(ArithmeticOperator::Min, self, other)
    }
    #[inline]
    fn join(self, other: Self) -> Self {
        binary(ArithmeticOperator::Max, self, other)
    }
    #[inline]
    fn complement(self) -> Self {
        Self::from_root_node(Node::ArithmeticOp {
            op: ArithmeticOperator::Sub,
            lhs: Uncertain::point(R::one()).into_tree(),
            rhs: self.into_tree(),
        })
    }
}
