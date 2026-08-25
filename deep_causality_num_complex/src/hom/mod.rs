/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The canonical embedding ℝ ↪ ℂ, and the projection ℂ → ℝ that is not its inverse.

use crate::Complex;
use core::marker::PhantomData;
use deep_causality_algebra::{Hom, Injective, RealField, RingHom, Surjective};

/// The canonical embedding **ℝ ↪ ℂ**, `r ↦ r + 0i`.
///
/// Injective and not surjective — `i` has no real preimage — so, like ℤ ↪ ℚ, it is a monomorphism
/// rather than an isomorphism. The body already existed as `ConjugateScalar::from_real`.
///
/// # Why it is a `RingHom`
///
/// `(a + b) + 0i = (a + 0i) + (b + 0i)`, and `(a·b) + 0i = (a + 0i)·(b + 0i)` because the imaginary
/// cross terms are zero. `1 ↦ 1 + 0i`.
pub struct RealToComplex<R>(PhantomData<R>);

impl<R> RealToComplex<R> {
    /// The embedding.
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<R> Hom for RealToComplex<R>
where
    R: RealField,
{
    type Domain = R;
    type Codomain = Complex<R>;

    fn apply(&self, re: R) -> Complex<R> {
        Complex::new(re, R::zero())
    }
}

impl<R> RingHom for RealToComplex<R> where R: RealField {}
impl<R> Injective for RealToComplex<R> where R: RealField {}
// Deliberately NOT `Surjective`: `i` is not `r + 0i` for any real `r`.

/// The projection **ℂ → ℝ**, `a + bi ↦ a`.
///
/// The other side of the embedding, and a good illustration of why the properties are separate
/// traits rather than bundled into one notion of "map".
///
/// It is [`Surjective`] — every real is the real part of some complex — and neither [`Injective`]
/// (`1 + i` and `1 + 2i` share a real part) nor a [`RingHom`]:
///
/// ```text
/// re(i · i) = re(-1) = -1     but     re(i) · re(i) = 0 · 0 = 0
/// ```
///
/// So it preserves addition but not multiplication. Stating that in the type is the point: a
/// generic conversion bounded on `RingHom` will not accept this map, which is correct.
pub struct ComplexToReal<R>(PhantomData<R>);

impl<R> ComplexToReal<R> {
    /// The projection.
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<R> Hom for ComplexToReal<R>
where
    R: RealField,
{
    type Domain = Complex<R>;
    type Codomain = R;

    fn apply(&self, z: Complex<R>) -> R {
        z.re
    }
}

impl<R> Surjective for ComplexToReal<R> where R: RealField {}
// Deliberately neither `Injective` nor `RingHom`; see the type documentation.

// The derives would place `R: Debug`, `R: Eq`, … bounds on these marker types because of the
// `PhantomData`. A map carries no data, so its parameter should not have to satisfy anything —
// and `R: Eq` would be unreachable, since `RealField` is satisfied by the floats and no float is
// `Eq`. The impls are written out to keep the parameter free.

macro_rules! marker_impls {
    ($name:ident, $label:literal) => {
        impl<R> core::fmt::Debug for $name<R> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str($label)
            }
        }

        impl<R> Clone for $name<R> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<R> Copy for $name<R> {}

        impl<R> Default for $name<R> {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<R> PartialEq for $name<R> {
            fn eq(&self, _: &Self) -> bool {
                true
            }
        }

        impl<R> Eq for $name<R> {}
    };
}

marker_impls!(RealToComplex, "RealToComplex");
marker_impls!(ComplexToReal, "ComplexToReal");
