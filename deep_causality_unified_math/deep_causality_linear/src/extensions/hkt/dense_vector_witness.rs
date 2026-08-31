/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::dense_vector::DenseVector;
use deep_causality_haft::{Applicative, CoMonad, Foldable, Functor, HKT, Monad, Pure};

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
    /// Pairs the functions with the values by position, and stops at the shorter of the two.
    ///
    /// `CsrMatrixWitness::apply` stops the same way. Demanding matching lengths made
    /// `apply(pure(f), v)` — the left-hand side of the applicative identity law, where `pure`
    /// builds a one-element vector — panic for every `v` of two entries or more.
    fn apply<A, B, Func>(ff: DenseVector<Func>, fa: DenseVector<A>) -> DenseVector<B>
    where
        Func: FnMut(A) -> B,
    {
        let mut fns = ff.into_data().into_iter();
        let mut out = alloc::vec::Vec::new();
        for a in fa.into_data() {
            match fns.next() {
                Some(mut g) => out.push(g(a)),
                None => break,
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
