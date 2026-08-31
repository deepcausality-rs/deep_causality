/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Complex;
use deep_causality_haft::{
    Convolutional, Foldable, Functor, HKT, LaxMonoidal, MonoidalApplicative,
    NoConstraint, Satisfies, Semigroupal,
};

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

impl Semigroupal<ComplexWitness> for ComplexWitness {
    /// Pairs component with component. Total, and every component of `fa` and `fb` is moved
    /// exactly once, so no payload needs `Clone`.
    ///
    /// Componentwise is not one option among several; it is the only lawful `φ`. For
    /// `F(A) = A^S` over a finite index set `S`, Yoneda gives every natural `φ` the form
    /// `φ(fa, fb)_s = (fa_{u(s)}, fb_{v(s)})` for fixed endofunctions `u, v` of `S`, and the
    /// two unit laws force `u = v = id`. See the module docs.
    fn zip_with<A, B, C, F>(fa: Complex<A>, fb: Complex<B>, mut f: F) -> Complex<C>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        F: FnMut(A, B) -> C,
    {
        Complex {
            re: f(fa.re, fb.re),
            im: f(fa.im, fb.im),
        }
    }
}

impl LaxMonoidal<ComplexWitness> for ComplexWitness {
    /// `η : I → F I`. `Complex<()>` has exactly one inhabitant across its 2 slots, so this is
    /// forced rather than chosen.
    fn unit() -> Complex<()> {
        Complex {
            re: (),
            im: (),
        }
    }
}

/// Promises that `φ` associates under Day convolution. Discharged by the law tests in
/// `tests/extensions/`.
impl Convolutional<ComplexWitness> for ComplexWitness {}

/// `apply` is the provided method; there is no body to write.
impl MonoidalApplicative<ComplexWitness> for ComplexWitness {}
