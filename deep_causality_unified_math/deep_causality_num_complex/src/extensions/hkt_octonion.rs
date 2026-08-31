/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Octonion;
use deep_causality_haft::{Foldable, Functor, HKT, NoConstraint, Satisfies};

/// HKT witness for [`Octonion`], a functor over its component type.
///
/// See the module docs for why this stops at `Functor` and `Foldable`.
pub struct OctonionWitness;

impl HKT for OctonionWitness {
    type Constraint = NoConstraint;
    type Type<T> = Octonion<T>;
}

impl Functor<OctonionWitness> for OctonionWitness {
    /// Maps all eight components, scalar part first.
    fn fmap<A, B, F>(fa: Octonion<A>, mut f: F) -> Octonion<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        F: FnMut(A) -> B,
    {
        Octonion {
            s: f(fa.s),
            e1: f(fa.e1),
            e2: f(fa.e2),
            e3: f(fa.e3),
            e4: f(fa.e4),
            e5: f(fa.e5),
            e6: f(fa.e6),
            e7: f(fa.e7),
        }
    }
}

impl Foldable<OctonionWitness> for OctonionWitness {
    /// Folds over the components in the order `s`, `e1` through `e7`.
    fn fold<A, B, F>(fa: Octonion<A>, init: B, mut f: F) -> B
    where
        A: Satisfies<NoConstraint>,
        F: FnMut(B, A) -> B,
    {
        let acc = f(init, fa.s);
        let acc = f(acc, fa.e1);
        let acc = f(acc, fa.e2);
        let acc = f(acc, fa.e3);
        let acc = f(acc, fa.e4);
        let acc = f(acc, fa.e5);
        let acc = f(acc, fa.e6);
        f(acc, fa.e7)
    }
}
