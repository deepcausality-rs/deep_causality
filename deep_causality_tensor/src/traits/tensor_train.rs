/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::causal_tensor_network::cross_config::CrossConfig;
use crate::types::causal_tensor_network::truncation::Truncation;
use crate::{CausalTensor, CausalTensorError};

/// Behaviour of a tensor train (matrix-product state).
///
/// This is the trait/`CausalTensorTrain` split that mirrors the existing
/// [`Tensor`](crate::Tensor) / [`CausalTensor`](crate::CausalTensor) pairing: the trait declares the
/// transformation and query surface, while constructors stay inherent on the concrete type. The
/// implementation is precision-generic over the scalar (`Scalar` bound at the impl site).
///
/// Lossless operations (`add`, `hadamard`) grow bond dimension exactly; the paired `*_rounded`
/// variants recompress to a [`Truncation`]. The algebraic laws they satisfy hold exactly without
/// truncation and to the truncation tolerance otherwise.
pub trait TensorTrain<T>: Sized {
    /// Contracts the train back to a dense tensor.
    ///
    /// # Errors
    /// [`CausalTensorError::RankExceeded`] if the dense tensor would exceed the element-count guard.
    fn to_dense(&self) -> Result<CausalTensor<T>, CausalTensorError>;

    /// Evaluates a single logical entry `A[index]` without materializing the dense tensor.
    ///
    /// # Errors
    /// [`CausalTensorError::DimensionMismatch`] if `index` length differs from the order, or
    /// [`CausalTensorError::IndexOutOfBounds`] if any component is out of range.
    fn eval(&self, index: &[usize]) -> Result<T, CausalTensorError>;

    /// Left-canonicalizes the train via a left-to-right QR sweep (cores `0..order-1` become
    /// left-orthonormal). Lossless; also removes redundant bond rank.
    fn left_canonicalize(&self) -> Result<Self, CausalTensorError>;

    /// Right-canonicalizes the train via a right-to-left LQ sweep.
    fn right_canonicalize(&self) -> Result<Self, CausalTensorError>;

    /// Brings the orthogonality centre to core `center` (mixed-canonical form).
    ///
    /// # Errors
    /// [`CausalTensorError::IndexOutOfBounds`] if `center >= order`.
    fn canonicalize_at(&self, center: usize) -> Result<Self, CausalTensorError>;

    /// Recompresses the train to the given [`Truncation`] via a left-canonicalize + right-to-left
    /// truncated-SVD sweep.
    fn round(&self, trunc: &Truncation<T>) -> Result<Self, CausalTensorError>;

    /// The Frobenius norm of the represented tensor.
    fn norm(&self) -> Result<T, CausalTensorError>;

    /// The inner product `⟨self, other⟩` of two trains over the same physical dimensions.
    ///
    /// # Errors
    /// [`CausalTensorError::ShapeMismatch`] if the physical dimensions differ.
    fn inner(&self, other: &Self) -> Result<T, CausalTensorError>;

    /// Adds two trains exactly (bond dimensions add). Use [`TensorTrain::add_rounded`] to recompress.
    ///
    /// # Errors
    /// [`CausalTensorError::ShapeMismatch`] if the physical dimensions differ.
    fn add(&self, other: &Self) -> Result<Self, CausalTensorError>;

    /// `add` followed by `round`.
    fn add_rounded(&self, other: &Self, trunc: &Truncation<T>) -> Result<Self, CausalTensorError>;

    /// Adds a scalar constant to every entry (exact affine offset via a rank-1 ones-train).
    fn add_scalar(&self, c: T) -> Result<Self, CausalTensorError>;

    /// Elementwise (Hadamard) product of two trains (bond dimensions multiply). Use
    /// [`TensorTrain::hadamard_rounded`] to recompress.
    ///
    /// # Errors
    /// [`CausalTensorError::ShapeMismatch`] if the physical dimensions differ.
    fn hadamard(&self, other: &Self) -> Result<Self, CausalTensorError>;

    /// `hadamard` followed by `round`.
    fn hadamard_rounded(
        &self,
        other: &Self,
        trunc: &Truncation<T>,
    ) -> Result<Self, CausalTensorError>;

    /// Sums out the given physical sites, returning a train over the remaining sites.
    ///
    /// # Errors
    /// - [`CausalTensorError::IndexOutOfBounds`] if any site is out of range.
    /// - [`CausalTensorError::InvalidParameter`] if every site would be summed out.
    fn marginalize(&self, sites: &[usize]) -> Result<Self, CausalTensorError>;

    /// Applies a general nonlinear scalar map `f` to every logical entry, returning a *new*
    /// approximate train and a sampled residual.
    ///
    /// A nonlinear map of a tensor train has no exact local form (and can inflate rank), so this
    /// re-approximates `f∘self` by TT-cross over the oracle `i ↦ f(self.eval(i))`. The returned
    /// residual makes the approximation explicit. For exact linear/affine maps use `scale` /
    /// `add_scalar` instead.
    ///
    /// # Errors
    /// Propagates [`CausalTensorError::CrossSampleFailure`] if `f` or evaluation produces a
    /// non-finite value, and other cross errors.
    fn apply_nonlinear<F>(
        &self,
        f: F,
        config: &CrossConfig<T>,
    ) -> Result<(Self, T), CausalTensorError>
    where
        F: FnMut(T) -> T;
}
