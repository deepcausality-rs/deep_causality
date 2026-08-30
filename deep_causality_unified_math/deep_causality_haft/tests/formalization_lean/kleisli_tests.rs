/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Kleisli.lean` (the Kleisli category of a monad:
//! `id = pure`, `compose = bind`).

use deep_causality_haft::{Category, Kleisli, OptionWitness, Pure};

/// THEOREM_MAP: haft.kleisli.category_laws
#[test]
fn test_kleisli_category_laws() {
    type K = Kleisli<OptionWitness>;
    // Kleisli arrows `A -> Option<B>`.
    let f = |x: i32| Some(x + 1);
    let g = |x: i32| if x > 0 { Some(x * 2) } else { None };
    let h = |x: i32| Some(x - 3);

    for x in [-2, 0, 5, 7] {
        // Composition is monadic bind: compose(f, g)(x) = bind(f(x), g).
        assert_eq!(K::compose(f, g)(x), f(x).and_then(g));
        // Left identity: compose(id, g) = g.
        assert_eq!(K::compose(K::id::<i32>(), g)(x), g(x));
        // Right identity: compose(f, id) = f.
        assert_eq!(K::compose(f, K::id::<i32>())(x), f(x));
        // Associativity.
        let lhs = K::compose(K::compose(f, g), h)(x);
        let rhs = K::compose(f, K::compose(g, h))(x);
        assert_eq!(lhs, rhs);
    }

    // The category identity is the monad `pure`.
    assert_eq!(K::id::<i32>()(5), OptionWitness::pure(5));
}
