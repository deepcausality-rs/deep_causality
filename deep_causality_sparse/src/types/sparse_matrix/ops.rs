/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CsrMatrix;
use crate::errors::SparseMatrixError;
use deep_causality_num::{One, Zero};
use std::ops::{Mul, Sub};

impl<T> CsrMatrix<T>
where
    T: Copy + Zero + PartialEq + Default,
{
    /// Performs matrix addition: \( C = A + B \).
    ///
    /// Given two matrices \( A \) (self) and \( B \) (other) of the same shape \( m \times n \),
    /// their sum \( C \) is a matrix of the same shape where each element \( C_{ij} \) is the
    /// sum of the corresponding elements in \( A \) and \( B \):
    /// \( C_{ij} = A_{ij} + B_{ij} \).
    ///
    /// Returns a new `CsrMatrix` representing the sum of the two matrices,
    /// or a `SparseMatrixError::ShapeMismatch` if their shapes are not compatible.
    ///
    /// # Arguments
    ///
    /// * `other` - The matrix to add.
    ///
    /// # Errors
    ///
    /// Returns `SparseMatrixError::ShapeMismatch` if the matrices have different dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_sparse::CsrMatrix;
    /// use deep_causality_num::Zero;
    ///
    /// let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    /// // A = [[1.0, 0.0], [0.0, 2.0]]
    ///
    /// let b = CsrMatrix::from_triplets(2, 2, &[(0, 1, 3.0), (1, 0, 4.0)]).unwrap();
    /// // B = [[0.0, 3.0], [4.0, 0.0]]
    ///
    /// let c = a.add_matrix(&b).unwrap();
    /// // C = A + B = [[1.0, 3.0], [4.0, 2.0]]
    ///
    /// assert_eq!(c.get_value_at(0, 0), 1.0);
    /// assert_eq!(c.get_value_at(0, 1), 3.0);
    /// assert_eq!(c.get_value_at(1, 0), 4.0);
    /// assert_eq!(c.get_value_at(1, 1), 2.0);
    /// ```
    pub fn add_matrix(&self, other: &Self) -> Result<Self, SparseMatrixError> {
        if self.shape != other.shape {
            return Err(SparseMatrixError::ShapeMismatch(self.shape, other.shape));
        }

        let rows = self.shape.0;
        let cols = self.shape.1;

        let mut new_row_indices = vec![0; rows + 1];
        let mut new_col_indices = Vec::new();
        let mut new_values = Vec::new();

        for i in 0..rows {
            let mut self_ptr = self.row_indices[i];
            let self_end = self.row_indices[i + 1];

            let mut other_ptr = other.row_indices[i];
            let other_end = other.row_indices[i + 1];

            while self_ptr < self_end || other_ptr < other_end {
                let self_col = if self_ptr < self_end {
                    self.col_indices[self_ptr]
                } else {
                    cols
                };
                let self_val = if self_ptr < self_end {
                    self.values[self_ptr]
                } else {
                    T::zero()
                };

                let other_col = if other_ptr < other_end {
                    other.col_indices[other_ptr]
                } else {
                    cols
                };
                let other_val = if other_ptr < other_end {
                    other.values[other_ptr]
                } else {
                    T::zero()
                };

                if self_col < other_col {
                    if self_val != T::zero() {
                        // Only add if not zero (might be a phantom zero from other_ptr)
                        new_col_indices.push(self_col);
                        new_values.push(self_val);
                    }
                    self_ptr += 1;
                } else if other_col < self_col {
                    if other_val != T::zero() {
                        // Only add if not zero (might be a phantom zero from self_ptr)
                        new_col_indices.push(other_col);
                        new_values.push(other_val);
                    }
                    other_ptr += 1;
                } else {
                    // self_col == other_col
                    let sum_val = self_val + other_val;
                    if sum_val != T::zero() {
                        new_col_indices.push(self_col);
                        new_values.push(sum_val);
                    }
                    self_ptr += 1;
                    other_ptr += 1;
                }
            }
            new_row_indices[i + 1] = new_values.len();
        }

        Ok(Self {
            row_indices: new_row_indices,
            col_indices: new_col_indices,
            values: new_values,
            shape: (rows, cols),
        })
    }
}

