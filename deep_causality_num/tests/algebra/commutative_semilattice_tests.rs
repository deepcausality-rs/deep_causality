/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Law-tests for `CommutativeMonoid` / `Idempotent` / `BoundedSemilattice`. Mirrors
//! `DeepCausalityFormal/Num/CommutativeMonoid.lean` + `BoundedSemilattice.lean`.

use deep_causality_num::{
    BoundedSemilattice, CommutativeMonoid, Conjunction, Count, Disjunction, Monoid, Prob,
};

fn assert_commutative<M: CommutativeMonoid + Clone + PartialEq + core::fmt::Debug>(x: M, y: M) {
    assert_eq!(x.clone().combine(y.clone()), y.combine(x));
}
fn assert_idempotent<M: BoundedSemilattice + Clone + PartialEq + core::fmt::Debug>(x: M) {
    assert_eq!(x.clone().combine(x.clone()), x);
}

/// THEOREM_MAP: num.commutative_monoid.comm
#[test]
fn test_commutativity() {
    assert_commutative(Conjunction(true), Conjunction(false));
    assert_commutative(Disjunction(true), Disjunction(false));
    assert_commutative(Count(2), Count(7));
    assert_commutative(Prob(0.3), Prob(0.9));
}

/// THEOREM_MAP: num.semilattice.idempotent
#[test]
fn test_idempotence_of_semilattices() {
    assert_idempotent(Conjunction(true));
    assert_idempotent(Conjunction(false));
    assert_idempotent(Disjunction(true));
    assert_idempotent(Disjunction(false));
}

#[test]
fn test_count_is_commutative_but_not_idempotent() {
    // Commutative (proved above) but combine(x,x) != x — so not a BoundedSemilattice.
    assert_eq!(Count(1).combine(Count(1)), Count(2));
    assert_ne!(Count(1).combine(Count(1)), Count(1));
}

/// Order-independence of a bounded-semilattice fold over a multiset (assumption #1's algebra):
/// follows from commutativity + associativity + idempotence.
#[test]
fn test_semilattice_fold_order_independent() {
    let items = [Conjunction(true), Conjunction(true), Conjunction(false)];
    let forward = items
        .iter()
        .copied()
        .fold(Conjunction::empty(), Monoid::combine);
    let reversed = items
        .iter()
        .rev()
        .copied()
        .fold(Conjunction::empty(), Monoid::combine);
    assert_eq!(forward, reversed);
    assert_eq!(forward, Conjunction(false)); // All: one false -> false
}
