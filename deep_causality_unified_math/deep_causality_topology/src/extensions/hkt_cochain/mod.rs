/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Cochain;
use deep_causality_haft::{Foldable, Functor, HKT};

/// The higher-kinded witness for [`Cochain`], completing the three cochain-adjacent siblings:
/// [`ChainWitness`](crate::ChainWitness) binds `Chain`, `ExteriorDerivativeWitness` binds
/// `DifferentialForm`, and this binds `Cochain`.
///
/// # Why the element type carries no bound
///
/// `Cochain<R>` declares no bound on `R`, and the two operations here move values without
/// computing on them: `fmap` maps `A` to an unrelated `B` and `fold` hands each value to the
/// caller's function. A cochain of labels maps as readily as one of `f64`.
///
/// # What is claimed, and what is not
///
/// `Functor` and `Foldable`, matching what `ChainWitness` claims and no more.
///
/// `Pure` is absent, and not merely by symmetry with the sibling. A cochain carries a degree
/// alongside its values; `pure` receives one value and no degree, so any degree it returned —
/// zero, or the degree of nothing — would be invented rather than derived. `Applicative`,
/// `Monad` and `CoMonad` follow `Pure` out for the same reason.
///
/// `Traversable` is also absent: `ChainWitness` does not carry it either, and nothing in the
/// workspace sequences a cochain.
pub struct CochainWitness;

impl HKT for CochainWitness {
    type Type<R> = Cochain<R>;
}

impl Functor<CochainWitness> for CochainWitness {
    /// Maps every value in index order, carrying the degree across unchanged.
    ///
    /// The degree is what distinguishes this type from a bare `Vec`, and preserving it is the
    /// functor's structural obligation: it is carried, never recomputed from the value count. The
    /// two are independent — a degree-2 cochain over a complex with five 2-cells has length five —
    /// so deriving one from the other would be wrong wherever they differ.
    fn fmap<A, B, Func>(fa: Cochain<A>, f: Func) -> Cochain<B>
    where
        Func: FnMut(A) -> B,
    {
        let degree = fa.degree();
        Cochain::new(fa.into_values().into_iter().map(f).collect(), degree)
    }
}

impl Foldable<CochainWitness> for CochainWitness {
    /// Folds the values in index order from `init`.
    ///
    /// A cochain carrying no values returns `init` unchanged, and `f` is never called.
    fn fold<A, B, Func>(fa: Cochain<A>, init: B, f: Func) -> B
    where
        Func: FnMut(B, A) -> B,
    {
        fa.into_values().into_iter().fold(init, f)
    }
}
