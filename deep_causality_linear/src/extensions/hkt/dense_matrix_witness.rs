/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::dense_matrix::DenseMatrix;
use deep_causality_haft::{
    Applicative, CoMonad, Foldable, Functor, HKT, Monad, NoConstraint, Pure, Satisfies,
};

/// The higher-kinded witness for [`DenseMatrix`].
///
/// A witness is a zero-sized stand-in for the type constructor `DenseMatrix<_>`, which Rust cannot
/// name directly. Every `deep_causality_haft` trait is implemented on the witness rather than on the
/// container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DenseMatrixWitness;

impl HKT for DenseMatrixWitness {
    type Constraint = NoConstraint;
    type Type<T> = DenseMatrix<T>;
}

// `Functor`, `Foldable`, `Pure`, `Applicative`, `Monad`, `CoMonad` and `Adjunction` follow, matching
// `CsrMatrixWitness` member for member.
//
// Two of them carry a shape decision that the laws do not settle, and which the tests pin:
//
//   `pure` has to choose a shape for a single value. `CsrMatrixWitness` makes a 1x1. This does the
//   same, so that a value round-tripping through `pure` then `extract` is unchanged and the monad
//   identities hold at a shape both sides agree on.
//
//   `extract` has to choose which entry of a matrix is "the" one. It is the (0, 0) entry, and it
//   panics on an empty matrix — a comonad has no counit for an empty container, and returning a
//   fabricated zero would break `extend(extract) == id`.

impl Functor<DenseMatrixWitness> for DenseMatrixWitness {
    fn fmap<A, B, Func>(m_a: DenseMatrix<A>, f: Func) -> DenseMatrix<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        let _ = (m_a, f);
        todo!("DenseMatrixWitness::fmap")
    }
}

impl Foldable<DenseMatrixWitness> for DenseMatrixWitness {
    fn fold<A, B, Func>(fa: DenseMatrix<A>, init: B, f: Func) -> B
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(B, A) -> B,
    {
        let _ = (fa, init, f);
        todo!("DenseMatrixWitness::fold")
    }
}

impl Pure<DenseMatrixWitness> for DenseMatrixWitness {
    /// Builds the smallest container holding one value, matching `CsrMatrixWitness`.
    ///
    /// The laws do not settle the shape; something has to choose it, and choosing the same shape
    /// the existing witness chooses is what lets a value round-trip through `pure` then `extract`
    /// unchanged.
    fn pure<T>(value: T) -> DenseMatrix<T>
    where
        T: Satisfies<NoConstraint>,
    {
        let _ = value;
        todo!("DenseMatrixWitness::pure")
    }
}

impl Applicative<DenseMatrixWitness> for DenseMatrixWitness {
    fn apply<A, B, Func>(ff: DenseMatrix<Func>, fa: DenseMatrix<A>) -> DenseMatrix<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B + Satisfies<NoConstraint>,
    {
        let _ = (ff, fa);
        todo!("DenseMatrixWitness::apply")
    }
}

impl Monad<DenseMatrixWitness> for DenseMatrixWitness {
    fn bind<A, B, Func>(fa: DenseMatrix<A>, f: Func) -> DenseMatrix<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> DenseMatrix<B>,
    {
        let _ = (fa, f);
        todo!("DenseMatrixWitness::bind")
    }
}

impl CoMonad<DenseMatrixWitness> for DenseMatrixWitness {
    /// The `(0, 0)` entry.
    ///
    /// # Panics
    ///
    /// On an empty container. A comonad has no counit for one, and returning a fabricated zero
    /// would break `extend(extract) == id`.
    fn extract<A>(fa: &DenseMatrix<A>) -> A
    where
        A: Satisfies<NoConstraint> + Clone,
    {
        let _ = fa;
        todo!("DenseMatrixWitness::extract")
    }

    fn extend<A, B, Func>(fa: &DenseMatrix<A>, f: Func) -> DenseMatrix<B>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        Func: FnMut(&DenseMatrix<A>) -> B,
    {
        let _ = (fa, f);
        todo!("DenseMatrixWitness::extend")
    }
}
