/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{AliasCoMonad, CoMonad, Functor, HKT, NoConstraint, Satisfies};

// Identity CoMonad
#[derive(Debug, PartialEq, Clone)]
struct Identity<T>(T);
struct IdentityWitness;
impl HKT for IdentityWitness {
    type Constraint = NoConstraint;
    type Type<T> = Identity<T>;
}

impl Functor<IdentityWitness> for IdentityWitness {
    fn fmap<A, B, Func>(m_a: Identity<A>, f: Func) -> Identity<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        // Identity Functor: just apply f to inner value
        let mut f = f;
        Identity(f(m_a.0))
    }
}

impl CoMonad<IdentityWitness> for IdentityWitness {
    fn extract<A>(fa: &Identity<A>) -> A
    where
        A: Satisfies<NoConstraint> + Clone,
    {
        fa.0.clone()
    }

    fn extend<A, B, F>(fa: &Identity<A>, mut f: F) -> Identity<B>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        F: FnMut(&Identity<A>) -> B,
    {
        Identity(f(fa))
    }
}

// AliasCoMonad is blanket impl

#[test]
fn test_alias_comonad_observe() {
    let val = Identity(42);
    // observe alias extract
    let res = IdentityWitness::observe(&val);
    assert_eq!(res, 42);
}

#[test]
fn test_alias_comonad_propagate() {
    let val = Identity(21);
    // propagate alias extend
    let res = IdentityWitness::propagate(&val, |id_a| id_a.0 * 2);
    assert_eq!(res, Identity(42));
}
