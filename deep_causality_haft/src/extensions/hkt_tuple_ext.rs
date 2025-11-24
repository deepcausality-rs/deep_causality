/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Bifunctor, HKT2Unbound, HKT3Unbound, Promonad};

// -----------------------------------------------------------------------------
// Tuple 2 (Pair) Extensions
// -----------------------------------------------------------------------------

/// `Tuple2Witness` acts as a witness for the `(A, B)` type constructor.
pub struct Tuple2Witness;

impl HKT2Unbound for Tuple2Witness {
    type Type<A, B> = (A, B);
}

impl Bifunctor<Tuple2Witness> for Tuple2Witness {
    fn bimap<A, B, C, D, F1, F2>(fab: (A, B), mut f1: F1, mut f2: F2) -> (C, D)
    where
        F1: FnMut(A) -> C,
        F2: FnMut(B) -> D,
    {
        (f1(fab.0), f2(fab.1))
    }
}

// -----------------------------------------------------------------------------
// Tuple 3 (Triple) Extensions
// -----------------------------------------------------------------------------

/// `Tuple3Witness` acts as a witness for the `(A, B, C)` type constructor.
pub struct Tuple3Witness;

impl HKT3Unbound for Tuple3Witness {
    type Type<A, B, C> = (A, B, C);
}

impl Promonad<Tuple3Witness> for Tuple3Witness {
    fn merge<A, B, C, F>(pa: (A, A, A), pb: (B, B, B), mut f: F) -> (C, C, C)
    where
        F: FnMut(A, B) -> C,
    {
        (f(pa.0, pb.0), f(pa.1, pb.1), f(pa.2, pb.2))
    }

    fn fuse<A, B, C>(_input_a: A, _input_b: B) -> (A, B, C) {
        // For a tuple, "fusion" without a merge function isn't well-defined in a general sense
        // unless we have a default C or can construct it.
        // However, Promonad::fuse is often used when C is a composite of A and B (like a Pair).
        // But here Type<A, B, C> is (A, B, C).
        // We can't produce (A, B, C) from A and B without C.
        // So this method is not applicable for a generic Triple where C is independent.
        // But if we interpret the Triple as a context where we want to combine them...
        // Actually, Promonad is best suited for things like `Interaction<A, B, C>`.
        // For a simple Tuple3, `fuse` is hard to implement meaningfully without extra info.
        // We will panic to indicate this limitation, similar to the test implementation.
        panic!("Tuple3Witness::fuse is not supported as C cannot be derived from A and B alone.")
    }
}
