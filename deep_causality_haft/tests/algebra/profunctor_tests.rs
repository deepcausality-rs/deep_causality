/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{HKT2Unbound, Profunctor};

// Local Function wrapper
struct Function<I, O>(Box<dyn Fn(I) -> O>);
struct FunctionWitness;

impl HKT2Unbound for FunctionWitness {
    type Type<A, B> = Function<A, B>;
}

impl Profunctor<FunctionWitness> for FunctionWitness {
    fn dimap<A, B, C, D, F1, F2>(pab: Function<A, B>, f_pre: F1, f_post: F2) -> Function<C, D>
    where
        F1: FnMut(C) -> A + 'static,
        F2: FnMut(B) -> D + 'static,
        A: 'static,
        B: 'static,
        C: 'static,
        D: 'static,
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

#[test]
fn test_profunctor_function() {
    // A function from i32 -> i32
    let f = Function(Box::new(|x: i32| x + 1));

    // dimap: Pre-process input (string -> int), Post-process output (int -> string)
    let dimapped = FunctionWitness::dimap(
        f,
        |s: String| s.len() as i32, // Pre: String length
        |n: i32| n.to_string(),     // Post: Convert to string
    );

    assert_eq!((dimapped.0)("hello".to_string()), "6"); // len 5 + 1 = 6
}
