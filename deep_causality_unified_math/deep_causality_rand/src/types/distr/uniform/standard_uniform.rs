/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Distribution, Rng};

/// Uniform on the real interval `[0, 1)`.
///
/// This type samples **real scalars only**. Machine words come from [`StandardWord`] and Booleans
/// from [`StandardBool`], because those are different mathematical objects: a word is the
/// generator's own output and carries no algebra, a Boolean is the two-element Boolean algebra,
/// and a uniform on `[0, 1)` is an element of a real field.
///
/// Keeping all three under one type is what prevented this crate from taking a blanket
/// implementation over the algebra tower: `impl<T: RealField> Distribution<T> for StandardUniform`
/// is `error[E0119]` against a `Distribution<u64>` implementation, because coherence cannot prove
/// that `u64` will never be a real field.
#[derive(Clone, Copy, Debug, Default)]
pub struct StandardUniform;

/// Uniform over the machine words a generator produces.
///
/// A word is not a real number and this type claims no algebra for it. Drawing an index, a seed or
/// a raw bit pattern goes through here.
#[derive(Clone, Copy, Debug, Default)]
pub struct StandardWord;

/// Uniform over `{false, true}`: the two-element Boolean algebra.
#[derive(Clone, Copy, Debug, Default)]
pub struct StandardBool;

impl Distribution<u64> for StandardWord {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u64 {
        rng.next_u64()
    }
}

impl Distribution<u32> for StandardWord {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u32 {
        rng.next_u32()
    }
}

impl Distribution<bool> for StandardBool {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> bool {
        // The top bit, not a parity test: `% 2` reads one bit of information from a whole word
        // and is sensitive to a generator with a weak low bit.
        rng.next_u64() >> 63 == 1
    }
}
