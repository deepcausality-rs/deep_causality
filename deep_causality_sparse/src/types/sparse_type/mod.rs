/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod default;
mod getters;
mod iterators;
mod ops;

use crate::SparseMatrixError;
use deep_causality_num::{One, Zero};
use std::ops::{Add, Mul, Sub};

/// A Compressed Sparse Row Matrix.
#[derive(Clone, Debug)]
pub struct CsrMatrix<T> {
    pub(crate) row_indices: Vec<usize>,
    pub(crate) col_indices: Vec<usize>,
    pub(crate) values: Vec<T>,
    pub(crate) shape: (usize, usize), // (Rows, Cols)
}

impl<T> CsrMatrix<T> {
    pub fn new() -> Self {
        Self {
            row_indices: Vec::new(),
            col_indices: Vec::new(),
            values: Vec::new(),
            shape: (0, 0),
        }
    }

    pub fn with_capacity(rows: usize, cols: usize, capacity: usize) -> Self {
        Self {
            row_indices: vec![0; rows + 1],
            col_indices: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            shape: (rows, cols),
        }
    }
}
impl<T> CsrMatrix<T>
where
    T: Copy
        + Clone
        + Mul<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Zero
        + One
        + PartialEq
        + Default,
{
    /// Creates a new `CsrMatrix` from a list of `(row, col, value)` triplets.
    ///
    /// The triplets are sorted, and duplicate `(row, col)` entries have their
    /// values summed.
    ///
    /// # Arguments
    /// * `rows` - The number of rows in the matrix.
    /// * `cols` - The number of columns in the matrix.
    /// * `triplets` - A slice of `(row_idx, col_idx, value)` tuples.
    ///
    /// # Errors
    /// Returns `SparseMatrixError::IndexOutOfBounds` if any triplet's indices
    /// are outside the specified matrix dimensions.
    ///
    /// # Examples
    /// ```
    /// use deep_causality_sparse::CsrMatrix;
    /// use deep_causality_num::Zero;
    ///
    /// // Example 1: Basic construction
    /// let triplets1 = vec![(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)];
    /// let matrix1 = CsrMatrix::from_triplets(2, 3, &triplets1).unwrap();
    /// // matrix1 will represent:
    /// // [1.0, 0.0, 2.0]
    /// // [0.0, 3.0, 0.0]
    /// assert_eq!(matrix1.values(), &vec![1.0, 2.0, 3.0]);
    /// assert_eq!(matrix1.col_indices(), &vec![0, 2, 1]);
    /// assert_eq!(matrix1.row_indices(), &vec![0, 2, 3]);
    /// assert_eq!(matrix1.shape(), (2, 3));
    ///
    /// // Example 2: With duplicate entries
    /// let triplets2 = vec![(0, 0, 1.0), (0, 0, 0.5), (1, 1, 3.0)];
    /// let matrix2 = CsrMatrix::from_triplets(2, 2, &triplets2).unwrap();
    /// // matrix2 will represent:
    /// // [1.5, 0.0]
    /// // [0.0, 3.0]
    /// assert_eq!(matrix2.values(), &vec![1.5, 3.0]);
    /// assert_eq!(matrix2.col_indices(), &vec![0, 1]);
    /// assert_eq!(matrix2.row_indices(), &vec![0, 1, 2]);
    /// assert_eq!(matrix2.shape(), (2, 2));
    ///
    /// // Example 3: With zero-valued entries after summation
    /// let triplets3 = vec![(0, 0, 1.0), (0, 0, -1.0), (1, 1, 3.0)];
    /// let matrix3 = CsrMatrix::from_triplets(2, 2, &triplets3).unwrap();
    /// // matrix3 will represent:
    /// // [0.0, 0.0]
    /// // [0.0, 3.0]
    /// assert_eq!(matrix3.values(), &vec![3.0]);
    /// assert_eq!(matrix3.col_indices(), &vec![1]);
    /// assert_eq!(matrix3.row_indices(), &vec![0, 0, 1]); // Note: row_indices[0]=0, row_indices[1]=0 as row 0 has no non-zeros
    /// assert_eq!(matrix3.shape(), (2, 2));
    /// ```
    pub fn from_triplets(
        rows: usize,
        cols: usize,
        triplets: &[(usize, usize, T)],
    ) -> Result<Self, SparseMatrixError> {
        let mut processed_triplets = Vec::with_capacity(triplets.len());

        // 1. Validate and filter out zero-value triplets initially
        for &(r, c, v) in triplets.iter() {
            if r >= rows || c >= cols {
                return Err(SparseMatrixError::IndexOutOfBounds(
                    r.max(c),
                    rows.max(cols),
                ));
            }
            if v != T::zero() {
                // Only include non-zero values
                processed_triplets.push((r, c, v));
            }
        }

        // 2. Sort triplets by row, then by column
        processed_triplets.sort_unstable_by_key(|&(r, c, _)| (r, c));

        // 3. Aggregate duplicates and build final lists
        let mut final_col_indices = Vec::new();
        let mut final_values = Vec::new();
        let mut row_ptr_counts = vec![0; rows + 1]; // To count non-zeros per row temporarily

        if processed_triplets.is_empty() {
            return Ok(Self {
                row_indices: vec![0; rows + 1],
                col_indices: final_col_indices,
                values: final_values,
                shape: (rows, cols),
            });
        }

        let mut iter = processed_triplets.into_iter().peekable();

        while let Some((r, c, mut v)) = iter.next() {
            // Sum duplicate entries
            while iter
                .peek()
                .is_some_and(|&(next_r, next_c, _)| next_r == r && next_c == c)
            {
                v = v + iter.next().unwrap().2;
            }

            if v != T::zero() {
                // Only add if the summed value is not zero
                final_col_indices.push(c);
                final_values.push(v);
                row_ptr_counts[r + 1] += 1; // Count non-zero in this row
            }
        }

        // Convert counts to cumulative sum for row_indices
        let mut actual_row_indices = vec![0; rows + 1];
        for i in 0..rows {
            actual_row_indices[i + 1] = actual_row_indices[i] + row_ptr_counts[i + 1];
        }

        Ok(Self {
            row_indices: actual_row_indices,
            col_indices: final_col_indices,
            values: final_values,
            shape: (rows, cols),
        })
    }
}
