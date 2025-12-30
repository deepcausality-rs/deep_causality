/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! MatrixRep trait for converting between coefficient and matrix representations.
//!
//! This trait enables the Matrix Isomorphism acceleration strategy,
//! mapping Clifford algebra operations to matrix multiplication.

use deep_causality_metric::Metric;
use deep_causality_tensor::{LinearAlgebraBackend, TensorData};

/// Converts between multivector coefficients and matrix representation.
///
/// This trait is the core of the Matrix Isomorphism Bridge, enabling
/// Clifford algebra operations to be accelerated via backend matmul.
///
/// # Theory
///
/// Every Clifford algebra Cl(p,q,r) has a faithful matrix representation.
/// A multivector A = Σ aᵢ eᵢ can be written as a matrix:
///
/// M(A) = Σ aᵢ Γᵢ
///
/// where Γᵢ are the gamma matrices (matrix representations of basis blades).
///
/// # Type Parameters
///
/// * `B` - The backend where the matrix will be stored.
/// * `T` - The scalar type of the coefficients.
///
/// # Example
///
/// ```rust,ignore
/// let mv = CausalMultiVector::<f64>::new(...)?;
/// let matrix = mv.to_matrix::<CpuBackend>();
/// // ... perform matmul operations ...
/// let recovered = CausalMultiVector::from_matrix::<CpuBackend>(matrix, metric);
/// ```
pub trait MatrixRep<B: LinearAlgebraBackend, T: TensorData> {
    /// Transforms coefficients to Matrix Representation on Device.
    ///
    /// Op: Tensor Contraction (Coeffs * Gammas)
    fn to_matrix(&self) -> B::Tensor<T>;

    /// Transforms Matrix Representation back to coefficients.
    ///
    /// Op: Projection / Trace
    fn from_matrix(matrix: B::Tensor<T>, metric: Metric) -> Self;
}
