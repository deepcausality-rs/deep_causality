/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::Real;

/// One slot per bit of an observation count: slot `i` holds the sum of exactly `2ⁱ` observations.
const SLOTS: usize = usize::BITS as usize;

/// A sum formed as a balanced binary tree.
///
/// # Why not left to right
///
/// A running total grows with the number of observations while each addend does not. Once the
/// total is large enough that an addend falls below its last place, every further addend rounds
/// away and the total stops moving — while staying finite, which is what makes the failure quiet.
/// Measured at `BFloat16` over seeded draws from N(100, 5), a left-to-right mean reads 114.5 at 256
/// observations, 32.75 at a thousand and 3.28 at ten thousand, against a true 100.
///
/// Summing a balanced tree keeps every addition between two partial sums of the same size, so the
/// error grows with `log n` rather than with `n`. The arrangement here is the one a binary counter
/// makes: an observation carries into slot 0, and an occupied slot adds and carries upward exactly
/// as a bit does. Which slot a partial sum lands in is therefore decided by the count alone, which
/// is what lets a streaming caller and a slice caller agree to the last bit.
///
/// The cost is `n − 1` additions, the same as a left-to-right fold, in `usize::BITS` slots of
/// memory rather than one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PairwiseSum<T> {
    slots: [Option<T>; SLOTS],
}

impl<T: Real> PairwiseSum<T> {
    /// A sum over no observations.
    pub(crate) fn new() -> Self {
        Self {
            slots: [None; SLOTS],
        }
    }

    /// Adds one observation.
    pub(crate) fn push(&mut self, value: T) {
        let mut carry = value;
        for slot in self.slots.iter_mut() {
            match slot.take() {
                Some(held) => carry = held + carry,
                None => {
                    *slot = Some(carry);
                    return;
                }
            }
        }
        // Reaching here needs every slot occupied, which takes 2^usize::BITS observations — one
        // more than a `usize` count can hold. Kept total rather than dropping the value.
        self.slots[SLOTS - 1] = Some(carry);
    }

    /// Which slots hold a partial sum, as a bit pattern.
    ///
    /// Equal to the number of observations, because a slot holds exactly `2ⁱ` of them and the
    /// carry is a binary increment. That identity is what makes two callers pushing the same
    /// sequence associate their additions the same way, so it is observable rather than implied.
    pub(crate) fn occupancy(&self) -> u128 {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.is_some())
            .map(|(i, _)| 1u128 << i)
            .sum()
    }

    /// The sum of everything added, smallest partial sum first.
    ///
    /// Folded from the first occupied slot rather than from zero, so that a sum which is
    /// genuinely negative zero stays one: `(−0) + (−0)` is `−0`, but `(+0) + (−0)` is `+0`, so a
    /// seeded fold would turn every negative zero positive on its way out. Zero is returned only
    /// when nothing was added, which is the one case that has no first slot.
    pub(crate) fn total(&self) -> T {
        let mut occupied = self.slots.iter().flatten();
        match occupied.next() {
            Some(&first) => occupied.fold(first, |acc, &partial| acc + partial),
            None => T::zero(),
        }
    }
}

/// Sums `f` over `xs` as a balanced tree. See [`PairwiseSum`] for why not left to right.
///
/// The mapping is taken here rather than applied by the caller so that a derived quantity — a
/// deviation, a squared deviation, a scaled observation, a surprisal — is summed the same way an
/// observation is, without materialising an intermediate slice for it. A term the caller wants to
/// leave out contributes `T::zero()`, which is what leaving it out of a sum means.
pub(crate) fn pairwise_sum<T, F>(xs: &[T], f: F) -> T
where
    T: Real,
    F: Fn(T) -> T,
{
    let mut sum = PairwiseSum::new();
    for &x in xs {
        sum.push(f(x));
    }
    sum.total()
}
