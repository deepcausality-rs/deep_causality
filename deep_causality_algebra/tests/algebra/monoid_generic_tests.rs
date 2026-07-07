/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Law-tests (witnesses) for the generic `Monoid` — left identity, right identity, associativity —
//! over the aggregation carriers. Mirrors `DeepCausalityFormal/Algebra/MonoidGeneric.lean`.

use deep_causality_algebra::{Conjunction, Count, Disjunction, Monoid, Prob};

fn assert_monoid_laws<M: Monoid + Clone + PartialEq + core::fmt::Debug>(x: M, y: M, z: M) {
    // left identity
    assert_eq!(M::empty().combine(x.clone()), x.clone());
    // right identity
    assert_eq!(x.clone().combine(M::empty()), x.clone());
    // associativity
    assert_eq!(
        x.clone().combine(y.clone()).combine(z.clone()),
        x.clone().combine(y.clone().combine(z.clone()))
    );
}

/// THEOREM_MAP: algebra.monoid.left_id
/// THEOREM_MAP: algebra.monoid.right_id
/// THEOREM_MAP: algebra.monoid.assoc
#[test]
fn test_num_monoid_laws() {
    assert_monoid_laws(Conjunction(true), Conjunction(false), Conjunction(true));
    assert_monoid_laws(Disjunction(false), Disjunction(true), Disjunction(false));
    assert_monoid_laws(Count(2), Count(3), Count(5));
    assert_monoid_laws(Prob(0.5), Prob(0.4), Prob(0.2));
}

#[test]
fn test_conjunction_is_and() {
    assert_eq!(
        Conjunction(true).combine(Conjunction(false)),
        Conjunction(false)
    );
    assert_eq!(Conjunction::empty(), Conjunction(true));
}

#[test]
fn test_disjunction_is_or() {
    assert_eq!(
        Disjunction(false).combine(Disjunction(true)),
        Disjunction(true)
    );
    assert_eq!(Disjunction::empty(), Disjunction(false));
}

#[test]
fn test_count_is_add() {
    assert_eq!(Count(2).combine(Count(3)), Count(5));
    assert_eq!(Count::empty(), Count(0));
}

#[test]
fn test_prob_is_product() {
    assert_eq!(Prob(0.5).combine(Prob(0.4)), Prob(0.2));
    assert_eq!(Prob::empty(), Prob(1.0));
}
