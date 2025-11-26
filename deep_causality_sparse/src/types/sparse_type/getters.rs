/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CsrMatrix;
use deep_causality_num::Zero;

impl<T> CsrMatrix<T> {
    pub fn row_ptrs(&self) -> &Vec<usize> {
        &self.row_indices
    }

    pub fn col_indices(&self) -> &Vec<usize> {
        &self.col_indices
    }

    pub fn values(&self) -> &Vec<T> {
        &self.values
    }

    pub fn shape(&self) -> (usize, usize) {
        self.shape
    }
}

impl<T> CsrMatrix<T> {
    /// Retrieves the value at the specified row and column index.
    /// Returns `T::zero()` if the index is out of bounds or the element is zero.
    ///
    /// # Arguments
    /// * `row_idx` - The row index.
    /// * `col_idx` - The column index.
    ///
    /// # Returns
    /// The value at `(row_idx, col_idx)` or `T::zero()` if not found or out of bounds.
    pub fn get_value_at(&self, row_idx: usize, col_idx: usize) -> T
    where
        T: Copy + Zero + PartialEq,
    {
        if row_idx >= self.shape.0 || col_idx >= self.shape.1 {
            return T::zero(); // Out of bounds
        }

        let start = self.row_indices[row_idx];
        let end = self.row_indices[row_idx + 1];

        // Perform a binary search or linear scan for the column index in the current row
        // Given that sparse rows can be short, linear scan is often fine.
        for i in start..end {
            if self.col_indices[i] == col_idx {
                return self.values[i];
            }
        }
        T::zero()
    }
}
