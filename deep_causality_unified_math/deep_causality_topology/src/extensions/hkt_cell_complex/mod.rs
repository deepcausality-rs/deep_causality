/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Cell;
use crate::CellComplex;
use deep_causality_haft::{Foldable, Functor, HKT};
use std::sync::Arc;

/// HKT witness for [`CellField`], the functor over values on a cell complex.
///
/// # Why the element type carries no bound
///
/// `CellField<C, T>` carries no bound on its value type `T`, and the operations here move values
/// without computing on them. `fmap` maps `A` to an unrelated `B`, so constraining `T` would forbid
/// mapping a field of labels, which is legitimate. See
/// `openspec/notes/archive/hkt_gat/hkt_gat_topology.md` §4.
///
/// # Why there is no `Pure`, `Monad` or `CoMonad`
///
/// A `CellField` is a complex paired with values, and `Pure::pure` receives one value and nothing
/// else. It would have to invent a `CellComplex` from nothing, and any complex it invented would be
/// wrong: `bind(m, pure)` would return a field over a fabricated complex rather than `m`, so monad
/// right identity would fail by construction. This is the defect measured on
/// `CausalMultiVectorWitness`, whose `pure` fabricates `Metric::Euclidean(0)`, recorded in
/// `openspec/notes/archive/unified_math/HKT-LAW-FINDINGS.md`.
///
/// `deep_causality_linear` set the precedent: it implements `Monad` only for `DenseVector`, the one
/// container with no context to fabricate. A missing instance is a smaller defect than an unlawful
/// one, so this witness stops at `Functor` and `Foldable`.
///
/// `CoMonad` is absent for a different reason: `extract` needs a distinguished cell, and
/// `CellField` carries no cursor to name one.
pub struct CellComplexWitness<C: Cell>(std::marker::PhantomData<C>);

impl<C: Cell> HKT for CellComplexWitness<C> {
    type Type<T> = CellField<C, T>;
}

/// A field over an arbitrary cell complex.
///
/// `C` is the cell type, which fixes the complex; `T` is the value carried on each cell. They are
/// independent, so mapping the values leaves the complex alone.
#[derive(Clone)]
pub struct CellField<C: Cell, T> {
    pub complex: Arc<CellComplex<C>>,
    pub values: Vec<T>,
}

/// `CellComplex` implements neither `Debug` nor `PartialEq`, so both are written by hand here.
///
/// Two fields are equal when they share the same complex and carry equal values. Sharing is
/// identity (`Arc::ptr_eq`) rather than structural equality, which is the strongest statement
/// available and the one the functor laws need: `fmap` moves the `Arc` through, so the result
/// shares the source's complex.
impl<C: Cell, T: PartialEq> PartialEq for CellField<C, T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.complex, &other.complex) && self.values == other.values
    }
}

impl<C: Cell, T: core::fmt::Debug> core::fmt::Debug for CellField<C, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CellField")
            .field("values", &self.values)
            .finish_non_exhaustive()
    }
}

impl<C: Cell, T> CellField<C, T> {
    /// Pairs a complex with one value per entry of `values`.
    pub fn new(complex: Arc<CellComplex<C>>, values: Vec<T>) -> Self {
        Self { complex, values }
    }

    pub fn complex(&self) -> &Arc<CellComplex<C>> {
        &self.complex
    }

    pub fn values(&self) -> &[T] {
        &self.values
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl<C: Cell> Functor<CellComplexWitness<C>> for CellComplexWitness<C> {
    /// Maps the values, carrying the complex across unchanged.
    fn fmap<A, B, F>(fa: CellField<C, A>, f: F) -> CellField<C, B>
    where
        F: FnMut(A) -> B,
    {
        CellField {
            complex: fa.complex,
            values: fa.values.into_iter().map(f).collect(),
        }
    }
}

impl<C: Cell> Foldable<CellComplexWitness<C>> for CellComplexWitness<C> {
    fn fold<A, B, F>(fa: CellField<C, A>, init: B, f: F) -> B
    where
        F: FnMut(B, A) -> B,
    {
        fa.values.into_iter().fold(init, f)
    }
}
