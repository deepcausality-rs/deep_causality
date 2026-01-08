/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Adjunction, HKT, NoConstraint, Satisfies};

// Simplified Identity for testing
#[derive(Debug, PartialEq, Clone)]
struct Identity<T>(T);
struct IdentityWitness;
impl HKT for IdentityWitness {
    type Constraint = NoConstraint;
    type Type<T> = Identity<T>;
}

struct IdentityAdjunction;

impl Adjunction<IdentityWitness, IdentityWitness, ()> for IdentityAdjunction {
    fn left_adjunct<A, B, F>(_ctx: &(), _a: A, _f: F) -> Identity<B>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        Identity<A>: Satisfies<NoConstraint>,
        F: Fn(Identity<A>) -> B,
    {
        // Simplified - not fully implemented for test
        panic!("Not implemented for test")
    }

    fn right_adjunct<A, B, F>(_ctx: &(), _la: Identity<A>, _f: F) -> B
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        Identity<B>: Satisfies<NoConstraint>,
        F: FnMut(A) -> Identity<B>,
    {
        // Simplified - not fully implemented for test
        panic!("Not implemented for test")
    }

    fn unit<A>(_ctx: &(), a: A) -> Identity<Identity<A>>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint> + Clone,
        Identity<A>: Satisfies<NoConstraint>,
    {
        Identity(Identity(a))
    }

    fn counit<B>(_ctx: &(), fa: Identity<Identity<B>>) -> B
    where
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint> + Clone,
        Identity<B>: Satisfies<NoConstraint>,
    {
        fa.0.0
    }
}

#[test]
fn test_adjunction_identity() {
    let val = 42;
    let unit = IdentityAdjunction::unit(&(), val);
    assert_eq!(unit, Identity(Identity(42)));

    let counit = IdentityAdjunction::counit(&(), unit);
    assert_eq!(counit, 42);
}
