/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::LatticeComplex;
use deep_causality_algebra::RealField;
use deep_causality_haft::{Foldable, Functor, HKT};
use std::sync::Arc;

/// HKT witness for [`LatticeField`], the functor over values on a lattice complex.
///
/// # Why `NoConstraint`
///
/// `LatticeField<D, R, T>` carries no bound on its value type `T`, and the operations here move
/// values without computing on them. The lattice's precision `R` is a separate parameter, so
/// mapping the values leaves the lattice and its metric alone.
///
/// # Why there is no `Pure`, `Monad` or `CoMonad`
///
/// `Pure::pure` receives one value and would have to invent a `LatticeComplex<D, R>` around it.
/// Any lattice it invented would be the wrong one, so `bind(m, pure)` would not return `m` and
/// monad right identity would fail by construction. `deep_causality_linear` set the precedent by
/// implementing `Monad` only for `DenseVector`, the one container with no context to fabricate;
/// see `openspec/notes/archive/unified_math/HKT-LAW-FINDINGS.md`.
///
/// `CoMonad` is absent because `extract` needs a distinguished cell and `LatticeField` carries no
/// cursor to name one.
pub struct LatticeComplexWitness<const D: usize, R: RealField>(std::marker::PhantomData<R>);

impl<const D: usize, R: RealField> HKT for LatticeComplexWitness<D, R> {
    type Type<T> = LatticeField<D, R, T>;
}

/// A field assignment over lattice cells, stored linearised.
///
/// `D` and `R` fix the lattice and its metric precision; `T` is the value carried on each cell.
/// They are independent, which is what lets `fmap` keep the lattice.
#[derive(Debug, Clone, PartialEq)]
pub struct LatticeField<const D: usize, R: RealField, T> {
    pub lattice: Arc<LatticeComplex<D, R>>,
    pub values: Vec<T>,
}

impl<const D: usize, R: RealField, T> LatticeField<D, R, T> {
    /// Pairs a lattice with one value per entry of `values`.
    pub fn new(lattice: Arc<LatticeComplex<D, R>>, values: Vec<T>) -> Self {
        Self { lattice, values }
    }

    pub fn lattice(&self) -> &Arc<LatticeComplex<D, R>> {
        &self.lattice
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

impl<const D: usize, R: RealField> Functor<LatticeComplexWitness<D, R>>
    for LatticeComplexWitness<D, R>
{
    /// Maps the values, carrying the lattice across unchanged.
    fn fmap<A, B, F>(fa: LatticeField<D, R, A>, f: F) -> LatticeField<D, R, B>
    where
        F: FnMut(A) -> B,
    {
        LatticeField {
            lattice: fa.lattice,
            values: fa.values.into_iter().map(f).collect(),
        }
    }
}

impl<const D: usize, R: RealField> Foldable<LatticeComplexWitness<D, R>>
    for LatticeComplexWitness<D, R>
{
    fn fold<A, B, F>(fa: LatticeField<D, R, A>, init: B, f: F) -> B
    where
        F: FnMut(B, A) -> B,
    {
        fa.values.into_iter().fold(init, f)
    }
}
