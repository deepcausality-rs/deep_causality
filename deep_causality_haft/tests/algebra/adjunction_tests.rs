/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Adjunction, HKT};

// Simplified Identity for testing
#[derive(Debug, PartialEq, Clone)]
struct Identity<T>(T);
struct IdentityWitness;
impl HKT for IdentityWitness {
    type Type<T> = Identity<T>;
}

struct IdentityAdjunction;

impl Adjunction<IdentityWitness, IdentityWitness> for IdentityAdjunction {
    fn unit<A>(a: A) -> Identity<Identity<A>> {
        Identity(Identity(a))
    }

    fn counit<A>(fa: Identity<Identity<A>>) -> A {
        fa.0.0
    }

    fn left_adjunct<A, B, F>(_a: A, _f: F) -> Identity<B>
    where
        F: Fn(Identity<A>) -> B,
    {
        // Simplified - not fully implemented for test
        panic!("Not implemented for test")
    }

    fn right_adjunct<A, B, F>(_la: Identity<A>, _f: F) -> B
    where
        F: Fn(A) -> Identity<B>,
    {
        // Simplified - not fully implemented for test
        panic!("Not implemented for test")
    }
}

#[test]
fn test_adjunction_identity() {
    let val = 42;
    let unit = IdentityAdjunction::unit(val);
    assert_eq!(unit, Identity(Identity(42)));

    let counit = IdentityAdjunction::counit(unit);
    assert_eq!(counit, 42);
}
