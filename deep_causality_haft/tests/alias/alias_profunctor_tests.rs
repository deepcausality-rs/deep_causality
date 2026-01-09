/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{AliasProfunctor, HKT2Unbound, NoConstraint, Profunctor, Satisfies};

// Function Wrapper needed for Profunctor
struct Function<I, O>(Box<dyn Fn(I) -> O>);
struct FunctionWitness;

impl HKT2Unbound for FunctionWitness {
    type Constraint = NoConstraint;
    type Type<A, B> = Function<A, B>;
}

impl Profunctor<FunctionWitness> for FunctionWitness {
    fn dimap<A, B, C, D, F1, F2>(pab: Function<A, B>, f_pre: F1, f_post: F2) -> Function<C, D>
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
        Function(Box::new(move |c| {
            let a = (f_pre.borrow_mut())(c);
            let b = inner(a);
            (f_post.borrow_mut())(b)
        }))
    }
}

// Blanket impl for AliasProfunctor

#[test]
fn test_alias_profunctor_adapt() {
    let f = Function(Box::new(|x: i32| x + 1));
    // adapt alias dimap
    let adapted = FunctionWitness::adapt(
        f,
        |s: String| s.len() as i32, // Pre
        |n: i32| n.to_string(),     // Post
    );
    assert_eq!((adapted.0)("hello".to_string()), "6");
}

#[test]
fn test_alias_profunctor_preprocess() {
    let f = Function(Box::new(|x: i32| x + 1));
    // preprocess alias lmap
    let preprocessed = FunctionWitness::preprocess(f, |s: String| s.len() as i32);
    // Result is String -> i32 (input modified, output same)
    assert_eq!((preprocessed.0)("hi".to_string()), 3);
}

#[test]
fn test_alias_profunctor_postprocess() {
    let f = Function(Box::new(|x: i32| x + 1));
    // postprocess alias rmap
    let postprocessed = FunctionWitness::postprocess(f, |n: i32| n * 2);
    // Result is i32 -> i32 (input same, output modified)
    assert_eq!((postprocessed.0)(10), 22);
}
