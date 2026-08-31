/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Quaternion;
use deep_causality_haft::{
    Convolutional, Foldable, Functor, HKT, LaxMonoidal, MonoidalApplicative, Semigroupal,
};

/// HKT witness for [`Quaternion`], a functor over its four component slots.
///
/// Implements `Functor` and `Foldable`, plus the lax monoidal stack: `Semigroupal`,
/// `LaxMonoidal`, `Convolutional` and `MonoidalApplicative`, which is where `apply` comes from.
///
/// `Pure`, `Applicative`, `Monad` and `CoMonad` are deliberately absent. Filling all four slots
/// from one moved value is the diagonal, and `Pure::pure` cannot reach it under `NoConstraint`;
/// `CoMonad::extend` has no canonical cursor to walk a product with. The module docs carry the
/// argument in full.
pub struct QuaternionWitness;

impl HKT for QuaternionWitness {
    type Type<T> = Quaternion<T>;
}

impl Functor<QuaternionWitness> for QuaternionWitness {
    /// Maps all four components, scalar part first.
    fn fmap<A, B, F>(fa: Quaternion<A>, mut f: F) -> Quaternion<B>
    where
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
        F: FnMut(B, A) -> B,
    {
        let acc = f(init, fa.w);
        let acc = f(acc, fa.x);
        let acc = f(acc, fa.y);
        f(acc, fa.z)
    }
}

impl Semigroupal<QuaternionWitness> for QuaternionWitness {
    /// Pairs component with component. Total, and every component of `fa` and `fb` is moved
    /// exactly once, so no payload needs `Clone`.
    ///
    /// Componentwise is not one option among several; it is the only lawful `φ`. For
    /// `F(A) = A^S` over a finite index set `S`, Yoneda gives every natural `φ` the form
    /// `φ(fa, fb)_s = (fa_{u(s)}, fb_{v(s)})` for fixed endofunctions `u, v` of `S`, and the
    /// two unit laws force `u = v = id`. See the module docs.
    fn zip_with<A, B, C, F>(fa: Quaternion<A>, fb: Quaternion<B>, mut f: F) -> Quaternion<C>
    where
        F: FnMut(A, B) -> C,
    {
        Quaternion {
            w: f(fa.w, fb.w),
            x: f(fa.x, fb.x),
            y: f(fa.y, fb.y),
            z: f(fa.z, fb.z),
        }
    }
}

impl LaxMonoidal<QuaternionWitness> for QuaternionWitness {
    /// `η : I → F I`. `Quaternion<()>` has exactly one inhabitant across its 4 slots, so this is
    /// forced rather than chosen.
    fn unit() -> Quaternion<()> {
        Quaternion {
            w: (),
            x: (),
            y: (),
            z: (),
        }
    }
}

/// Promises that `φ` associates under Day convolution. Discharged by the law tests in
/// `tests/extensions/`.
impl Convolutional<QuaternionWitness> for QuaternionWitness {}

/// `apply` is the provided method; there is no body to write.
impl MonoidalApplicative<QuaternionWitness> for QuaternionWitness {}
