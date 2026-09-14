/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{HKT, Semigroupal};

/// Turning a structure inside out along the **diagonal**: the i-th slot of each inner container
/// meets the i-th slot of every other.
///
/// [`Traversable::sequence`](crate::Traversable::sequence) already flips `F<M<A>>` into `M<F<A>>`,
/// and for many `M` it is the operation wanted. It is not this one. `sequence` is bounded on
/// `M: Applicative`, and an applicative that also carries [`Monad`](crate::Monad) owes the
/// coherence law `apply(ff, fa) == bind(ff, |f| fmap(fa, f))`, which forces the **cartesian**
/// reading: every element against every element. Flipping a 2×2 structure whose four slots each
/// hold 50 values that way gives `50^4` results — every combination of the four — rather than 50.
///
/// When the inner containers are parallel runs of the same length, the cartesian product is not
/// what the caller means. Value *i* of one belongs with value *i* of the others; the pairing is by
/// index. That reading is [`Semigroupal::zip_with`], and this trait is `sequence` driven by it.
///
/// # Why it needs its own trait, and its own argument
///
/// A witness whose `zip` is positional cannot carry [`Pure`](crate::Pure). The unit of a positional
/// zip is the value that pairs with every slot of any container at any length — the infinite repeat
/// — and a finite carrier cannot represent it; `crate::lax_monoidal` sets out why inventing one
/// fails the unit laws at every value rather than in a corner. So the zip witnesses implement
/// [`Semigroupal`] and stop, and cannot drive `sequence`:
///
/// ```text
/// error[E0277]: the trait bound `ZipTensorWitness: Applicative<ZipTensorWitness>` is not satisfied
/// ```
///
/// With no `Pure` there is nothing to build a starting accumulator from, so the caller supplies
/// one. That is where the missing unit surfaces, and it is honest rather than a wart: the length
/// of the result is a decision the caller is making, and here they make it in the open.
pub trait DiagonalTraversable<F: HKT> {
    /// Flip `F<M<A>>` into `M<F<A>>` by pairing index with index.
    ///
    /// `seed` is the starting accumulator, an `M` of as many `F`s as the result should hold. Each
    /// slot of `fa` is zipped into it in turn, so every `F` in the result gains one entry per slot
    /// of the original structure, taken from the same position of that slot's container.
    ///
    /// # Length
    ///
    /// Zipping truncates to the shorter side, so the result holds as many `F`s as the shortest of
    /// the seed and the inner containers. **The truncation is of the result, never of the
    /// structure**: every `F` that survives holds every slot of `fa`. Losing a slot would change
    /// what each result *is*; losing a position only changes how many there are.
    ///
    /// # An empty structure
    ///
    /// A structure with no slots leaves nothing to zip, and the result is the seed as given.
    fn sequence_zip<A, M>(fa: F::Type<M::Type<A>>, seed: M::Type<F::Type<A>>) -> M::Type<F::Type<A>>
    where
        M: Semigroupal<M> + HKT;
}
