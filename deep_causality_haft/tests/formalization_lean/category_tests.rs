/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Category.lean` (the function category `Fun` — the
//! semantic category the value-level `Arrow` runs in).

use deep_causality_haft::{Category, Fun};

/// THEOREM_MAP: haft.category.laws
#[test]
fn test_fun_category_laws() {
    let f = |x: i32| x + 1;
    let g = |x: i32| x * 2;
    let h = |x: i32| x - 3;
    for x in [-2, 0, 5, 7] {
        // Left identity: compose(id, g) = g.
        assert_eq!(Fun::compose(Fun::id::<i32>(), g)(x), g(x));
        // Right identity: compose(f, id) = f.
        assert_eq!(Fun::compose(f, Fun::id::<i32>())(x), f(x));
        // Associativity: compose(compose(f, g), h) = compose(f, compose(g, h)).
        let lhs = Fun::compose(Fun::compose(f, g), h)(x);
        let rhs = Fun::compose(f, Fun::compose(g, h))(x);
        assert_eq!(lhs, rhs);
    }
}
