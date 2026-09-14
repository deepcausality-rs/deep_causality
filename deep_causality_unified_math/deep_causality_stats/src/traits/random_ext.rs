/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The numerical draw, as an extension on any generator.

use crate::StandardUniform;
use deep_causality_rand::{Distribution, Iter, Map, Rng};

/// A numerical draw at the caller's scalar, on any generator.
///
/// This is an *extension* trait rather than a second `Rng`: it adds methods to every existing
/// generator and introduces no new bound to satisfy. `deep_causality_rand::Rng` deliberately does
/// not carry these, because a value on `[0, 1)` is a real number and that crate makes no claim
/// about real-valued distributions — it offers bits, words, Booleans and ranges.
///
/// Blanket-implemented, so every `Rng` gains it by importing this trait.
pub trait RandomExt: Rng {
    /// A uniform draw on `[0, 1)` at the caller's scalar.
    #[inline]
    fn random<T>(&mut self) -> T
    where
        StandardUniform: Distribution<T>,
    {
        StandardUniform.sample(self)
    }

    /// An iterator of uniform draws on `[0, 1)`.
    #[inline]
    fn random_iter<T>(&mut self) -> Iter<StandardUniform, &mut Self, T>
    where
        Self: Sized,
        StandardUniform: Distribution<T>,
    {
        StandardUniform.sample_iter(self)
    }

    /// Map a function over a stream of uniform draws.
    fn map_random<T, S, F>(&mut self, func: F) -> Map<StandardUniform, F, T, S>
    where
        StandardUniform: Distribution<T>,
        F: Fn(T) -> S,
    {
        Map {
            distr: StandardUniform,
            func,
            phantom: core::marker::PhantomData,
        }
    }
}

impl<R: Rng + ?Sized> RandomExt for R {}
