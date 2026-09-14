/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_stats::{RandomExt, Rng};

/// A trait for generating random field elements uniformly.
///
/// Generic generation of both real (`f64`) and complex (`Complex<f64>`) scalars with components
/// in the range `[-0.5, 0.5]`.
///
/// The draw comes from `deep_causality_stats`, which owns the distributions; the machine words
/// behind it are the entropy crate's business and this crate no longer names that crate at all.
pub trait RandomField {
    /// Generate a random value with components in the range [-0.5, 0.5].
    fn generate_uniform<R: Rng>(rng: &mut R) -> Self;
}

impl RandomField for f64 {
    fn generate_uniform<R: Rng>(rng: &mut R) -> Self {
        // rng.random returns [0, 1), so output is [-0.5, 0.5)
        rng.random::<f64>() - 0.5
    }
}

impl<T> RandomField for Complex<T>
where
    T: RandomField + deep_causality_algebra::RealField + Copy,
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
