/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::dense_vector::DenseVector;
use deep_causality_haft::{Convolutional, Functor, HKT, MonoidalApplicative, Semigroupal};

/// The elementwise higher-kinded view of [`DenseVector`].
///
/// This witness stands for the same container as
/// [`DenseVectorWitness`](crate::DenseVectorWitness) and differs only in what `apply` means. The
/// two cannot be one witness, and the reason is a law rather than a preference.
///
/// `DenseVectorWitness` carries [`Monad`](deep_causality_haft::Monad), whose `bind` is list
/// concatenation. That owes the applicative/monad coherence law
/// `apply(ff, fa) == bind(ff, |f| fmap(fa, f))`, and `bind` runs the continuation once per
/// function, so coherence admits only the cartesian product: every function against every value.
/// Pairing position by position is the reading a caller usually wants from a vector, and it is a
/// different applicative, so it lives here.
///
/// # Why there is no `Pure` and no `LaxMonoidal`
///
/// The unit of a positional zip is the infinite repeat: the value that pairs with every element of
/// any vector, at any length. A finite `DenseVector` cannot represent it. Anything else invented
/// for `unit` fails the unit laws at every non-matching length rather than in some corner, so this
/// witness implements [`Semigroupal`] and stops there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ZipDenseVectorWitness;

impl HKT for ZipDenseVectorWitness {
    type Type<T> = DenseVector<T>;
}

impl Functor<ZipDenseVectorWitness> for ZipDenseVectorWitness {
    fn fmap<A, B, Func>(fa: DenseVector<A>, f: Func) -> DenseVector<B>
    where
        Func: FnMut(A) -> B,
    {
        DenseVector::from_vec(fa.into_data().into_iter().map(f).collect())
    }
}

impl Semigroupal<ZipDenseVectorWitness> for ZipDenseVectorWitness {
    /// Pairs the two vectors position by position, stopping at the shorter.
    ///
    /// Truncation keeps both laws: `min` is associative, so `zip` associates, and naturality holds
    /// because `fmap` does not change a vector's length.
    fn zip_with<A, B, C, Func>(
        fa: DenseVector<A>,
        fb: DenseVector<B>,
        mut f: Func,
    ) -> DenseVector<C>
    where
        Func: FnMut(A, B) -> C,
    {
        DenseVector::from_vec(
            fa.into_data()
                .into_iter()
                .zip(fb.into_data())
                .map(|(a, b)| f(a, b))
                .collect(),
        )
    }
}

impl Convolutional<ZipDenseVectorWitness> for ZipDenseVectorWitness {}

impl MonoidalApplicative<ZipDenseVectorWitness> for ZipDenseVectorWitness {}
