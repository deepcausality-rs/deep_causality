/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CsrMatrix, SparseMatrixError};
use deep_causality_num::Zero;
use std::ops::Mul;

impl<T> CsrMatrix<T> {
    /// Performs matrix-vector multiplication: \( y = Ax \).
    ///
    /// Given a matrix \( A \) of shape \( m \times n \) (self) and a vector \( x \) of length \( n \),
    /// their product \( y \) is a vector of length \( m \), where each element \( y_i \) is the
    /// dot product of the \( i \)-th row of \( A \) and the vector \( x \):
    /// \( y_i = \sum_{j=0}^{n-1} A_{ij} x_j \).
    ///
    /// # Arguments
    /// * `x` - The vector to multiply by. It is expected to have a length equal to the number of columns in the matrix.
    ///
    /// # Returns
    /// A `Result<Vec<T>, SparseMatrixError>` representing the resulting vector, or an error.
    ///
    /// # Errors
    /// Returns `SparseMatrixError::DimensionMismatch` if the length of `x` does not match
    /// the number of columns in the matrix (`self.shape.1`).
    ///
    /// # Examples
    /// ```
    /// use deep_causality_sparse::CsrMatrix;
    /// use deep_causality_num::Zero;
    ///
    /// let a = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    /// // A = [[1.0, 0.0, 2.0], [0.0, 3.0, 0.0]]
    ///
    /// let x = vec![1.0, 2.0, 3.0];
    ///
    /// let y = a.vec_mult(&x).unwrap();
    /// // y = Ax = [(1.0*1.0 + 0.0*2.0 + 2.0*3.0), (0.0*1.0 + 3.0*2.0 + 0.0*3.0)] = [7.0, 6.0]
    ///
    /// assert_eq!(y, vec![7.0, 6.0]);
    ///
    /// // Example of error handling
    /// let x_invalid = vec![1.0, 2.0]; // Incorrect length
    /// let err = a.vec_mult(&x_invalid).unwrap_err();
    /// assert!(matches!(err, deep_causality_sparse::SparseMatrixError::DimensionMismatch(3, 2)));
    /// ```
    pub(crate) fn vec_mult_impl(&self, x: &[T]) -> Result<Vec<T>, SparseMatrixError>
    where
        T: Copy + Zero + std::ops::Add<Output = T> + Mul<Output = T>,
    {
        if x.len() != self.shape.1 {
            return Err(SparseMatrixError::DimensionMismatch(self.shape.1, x.len()));
        }

        let rows = self.shape.0;
        let mut y = Vec::with_capacity(rows);

        // We assume T is cheap to copy (f32, f64, etc).
        // Iterate through rows.
        for i in 0..rows {
            let start = self.row_indices[i];
            let end = self.row_indices[i + 1];

            // Create slice views for the current row.
            // This helps the compiler eliminate bounds checks on 'self' arrays
            // within the inner loop, as the slice length is known.
            let row_cols = &self.col_indices[start..end];
            let row_vals = &self.values[start..end];

            // 4 Independent accumulators to break dependency chains.
            // This allows the CPU to pipeline the math operations.
            let mut sum0 = T::zero();
            let mut sum1 = T::zero();
            let mut sum2 = T::zero();
            let mut sum3 = T::zero();

            // Process 4 elements at a time.
            let mut chunks_cols = row_cols.chunks_exact(4);
            let mut chunks_vals = row_vals.chunks_exact(4);

            // The main "Pseudo-SIMD" loop.
            // The iterator handles the loop logic efficiently.
            while let (Some(c), Some(v)) = (chunks_cols.next(), chunks_vals.next()) {
                // Note: x[c[0]] involves a bounds check.
                // Without 'unsafe', we cannot remove it, but splitting the
                // accumulators helps hide the latency of that check/load.
                sum0 = sum0 + (v[0] * x[c[0]]);
                sum1 = sum1 + (v[1] * x[c[1]]);
                sum2 = sum2 + (v[2] * x[c[2]]);
                sum3 = sum3 + (v[3] * x[c[3]]);
            }

            // Reduce the parallel accumulators
            let mut final_sum = (sum0 + sum1) + (sum2 + sum3);

            // Handle the remainder (0 to 3 elements left)
            // The compiler will likely unroll this small loop or turn it into a jump table.
            let rem_cols = chunks_cols.remainder();
            let rem_vals = chunks_vals.remainder();

            for (&c, &v) in rem_cols.iter().zip(rem_vals.iter()) {
                final_sum = final_sum + (v * x[c]);
            }

            y.push(final_sum);
        }
        Ok(y)
    }
}
