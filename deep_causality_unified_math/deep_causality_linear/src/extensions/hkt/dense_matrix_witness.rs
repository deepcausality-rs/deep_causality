/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::dense_matrix::DenseMatrix;
use deep_causality_haft::{
    Applicative, CoMonad, Foldable, Functor, HKT, NoConstraint, Pure, Satisfies,
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
        let (r, c) = (m_a.rows_pub(), m_a.cols_pub());
        let mapped: alloc::vec::Vec<B> = m_a.into_data().into_iter().map(f).collect();
        DenseMatrix::from_vec(mapped, r, c).expect("fmap preserves the element count")
    }
}

impl Foldable<DenseMatrixWitness> for DenseMatrixWitness {
    fn fold<A, B, Func>(fa: DenseMatrix<A>, init: B, f: Func) -> B
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(B, A) -> B,
    {
        fa.into_data().into_iter().fold(init, f)
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
        DenseMatrix::from_vec(alloc::vec![value], 1, 1).expect("1x1 holds exactly one value")
    }
}

impl Applicative<DenseMatrixWitness> for DenseMatrixWitness {
    fn apply<A, B, Func>(ff: DenseMatrix<Func>, fa: DenseMatrix<A>) -> DenseMatrix<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B + Satisfies<NoConstraint>,
    {
        let (r, c) = (fa.rows_pub(), fa.cols_pub());
        let mut fns = ff.into_data().into_iter();
        let mut out = alloc::vec::Vec::with_capacity(r * c);
        for a in fa.into_data() {
            let mut g = fns.next().expect("apply requires matching shapes");
            out.push(g(a));
        }
        DenseMatrix::from_vec(out, r, c).expect("apply preserves the element count")
    }
}

// `Monad` is deliberately absent, and this is the trait `linear-hkt-composition` anticipated a
// container might not support.
//
// `pure` has to choose a shape for a single value, and a shaped container has no canonical one. Take
// `pure(a)` to be the 1x1 — the only defensible choice — and monad right identity requires
// `bind(m, pure) == m`, so `bind` must reassemble an `m x n` matrix out of `m*n` one-by-ones. Any
// `bind` general enough to accept an `f` returning other shapes cannot also do that.
//
// `deep_causality_sparse::CsrMatrixWitness` claims `Monad` and does not satisfy the law: its `bind`
// flattens to `1 x count`, so `bind(m, pure)` turns a 2x2 into a 1x4. Verified by probe against the
// published crate. That is a defect in the code being moved, recorded in
// `openspec/notes/unified_math/HKT-LAW-FINDINGS.md` and to be decided at task 4.11 rather than copied
// here.
//
// `DenseVectorWitness` does claim `Monad` and does satisfy the laws, because a vector's `bind` is
// list concatenation and a vector has no shape beyond its length.

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
        fa.as_slice()
            .first()
            .cloned()
            .expect("a comonad has no counit for an empty container")
    }

    fn extend<A, B, Func>(fa: &DenseMatrix<A>, mut f: Func) -> DenseMatrix<B>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        Func: FnMut(&DenseMatrix<A>) -> B,
    {
        // For each position, build the view that focuses it at (0, 0) and apply `f` there. That is
        // what makes `extend(extract) == id` hold: `extract` of the view focused at (i, j) is the
        // entry at (i, j), so the result reproduces the original.
        //
        // Applying `f` to the whole container at every position instead would fill the result with
        // `f(fa)` repeated, and the comonad law would fail.
        let (r, c) = (fa.rows_pub(), fa.cols_pub());
        let mut out = alloc::vec::Vec::with_capacity(r * c);
        for i in 0..r {
            for j in 0..c {
                let view = shifted_view(fa, i, j);
                out.push(f(&view));
            }
        }
        DenseMatrix::from_vec(out, r, c).expect("extend preserves the shape")
    }
}

/// The matrix `fa` with `(row, col)` rotated to `(0, 0)`.
///
/// The focus a comonad needs. `DenseMatrix` carries no cursor, so the focus is expressed by moving
/// the entry rather than by pointing at it, which is the arrangement `CsrMatrixWitness` uses.
fn shifted_view<A: Clone>(fa: &DenseMatrix<A>, row: usize, col: usize) -> DenseMatrix<A> {
    let (r, c) = (fa.rows_pub(), fa.cols_pub());
    let mut out = alloc::vec::Vec::with_capacity(r * c);
    for i in 0..r {
        for j in 0..c {
            let si = (i + row) % r;
            let sj = (j + col) % c;
            out.push(fa.as_slice()[si * c + sj].clone());
        }
    }
    DenseMatrix::from_vec(out, r, c).expect("a rotation preserves the element count")
}
