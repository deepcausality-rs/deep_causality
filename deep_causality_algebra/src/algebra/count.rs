/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
//! `Count(u64)` — the additive count monoid (the `AggregateLogic::Some(k)` carrier: fold then
//! threshold). Commutative but NOT idempotent, so deliberately not a `BoundedSemilattice`.

use crate::algebra::operator::Combining;
use crate::{Associative, Commutative, CommutativeMonoid, Monoid};

/// The count monoid `(ℕ, +, 0)`: identity `0`, `combine = +`. Commutative, not idempotent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Count(pub u64);

impl Monoid for Count {
    #[inline]
    fn empty() -> Self {
        Count(0)
    }
    #[inline]
    fn combine(self, other: Self) -> Self {
        Count(self.0 + other.0)
    }
}
// `m + n = n + m` — the marker `CommutativeMonoid` requires.
// The bare `Commutative` here promised `a * b == b * a` while meaning
// `x.combine(y) == y.combine(x)`. The claim is now stated on the operation it is
// actually about, above.
// `combine` is associative and commutative; the operator names which operation the
// laws are about, since `combine` is neither `Add` nor `Mul`.
impl Associative<Combining> for Count {}
impl Commutative<Combining> for Count {}
impl CommutativeMonoid for Count {}
// Deliberately NOT `Idempotent`/`BoundedSemilattice`: `Count(1).combine(Count(1)) = Count(2) ≠ Count(1)`.
