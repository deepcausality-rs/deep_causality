/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::csr_matrix::CsrMatrix;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_haft::{
    Applicative, CoMonad, Foldable, Functor, HKT, NoConstraint, Pure, Satisfies,
};

/// The higher-kinded witness for [`CsrMatrix`].
///
/// Moves here from `deep_causality_sparse` unchanged, so that its trait impls and their results are
/// identical to what they were before the move.
///
/// # `fmap` maps the stored entries
///
/// A sparse matrix stores only its non-zeros, so a function that does not fix zero changes which
/// entries are structurally present. This witness maps the **stored** entries and leaves the
/// structural zeros alone, which keeps the result sparse. A caller who wants a function applied to
/// the whole logical matrix densifies first, and that conversion is explicit.
///
/// The shape is preserved either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CsrMatrixWitness;

impl HKT for CsrMatrixWitness {
    type Constraint = NoConstraint;
    type Type<T> = CsrMatrix<T>;
}

impl Functor<CsrMatrixWitness> for CsrMatrixWitness {
    /// Maps the **stored** entries, leaving the structural zeros alone.
    fn fmap<A, B, Func>(m_a: CsrMatrix<A>, f: Func) -> CsrMatrix<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        let (ri, ci, values, shape) = m_a.into_parts();
        CsrMatrix::from_raw_parts(ri, ci, values.into_iter().map(f).collect(), shape)
    }
}

impl Foldable<CsrMatrixWitness> for CsrMatrixWitness {
    /// Folds over the stored entries. A structural zero contributes nothing, which is the choice
    /// `fmap` makes and for the same reason.
    fn fold<A, B, Func>(fa: CsrMatrix<A>, init: B, f: Func) -> B
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(B, A) -> B,
    {
        let (_, _, values, _) = fa.into_parts();
        values.into_iter().fold(init, f)
    }
}

impl Pure<CsrMatrixWitness> for CsrMatrixWitness {
    /// The 1×1 holding the value, matching what the crate this moves from produces.
    fn pure<T>(value: T) -> CsrMatrix<T>
    where
        T: Satisfies<NoConstraint>,
    {
        CsrMatrix::from_raw_parts(vec![0, 1], vec![0], vec![value], (1, 1))
    }
}

impl Applicative<CsrMatrixWitness> for CsrMatrixWitness {
    /// Applies the stored functions to the stored entries, position by position.
    fn apply<A, B, Func>(ff: CsrMatrix<Func>, fa: CsrMatrix<A>) -> CsrMatrix<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B + Satisfies<NoConstraint>,
    {
        let (_, _, fns, _) = ff.into_parts();
        let (ri, ci, values, shape) = fa.into_parts();
        let mut it = fns.into_iter();
        let mut out = Vec::with_capacity(values.len());
        for a in values {
            match it.next() {
                Some(mut g) => out.push(g(a)),
                None => break,
            }
        }
        let kept = out.len();
        CsrMatrix::from_raw_parts(ri, ci.into_iter().take(kept).collect(), out, shape)
    }
}

impl CoMonad<CsrMatrixWitness> for CsrMatrixWitness {
    /// The first stored entry — the focus.
    ///
    /// # Panics
    ///
    /// On a matrix storing nothing. A comonad has no counit for an empty container, and returning a
    /// fabricated zero would break `extend(extract) == id`.
    fn extract<A>(fa: &CsrMatrix<A>) -> A
    where
        A: Satisfies<NoConstraint> + Clone,
    {
        fa.values()
            .first()
            .cloned()
            .expect("Comonad::extract cannot be called on a CsrMatrix that stores nothing")
    }

    /// Applies `f` at each stored position in turn, with that position rotated to the front.
    ///
    /// The shifted-view focus, which is what makes `extend(extract) == id` hold.
    fn extend<A, B, Func>(fa: &CsrMatrix<A>, mut f: Func) -> CsrMatrix<B>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        Func: FnMut(&CsrMatrix<A>) -> B,
    {
        let n = fa.values().len();
        let mut out = Vec::with_capacity(n);
        for k in 0..n {
            let rotated: Vec<A> = (0..n).map(|i| fa.values()[(i + k) % n].clone()).collect();
            let view = CsrMatrix::from_raw_parts(
                fa.row_indices().clone(),
                fa.col_indices().clone(),
                rotated,
                fa.shape(),
            );
            out.push(f(&view));
        }
        CsrMatrix::from_raw_parts(
            fa.row_indices().clone(),
            fa.col_indices().clone(),
            out,
            fa.shape(),
        )
    }
}

// `Monad` and `Adjunction` are deliberately absent, matching `DenseMatrixWitness`.
//
// The crate this moves from claims both. Its `bind` flattens to `1 x count`, so `bind(m, pure)`
// turns a 2x2 into a 1x4 and monad right identity fails — verified by probe against the published
// crate. `Adjunction`'s `counit` is written in terms of that `bind`, so it inherits the defect.
//
// The cause is structural rather than careless: `pure` must choose a shape for one value and a
// shaped container has no canonical one. `openspec/notes/unified_math/HKT-LAW-FINDINGS.md` carries the
// reasoning and the decision owed when the surface is retired.
//
// Nothing outside the two crates' own tests uses either trait, so the omission reaches no consumer.