impl<T> CsrMatrix<T>
where
    T: Copy + Sub<Output = T> + Zero + PartialEq + Default,
{
    /// Performs matrix subtraction: \( C = A - B \).
    ///
    /// Given two matrices \( A \) (self) and \( B \) (other) of the same shape \( m \times n \),
    /// their difference \( C \) is a matrix of the same shape where each element \( C_{ij} \) is the
    /// difference of the corresponding elements in \( A \) and \( B \):
    /// \( C_{ij} = A_{ij} - B_{ij} \).
    ///
    /// Returns a new `CsrMatrix` representing the difference of the two matrices,
    /// or a `SparseMatrixError::ShapeMismatch` if their shapes are not compatible.
    ///
    /// # Arguments
    ///
    /// * `other` - The matrix to subtract.
    ///
    /// # Errors
    ///
    /// Returns `SparseMatrixError::ShapeMismatch` if the matrices have different dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_sparse::CsrMatrix;
    /// use deep_causality_num::Zero;
    ///
    /// let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 5.0), (0, 1, 2.0), (1, 1, 3.0)]).unwrap();
    /// // A = [[5.0, 2.0], [0.0, 3.0]]
    ///
    /// let b = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 1.0)]).unwrap();
    /// // B = [[1.0, 0.0], [0.0, 1.0]]
    ///
    /// let c = a.sub_matrix(&b).unwrap();
    /// // C = A - B = [[4.0, 2.0], [0.0, 2.0]]
    ///
    /// assert_eq!(c.get_value_at(0, 0), 4.0);
    /// assert_eq!(c.get_value_at(0, 1), 2.0);
    /// assert_eq!(c.get_value_at(1, 0), 0.0);
    /// assert_eq!(c.get_value_at(1, 1), 2.0);
    /// ```
    pub fn sub_matrix(&self, other: &Self) -> Result<Self, SparseMatrixError> {
        if self.shape != other.shape {
            return Err(SparseMatrixError::ShapeMismatch(self.shape, other.shape));
        }

        let rows = self.shape.0;
        let cols = self.shape.1;

        let mut new_row_indices = vec![0; rows + 1];
        let mut new_col_indices = Vec::new();
        let mut new_values = Vec::new();

        for i in 0..rows {
            let mut self_ptr = self.row_indices[i];
            let self_end = self.row_indices[i + 1];

            let mut other_ptr = other.row_indices[i];
            let other_end = other.row_indices[i + 1];

            while self_ptr < self_end || other_ptr < other_end {
                let self_col = if self_ptr < self_end {
                    self.col_indices[self_ptr]
                } else {
                    cols
                };
                let self_val = if self_ptr < self_end {
                    self.values[self_ptr]
                } else {
                    T::zero()
                };

                let other_col = if other_ptr < other_end {
                    other.col_indices[other_ptr]
                } else {
                    cols
                };
                let other_val = if other_ptr < other_end {
                    other.values[other_ptr]
                } else {
                    T::zero()
                };

                if self_col < other_col {
                    if self_val != T::zero() {
                        new_col_indices.push(self_col);
                        new_values.push(self_val);
                    }
                    self_ptr += 1;
                } else if other_col < self_col {
                    let sub_val = T::zero() - other_val; // For elements only in other, treat as 0 - other_val
                    if sub_val != T::zero() {
                        new_col_indices.push(other_col);
                        new_values.push(sub_val);
                    }
                    other_ptr += 1;
                } else {
                    // self_col == other_col
                    let sub_val = self_val - other_val;
                    if sub_val != T::zero() {
                        new_col_indices.push(self_col);
                        new_values.push(sub_val);
                    }
                    self_ptr += 1;
                    other_ptr += 1;
                }
            }
            new_row_indices[i + 1] = new_values.len();
        }

        Ok(Self {
            row_indices: new_row_indices,
            col_indices: new_col_indices,
            values: new_values,
            shape: (rows, cols),
        })
    }
}

