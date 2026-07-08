/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Algebra/CommutativeMonoid.lean` (Mathlib `CommMonoid`
//! and the boolean semilattice).

use deep_causality_algebra::{
    BoundedSemilattice, CommutativeMonoid, Conjunction, Count, Disjunction, Monoid, Prob,
};

/// THEOREM_MAP: algebra.commutative_monoid.comm
#[test]
fn test_commutative_monoid_comm() {
    // x.combine(y) = y.combine(x)
    fn assert_comm<M: CommutativeMonoid + Clone + PartialEq + core::fmt::Debug>(x: M, y: M) {
        assert_eq!(x.clone().combine(y.clone()), y.combine(x));
    }
    assert_comm(Conjunction(true), Conjunction(false));
    assert_comm(Disjunction(true), Disjunction(false));
    assert_comm(Count(2), Count(7));
    assert_comm(Prob(0.3), Prob(0.9));
}

/// THEOREM_MAP: algebra.semilattice.idempotent
#[test]
fn test_semilattice_idempotent() {
    // x.combine(x) = x
    fn assert_idempotent<M: BoundedSemilattice + Clone + PartialEq + core::fmt::Debug>(x: M) {
        assert_eq!(x.clone().combine(x.clone()), x);
    }
    assert_idempotent(Conjunction(true));
    assert_idempotent(Conjunction(false));
    assert_idempotent(Disjunction(true));
    assert_idempotent(Disjunction(false));
}

/// THEOREM_MAP: algebra.semilattice.assoc
/// THEOREM_MAP: algebra.semilattice.comm
#[test]
fn test_semilattice_assoc_and_comm() {
    // Associativity: (x ∧ y) ∧ z = x ∧ (y ∧ z)
    let (x, y, z) = (Conjunction(true), Conjunction(false), Conjunction(true));
    assert_eq!(
        x.combine(y).combine(z),
        x.combine(y.combine(z))
    );
    // Commutativity: x ∧ y = y ∧ x (boolean ∧-semilattice)
    assert_eq!(x.combine(y), y.combine(x));
}
