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