impl<T> CsrMatrix<T>
where
    T: Copy + Mul<Output = T> + Zero + PartialEq + Default,
{
    /// Performs scalar multiplication: \( B = s \cdot A \).
    ///
    /// Given a matrix \( A \) and a scalar \( s \), their product \( B \) is a matrix
    /// of the same shape as \( A \), where each element \( B_{ij} \) is the product of
    /// the scalar \( s \) and the corresponding element \( A_{ij} \):
    /// \( B_{ij} = s \cdot A_{ij} \).
    ///
    /// Returns a new `CsrMatrix` where each element is multiplied by the scalar.
    ///
    /// # Arguments
    ///
    /// * `scalar` - The scalar value to multiply by.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_sparse::CsrMatrix;
    /// use deep_causality_num::Zero;
    ///
    /// let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    /// // A = [[1.0, 0.0], [0.0, 2.0]]
    ///
    /// let c = a.scalar_mult(3.0);
    /// // C = 3 * A = [[3.0, 0.0], [0.0, 6.0]]
    ///
    /// assert_eq!(c.get_value_at(0, 0), 3.0);
    /// assert_eq!(c.get_value_at(1, 1), 6.0);
    /// ```
    pub fn scalar_mult(&self, scalar: T) -> Self {
        let new_values: Vec<T> = self.values.iter().map(|&val| val * scalar).collect();
        Self {
            row_indices: self.row_indices.clone(),
            col_indices: self.col_indices.clone(),
            values: new_values,
            shape: self.shape,
        }
    }

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
    /// A `Vec<T>` representing the resulting vector.
    ///
    /// # Panics
    /// Panics if the length of `x` does not match the number of columns in the matrix.
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
    /// let y = a.vec_mult(&x);
    /// // y = Ax = [(1.0*1.0 + 0.0*2.0 + 2.0*3.0), (0.0*1.0 + 3.0*2.0 + 0.0*3.0)] = [7.0, 6.0]
    ///
    /// assert_eq!(y, vec![7.0, 6.0]);
    /// ```
    pub fn vec_mult(&self, x: &[T]) -> Vec<T> {
        let rows = self.shape.0;
        let mut y = Vec::with_capacity(rows);

        for i in 0..rows {
            let start = self.row_indices[i];
            let end = self.row_indices[i + 1];

            let mut sum = T::zero();

            // This loop is highly vectorizable due to SoA layout
            for j in start..end {
                let col = self.col_indices[j];
                let val = self.values[j];
                sum = sum + (val * x[col]);
            }
            y.push(sum);
        }
        y
    }

    /// Performs matrix multiplication: \( C = A \cdot B \).
    ///
    /// Given two matrices \( A \) (self) of shape \( m \times k \) and \( B \) (other) of shape \( k \times n \),
    /// their product \( C \) is a matrix of shape \( m \times n \). Each element \( C_{ij} \) is the
    /// dot product of the \( i \)-th row of \( A \) and the \( j \)-th column of \( B \):
    /// \( C_{ij} = \sum_{p=0}^{k-1} A_{ip} B_{pj} \).
    ///
    /// Returns a new `CsrMatrix` representing the product of the two matrices,
    /// or a `SparseMatrixError::DimensionMismatch` if their dimensions are not compatible.
    ///
    /// # Arguments
    ///
    /// * `other` - The matrix to multiply by.
    ///
    /// # Errors
    ///
    /// Returns `SparseMatrixError::DimensionMismatch` if the matrices have incompatible dimensions
    /// for multiplication (self.cols != other.rows).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_sparse::CsrMatrix;
    /// use deep_causality_num::Zero;
    ///
    /// let a = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    /// // A (2x3) = [[1.0, 0.0, 2.0], [0.0, 3.0, 0.0]]
    ///
    /// let b = CsrMatrix::from_triplets(3, 2, &[(0, 0, 4.0), (1, 1, 5.0), (2, 0, 6.0)]).unwrap();
    /// // B (3x2) = [[4.0, 0.0], [0.0, 5.0], [6.0, 0.0]]
    ///
    /// let c = a.mat_mult(&b).unwrap();
    /// // C = A * B (2x2) = [[(1*4+0*0+2*6), (1*0+0*5+2*0)], [(0*4+3*0+0*6), (0*0+3*5+0*0)]]
    /// //                 = [[16.0, 0.0], [0.0, 15.0]]
    ///
    /// assert_eq!(c.get_value_at(0, 0), 16.0);
    /// assert_eq!(c.get_value_at(0, 1), 0.0);
    /// assert_eq!(c.get_value_at(1, 0), 0.0);
    /// assert_eq!(c.get_value_at(1, 1), 15.0);
    /// ```
    pub fn mat_mult(&self, other: &Self) -> Result<Self, SparseMatrixError> {
        if self.shape.1 != other.shape.0 {
            return Err(SparseMatrixError::DimensionMismatch(
                self.shape.1,
                other.shape.0,
            ));
        }

        let rows = self.shape.0;
        let cols = other.shape.1; // other.cols

        let mut new_row_indices = vec![0; rows + 1];
        let mut new_col_indices = Vec::new();
        let mut new_values = Vec::new();

        // Max number of non-zero elements in a row of the result matrix can be 'cols'
        // This temporary array will store the values for the current row being computed.
        // Initialize with T::zero() or a way to mark "empty"
        let mut temp_row_values = vec![T::zero(); cols];
        let mut temp_row_cols = Vec::with_capacity(cols); // To store actual non-zero columns

        for i in 0..rows {
            // Iterate through rows of self (result matrix row index)
            // Reset temp_row for current row calculation
            for &c in temp_row_cols.iter() {
                temp_row_values[c] = T::zero(); // Clear previous values
            }
            temp_row_cols.clear();

            let self_row_start = self.row_indices[i];
            let self_row_end = self.row_indices[i + 1];

            for k_ptr in self_row_start..self_row_end {
                // Iterate non-zeros in self's row 'i'
                let k_col = self.col_indices[k_ptr]; // Column index in self (also row index in other)
                let self_val = self.values[k_ptr]; // Value A_ik

                let other_row_start = other.row_indices[k_col];
                let other_row_end = other.row_indices[k_col + 1];

                for j_ptr in other_row_start..other_row_end {
                    // Iterate non-zeros in other's row 'k_col'
                    let j_col = other.col_indices[j_ptr]; // Column index in other (also column index in result)
                    let other_val = other.values[j_ptr]; // Value B_kj

                    let product = self_val * other_val;
                    if product != T::zero() {
                        // If this is the first time we're adding to this column in the temp_row
                        if temp_row_values[j_col] == T::zero() {
                            temp_row_cols.push(j_col);
                        }
                        temp_row_values[j_col] = temp_row_values[j_col] + product;
                    }
                }
            }

            // Add non-zero elements from temp_row to result CSR structure
            temp_row_cols.sort_unstable(); // Ensure columns are sorted for CSR format
            for &c in temp_row_cols.iter() {
                let val = temp_row_values[c];
                if val != T::zero() {
                    // Only add non-zero values
                    new_col_indices.push(c);
                    new_values.push(val);
                }
            }
            new_row_indices[i + 1] = new_values.len();
        }

        Ok(Self {
            row_indices: new_row_indices,
            col_indices: new_col_indices,
            values: new_values,
            shape: (rows, cols),
        })
    }
}

