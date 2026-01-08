/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! LinearAlgebraBackend trait for advanced matrix operations.

use super::backend_tensor::TensorBackend;
use super::tensor_data::TensorData;

/// Advanced linear algebra operations.
///
/// This trait extends `TensorBackend` with matrix operations required for
/// physics simulations and geometric algebra.
///
/// # Operations
///
/// - `matmul` - General matrix multiplication with broadcasting
/// - `qr` - QR decomposition (Householder reflections)
/// - `svd` - Singular value decomposition (power iteration)
/// - `inverse` - Matrix inversion (Gauss-Jordan)
///
/// # Note on Bounds
///
/// Linear algebra operations require additional trait bounds beyond `TensorData`:
/// - `matmul` requires `Ring` (multiplicative structure)
/// - `qr`, `svd`, `inverse` require `RealField` (floating-point operations)
pub trait LinearAlgebraBackend: TensorBackend {
    /// General matrix multiplication (GEMM).
    ///
    /// Supports broadcasting for batch matrix multiplication:
    /// `[B, M, K] @ [B, K, N] -> [B, M, N]`
    ///
    /// # Type Bounds
    /// - `Ring`: Provides multiplication and addition with identity
    /// - `PartialOrd`: Required for internal dispatch logic
    fn matmul<T>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + deep_causality_num::Ring + Default + PartialOrd;

    /// QR decomposition using Householder reflections.
    ///
    /// Returns (Q, R) where A = Q * R:
    /// - Q is orthogonal (m x m)
    /// - R is upper triangular (m x n)
    fn qr<T>(input: &Self::Tensor<T>) -> (Self::Tensor<T>, Self::Tensor<T>)
    where
        T: TensorData + deep_causality_num::RealField + core::iter::Sum + PartialEq;

    /// Singular Value Decomposition.
    ///
    /// Returns (U, S, Vt) where A ≈ U * diag(S) * Vt:
    /// - U (m x k) — left singular vectors
    /// - S (k,) — singular values
    /// - Vt (k x n) — right singular vectors transposed
    fn svd<T>(input: &Self::Tensor<T>) -> (Self::Tensor<T>, Self::Tensor<T>, Self::Tensor<T>)
    where
        T: TensorData + deep_causality_num::RealField + core::iter::Sum + PartialEq;

    /// Matrix inversion.
    ///
    /// # Panics
    /// Panics if the matrix is singular.
    ///
    /// # Type Bounds
    /// - `RealField`: Floating-point operations for Gaussian elimination
    fn inverse<T>(input: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + deep_causality_num::RealField + core::iter::Sum + PartialEq;

    /// Cholesky decomposition.
    ///
    /// Returns L such that A = L * L^T.
    fn cholesky_decomposition<T>(input: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + deep_causality_num::RealField + core::iter::Sum + PartialEq;

    /// Solves linear least squares using Cholesky decomposition.
    fn solve_least_squares_cholsky<T>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + deep_causality_num::RealField + core::iter::Sum + PartialEq;

    /// Tensor product (outer product).
    fn tensor_product<T>(lhs: &Self::Tensor<T>, rhs: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + deep_causality_num::Ring + Default + PartialOrd;
}
