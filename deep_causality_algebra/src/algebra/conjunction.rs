/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
//! `Conjunction(bool)` — the boolean `∧` bounded semilattice (the `AggregateLogic::All` carrier).

use crate::algebra::operator::Combining;
use crate::{Associative, BoundedSemilattice, Commutative, CommutativeMonoid, Idempotent, Monoid};

/// The boolean conjunction (`∧`) monoid: identity `true`, `combine = &&`. A bounded semilattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conjunction(pub bool);

impl Monoid for Conjunction {
    #[inline]
    fn empty() -> Self {
        Conjunction(true)
    }
    #[inline]
    fn combine(self, other: Self) -> Self {
        Conjunction(self.0 && other.0)
    }
}
// `a ∧ b = b ∧ a` — the marker `CommutativeMonoid` requires.
// The bare `Commutative` here promised `a * b == b * a` while meaning
// `x.combine(y) == y.combine(x)`. The claim is now stated on the operation it is
// actually about, above.
// `combine` is associative and commutative; the operator names which operation the
// laws are about, since `combine` is neither `Add` nor `Mul`.
impl Associative<Combining> for Conjunction {}
impl Commutative<Combining> for Conjunction {}
impl CommutativeMonoid for Conjunction {}
impl Idempotent<Combining> for Conjunction {}
impl BoundedSemilattice for Conjunction {}
