/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
//! `Disjunction(bool)` — the boolean `∨` bounded semilattice (the `AggregateLogic::Any` carrier).

use crate::algebra::operator::Combining;
use crate::{Associative, BoundedSemilattice, Commutative, CommutativeMonoid, Idempotent, Monoid};

/// The boolean disjunction (`∨`) monoid: identity `false`, `combine = ||`. A bounded semilattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Disjunction(pub bool);

impl Monoid for Disjunction {
    #[inline]
    fn empty() -> Self {
        Disjunction(false)
    }
    #[inline]
    fn combine(self, other: Self) -> Self {
        Disjunction(self.0 || other.0)
    }
}
// `a ∨ b = b ∨ a` — the marker `CommutativeMonoid` requires.
// The bare `Commutative` here promised `a * b == b * a` while meaning
// `x.combine(y) == y.combine(x)`. The claim is now stated on the operation it is
// actually about, above.
// `combine` is associative and commutative; the operator names which operation the
// laws are about, since `combine` is neither `Add` nor `Mul`.
impl Associative<Combining> for Disjunction {}
impl Commutative<Combining> for Disjunction {}
impl CommutativeMonoid for Disjunction {}
impl Idempotent for Disjunction {}
impl BoundedSemilattice for Disjunction {}
