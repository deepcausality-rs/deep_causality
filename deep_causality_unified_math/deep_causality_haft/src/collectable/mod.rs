/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::HKT;

/// Building a structure from a sequence of values: the direction [`Foldable`](crate::Foldable)
/// cannot run.
///
/// `Foldable::fold` takes `F<A>` to a summary. Nothing on the surface takes a sequence of `A` back
/// to an `F<A>`, and a caller that produces values one at a time — a draw per sample, a row per
/// record — has to name a concrete container to put them in. Naming one is what forces a function
/// that should be generic in its carrier to pick a crate's container and stop being generic.
///
/// # Why no existing capability does this
///
/// Each of the neighbours covers a different direction, and the gap is not an oversight in any of
/// them.
///
/// - [`Foldable`](crate::Foldable) consumes a structure. It is this trait's opposite, and the two
///   compose into a round trip.
/// - [`Pure`](crate::Pure) builds a structure holding exactly one value. There is no operation on a
///   bare container to extend it by one more, so `pure` cannot be iterated into a sequence.
/// - [`Semigroupal::zip_with`](crate::Semigroupal::zip_with) pairs the slots two structures already
///   have. It never adds a slot, so it cannot grow a container from nothing.
/// - [`Monad::bind`](crate::Monad::bind) on a list-shaped witness does concatenate, so a fold over
///   `bind` could build one. That route is unavailable where it is most needed: `bind` requires
///   `Pure`, and the zip witnesses deliberately have none. It would also tie construction to the
///   cartesian reading, which is the wrong one for parallel runs.
///
/// # Laws (Informal)
///
/// 1. **Round trip**: `F::fold(F::collect(xs), init, f) == xs.into_iter().fold(init, f)` — collecting
///    a sequence and folding the result is folding the sequence, for any witness that also
///    implements [`Foldable`](crate::Foldable).
/// 2. **Order**: the i-th value of the sequence occupies the i-th slot of the structure. `collect`
///    preserves order; it is not a set.
/// 3. **Empty**: collecting no values yields the carrier's empty structure rather than failing, so
///    folding that structure returns the initial accumulator unchanged.
///
/// Laws are stated for pure functions; a stateful iterator voids them.
///
/// # Rank
///
/// For a carrier with a shape, `collect` produces the **rank-1** structure of the given length.
/// A carrier that can hold higher-rank data reshapes afterwards; deciding a shape from a flat
/// sequence is not something this operation can do, and guessing one would be wrong more often
/// than right.
///
/// # Type Parameters
///
/// *   `F`: A Higher-Kinded Type (HKT) witness that represents the type constructor.
pub trait Collectable<F: HKT> {
    /// Collects `items` into `F::Type<T>`, in the order the iterator yields them.
    ///
    /// # Arguments
    ///
    /// *   `items`: the values to place into the structure, in order.
    ///
    /// # Returns
    ///
    /// The structure holding every yielded value, one per slot, in iteration order. An iterator
    /// that yields nothing gives the carrier's empty structure.
    ///
    /// # Type Parameters
    ///
    /// *   `T`: the element type.
    /// *   `I`: anything iterable over `T`, so a caller can pass a `Vec`, a slice's iterator, a
    ///     map, or a lazily generated sequence without collecting it first.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use deep_causality_haft::{Collectable, Foldable, VecWitness};
    ///
    /// let xs: Vec<i32> = VecWitness::collect([1, 2, 3, 4]);
    /// assert_eq!(xs, vec![1, 2, 3, 4]);
    ///
    /// // A sequence that was never a collection: no intermediate `Vec` is built.
    /// let squares: Vec<i32> = VecWitness::collect((0..4).map(|i| i * i));
    /// assert_eq!(squares, vec![0, 1, 4, 9]);
    ///
    /// // The round-trip law, on a fold that is not commutative.
    /// let folded = VecWitness::fold(VecWitness::collect([1, 2, 3]), 0, |acc, x| acc * 2 + x);
    /// assert_eq!(folded, [1, 2, 3].into_iter().fold(0, |acc, x| acc * 2 + x));
    ///
    /// // Nothing collected is the empty structure, not a failure.
    /// let none: Vec<i32> = VecWitness::collect(Vec::new());
    /// assert!(none.is_empty());
    /// ```
    fn collect<T, I>(items: I) -> F::Type<T>
    where
        I: IntoIterator<Item = T>;
}
