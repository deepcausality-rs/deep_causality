/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Applicative.lean` (McBride–Paterson 2008).

use deep_causality_haft::{Applicative, Functor, OptionWitness, Pure};

/// THEOREM_MAP: haft.applicative.laws
#[test]
fn test_applicative_laws() {
    fn double(x: i32) -> i32 {
        x * 2
    }
    fn inc(x: i32) -> i32 {
        x + 1
    }

    // Identity: pure id <*> v = v
    for v in [Some(7), None::<i32>] {
        assert_eq!(OptionWitness::apply(Some(|a: i32| a), v), v);
    }

    // Homomorphism: pure f <*> pure x = pure (f x)
    assert_eq!(
        OptionWitness::apply(OptionWitness::pure(double as fn(i32) -> i32), Some(5)),
        OptionWitness::pure(double(5))
    );

    // Interchange: u <*> pure y = pure (|f| f y) <*> u
    for u in [Some(double as fn(i32) -> i32), None] {
        assert_eq!(
            OptionWitness::apply(u, OptionWitness::pure(9)),
            OptionWitness::apply(OptionWitness::pure(|f: fn(i32) -> i32| f(9)), u)
        );
    }

    // Composition: pure (∘) <*> u <*> v <*> w = u <*> (v <*> w)
    // (the law ABSENT from the trait docstring — deviation D1)
    let compose = |f: fn(i32) -> i32| move |g: fn(i32) -> i32| move |a: i32| f(g(a));
    for (u, v, w) in [
        (
            Some(double as fn(i32) -> i32),
            Some(inc as fn(i32) -> i32),
            Some(5),
        ),
        (None, Some(inc as fn(i32) -> i32), Some(5)),
        (Some(double as fn(i32) -> i32), None, Some(5)),
        (
            Some(double as fn(i32) -> i32),
            Some(inc as fn(i32) -> i32),
            None,
        ),
    ] {
        let s1 = OptionWitness::apply(Some(compose), u);
        let s2 = OptionWitness::apply(s1, v);
        let lhs = OptionWitness::apply(s2, w);
        let rhs = OptionWitness::apply(u, OptionWitness::apply(v, w));
        assert_eq!(lhs, rhs);
    }
}

/// THEOREM_MAP: haft.applicative.functor_compat
#[test]
fn test_applicative_functor_compat() {
    // fmap f x = pure f <*> x — apply and fmap present one functor.
    let f = |a: i32| a - 4;
    for x in [Some(10), None::<i32>] {
        assert_eq!(
            OptionWitness::fmap(x, f),
            OptionWitness::apply(OptionWitness::pure(f), x)
        );
    }
}
