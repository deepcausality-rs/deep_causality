/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Monad.lean` (Moggi 1991; Wadler 1995).

use deep_causality_haft::{Applicative, Functor, Monad, OptionWitness, Pure};

/// THEOREM_MAP: haft.monad.laws
#[test]
fn test_monad_laws() {
    let f = |a: i32| if a > 0 { Some(a + 1) } else { None };
    let g = |b: i32| if b % 2 == 0 { Some(b * 10) } else { None };

    // Left identity: bind (pure a) f = f a
    for a in [5, -5] {
        assert_eq!(OptionWitness::bind(OptionWitness::pure(a), f), f(a));
    }

    // Right identity: bind m pure = m
    for m in [Some(5), None::<i32>] {
        assert_eq!(OptionWitness::bind(m, OptionWitness::pure), m);
    }

    // Associativity: bind (bind m f) g = bind m (|a| bind (f a) g)
    for m in [Some(5), Some(-5), None::<i32>] {
        assert_eq!(
            OptionWitness::bind(OptionWitness::bind(m, f), g),
            OptionWitness::bind(m, |a| OptionWitness::bind(f(a), g))
        );
    }
}

/// THEOREM_MAP: haft.monad.applicative_coherence
#[test]
fn test_monad_applicative_coherence() {
    // apply f_ab f_a = bind f_ab (|f| fmap f f_a) — the coherence owed by the
    // `Monad: Functor + Pure` hierarchy (in place of `Monad: Applicative`).
    let fa = Some(6);
    for fab in [Some(|a: i32| a * 7), None] {
        assert_eq!(
            OptionWitness::apply(fab, fa),
            OptionWitness::bind(fab, |f| OptionWitness::fmap(fa, f))
        );
    }
}
