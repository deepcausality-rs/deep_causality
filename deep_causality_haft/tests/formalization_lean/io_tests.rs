/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Io.lean` (Moggi 1991, over `Result`; laws hold
//! on the `run` denotation, per the crate's io module docs).

use deep_causality_haft::{IoAction, io_fail, io_pure};

/// THEOREM_MAP: haft.io.monad_laws
#[test]
fn test_io_monad_laws() {
    type E = &'static str;

    // Left identity: run (pure a >>= f) = run (f a)
    let f = |x: i32| io_pure::<i32, E>(x + 1);
    assert_eq!(io_pure::<i32, E>(5).and_then(f).run(), f(5).run());

    // Right identity: run (m >>= pure) = run m
    assert_eq!(
        io_pure::<i32, E>(7).and_then(io_pure::<i32, E>).run(),
        io_pure::<i32, E>(7).run()
    );
    let failing = io_fail::<i32, E>("err");
    assert_eq!(
        failing.and_then(io_pure::<i32, E>).run(),
        io_fail::<i32, E>("err").run()
    );

    // Associativity: run ((m >>= f) >>= g) = run (m >>= |x| f x >>= g)
    let g = |x: i32| io_pure::<i32, E>(x * 2);
    assert_eq!(
        io_pure::<i32, E>(3).and_then(f).and_then(g).run(),
        io_pure::<i32, E>(3).and_then(|x| f(x).and_then(g)).run()
    );
}
