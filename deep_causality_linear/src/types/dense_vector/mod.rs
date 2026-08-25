/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A dense vector.

pub mod algebra;
pub mod ops;

use crate::errors::linear_error::LinearError;
use alloc::vec::Vec;

/// A dense vector, carrying its length.
///
/// # The larger half of the census
///
/// Of the 118 `CausalTensor` constructions across the consumer crates, **60 are rank-1** against 46
/// rank-2. Every one of those is a vector expressed as a tensor that happens to have one dimension,
/// so every access pays a runtime rank check and every signature that takes one also admits a
/// matrix. A vector type is not an ornament on the matrix work.
///
/// # Distinct from a one-column matrix
///
/// A function that wants a vector will not take a `DenseMatrix`, and the reverse. The two are
/// different shapes and the outer product is the one operation that has to know about both — it
/// takes two vectors and produces a matrix, which is why it is declared with the vector rather than
/// with the matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenseVector<T> {
    data: Vec<T>,
}

impl<T> DenseVector<T> {
    /// Builds from a buffer.
    pub fn from_vec(data: Vec<T>) -> Self {
        Self { data }
    }

    /// Consumes the vector and yields its entries.
    pub(crate) fn into_data(self) -> Vec<T> {
        self.data
    }

    /// The entries, mutably. Internal.
    pub(crate) fn as_mut_data(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// The number of entries.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether the vector has no entries.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// The entries.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// The entry at `index`.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if the index is outside the length.
    pub fn get(&self, index: usize) -> Result<T, LinearError>
    where
        T: Clone,
    {
        self.data
            .get(index)
            .cloned()
            .ok_or(LinearError::IndexOutOfBounds(
                (index, 0),
                (self.data.len(), 1),
            ))
    }
}

impl<T> DenseVector<T>
where
    T: deep_causality_algebra::CommutativeSemiring + Copy,
{
    /// The dot product, `Σ aᵢ bᵢ`.
    ///
    /// Bounded on `CommutativeSemiring`: it adds and multiplies and does nothing else, so it is
    /// available over ℕ. This is **not** an inner product over ℂ — see
    /// [`hermitian_inner`](Self::hermitian_inner).
    ///
    /// # Errors
    ///
    /// [`LinearError::LengthMismatch`] if the lengths differ, rather than truncating to the shorter.
    pub fn dot(&self, other: &Self) -> Result<T, LinearError> {
        if self.data.len() != other.data.len() {
            return Err(LinearError::LengthMismatch(
                self.data.len(),
                other.data.len(),
            ));
        }
        let mut acc = T::zero();
        for (a, b) in self.data.iter().zip(other.data.iter()) {
            acc = acc + *a * *b;
        }
        Ok(acc)
    }

    /// The outer product, an `m x n` matrix from an `m`-vector and an `n`-vector.
    ///
    /// The one operation that makes the vector and the matrix know about each other, which is why it
    /// is declared here rather than on the matrix.
    pub fn outer(&self, other: &Self) -> crate::types::dense_matrix::DenseMatrix<T> {
        let (m, n) = (self.data.len(), other.data.len());
        let mut out = Vec::with_capacity(m * n);
        for a in &self.data {
            for b in &other.data {
                out.push(*a * *b);
            }
        }
        crate::types::dense_matrix::DenseMatrix::from_vec(out, m, n)
            .expect("the buffer is built from the two lengths")
    }
}

impl<T> DenseVector<T>
where
    T: deep_causality_algebra::CommutativeRing + Copy,
{
    /// Entrywise addition.
    ///
    /// # Errors
    ///
    /// [`LinearError::LengthMismatch`] if the lengths differ.
    pub fn add(&self, other: &Self) -> Result<Self, LinearError> {
        if self.data.len() != other.data.len() {
            return Err(LinearError::LengthMismatch(
                self.data.len(),
                other.data.len(),
            ));
        }
        Ok(Self {
            data: self
                .data
                .iter()
                .zip(other.data.iter())
                .map(|(a, b)| *a + *b)
                .collect(),
        })
    }

    /// Entrywise subtraction.
    ///
    /// Bounded on `CommutativeRing` rather than `CommutativeSemiring`, which is the whole difference
    /// between the two bands: ℕ has no additive inverses, and `3u64 - 5u64` has no value.
    ///
    /// # Errors
    ///
    /// [`LinearError::LengthMismatch`] if the lengths differ.
    pub fn sub(&self, other: &Self) -> Result<Self, LinearError> {
        if self.data.len() != other.data.len() {
            return Err(LinearError::LengthMismatch(
                self.data.len(),
                other.data.len(),
            ));
        }
        Ok(Self {
            data: self
                .data
                .iter()
                .zip(other.data.iter())
                .map(|(a, b)| *a - *b)
                .collect(),
        })
    }

    /// Multiplies every entry by `scalar`.
    pub fn scale(&self, scalar: T) -> Self {
        Self {
            data: self.data.iter().map(|a| *a * scalar).collect(),
        }
    }
}

impl<T> DenseVector<T>
where
    T: deep_causality_algebra::ConjugateScalar,
{
    /// The Hermitian inner product, `Σ conj(aᵢ) · bᵢ`.
    ///
    /// # Distinct from the dot product, deliberately
    ///
    /// Over ℂ the plain dot product is not an inner product: `⟨v, v⟩` is neither real nor
    /// non-negative, so it induces no norm. `deep_causality_quantum` works in `Complex<R>`
    /// throughout, so a single `dot` that silently did the wrong thing there would be a defect
    /// waiting on its first complex caller.
    ///
    /// Over the reals the two agree, because conjugation is the identity there.
    ///
    /// # Errors
    ///
    /// [`LinearError::LengthMismatch`] if the lengths differ.
    pub fn hermitian_inner(&self, other: &Self) -> Result<T, LinearError> {
        if self.data.len() != other.data.len() {
            return Err(LinearError::LengthMismatch(
                self.data.len(),
                other.data.len(),
            ));
        }
        let mut acc = T::zero();
        for (a, b) in self.data.iter().zip(other.data.iter()) {
            acc += a.conjugate() * *b;
        }
        Ok(acc)
    }
}

impl<T> DenseVector<T>
where
    T: deep_causality_algebra::NormedScalar,
{
    /// The 1-norm, `Σ |aᵢ|`.
    ///
    /// Delegates to [`vector_norm_l1`](crate::vector_norm_l1). The body is there so that a caller
    /// holding a slice — which is what `deep_causality_multivector` holds — reaches the same norm
    /// without allocating a vector to put it in.
    pub fn norm_l1(&self) -> <T as deep_causality_algebra::Normed>::Real {
        crate::algorithms::norms::vector_norm_l1(&self.data)
    }

    /// The 2-norm, `sqrt(Σ |aᵢ|²)`.
    ///
    /// Delegates to [`vector_norm_l2`](crate::vector_norm_l2).
    pub fn norm_l2(&self) -> <T as deep_causality_algebra::Normed>::Real {
        crate::algorithms::norms::vector_norm_l2(&self.data)
    }

    /// The squared 2-norm, without the square root.
    ///
    /// Delegates to [`vector_norm_sq`](crate::vector_norm_sq). Available separately because the
    /// square root is the expensive part and comparisons rarely need it.
    pub fn norm_sq(&self) -> <T as deep_causality_algebra::Normed>::Real {
        crate::algorithms::norms::vector_norm_sq(&self.data)
    }

    /// The ∞-norm, `max |aᵢ|`.
    ///
    /// Delegates to [`vector_norm_inf`](crate::vector_norm_inf). Zero for the empty vector, and
    /// never `NaN`.
    pub fn norm_inf(&self) -> <T as deep_causality_algebra::Normed>::Real {
        crate::algorithms::norms::vector_norm_inf(&self.data)
    }
}
