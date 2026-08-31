/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Octonion;
use deep_causality_haft::{
    Convolutional, Foldable, Functor, HKT, LaxMonoidal, MonoidalApplicative, Semigroupal,
};

/// HKT witness for [`Octonion`], a functor over its eight component slots.
///
/// Implements `Functor` and `Foldable`, plus the lax monoidal stack: `Semigroupal`,
/// `LaxMonoidal`, `Convolutional` and `MonoidalApplicative`, which is where `apply` comes from.
///
/// `Pure`, `Applicative`, `Monad` and `CoMonad` are deliberately absent. Filling all eight slots
/// from one moved value is the diagonal, and `Pure::pure` cannot reach it under `NoConstraint`;
/// `CoMonad::extend` has no canonical cursor to walk a product with. The module docs carry the
/// argument in full.
pub struct OctonionWitness;

impl HKT for OctonionWitness {
    type Type<T> = Octonion<T>;
}

impl Functor<OctonionWitness> for OctonionWitness {
    /// Maps all eight components, scalar part first.
    fn fmap<A, B, F>(fa: Octonion<A>, mut f: F) -> Octonion<B>
    where
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

impl Semigroupal<OctonionWitness> for OctonionWitness {
    /// Pairs component with component. Total, and every component of `fa` and `fb` is moved
    /// exactly once, so no payload needs `Clone`.
    ///
    /// Componentwise is not one option among several; it is the only lawful `φ`. For
    /// `F(A) = A^S` over a finite index set `S`, Yoneda gives every natural `φ` the form
    /// `φ(fa, fb)_s = (fa_{u(s)}, fb_{v(s)})` for fixed endofunctions `u, v` of `S`, and the
    /// two unit laws force `u = v = id`. See the module docs.
    fn zip_with<A, B, C, F>(fa: Octonion<A>, fb: Octonion<B>, mut f: F) -> Octonion<C>
    where
        F: FnMut(A, B) -> C,
    {
        Octonion {
            s: f(fa.s, fb.s),
            e1: f(fa.e1, fb.e1),
            e2: f(fa.e2, fb.e2),
            e3: f(fa.e3, fb.e3),
            e4: f(fa.e4, fb.e4),
            e5: f(fa.e5, fb.e5),
            e6: f(fa.e6, fb.e6),
            e7: f(fa.e7, fb.e7),
        }
    }
}

impl LaxMonoidal<OctonionWitness> for OctonionWitness {
    /// `η : I → F I`. `Octonion<()>` has exactly one inhabitant across its 8 slots, so this is
    /// forced rather than chosen.
    fn unit() -> Octonion<()> {
        Octonion {
            s: (),
            e1: (),
            e2: (),
            e3: (),
            e4: (),
            e5: (),
            e6: (),
            e7: (),
        }
    }
}

/// Promises that `φ` associates under Day convolution. Discharged by the law tests in
/// `tests/extensions/`.
impl Convolutional<OctonionWitness> for OctonionWitness {}

/// `apply` is the provided method; there is no body to write.
impl MonoidalApplicative<OctonionWitness> for OctonionWitness {}