impl<T> CsrMatrix<T>
where
    T: Copy + Zero + One + PartialEq + Default,
{
    /// Computes the transpose of the matrix: \( B = A^T \).
    ///
    /// Given a matrix \( A \) of shape \( m \times n \), its transpose \( B \) is a matrix
    /// of shape \( n \times m \), where the rows of \( A \) become the columns of \( B \)
    /// and the columns of \( A \) become the rows of \( B \). Formally,
    /// \( B_{ij} = A_{ji} \).
    ///
    /// Returns a new `CsrMatrix` representing the transpose.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_sparse::CsrMatrix;
    /// use deep_causality_num::Zero;
    ///
    /// let a = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    /// // A (2x3) = [[1.0, 0.0, 2.0], [0.0, 3.0, 0.0]]
    ///
    /// let a_t = a.transpose();
    /// // A^T (3x2) = [[1.0, 0.0], [0.0, 3.0], [2.0, 0.0]]
    ///
    /// assert_eq!(a_t.shape(), (3, 2));
    /// assert_eq!(a_t.get_value_at(0, 0), 1.0);
    /// assert_eq!(a_t.get_value_at(1, 1), 3.0);
    /// assert_eq!(a_t.get_value_at(2, 0), 2.0);
    /// ```
    pub fn transpose(&self) -> Self {
        let (rows, cols) = self.shape;
        let num_elements = self.values.len();

        let mut b_col_indices = vec![0; num_elements];
        let mut b_values = vec![T::default(); num_elements]; // Use T::default() here
        let mut b_row_ptrs = vec![0; cols + 1];

        // Count elements per column in original matrix (which will be rows in transpose)
        for &col_idx in self.col_indices.iter() {
            b_row_ptrs[col_idx + 1] += 1;
        }

        // Cumulative sum to get row pointers for transpose
        for i in 0..cols {
            b_row_ptrs[i + 1] += b_row_ptrs[i];
        }

        // Fill col_indices and values for transpose
        let mut current_pos = b_row_ptrs.clone(); // Use as write pointers for each row
        for i in 0..rows {
            let start = self.row_indices[i];
            let end = self.row_indices[i + 1];

            for k in start..end {
                let col = self.col_indices[k];
                let val = self.values[k];

                b_col_indices[current_pos[col]] = i; // Original row becomes new column
                b_values[current_pos[col]] = val;
                current_pos[col] += 1;
            }
        }

        Self {
            row_indices: b_row_ptrs,
            col_indices: b_col_indices,
            values: b_values,
            shape: (cols, rows), // Transposed shape
        }
    }
}
