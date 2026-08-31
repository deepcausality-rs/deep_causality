/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Dual;
use deep_causality_haft::{
    Convolutional, Foldable, Functor, HKT, LaxMonoidal, MonoidalApplicative, Semigroupal,
};

/// HKT witness for [`Dual`], a functor over its component type.
///
/// See the module docs for why this stops where it does, and in particular why `CoMonad` is
/// deferred rather than missing.
pub struct DualWitness;

impl HKT for DualWitness {
    type Type<T> = Dual<T>;
}

impl Functor<DualWitness> for DualWitness {
    /// Maps both channels independently. The real part is mapped first, which is the order
    /// [`Foldable::fold`] visits them in.
    ///
    /// This is the pair functor and carries no chain rule; see the module docs.
    fn fmap<A, B, F>(fa: Dual<A>, mut f: F) -> Dual<B>
    where
        F: FnMut(A) -> B,
    {
        Dual {
            re: f(fa.re),
            du: f(fa.du),
        }
    }
}

impl Foldable<DualWitness> for DualWitness {
    /// Folds over the channels in the order `re`, `du`.
    fn fold<A, B, F>(fa: Dual<A>, init: B, mut f: F) -> B
    where
        F: FnMut(B, A) -> B,
    {
        let acc = f(init, fa.re);
        f(acc, fa.du)
    }
}

impl Semigroupal<DualWitness> for DualWitness {
    /// Pairs channel with channel. Total, and every component of `fa` and `fb` is moved exactly
    /// once, so no payload needs `Clone`.
    ///
    /// Componentwise is the only lawful `φ` here, as for the Cayley-Dickson family: `Dual<A>` is
    /// `A^S` over the two-element index set, so by Yoneda every natural `φ` is
    /// `φ(fa, fb)_s = (fa_{u(s)}, fb_{v(s)})` for fixed endofunctions `u, v` of `S`, and the two
    /// unit laws force `u = v = id`. The swap variant
    /// `Dual { re: (fa.re, fb.du), du: (fa.du, fb.re) }` fails the left unit law.
    fn zip_with<A, B, C, F>(fa: Dual<A>, fb: Dual<B>, mut f: F) -> Dual<C>
    where
        F: FnMut(A, B) -> C,
    {
        Dual {
            re: f(fa.re, fb.re),
            du: f(fa.du, fb.du),
        }
    }
}

impl LaxMonoidal<DualWitness> for DualWitness {
    /// `η : I → F I`. `Dual<()>` has exactly one inhabitant, so this is forced rather than chosen.
    fn unit() -> Dual<()> {
        Dual { re: (), du: () }
    }
}

/// Promises that `φ` associates under Day convolution. Discharged by the law tests in
/// `tests/extensions/`.
impl Convolutional<DualWitness> for DualWitness {}

/// `apply` is the provided method; there is no body to write.
impl MonoidalApplicative<DualWitness> for DualWitness {}
