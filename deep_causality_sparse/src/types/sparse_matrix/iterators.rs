/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CsrMatrix;

impl<T> CsrMatrix<T>
where
    T: Copy + PartialEq + Default,
{
    /// Returns an iterator over all non-zero elements, yielding `(row_index, col_index, &value)`.
    pub fn iter_non_zeros(&self) -> NonZeroIterator<'_, T> {
        NonZeroIterator::new(self)
    }

    /// Returns an iterator over the rows of the matrix. Each item is an iterator
    /// over the non-zero `(col_index, &value)` pairs in that row.
    pub fn iter_rows(&self) -> RowIterator<'_, T> {
        RowIterator::new(self)
    }

    /// Returns an iterator over the non-zero `(row_index, &value)` pairs for a specific column.
    /// Note: This is generally inefficient for CSR matrices.
    ///
    /// # Arguments
    /// * `col_idx` - The index of the column to iterate over.
    ///
    /// # Panics
    /// Panics if `col_idx` is out of bounds.
    pub fn iter_cols(&self, col_idx: usize) -> ColIterator<'_, T> {
        assert!(
            col_idx < self.shape.1,
            "Column index out of bounds: {} for matrix with {} columns",
            col_idx,
            self.shape.1
        );
        ColIterator::new(self, col_idx)
    }
}

// --- NonZeroIterator ---

/// An iterator that yields `(row_index, col_index, &value)` for all non-zero elements
/// in a `CsrMatrix`.
pub struct NonZeroIterator<'a, T> {
    matrix: &'a CsrMatrix<T>,
    current_row: usize,
    current_ptr: usize, // Pointer into matrix.col_indices and matrix.values
}

impl<'a, T> NonZeroIterator<'a, T> {
    pub(crate) fn new(matrix: &'a CsrMatrix<T>) -> Self {
        Self {
            matrix,
            current_row: 0,
            current_ptr: 0,
        }
    }
}

impl<'a, T> Iterator for NonZeroIterator<'a, T> {
    type Item = (usize, usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        // Skip empty rows until we find one with non-zero elements or reach the end
        while self.current_row < self.matrix.shape.0 {
            let start_ptr_for_current_row = self.matrix.row_indices[self.current_row];
            let end_ptr_for_current_row = self.matrix.row_indices[self.current_row + 1];

            if self.current_ptr < start_ptr_for_current_row
                || self.current_ptr >= end_ptr_for_current_row
            {
                // Either current_ptr is before this row's elements (e.g., from advancing rows)
                // or current_ptr has gone past this row's elements.
                // Reset current_ptr to the start of this row's elements.
                self.current_ptr = start_ptr_for_current_row;
            }

            if self.current_ptr < end_ptr_for_current_row {
                let col_idx = self.matrix.col_indices[self.current_ptr];
                let value = &self.matrix.values[self.current_ptr];
                let row_idx = self.current_row;

                self.current_ptr += 1;
                return Some((row_idx, col_idx, value));
            } else {
                // This row is exhausted, move to the next.
                self.current_row += 1;
            }
        }
        None // End of matrix
    }
}

// --- SingleRowIterator (Helper for RowIterator) ---

/// An iterator that yields `(col_index, &value)` for non-zero elements
/// in a single row of a `CsrMatrix`.
pub struct SingleRowIterator<'a, T> {
    matrix: &'a CsrMatrix<T>,
    current_ptr: usize,
    end_ptr: usize,
}

impl<'a, T> Iterator for SingleRowIterator<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_ptr < self.end_ptr {
            let col_idx = self.matrix.col_indices[self.current_ptr];
            let value = &self.matrix.values[self.current_ptr];
            self.current_ptr += 1;
            Some((col_idx, value))
        } else {
            None
        }
    }
}

// --- RowIterator ---

/// An iterator that yields a `SingleRowIterator` for each row of a `CsrMatrix`.
pub struct RowIterator<'a, T> {
    matrix: &'a CsrMatrix<T>,
    current_row: usize,
}

impl<'a, T> RowIterator<'a, T> {
    pub(crate) fn new(matrix: &'a CsrMatrix<T>) -> Self {
        Self {
            matrix,
            current_row: 0,
        }
    }
}

impl<'a, T> Iterator for RowIterator<'a, T> {
    type Item = SingleRowIterator<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row < self.matrix.shape.0 {
            let start_ptr = self.matrix.row_indices[self.current_row];
            let end_ptr = self.matrix.row_indices[self.current_row + 1];
            self.current_row += 1;
            Some(SingleRowIterator {
                matrix: self.matrix,
                current_ptr: start_ptr,
                end_ptr,
            })
        } else {
            None
        }
    }
}

// --- ColIterator (conceptual/less efficient in CSR) ---

/// An iterator that yields `(row_index, &value)` for all non-zero elements
/// in a specific column of a `CsrMatrix`.
/// This is generally less efficient for CSR matrices.
pub struct ColIterator<'a, T> {
    matrix: &'a CsrMatrix<T>,
    target_col: usize,
    current_row_idx: usize,        // Current row being checked
    current_col_ptr_in_row: usize, // Pointer into matrix.col_indices for the current_row_idx
}

impl<'a, T> ColIterator<'a, T> {
    pub(crate) fn new(matrix: &'a CsrMatrix<T>, target_col: usize) -> Self {
        Self {
            matrix,
            target_col,
            current_row_idx: 0,
            current_col_ptr_in_row: 0,
        }
    }
}

impl<'a, T> Iterator for ColIterator<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_row_idx < self.matrix.shape.0 {
            let row_start_ptr = self.matrix.row_indices[self.current_row_idx];
            let row_end_ptr = self.matrix.row_indices[self.current_row_idx + 1];

            // If starting a new row or already exhausted elements from previous check in this row
            if self.current_col_ptr_in_row < row_start_ptr
                || self.current_col_ptr_in_row >= row_end_ptr
            {
                self.current_col_ptr_in_row = row_start_ptr;
            }

            // Iterate through non-zero elements of the current row
            while self.current_col_ptr_in_row < row_end_ptr {
                let col_idx = self.matrix.col_indices[self.current_col_ptr_in_row];
                let value = &self.matrix.values[self.current_col_ptr_in_row];

                if col_idx == self.target_col {
                    let result_row_idx = self.current_row_idx;
                    self.current_col_ptr_in_row += 1; // Advance for next search
                    return Some((result_row_idx, value));
                }
                self.current_col_ptr_in_row += 1;
            }

            // Current row exhausted, move to the next row
            self.current_row_idx += 1;
        }
        None // End of matrix
    }
}
