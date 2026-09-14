/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Distribution, Rng};

/// Uniform over the machine words a generator produces.
///
/// A word is not a real number. Drawing an index, a seed or
/// a raw bit pattern goes through here.
/// The uniform on `[0, 1)` is a different object and lives in
/// `deep_causality_stats` with the other distributions.
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
