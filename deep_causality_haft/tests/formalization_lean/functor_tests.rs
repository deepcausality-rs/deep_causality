/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Functor.lean` (Mac Lane, CWM §I.3).

use deep_causality_haft::{Functor, OptionWitness};

/// THEOREM_MAP: haft.functor.laws
#[test]
fn test_functor_laws() {
    // Identity: fmap id = id
    assert_eq!(OptionWitness::fmap(Some(5), |a: i32| a), Some(5));
    assert_eq!(OptionWitness::fmap(None::<i32>, |a: i32| a), None);

    // Composition: fmap (g ∘ f) = fmap g ∘ fmap f
    let f = |a: i32| a + 1;
    let g = |b: i32| b * 2;
    for x in [Some(5), None::<i32>] {
        assert_eq!(
            OptionWitness::fmap(x, |a| g(f(a))),
            OptionWitness::fmap(OptionWitness::fmap(x, f), g)
        );
    }
}
