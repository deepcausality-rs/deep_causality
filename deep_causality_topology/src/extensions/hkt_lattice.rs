/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Lattice;
use deep_causality_haft::HKT;
use std::sync::Arc;

/// HKT witness for Lattice<D> as a functor over field values.
pub struct LatticeWitness<const D: usize>;

impl<const D: usize> HKT for LatticeWitness<D> {
    /// Lattice with field values of type T at each k-cell.
    /// This requires a definition of LatticeField which maps cells to values.
    /// For now, we define the type alias or struct here or import it.
    /// The spec defined LatticeField. We should define it in types/lattice/lattice_field.rs?
    /// Or inline if simple.
    type Type<T> = LatticeField<D, T>;
}

/// A field assignment over lattice cells.
/// Simplified implementation: map from cell indices (or linearized index) to value.
pub struct LatticeField<const D: usize, T> {
    pub lattice: Arc<Lattice<D>>,
    pub values: Vec<T>, // Linearized values
}
