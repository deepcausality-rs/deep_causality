/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Bifunctor.lean` (Mac Lane, CWM §II.3).

use deep_causality_haft::{Bifunctor, ResultUnboundWitness};

/// THEOREM_MAP: haft.bifunctor.laws
#[test]
fn test_bifunctor_laws() {
    let cases: [Result<i32, String>; 2] = [Ok(3), Err("e".to_string())];

    for x in cases {
        // Identity: bimap id id = id
        assert_eq!(
            ResultUnboundWitness::bimap(x.clone(), |a: i32| a, |b: String| b),
            x
        );

        // Composition: bimap (f' ∘ f) (g' ∘ g) = bimap f' g' ∘ bimap f g
        let f = |a: i32| a + 1;
        let fp = |c: i32| c * 2;
        let g = |b: String| b.len();
        let gp = |d: usize| d + 10;
        assert_eq!(
            ResultUnboundWitness::bimap(x.clone(), |a| fp(f(a)), |b| gp(g(b))),
            ResultUnboundWitness::bimap(ResultUnboundWitness::bimap(x, f, g), fp, gp)
        );
    }
}
