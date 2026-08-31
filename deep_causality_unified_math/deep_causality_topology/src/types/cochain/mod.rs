/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::topology_error::{TopologyError, TopologyErrorEnum};

/// A `k`-cochain: values indexed by cell index within the `k`-skeleton, carrying `k`.
///
/// # What this type is for
///
/// The cup product used to take a cochain and its degree as separate arguments, so a binary product
/// was five parameters and the `n`-fold form was a slice of tuples paired by convention. Nothing
/// bound the data to the degree, and a mismatch was a runtime check rather than a type error. This
/// binds them.
///
/// # The representation is unchanged
///
/// The values are the same flat slice indexed by cell index that `deep_causality_physics` uses for
/// velocity one-forms and pressure zero-forms. [`values`](Self::values) hands that slice back
/// unwrapped, and [`from_values`](Self::from_values) takes one, so a caller holding the bare
/// convention converts without copying its layout. The wrapper adds the degree and nothing else.
///
/// # No complex is carried
///
/// A cochain's length must match the cell count of its degree, but which complex it belongs to is
/// not part of the value: the cup product takes the complex separately and validates the length
/// there. Carrying an `Arc` to a complex would make two cochains over equal complexes unequal, and
/// would put a lifetime or a refcount on a type that is otherwise a `Vec` and a `usize`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cochain<R> {
    values: Vec<R>,
    degree: usize,
}

impl<R> Cochain<R> {
    /// A cochain of `degree` over `values`.
    pub fn new(values: Vec<R>, degree: usize) -> Self {
        Self { values, degree }
    }

    /// A cochain of `degree` copied from a slice.
    pub fn from_values(values: &[R], degree: usize) -> Self
    where
        R: Clone,
    {
        Self {
            values: values.to_vec(),
            degree,
        }
    }

    /// The degree.
    pub fn degree(&self) -> usize {
        self.degree
    }

    /// The values, indexed by cell index within the `degree`-skeleton.
    pub fn values(&self) -> &[R] {
        &self.values
    }

    /// The number of values, which is the cell count of the degree it is valid over.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the cochain carries no values.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Consumes the cochain and returns its values.
    pub fn into_values(self) -> Vec<R> {
        self.values
    }

    /// Rejects a cochain whose length does not match `expected`.
    ///
    /// # Errors
    ///
    /// [`TopologyErrorEnum::DimensionMismatch`] naming `side`, the degree and both lengths.
    pub(crate) fn check_len(&self, expected: usize, side: &str) -> Result<(), TopologyError> {
        if self.values.len() != expected {
            return Err(TopologyError(TopologyErrorEnum::DimensionMismatch(
                format!(
                    "{side} cochain of degree {} has length {}, but the complex has {expected} \
                     cells of that degree",
                    self.degree,
                    self.values.len()
                ),
            )));
        }
        Ok(())
    }
}
