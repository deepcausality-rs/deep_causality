/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CausalMultiField: Field of multivectors with tensor storage.
//!
//! This module provides `CausalMultiField<T>`, a spatial grid of multivectors stored
//! in Matrix Isomorphism representation for efficient Clifford algebra operations.
//!
//! # Architecture
//!
//! Unlike `CausalMultiVector<T>` which stores coefficients directly, `CausalMultiField`
//! stores data in Matrix Representation using `CausalTensor`.
//!
//! # Memory Layout
//!
//! Shape: `[Nx, Ny, Nz, Matrix_Dim, Matrix_Dim]`
//!
//! - Grid dimensions: `Nx, Ny, Nz` (spatial)
//! - Algebra dimensions: `Matrix_Dim × Matrix_Dim` (per-cell multivector as matrix)
//!
//! # Example
//!
//! ```rust,ignore
//! use deep_causality_multivector::CausalMultiField;
//! use deep_causality_metric::Metric;
//!
//! let field = CausalMultiField::<f64>::zeros(
//!     [128, 128, 128],
//!     Metric::Euclidean(3),
//!     [1.0, 1.0, 1.0],
//! );
//! let curl = field.curl();
//! ```
mod algebra;
mod api;
mod arithmetic;
pub mod ops;

use deep_causality_metric::Metric;
use deep_causality_tensor::CausalTensor;

/// A field of multivectors stored as tensors.
///
/// `CausalMultiField` stores multivector data in Matrix Isomorphism representation
/// using `CausalTensor` for efficient computation.
///
/// # Type Parameters
///
/// * `T` - The scalar type (typically `f32` or `f64`).
///
/// # Memory Layout
///
/// Shape: `[Nx, Ny, Nz, Matrix_Dim, Matrix_Dim]`
#[derive(Debug, Clone, PartialEq)]
pub struct CausalMultiField<A, S = A> {
    /// Storage: [Nx, Ny, Nz, Matrix_Dim, Matrix_Dim]
    /// Stored entirely in Matrix Isomorphism representation
    pub(crate) data: CausalTensor<A>,

    /// The metric signature of the underlying Clifford algebra
    pub(crate) metric: Metric,

    /// Grid spacing for differential operators [dx, dy, dz].
    ///
    /// Typed by `S`. Spacing describes the *grid*, not the
    /// field living on it, so a functorial map of the coefficients must leave it alone. When
    /// `A` and `S` coincide, which is every use outside the HKT witness, `S` defaults to `A`
    /// and `CausalMultiField<T>` means exactly what it did before.
    pub(crate) dx: [S; 3],

    /// Grid shape [Nx, Ny, Nz]
    pub(crate) shape: [usize; 3],
}
