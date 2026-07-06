/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Endomorphism.lean` (Mac Lane, CWM §I.1).

use deep_causality_haft::{Arrow, EndoArrow, Endomorphism, FnMorphism, Id, Lift};

/// THEOREM_MAP: haft.endo.monoid
#[test]
fn test_endo_monoid() {
    // End(T) is a monoid under composition: identity unit + associativity.
    let f = Lift::new(|x: i32| x + 1);
    for x in [0, 9, -4] {
        assert_eq!(
            Id::new().compose(Lift::new(|x: i32| x + 1)).run(x),
            f.run(x)
        );
        assert_eq!(
            Lift::new(|x: i32| x + 1).compose(Id::new()).run(x),
            f.run(x)
        );
        assert_eq!(
            Lift::new(|x: i32| x + 1)
                .compose(Lift::new(|x: i32| x * 2))
                .compose(Lift::new(|x: i32| x - 3))
                .run(x),
            Lift::new(|x: i32| x + 1)
                .compose(Lift::new(|x: i32| x * 2).compose(Lift::new(|x: i32| x - 3)))
                .run(x)
        );
    }
}

/// THEOREM_MAP: haft.endo.iterate_add
#[test]
fn test_endo_iterate_add() {
    // Monoid power law: iterate_n f (m + n) x = iterate_n f n (iterate_n f m x)
    // Checked on both endo layers: witness-level (Endomorphism / FnMorphism) and
    // value-level (EndoArrow).
    fn step(x: i32) -> i32 {
        x * 2 + 1
    }
    let arrow: fn(i32) -> i32 = step;
    for (m, n) in [(0, 3), (2, 2), (5, 0), (3, 4)] {
        let lhs = FnMorphism::iterate_n(&arrow, 1, m + n);
        let rhs = FnMorphism::iterate_n(&arrow, FnMorphism::iterate_n(&arrow, 1, m), n);
        assert_eq!(lhs, rhs);

        let lifted = Lift::new(step);
        assert_eq!(
            lifted.iterate_n(1, m + n),
            lifted.iterate_n(lifted.iterate_n(1, m), n)
        );
    }
}
