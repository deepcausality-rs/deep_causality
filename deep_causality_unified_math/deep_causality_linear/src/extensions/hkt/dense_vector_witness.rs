/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::dense_vector::DenseVector;
use deep_causality_haft::{
    Applicative, CoMonad, DiagonalTraversable, Foldable, Functor, HKT, Monad, Pure, Semigroupal,
    Traversable,
};

/// The higher-kinded witness for [`DenseVector`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DenseVectorWitness;

impl HKT for DenseVectorWitness {
    type Type<T> = DenseVector<T>;
}

// As for the matrix witness: `pure` builds a one-element vector, `extract` reads index 0 and panics
// on an empty vector, and `fmap` preserves the length.

impl Functor<DenseVectorWitness> for DenseVectorWitness {
    fn fmap<A, B, Func>(m_a: DenseVector<A>, f: Func) -> DenseVector<B>
    where
        Func: FnMut(A) -> B,
    {
        DenseVector::from_vec(m_a.into_data().into_iter().map(f).collect())
    }
}

impl Foldable<DenseVectorWitness> for DenseVectorWitness {
    fn fold<A, B, Func>(fa: DenseVector<A>, init: B, f: Func) -> B
    where
        Func: FnMut(B, A) -> B,
    {
        fa.into_data().into_iter().fold(init, f)
    }
}

impl Pure<DenseVectorWitness> for DenseVectorWitness {
    /// Builds the smallest container holding one value, matching `CsrMatrixWitness`.
    ///
    /// The laws do not settle the shape; something has to choose it, and choosing the same shape
    /// the existing witness chooses is what lets a value round-trip through `pure` then `extract`
    /// unchanged.
    fn pure<T>(value: T) -> DenseVector<T> {
        DenseVector::from_vec(alloc::vec![value])
    }
}

impl Applicative<DenseVectorWitness> for DenseVectorWitness {
    /// The cartesian applicative: every function against every value, function-major.
    ///
    /// This witness also carries [`Monad`], whose `bind` is list concatenation, and that pins the
    /// applicative. Coherence requires `apply(ff, fa) == bind(ff, |f| fmap(fa, f))`, and `bind`
    /// runs the continuation once per function, so the only answer that agrees with it is the
    /// cartesian product. The elementwise reading is a different applicative and lives on
    /// [`ZipDenseVectorWitness`](crate::ZipDenseVectorWitness), which carries no `Monad` and so
    /// owes no coherence.
    ///
    /// The identity law still holds: `pure` builds the one-element vector, so `apply(pure(id), v)`
    /// runs one function across every element and returns `v`.
    fn apply<A, B, Func>(ff: DenseVector<Func>, fa: DenseVector<A>) -> DenseVector<B>
    where
        A: Clone,
        Func: FnMut(A) -> B,
    {
        let fns = ff.into_data();
        let vals = fa.into_data();
        let mut out: alloc::vec::Vec<B> = alloc::vec::Vec::with_capacity(fns.len() * vals.len());
        for mut g in fns {
            for a in vals.iter() {
                out.push(g(a.clone()));
            }
        }
        DenseVector::from_vec(out)
    }
}

impl Monad<DenseVectorWitness> for DenseVectorWitness {
    fn bind<A, B, Func>(fa: DenseVector<A>, mut f: Func) -> DenseVector<B>
    where
        Func: FnMut(A) -> DenseVector<B>,
    {
        let mut out = alloc::vec::Vec::new();
        for a in fa.into_data() {
            out.extend(f(a).into_data());
        }
        DenseVector::from_vec(out)
    }
}

impl Traversable<DenseVectorWitness> for DenseVectorWitness {
    /// Flips `DenseVector<M<A>>` into `M<DenseVector<A>>` by folding an accumulator through `M`
    /// from left to right, so the effects run in index order and the result keeps that order.
    ///
    /// An element in a failing state collapses the whole traversal, and the first such element in
    /// index order is the one reported. The empty vector yields `M::pure` of the empty vector.
    ///
    /// # Cost
    ///
    /// The accumulator is cloned once per step and holds `k` elements at step `k`, so the fold
    /// performs `n(n-1)/2` element clones — quadratic — for an `n`-element input. The clone is
    /// forced by [`Applicative::apply`]'s `Func: FnMut` bound, not by this trait's `A: Clone`: an
    /// `FnMut` may be invoked repeatedly, so the closure cannot move its captured accumulator out,
    /// and this witness's own cartesian `apply` does invoke it once per element. An `A: Copy`
    /// bound would not help, because the cloned value is the accumulator `Vec`, never `Copy`.
    fn sequence<A, M>(fa: DenseVector<M::Type<A>>) -> M::Type<DenseVector<A>>
    where
        M: Applicative<M> + HKT,
        A: Clone,
    {
        let mut acc: M::Type<alloc::vec::Vec<A>> = M::pure(alloc::vec::Vec::new());
        for m_a in fa.into_data() {
            acc = M::apply(
                M::fmap(acc, |v: alloc::vec::Vec<A>| {
                    move |a: A| {
                        let mut v = v.clone();
                        v.push(a);
                        v
                    }
                }),
                m_a,
            );
        }
        M::fmap(acc, DenseVector::from_vec)
    }
}

impl CoMonad<DenseVectorWitness> for DenseVectorWitness {
    /// The `(0, 0)` entry.
    ///
    /// # Panics
    ///
    /// On an empty container. A comonad has no counit for one, and returning a fabricated zero
    /// would break `extend(extract) == id`.
    fn extract<A>(fa: &DenseVector<A>) -> A
    where
        A: Clone,
    {
        fa.as_slice()
            .first()
            .cloned()
            .expect("a comonad has no counit for an empty container")
    }

    fn extend<A, B, Func>(fa: &DenseVector<A>, mut f: Func) -> DenseVector<B>
    where
        A: Clone,
        Func: FnMut(&DenseVector<A>) -> B,
    {
        // The same focus rule as the matrix: rotate position `i` to the front and apply `f` there,
        // so that `extend(extract) == id`.
        let n = fa.as_slice().len();
        let mut out = alloc::vec::Vec::with_capacity(n);
        for i in 0..n {
            let view = shifted_view(fa, i);
            out.push(f(&view));
        }
        DenseVector::from_vec(out)
    }
}

/// The vector `fa` rotated so that `index` is first.
fn shifted_view<A: Clone>(fa: &DenseVector<A>, index: usize) -> DenseVector<A> {
    let s = fa.as_slice();
    let n = s.len();
    let mut out = alloc::vec::Vec::with_capacity(n);
    for i in 0..n {
        out.push(s[(i + index) % n].clone());
    }
    DenseVector::from_vec(out)
}

impl DiagonalTraversable<DenseVectorWitness> for DenseVectorWitness {
    /// Zips each slot's run into the accumulator in index order.
    ///
    /// A vector carries no shape beyond its length, so unlike the tensor impl there is nothing to
    /// restore afterwards and the fold is the whole operation. An empty vector leaves nothing to
    /// zip and the seed is returned as given — with no [`Pure`] there is nothing else it could be.
    fn sequence_zip<A, M>(
        fa: DenseVector<M::Type<A>>,
        seed: M::Type<DenseVector<A>>,
    ) -> M::Type<DenseVector<A>>
    where
        M: Semigroupal<M> + HKT,
    {
        let mut acc = seed;
        for cell in fa.into_data() {
            acc = M::zip_with(acc, cell, |slot, a| {
                let mut values = slot.into_data();
                values.push(a);
                DenseVector::from_vec(values)
            });
        }
        acc
    }
}
