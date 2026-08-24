/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::dense_vector::DenseVector;
use deep_causality_haft::{
    Applicative, CoMonad, Foldable, Functor, HKT, Monad, NoConstraint, Pure, Satisfies,
};

/// The higher-kinded witness for [`DenseVector`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DenseVectorWitness;

impl HKT for DenseVectorWitness {
    type Constraint = NoConstraint;
    type Type<T> = DenseVector<T>;
}

// As for the matrix witness: `pure` builds a one-element vector, `extract` reads index 0 and panics
// on an empty vector, and `fmap` preserves the length.

impl Functor<DenseVectorWitness> for DenseVectorWitness {
    fn fmap<A, B, Func>(m_a: DenseVector<A>, f: Func) -> DenseVector<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        let _ = (m_a, f);
        todo!("DenseVectorWitness::fmap")
    }
}

impl Foldable<DenseVectorWitness> for DenseVectorWitness {
    fn fold<A, B, Func>(fa: DenseVector<A>, init: B, f: Func) -> B
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(B, A) -> B,
    {
        let _ = (fa, init, f);
        todo!("DenseVectorWitness::fold")
    }
}

impl Pure<DenseVectorWitness> for DenseVectorWitness {
    /// Builds the smallest container holding one value, matching `CsrMatrixWitness`.
    ///
    /// The laws do not settle the shape; something has to choose it, and choosing the same shape
    /// the existing witness chooses is what lets a value round-trip through `pure` then `extract`
    /// unchanged.
    fn pure<T>(value: T) -> DenseVector<T>
    where
        T: Satisfies<NoConstraint>,
    {
        let _ = value;
        todo!("DenseVectorWitness::pure")
    }
}

impl Applicative<DenseVectorWitness> for DenseVectorWitness {
    fn apply<A, B, Func>(ff: DenseVector<Func>, fa: DenseVector<A>) -> DenseVector<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B + Satisfies<NoConstraint>,
    {
        let _ = (ff, fa);
        todo!("DenseVectorWitness::apply")
    }
}

impl Monad<DenseVectorWitness> for DenseVectorWitness {
    fn bind<A, B, Func>(fa: DenseVector<A>, f: Func) -> DenseVector<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> DenseVector<B>,
    {
        let _ = (fa, f);
        todo!("DenseVectorWitness::bind")
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
        A: Satisfies<NoConstraint> + Clone,
    {
        let _ = fa;
        todo!("DenseVectorWitness::extract")
    }

    fn extend<A, B, Func>(fa: &DenseVector<A>, f: Func) -> DenseVector<B>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        Func: FnMut(&DenseVector<A>) -> B,
    {
        let _ = (fa, f);
        todo!("DenseVectorWitness::extend")
    }
}
