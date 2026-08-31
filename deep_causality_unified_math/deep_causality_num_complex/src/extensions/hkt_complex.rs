/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Complex;
use deep_causality_haft::{Foldable, Functor, HKT, NoConstraint, Satisfies};

/// HKT witness for [`Complex`], a functor over its component type.
///
/// See the module docs for why this stops at `Functor` and `Foldable`.
pub struct ComplexWitness;

impl HKT for ComplexWitness {
    type Constraint = NoConstraint;
    type Type<T> = Complex<T>;
}

impl Functor<ComplexWitness> for ComplexWitness {
    /// Maps both components. The real part is mapped first, which is the order [`Foldable::fold`]
    /// visits them in.
    fn fmap<A, B, F>(fa: Complex<A>, mut f: F) -> Complex<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        F: FnMut(A) -> B,
    {
        Complex {
            re: f(fa.re),
            im: f(fa.im),
        }
    }
}

impl Foldable<ComplexWitness> for ComplexWitness {
    /// Folds over the components in the order `re`, `im`.
    fn fold<A, B, F>(fa: Complex<A>, init: B, mut f: F) -> B
    where
        A: Satisfies<NoConstraint>,
        F: FnMut(B, A) -> B,
    {
        let acc = f(init, fa.re);
        f(acc, fa.im)
    }
}
