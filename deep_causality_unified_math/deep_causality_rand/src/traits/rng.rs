/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Distribution, Fill, RngCore, SampleRange, SampleUniform};
use crate::{Iter, Map};

impl<T: Rng> Rng for &mut T {}

pub trait Rng: RngCore {
    /// A raw machine word. Not a real number, so it is a separate operation from [`Rng::random`].
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

    /// An iterator of raw machine words, the word-sided sibling of [`Rng::random_iter`].
    #[inline]
    fn random_word_iter<T>(&mut self) -> Iter<crate::StandardWord, &mut Self, T>
    where
        Self: Sized,
        crate::StandardWord: Distribution<T>,
    {
        crate::StandardWord.sample_iter(self)
    }

    #[track_caller]
    fn random_range<T, R>(&mut self, range: R) -> T
    where
        T: SampleUniform,
        R: SampleRange<T>,
    {
        assert!(!range.is_empty(), "cannot sample empty range");
        range.sample_single(self).unwrap()
    }

    #[inline]
    #[track_caller]
    fn random_bool(&mut self, p: f64) -> bool {
        if !(0.0..=1.0).contains(&p) {
            panic!("p={} is outside range [0.0, 1.0]", p);
        }
        self.next_u64() as f64 / (u64::MAX as f64) <= p
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

    /// Map over raw machine words, the word-sided sibling of [`Rng::map`].
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
