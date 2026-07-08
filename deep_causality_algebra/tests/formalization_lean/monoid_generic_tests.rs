/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Algebra/MonoidGeneric.lean` (Mathlib `Monoid`).

use deep_causality_algebra::{Conjunction, Disjunction, Monoid};

fn assert_monoid_laws<M: Monoid + Clone + PartialEq + core::fmt::Debug>(x: M, y: M, z: M) {
    // left identity: empty().combine(x) = x
    assert_eq!(M::empty().combine(x.clone()), x.clone());
    // right identity: x.combine(empty()) = x
    assert_eq!(x.clone().combine(M::empty()), x.clone());
    // associativity: x.combine(y).combine(z) = x.combine(y.combine(z))
    assert_eq!(
        x.clone().combine(y.clone()).combine(z.clone()),
        x.clone().combine(y.clone().combine(z.clone()))
    );
}

/// THEOREM_MAP: algebra.monoid.left_id
/// THEOREM_MAP: algebra.monoid.right_id
/// THEOREM_MAP: algebra.monoid.assoc
#[test]
fn test_generic_monoid_laws() {
    assert_monoid_laws(Conjunction(true), Conjunction(false), Conjunction(true));
    assert_monoid_laws(Disjunction(false), Disjunction(true), Disjunction(false));
}
