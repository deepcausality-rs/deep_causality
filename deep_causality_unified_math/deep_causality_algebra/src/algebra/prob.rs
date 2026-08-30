/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
//! `Prob(f64)` — a probability in `[0, 1]`: a product `CommutativeMonoid` (the `All` reducer) and
//! an MV-algebra [`Verdict`](crate::Verdict) (`meet = min`, `join = max`, `complement = 1 − p`).

use crate::algebra::operator::Combining;
use crate::{Associative, Commutative, CommutativeMonoid, Monoid, Verdict};

/// A probability in `[0, 1]`. The `Monoid` is the product t-norm (identity `1`), giving the
/// `AggregateLogic::All` reducer `∏ pᵢ`; commutative, not idempotent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Prob(pub f64);

impl Monoid for Prob {
    #[inline]
    fn empty() -> Self {
        Prob(1.0)
    }
    #[inline]
    fn combine(self, other: Self) -> Self {
        Prob(self.0 * other.0)
    }
}
// `p · q = q · p` — the marker `CommutativeMonoid` requires.
// The bare `Commutative` here promised `a * b == b * a` while meaning
// `x.combine(y) == y.combine(x)`. The claim is now stated on the operation it is
// actually about, above.
// `combine` is associative and commutative; the operator names which operation the
// laws are about, since `combine` is neither `Add` nor `Mul`.
impl Associative<Combining> for Prob {}
impl Commutative<Combining> for Prob {}
impl CommutativeMonoid for Prob {}

// MV-algebra (not Boolean): complement is `1 − p`, meet/join are the [0,1] bounded lattice.
impl Verdict for Prob {
    #[inline]
    fn bottom() -> Self {
        Prob(0.0)
    }
    #[inline]
    fn top() -> Self {
        Prob(1.0)
    }
    #[inline]
    fn meet(self, other: Self) -> Self {
        Prob(self.0.min(other.0))
    }
    #[inline]
    fn join(self, other: Self) -> Self {
        Prob(self.0.max(other.0))
    }
    #[inline]
    fn complement(self) -> Self {
        Prob(1.0 - self.0)
    }
}
