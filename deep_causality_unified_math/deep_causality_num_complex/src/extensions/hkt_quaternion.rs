/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Quaternion;
use deep_causality_haft::{Foldable, Functor, HKT, NoConstraint, Satisfies};

/// HKT witness for [`Quaternion`], a functor over its component type.
///
/// See the module docs for why this stops at `Functor` and `Foldable`.
pub struct QuaternionWitness;

impl HKT for QuaternionWitness {
    type Constraint = NoConstraint;
    type Type<T> = Quaternion<T>;
}

impl Functor<QuaternionWitness> for QuaternionWitness {
    /// Maps all four components, scalar part first.
    fn fmap<A, B, F>(fa: Quaternion<A>, mut f: F) -> Quaternion<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        F: FnMut(A) -> B,
    {
        Quaternion {
            w: f(fa.w),
            x: f(fa.x),
            y: f(fa.y),
            z: f(fa.z),
        }
    }
}

impl Foldable<QuaternionWitness> for QuaternionWitness {
    /// Folds over the components in the order `w`, `x`, `y`, `z`.
    fn fold<A, B, F>(fa: Quaternion<A>, init: B, mut f: F) -> B
    where
        A: Satisfies<NoConstraint>,
        F: FnMut(B, A) -> B,
    {
        let acc = f(init, fa.w);
        let acc = f(acc, fa.x);
        let acc = f(acc, fa.y);
        f(acc, fa.z)
    }
}
