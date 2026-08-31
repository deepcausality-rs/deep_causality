/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::CausalTensor;
use deep_causality_haft::{Convolutional, Functor, HKT, MonoidalApplicative, Semigroupal};

// ============================================================================
// HKT Witness Implementation
// ============================================================================

/// The elementwise higher-kinded view of [`CausalTensor`].
///
/// This witness stands for the same container as [`CausalTensorWitness`](crate::CausalTensorWitness)
/// and differs only in what `apply` means. The two cannot be one witness, and the reason is a law
/// rather than a preference.
///
/// `CausalTensorWitness` carries [`Monad`](deep_causality_haft::Monad), which owes the
/// applicative/monad coherence law `apply(ff, fa) == bind(ff, |f| fmap(fa, f))`. `bind` runs the
/// continuation once per function, so coherence forces the *cartesian* applicative: every function
/// against every argument. That is the right answer for the monadic reading and the wrong one for
/// a tensor library, where applying a tensor of functions to a tensor of values elementwise is
/// what a caller means.
///
/// So the elementwise reading lives here, on the monoidal route. [`Semigroupal::zip_with`] pairs
/// slot with slot and consumes nothing twice, and [`MonoidalApplicative::apply`] is derived from
/// it. There is no `Monad` on this witness, so no coherence is owed and the two `apply`s are free
/// to differ.
///
/// # Why there is no `Pure` and no `LaxMonoidal`
///
/// The unit of a positional zip is the infinite repeat: the value that pairs with every element of
/// any tensor, at any shape. A finite `CausalTensor` cannot represent it. Anything else invented
/// for `unit` fails the unit laws at every non-matching shape rather than in some corner, so this
/// witness implements [`Semigroupal`] and stops there, exactly as
/// `deep_causality_haft::lax_monoidal` prescribes for a shape-carrying carrier.
///
/// [`CausalTensorWitness`]: crate::CausalTensorWitness
pub struct ZipTensorWitness;

impl HKT for ZipTensorWitness {
    type Type<T> = CausalTensor<T>;
}

impl Functor<ZipTensorWitness> for ZipTensorWitness {
    fn fmap<A, B, Func>(m_a: CausalTensor<A>, f: Func) -> CausalTensor<B>
    where
        Func: FnMut(A) -> B,
    {
        let shape = m_a.shape().to_vec();
        let new_data: Vec<B> = m_a.into_vec().into_iter().map(f).collect();
        CausalTensor::from_vec(new_data, &shape)
    }
}

impl Semigroupal<ZipTensorWitness> for ZipTensorWitness {
    /// Pairs the two tensors position by position, stopping at the shorter.
    ///
    /// Equal shapes keep that shape, which is the case callers mean. Two tensors of differing
    /// extent have no common shape to report, so the result is the flat `[len]` of the overlap.
    /// Truncation keeps both laws: `min` is associative, so `zip` associates, and naturality
    /// holds because `fmap` does not change a tensor's length.
    fn zip_with<A, B, C, Func>(
        fa: CausalTensor<A>,
        fb: CausalTensor<B>,
        mut f: Func,
    ) -> CausalTensor<C>
    where
        Func: FnMut(A, B) -> C,
    {
        let same_shape = fa.shape() == fb.shape();
        let shape = fa.shape().to_vec();
        let data: Vec<C> = fa
            .into_vec()
            .into_iter()
            .zip(fb.into_vec())
            .map(|(a, b)| f(a, b))
            .collect();
        if same_shape {
            CausalTensor::from_vec(data, &shape)
        } else {
            let len = data.len();
            CausalTensor::from_vec(data, &[len])
        }
    }
}

impl Convolutional<ZipTensorWitness> for ZipTensorWitness {}

impl MonoidalApplicative<ZipTensorWitness> for ZipTensorWitness {}
