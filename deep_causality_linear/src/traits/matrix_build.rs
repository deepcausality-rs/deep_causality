/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::linear_error::LinearError;
use crate::traits::matrix_view::MatrixView;
use deep_causality_num::One;

/// Construction and mutation of a matrix, position by position.
///
/// This is what lets an algorithm return a *new* matrix — a kernel basis, an image basis, an
/// inverse — without naming a concrete representation. It is separate from
/// [`MatrixView`](crate::traits::matrix_view::MatrixView) because reading and building are needed
/// independently: a caller may read a `CausalTensor` it did not build, and an algorithm may build a
/// result it never reads back.
pub trait MatrixBuild: MatrixView {
    /// A matrix of the given shape with every entry zero.
    ///
    /// For a sparse representation this stores nothing, so the cost is the row-pointer array rather
    /// than `rows * cols`.
    fn zeros(rows: usize, cols: usize) -> Self;

    /// Writes `value` at `(row, col)`.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if the position is outside the shape.
    ///
    /// [`LinearError::NotBinary`] if the representation cannot hold the value — the bit-packed 𝔽₂
    /// matrix is the case, and it names the offending position rather than reporting only that
    /// something was wrong.
    fn set(&mut self, row: usize, col: usize, value: Self::Scalar) -> Result<(), LinearError>;

    /// The `n x n` identity.
    ///
    /// Defaulted, because it is `zeros` followed by writing `one` down the diagonal for every
    /// representation. A representation with a cheaper construction may override it; none here
    /// does, and the default is written so that overriding is a performance decision rather than a
    /// correctness one.
    fn identity(n: usize) -> Self
    where
        Self: Sized,
        Self::Scalar: One,
    {
        let mut m = Self::zeros(n, n);
        for i in 0..n {
            // The index is in bounds by construction, so this cannot fail. Written as a `let _`
            // rather than an `unwrap` so that a representation whose `set` grows new failure modes
            // does not turn this into a panic without anyone noticing.
            let _ = m.set(i, i, Self::Scalar::one());
        }
        m
    }
}
