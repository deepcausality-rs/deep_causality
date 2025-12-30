/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CausalMultiField: Hardware-accelerated field of multivectors.
//!
//! This module provides `CausalMultiField<B, T>`, a spatial grid of multivectors stored
//! in Matrix Isomorphism representation for efficient GPU-accelerated Clifford algebra operations.
//!
//! # Architecture
//!
//! Unlike `CausalMultiVector<T>` which stores coefficients directly, `CausalMultiField`
//! stores data in Matrix Representation on the backend device. This enables:
//!
//! - All Clifford operations map to `LinearAlgebraBackend::matmul`
//! - Zero per-cell conversion overhead for field operations
//! - Transparent MLX/CUDA acceleration via feature flags
//!
//! # Memory Layout
//!
//! Shape: `[Nx, Ny, Nz, Matrix_Dim, Matrix_Dim]`
//!
//! - Grid dimensions: `Nx, Ny, Nz` (spatial)
//! - Algebra dimensions: `Matrix_Dim × Matrix_Dim` (per-cell multivector as matrix)
//!
//! # Precision
//!
//! MLX backend operates in f32. When using `CausalMultiField<MlxBackend, f64>`,
//! data is automatically downcast to f32 for GPU execution and upcast on retrieval.
//!
//! # Example
//!
//! ```rust,ignore
//! use deep_causality_multivector::{CausalMultiField, CpuBackend};
//! use deep_causality_metric::Metric;
//!
//! let field = CausalMultiField::<CpuBackend, f64>::zeros(
//!     [128, 128, 128],
//!     Metric::Euclidean(3),
//!     [1.0, 1.0, 1.0],
//! );
//! let curl = field.curl();
//! ```
mod arithmetic;

pub mod gamma;
mod ops;

use deep_causality_metric::Metric;
use deep_causality_tensor::{LinearAlgebraBackend, TensorData};

/// A hardware-accelerated field of multivectors.
///
/// `CausalMultiField` stores multivector data in Matrix Isomorphism representation
/// directly on the backend device, enabling efficient GPU/accelerator computation
/// of Clifford algebra operations.
///
/// # Type Parameters
///
/// * `B` - The compute backend ([`CpuBackend`], [`MlxBackend`]).
/// * `T` - The scalar type (typically `f32` or `f64`).
///
/// # Memory Layout
///
/// Shape: `[Nx, Ny, Nz, Matrix_Dim, Matrix_Dim]`
#[derive(Debug, Clone)]
pub struct CausalMultiField<B: LinearAlgebraBackend, T: TensorData> {
    /// Storage: [Nx, Ny, Nz, Matrix_Dim, Matrix_Dim]
    /// Stored entirely in Matrix Isomorphism representation on Device
    data: B::Tensor<T>,

    /// The metric signature of the underlying Clifford algebra
    metric: Metric,

    /// Grid spacing for differential operators [dx, dy, dz]
    dx: [T; 3],

    /// Grid shape [Nx, Ny, Nz]
    shape: [usize; 3],
}

impl<B: LinearAlgebraBackend, T: TensorData> CausalMultiField<B, T> {
    /// Returns the metric signature of the algebra.
    #[inline]
    pub fn metric(&self) -> Metric {
        self.metric
    }

    /// Returns the grid spacing.
    #[inline]
    pub fn dx(&self) -> &[T; 3] {
        &self.dx
    }

    /// Returns the grid shape [Nx, Ny, Nz].
    #[inline]
    pub fn shape(&self) -> &[usize; 3] {
        &self.shape
    }

    /// Returns the total number of grid cells.
    #[inline]
    pub fn num_cells(&self) -> usize {
        self.shape[0] * self.shape[1] * self.shape[2]
    }

    /// Returns the matrix dimension for the algebra.
    ///
    /// For Cl(p,q,r), the matrix dimension is 2^⌈N/2⌉ where N = p + q + r.
    #[inline]
    pub fn matrix_dim(&self) -> usize {
        let n = self.metric.dimension();
        1 << n.div_ceil(2)
    }

    /// Returns a reference to the underlying backend tensor.
    #[inline]
    pub fn data(&self) -> &B::Tensor<T> {
        &self.data
    }
}
