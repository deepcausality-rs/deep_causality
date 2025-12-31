/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Cell;
use crate::CellComplex;
use deep_causality_haft::HKT;
use std::sync::Arc;

/// HKT witness for CellComplex<C> as a functor over field values.
pub struct CellComplexWitness<C: Cell>(std::marker::PhantomData<C>);

impl<C: Cell> HKT for CellComplexWitness<C> {
    type Type<T> = CellField<C, T>;
}

/// A field over an arbitrary cell complex.
pub struct CellField<C: Cell, T> {
    pub complex: Arc<CellComplex<C>>,
    pub values: Vec<T>,
}
