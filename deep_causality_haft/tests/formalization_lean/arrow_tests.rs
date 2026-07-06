/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Arrow.lean` (Hughes 2000; Paterson 2001).

use deep_causality_haft::{Arrow, Id, Lift};

fn arr_f(x: i32) -> i32 {
    x + 1
}
fn arr_g(x: i32) -> i32 {
    x * 2
}

/// THEOREM_MAP: haft.arrow.category_laws
#[test]
fn test_arrow_category_laws() {
    for x in [0, 5, -7] {
        // id >>> f = f
        assert_eq!(
            Id::new().compose(Lift::new(arr_f)).run(x),
            Lift::new(arr_f).run(x)
        );
        // f >>> id = f
        assert_eq!(
            Lift::new(arr_f).compose(Id::new()).run(x),
            Lift::new(arr_f).run(x)
        );
        // (f >>> g) >>> h = f >>> (g >>> h)
        let h = |x: i32| x - 3;
        assert_eq!(
            Lift::new(arr_f)
                .compose(Lift::new(arr_g))
                .compose(Lift::new(h))
                .run(x),
            Lift::new(arr_f)
                .compose(Lift::new(arr_g).compose(Lift::new(h)))
                .run(x)
        );
    }
}

/// THEOREM_MAP: haft.arrow.arr_functor
#[test]
fn test_arrow_arr_functor() {
    for x in [0, 5, -7] {
        // arr id = id
        assert_eq!(Lift::new(|a: i32| a).run(x), Id::new().run(x));
        // arr (g ∘ f) = arr f >>> arr g
        assert_eq!(
            Lift::new(|a: i32| arr_g(arr_f(a))).run(x),
            Lift::new(arr_f).compose(Lift::new(arr_g)).run(x)
        );
    }
}

/// THEOREM_MAP: haft.arrow.strength_laws
#[test]
fn test_arrow_strength_laws() {
    let samples = [(4, "ctx"), (-2, "other")];
    for (a, c) in samples {
        // first (arr f) = arr (f × id)
        assert_eq!(
            Lift::new(arr_f).first::<&str>().run((a, c)),
            Lift::new(|p: (i32, &str)| (arr_f(p.0), p.1)).run((a, c))
        );
        // first (f >>> g) = first f >>> first g
        assert_eq!(
            Lift::new(arr_f)
                .compose(Lift::new(arr_g))
                .first::<&str>()
                .run((a, c)),
            Lift::new(arr_f)
                .first::<&str>()
                .compose(Lift::new(arr_g).first::<&str>())
                .run((a, c))
        );
        // Exchange: first f >>> arr (id × g) = arr (id × g) >>> first f
        let g_str = |s: &str| s.len();
        assert_eq!(
            Lift::new(arr_f)
                .first::<&str>()
                .compose(Lift::new(move |p: (i32, &str)| (p.0, g_str(p.1))))
                .run((a, c)),
            Lift::new(move |p: (i32, &str)| (p.0, g_str(p.1)))
                .compose(Lift::new(arr_f).first::<usize>())
                .run((a, c))
        );
        // Unit: first f >>> arr fst = arr fst >>> f
        assert_eq!(
            Lift::new(arr_f)
                .first::<&str>()
                .compose(Lift::new(|p: (i32, &str)| p.0))
                .run((a, c)),
            Lift::new(|p: (i32, &str)| p.0)
                .compose(Lift::new(arr_f))
                .run((a, c))
        );
    }
    // Association: first (first f) >>> arr assoc = arr assoc >>> first f
    let assoc = |p: ((i32, u8), bool)| (p.0.0, (p.0.1, p.1));
    let input = ((3, 9u8), true);
    assert_eq!(
        Lift::new(arr_f)
            .first::<u8>()
            .first::<bool>()
            .compose(Lift::new(assoc))
            .run(input),
        Lift::new(assoc)
            .compose(Lift::new(arr_f).first::<(u8, bool)>())
            .run(input)
    );
}

/// THEOREM_MAP: haft.arrow.derived_combinators
#[test]
fn test_arrow_derived_combinators() {
    // second f = arr swap >>> first f >>> arr swap
    let input = ("ctx", 4);
    assert_eq!(
        Lift::new(arr_f).second::<&str>().run(input),
        Lift::new(|p: (&str, i32)| (p.1, p.0))
            .compose(Lift::new(arr_f).first::<&str>())
            .compose(Lift::new(|p: (i32, &str)| (p.1, p.0)))
            .run(input)
    );
    // f *** g = first f >>> second g
    let input = (4, 10);
    assert_eq!(
        Lift::new(arr_f).split(Lift::new(arr_g)).run(input),
        Lift::new(arr_f)
            .first::<i32>()
            .compose(Lift::new(arr_g).second::<i32>())
            .run(input)
    );
    // f &&& g = arr dup >>> (f *** g)
    assert_eq!(
        Lift::new(arr_f).fanout(Lift::new(arr_g)).run(6),
        Lift::new(|a: i32| (a, a))
            .compose(Lift::new(arr_f).split(Lift::new(arr_g)))
            .run(6)
    );
}
