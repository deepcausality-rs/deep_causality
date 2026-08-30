/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Adjunction, AliasAdjunction, HKT, NoConstraint, Satisfies};

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
    fn left_adjunct<A, B, F>(_ctx: &(), a: A, f: F) -> Identity<B>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        Identity<A>: Satisfies<NoConstraint>,
        F: Fn(Identity<A>) -> B,
    {
        let res_b = f(Identity(a));
        Identity(res_b)
    }

    fn right_adjunct<A, B, F>(_ctx: &(), la: Identity<A>, mut f: F) -> B
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        Identity<B>: Satisfies<NoConstraint>,
        F: FnMut(A) -> Identity<B>,
    {
        let a = la.0;
        let id_b = f(a);
        id_b.0
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

// AliasAdjunction is automatically implemented for any Adjunction types

#[test]
fn test_alias_adjunction_integrate() {
    let val = 10;
    // integrate alias left_adjunct
    // (L<A> -> B) -> (A -> R<B>)
    // Here L=Id, R=Id. (Id<i32> -> i32) -> (i32 -> Id<i32>)
    let res = IdentityAdjunction::integrate(&(), val, |x: Identity<i32>| x.0 * 2);
    assert_eq!(res, Identity(20));
}

#[test]
fn test_alias_adjunction_differentiate() {
    let val = Identity(10);
    // differentiate alias right_adjunct
    // (A -> R<B>) -> (L<A> -> B)
    // Here L=Id, R=Id. (i32 -> Id<i32>) -> (Id<i32> -> i32)
    let res = IdentityAdjunction::differentiate(&(), val, |x: i32| Identity(x * 2));
    assert_eq!(res, 20);
}
