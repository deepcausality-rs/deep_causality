/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Lattice;
use deep_causality_haft::{HKT, NoConstraint, Satisfies};
use std::sync::Arc;

/// HKT witness for `Lattice<D>` as a functor over field values.
pub struct LatticeWitness<const D: usize>;

impl<const D: usize> HKT for LatticeWitness<D> {
    type Constraint = NoConstraint;
    type Type<T>
        = LatticeField<D, T>
    where
        T: Satisfies<NoConstraint>;
}

/// A field assignment over lattice cells.
/// Simplified implementation: map from cell indices (or linearized index) to value.
pub struct LatticeField<const D: usize, T> {
    pub lattice: Arc<Lattice<D>>,
    pub values: Vec<T>, // Linearized values
}
