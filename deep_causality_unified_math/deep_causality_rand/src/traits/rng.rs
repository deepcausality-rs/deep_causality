/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Distribution, Fill, RandScalar, RngCore, SampleRange, SampleUniform};
use crate::{Iter, Map};

impl<T: Rng> Rng for &mut T {}

pub trait Rng: RngCore {
    /// A raw machine word. Not a real number, so it is a separate operation from the numerical draw,
    /// which lives in `deep_causality_stats` as `RandomExt::random` — this crate states no
    /// distribution.
    ///
    /// This is what an index draw, a seed draw or a bit-pattern draw wants.
    #[inline]
    fn random_word<T>(&mut self) -> T
    where
        crate::StandardWord: Distribution<T>,
    {
        crate::StandardWord.sample(self)
    }

    /// A Boolean draw: the two-element Boolean algebra, not a real field.
    #[inline]
    fn random_boolean(&mut self) -> bool {
        crate::StandardBool.sample(self)
    }

    /// An iterator of raw machine words, the word-sided sibling of `RandomExt::random_iter`.
    #[inline]
    fn random_word_iter<T>(&mut self) -> Iter<crate::StandardWord, &mut Self, T>
    where
        Self: Sized,
        crate::StandardWord: Distribution<T>,
    {
        crate::StandardWord.sample_iter(self)
    }

    #[track_caller]
    /// A uniform draw from a range, at whatever scalar the range holds.
    ///
    /// `K` is the tower marker and is inferred from the range; no call site names it.
    ///
    /// # Panics
    ///
    /// If the range is empty, because there is no value to return and no sentinel that would not
    /// be a lie. `#[track_caller]` puts the panic at the call site rather than here.
    ///
    /// A range written with distinct endpoints can still be empty, and a narrow scalar is where
    /// that happens: `1000.0..1001.0` at `BFloat16` has both endpoints at 1000, because the
    /// spacing there is 8. The endpoints are a fact about the scalar, not about this function, so
    /// widening the range or nudging an endpoint would put mass where the scalar says there is
    /// none.
    ///
    /// [`Uniform::new`](crate::Uniform::new) is the fallible form of the same construction and
    /// returns `UniformDistributionError::EmptyRange` for these endpoints. The two differ on
    /// purpose: this one is an infallible convenience over a range the caller wrote, and that one
    /// is a constructor that can be handed bounds computed at runtime.
    fn random_range<T, K, R>(&mut self, range: R) -> T
    where
        T: SampleUniform<K>,
        R: SampleRange<T, K>,
    {
        assert!(!range.is_empty(), "cannot sample empty range");
        range.sample_single(self).unwrap()
    }

    /// A Bernoulli trial at probability `p`, stated in the caller's scalar.
    ///
    /// The draw is a unit value compared against `p`. That makes `p = 0` exact — no unit draw is
    /// below zero, so an impossible event cannot fire. The ratio form this replaced,
    /// `word / u64::MAX <= p`, read `0 <= 0` on a zero word and fired an event of probability
    /// zero once in every `2^64` draws.
    ///
    /// `p = 1` is special-cased rather than left to the comparison, for the reason
    /// [`RandScalar::rand_float_gen`] documents: a narrow significand can round a draw from
    /// `[0, 1)` onto exactly `1.0`, and `1.0 < 1.0` is false. At `f32` that happens about once in
    /// `2^25` draws, so without the case a certain event would occasionally fail to occur.
    ///
    /// Resolution follows the scalar: `p` is honoured to as many bits as the unit draw carries —
    /// 24 at `f32`, 53 at `f64`, 106 at `Float106`.
    #[inline]
    #[track_caller]
    fn random_bool<T: RandScalar>(&mut self, p: T) -> bool {
        if !(T::zero()..=T::one()).contains(&p) {
            panic!(
                "p={} is outside range [0.0, 1.0]",
                p.to_f64().unwrap_or(f64::NAN)
            );
        }
        if p == T::one() {
            return true;
        }
        T::rand_float_gen(self) < p
    }

    #[inline]
    #[track_caller]
    fn random_ratio(&mut self, numerator: u32, denominator: u32) -> bool {
        if denominator == 0 || numerator > denominator {
            panic!(
                "p={}/{} is outside range [0.0, 1.0]",
                numerator, denominator
            );
        }
        self.next_u64() % (denominator as u64) < (numerator as u64)
    }

    fn sample<T, D: Distribution<T>>(&mut self, distr: D) -> T {
        distr.sample(self)
    }

    fn sample_iter<T, D>(&mut self, distr: D) -> Iter<D, &mut Self, T>
    where
        D: Distribution<T>,
        Self: Sized,
    {
        distr.sample_iter(self)
    }

    #[track_caller]
    fn fill<T: Fill + ?Sized>(&mut self, dest: &mut T) {
        dest.fill(self)
    }

    /// Map over raw machine words, the word-sided sibling of `RandomExt::map_random`.
    fn map_word<T, S, F>(&mut self, func: F) -> Map<crate::StandardWord, F, T, S>
    where
        crate::StandardWord: Distribution<T>,
        F: Fn(T) -> S,
    {
        Map {
            distr: crate::StandardWord,
            func,
            phantom: core::marker::PhantomData,
        }
    }
}
