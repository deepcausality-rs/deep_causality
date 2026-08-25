/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The canonical embedding ℤ ↪ ℚ.

use crate::{Rational, RationalScalar};
use core::marker::PhantomData;
use deep_causality_algebra::{Hom, Injective, RingHom};

/// The canonical embedding **ℤ ↪ ℚ**, `n ↦ n/1`.
///
/// This is the map that motivated giving morphisms a home. It is injective and **not** surjective —
/// `1/2` has no integer preimage — so it can never be an isomorphism, and the pair-shaped
/// `*Iso` traits could not express it. The body already existed as
/// [`Rational::from_integer`](crate::Rational::from_integer); this states what it is.
///
/// # Why it is a `RingHom`
///
/// `(a + b)/1 = a/1 + b/1`, `(a·b)/1 = (a/1)·(b/1)`, and `1 ↦ 1/1`. Every rational built this way
/// is already in canonical form, so no normalisation step can perturb the laws.
///
/// ℚ is the field of fractions of ℤ, and this is the embedding that makes it so.
///
/// # Panics
///
/// [`apply`](Hom::apply) panics on `T::MIN`. A canonical `Rational<T>` excludes a numerator of
/// `T::MIN` so that negation stays total, so the domain this map covers is
/// `[T::MIN + 1, T::MAX]`. That is the fixed-width edge of the promise to ℤ, and
/// [`Rational::try_from_integer`](crate::Rational::try_from_integer) is the total form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IntToRational<T>(PhantomData<T>);

impl<T> IntToRational<T> {
    /// The embedding.
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Hom for IntToRational<T>
where
    T: RationalScalar,
{
    type Domain = T;
    type Codomain = Rational<T>;

    /// # Panics
    ///
    /// On `T::MIN`, which a canonical `Rational<T>` cannot hold as a numerator.
    fn apply(&self, n: T) -> Rational<T> {
        Rational::from_integer(n)
    }
}

impl<T> RingHom for IntToRational<T> where T: RationalScalar {}

// Injective: `a/1 == b/1` forces `a == b`, since both sides are already canonical.
impl<T> Injective for IntToRational<T> where T: RationalScalar {}

// Deliberately NOT `Surjective`: `1/2` is not `n/1` for any integer `n`.
