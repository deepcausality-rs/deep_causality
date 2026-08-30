/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Profunctor.lean` (Loregian, (Co)end Calculus §5).

use deep_causality_haft::{HKT2Unbound, NoConstraint, Profunctor, Satisfies};

// Local function carrier, mirroring the crate's canonical profunctor test.
struct Fun<I, O>(Box<dyn Fn(I) -> O>);
struct FunWitness;

impl HKT2Unbound for FunWitness {
    type Constraint = NoConstraint;
    type Type<A, B> = Fun<A, B>;
}

impl Profunctor<FunWitness> for FunWitness {
    fn dimap<A, B, C, D, F1, F2>(pab: Fun<A, B>, f_pre: F1, f_post: F2) -> Fun<C, D>
    where
        A: 'static + Satisfies<NoConstraint>,
        B: 'static + Satisfies<NoConstraint>,
        C: 'static + Satisfies<NoConstraint>,
        D: 'static + Satisfies<NoConstraint>,
        F1: FnMut(C) -> A + 'static,
        F2: FnMut(B) -> D + 'static,
    {
        let inner = pab.0;
        let f_pre = std::cell::RefCell::new(f_pre);
        let f_post = std::cell::RefCell::new(f_post);
        Fun(Box::new(move |c| {
            let a = (f_pre.borrow_mut())(c);
            (f_post.borrow_mut())(inner(a))
        }))
    }
}

/// THEOREM_MAP: haft.profunctor.laws
#[test]
fn test_profunctor_laws() {
    // Identity: dimap id id = id (checked pointwise — arrows compare by behavior)
    let p = Fun(Box::new(|x: i32| x + 1));
    let p_id = FunWitness::dimap(p, |c: i32| c, |b: i32| b);
    for x in [0, 5, -3] {
        assert_eq!((p_id.0)(x), x + 1);
    }

    // Composition with the contravariant twist:
    // dimap pre' post' (dimap pre post p) = dimap (pre ∘ pre') (post' ∘ post) p
    let pre = |c: i32| c * 2;
    let post = |b: i32| b + 10;
    let pre_p = |a: i32| a - 1;
    let post_p = |d: i32| d * 100;

    let lhs = FunWitness::dimap(
        FunWitness::dimap(Fun(Box::new(|x: i32| x + 1)), pre, post),
        pre_p,
        post_p,
    );
    let rhs = FunWitness::dimap(
        Fun(Box::new(|x: i32| x + 1)),
        move |a: i32| pre(pre_p(a)),
        move |b: i32| post_p(post(b)),
    );
    for x in [0, 5, -3] {
        assert_eq!((lhs.0)(x), (rhs.0)(x));
    }
}
