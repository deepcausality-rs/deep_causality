/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{CoMonad, Functor, HKT, NoConstraint, Satisfies};

// Mock Identity Comonad (since standard types don't implement Comonad easily in Rust)
#[derive(Debug, PartialEq, Clone)]
struct Identity<T>(T);

struct IdentityWitness;
impl HKT for IdentityWitness {
    type Constraint = NoConstraint;
    type Type<T> = Identity<T>;
}
impl Functor<IdentityWitness> for IdentityWitness {
    fn fmap<A, B, Func>(fa: Identity<A>, mut f: Func) -> Identity<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        Identity(f(fa.0))
    }
}
impl CoMonad<IdentityWitness> for IdentityWitness {
    fn extract<A>(wa: &Identity<A>) -> A
    where
        A: Satisfies<NoConstraint> + Clone,
    {
        wa.0.clone()
    }
    fn extend<A, B, Func>(wa: &Identity<A>, mut f: Func) -> Identity<B>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        Func: FnMut(&Identity<A>) -> B,
    {
        Identity(f(wa))
    }
}

#[test]
fn test_comonad_identity() {
    let id = Identity(10);

    // Extract
    assert_eq!(IdentityWitness::extract(&id), 10);

    // Extend
    let extended = IdentityWitness::extend(&id, |wa| IdentityWitness::extract(wa) + 1);
    assert_eq!(extended, Identity(11));
}
