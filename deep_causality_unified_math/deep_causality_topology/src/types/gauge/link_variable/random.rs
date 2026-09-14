/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::RealField;
use deep_causality_num::Float106;
use deep_causality_num_complex::Complex;
use deep_causality_stats::{Distribution, RandScalar, Rng, StandardUniform};

/// A trait for generating random field elements uniformly.
///
/// Generic generation of both real and complex scalars with components in the range `[-0.5, 0.5]`.
///
/// The draw comes from `deep_causality_stats`, which owns the distributions; the machine words
/// behind it are the entropy crate's business and this crate no longer names that crate at all.
pub trait RandomField {
    /// Generate a random value with components in the range [-0.5, 0.5].
    fn generate_uniform<R: Rng>(rng: &mut R) -> Self;
}

/// The body, written once for every scalar.
///
/// A centred unit draw: `StandardUniform` on `[0, 1)` less one half. It is a `stats` draw at the
/// caller's scalar rather than an `f64` draw that everything else is converted from, so a lattice
/// at `Float106` gets a proposal with a double-double's worth of entropy behind it instead of a
/// widened `f64`.
#[inline]
fn centred_unit<T, R>(rng: &mut R) -> T
where
    T: RandScalar,
    R: Rng + ?Sized,
{
    let u: T = StandardUniform.sample(rng);
    u - T::from_f64(0.5).expect("one half converts to every supported scalar")
}

// The three impls below are what coherence leaves. A blanket `impl<T: RealField> RandomField for T`
// would collide with the `Complex<T>` impl beneath it: the compiler cannot prove that `Complex<T>`
// will never be a real field, so the two impls overlap as far as it can tell. The same rule keeps
// `SampleUniform`'s bindings in the entropy crate. What matters is that the *body* is written
// once — these are bindings, not implementations, and adding a scalar is one line.

impl RandomField for f32 {
    #[inline]
    fn generate_uniform<R: Rng>(rng: &mut R) -> Self {
        centred_unit(rng)
    }
}

impl RandomField for f64 {
    #[inline]
    fn generate_uniform<R: Rng>(rng: &mut R) -> Self {
        centred_unit(rng)
    }
}

impl RandomField for Float106 {
    #[inline]
    fn generate_uniform<R: Rng>(rng: &mut R) -> Self {
        centred_unit(rng)
    }
}

impl<T> RandomField for Complex<T>
where
    T: RandomField + RealField + Copy,
{
    fn generate_uniform<R: Rng>(rng: &mut R) -> Self {
        // Generate random real and imaginary parts
        // T::generate_uniform calls the implementation for the underlying real type
        // e.g., for Complex<f64>, T is f64
        let re = T::generate_uniform(rng);
        let im = T::generate_uniform(rng);
        Complex::new(re, im)
    }
}
