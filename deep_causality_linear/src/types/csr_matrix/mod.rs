/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A compressed-sparse-row matrix.

pub mod algebra;
pub mod ops;

use crate::errors::linear_error::LinearError;
use alloc::vec::Vec;
use deep_causality_algebra::CommutativeSemiring;
use deep_causality_num::Zero;

/// A matrix in compressed sparse row form.
///
/// Moves here from `deep_causality_sparse` with its public surface unchanged, so that code written
/// against the retired crate compiles against the new path and returns identical results.
///
/// # The read side only
///
/// This type implements [`MatrixView`](crate::MatrixView) and deliberately **not**
/// [`RowOps`](crate::RowOps). `swap_rows` on CSR is fine; `axpy_rows` is not, because adding a
/// multiple of one sparse row to another changes that row's non-zero pattern, which in CSR means
/// reallocating every row after it. Sparse elimination needs a fill-reducing ordering and a symbolic
/// factorisation, which is a different algorithm and a separate proposal.
///
/// A caller who wants to eliminate on a sparse matrix converts to a dense layout, and writes that
/// conversion at the call site so its cost is visible rather than hidden inside an algorithm.
///
/// # What it is for
///
/// Matrix–vector products against a matrix that is mostly zeros. `deep_causality_topology`'s
/// boundary and coboundary operators are `CsrMatrix<i8>` with entries in `{-1, 0, 1}`, and
/// `deep_causality_physics` applies Hodge-star and coboundary operators to fields. The work is
/// proportional to the number of stored entries rather than to `rows * cols`, which is the whole
/// reason the representation exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsrMatrix<T> {
    row_indices: Vec<usize>,
    col_indices: Vec<usize>,
    values: Vec<T>,
    shape: (usize, usize),
}

impl<T> CsrMatrix<T> {
    /// An empty matrix with shape `(0, 0)`.
    pub fn new() -> Self {
        Self {
            row_indices: alloc::vec![0],
            col_indices: Vec::new(),
            values: Vec::new(),
            shape: (0, 0),
        }
    }

    /// A matrix of the given shape with room reserved for `capacity` stored entries.
    pub fn with_capacity(rows: usize, cols: usize, capacity: usize) -> Self {
        Self {
            row_indices: alloc::vec![0; rows + 1],
            col_indices: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            shape: (rows, cols),
        }
    }

    /// Rebuilds from the three arrays and the shape.
    ///
    /// Internal: the caller is responsible for the CSR invariants, which is why this is not public.
    pub(crate) fn from_raw_parts(
        row_indices: Vec<usize>,
        col_indices: Vec<usize>,
        values: Vec<T>,
        shape: (usize, usize),
    ) -> Self {
        Self {
            row_indices,
            col_indices,
            values,
            shape,
        }
    }

    /// The shape, as `(rows, cols)`.
    pub fn shape(&self) -> (usize, usize) {
        self.shape
    }

    /// The row-pointer array, of length `rows + 1`.
    pub fn row_indices(&self) -> &Vec<usize> {
        &self.row_indices
    }

    /// The column index of each stored entry.
    pub fn col_indices(&self) -> &Vec<usize> {
        &self.col_indices
    }

    /// The stored entries, in row-major order of position.
    pub fn values(&self) -> &Vec<T> {
        &self.values
    }

    /// Decomposes into the three arrays and the shape.
    pub fn into_parts(self) -> (Vec<usize>, Vec<usize>, Vec<T>, (usize, usize)) {
        (self.row_indices, self.col_indices, self.values, self.shape)
    }

    /// Applies `f` to every **stored** entry.
    ///
    /// Structural zeros are not visited. A function that does not fix zero therefore changes the
    /// matrix this represents, and the caller is choosing that by calling this rather than
    /// densifying first.
    pub fn map_values<U, F>(self, f: F) -> CsrMatrix<U>
    where
        F: Fn(T) -> U,
    {
        CsrMatrix {
            row_indices: self.row_indices,
            col_indices: self.col_indices,
            values: self.values.into_iter().map(f).collect(),
            shape: self.shape,
        }
    }
}

impl<T> CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    /// Builds from `(row, col, value)` triplets.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if any triplet names a position outside the shape.
    pub fn from_triplets(
        rows: usize,
        cols: usize,
        triplets: &[(usize, usize, T)],
    ) -> Result<Self, LinearError> {
        for &(r, c, _) in triplets {
            if r >= rows || c >= cols {
                return Err(LinearError::IndexOutOfBounds {
                    index: (r, c),
                    shape: (rows, cols),
                });
            }
        }
        let mut sorted: Vec<(usize, usize, T)> = triplets.to_vec();
        sorted.sort_by_key(|&(r, c, _)| (r, c));

        let mut row_indices = alloc::vec![0usize; rows + 1];
        let mut col_indices = Vec::with_capacity(sorted.len());
        let mut values = Vec::with_capacity(sorted.len());
        for (r, c, v) in sorted {
            if v == T::zero() {
                continue;
            }
            col_indices.push(c);
            values.push(v);
            row_indices[r + 1] += 1;
        }
        for i in 0..rows {
            row_indices[i + 1] += row_indices[i];
        }
        Ok(Self {
            row_indices,
            col_indices,
            values,
            shape: (rows, cols),
        })
    }

    /// The entry at `(row, col)`, returning the scalar zero for a position outside the stored
    /// pattern.
    pub fn get_value_at(&self, row_idx: usize, col_idx: usize) -> T
    where
        T: Zero,
    {
        if row_idx >= self.shape.0 || col_idx >= self.shape.1 {
            return T::zero();
        }
        let (start, end) = (self.row_indices[row_idx], self.row_indices[row_idx + 1]);
        for k in start..end {
            if self.col_indices[k] == col_idx {
                return self.values[k];
            }
        }
        T::zero()
    }

    /// The transpose.
    ///
    /// Bounded on `CommutativeSemiring` rather than on `Field`: transposing moves entries and
    /// performs no arithmetic at all, so it is available over ℕ.
    pub fn transpose(&self) -> Self {
        let (r, c) = self.shape;
        let mut triplets = Vec::with_capacity(self.values.len());
        for i in 0..r {
            for k in self.row_indices[i]..self.row_indices[i + 1] {
                triplets.push((self.col_indices[k], i, self.values[k]));
            }
        }
        Self::from_triplets(c, r, &triplets).expect("transposed indices stay inside the shape")
    }

    /// The matrix–vector product.
    ///
    /// Work is proportional to the number of stored entries. The result is dense, because a sparse
    /// matrix times a dense vector generally is.
    ///
    /// # Errors
    ///
    /// [`LinearError::LengthMismatch`] if the vector's length is not the column count.
    pub fn vec_mult(&self, vector: &[T]) -> Result<Vec<T>, LinearError> {
        let (r, c) = self.shape;
        if vector.len() != c {
            return Err(LinearError::LengthMismatch {
                expected: c,
                found: vector.len(),
            });
        }
        // Proportional to the stored entries, which is what the representation is for.
        let out: Vec<T> = (0..r)
            .map(|i| {
                let mut acc = T::zero();
                for k in self.row_indices[i]..self.row_indices[i + 1] {
                    acc = acc + self.values[k] * vector[self.col_indices[k]];
                }
                acc
            })
            .collect();
        Ok(out)
    }

    /// The matrix product.
    ///
    /// # Errors
    ///
    /// [`LinearError::InnerDimensionMismatch`] if the inner dimensions do not meet.
    pub fn mat_mult(&self, other: &Self) -> Result<Self, LinearError> {
        if self.shape.1 != other.shape.0 {
            return Err(LinearError::InnerDimensionMismatch {
                left_cols: self.shape.1,
                right_rows: other.shape.0,
            });
        }
        let (m, n) = (self.shape.0, other.shape.1);
        let mut triplets = Vec::new();
        for i in 0..m {
            let mut acc = alloc::vec![T::zero(); n];
            for k in self.row_indices[i]..self.row_indices[i + 1] {
                let (a_col, a_val) = (self.col_indices[k], self.values[k]);
                for kk in other.row_indices[a_col]..other.row_indices[a_col + 1] {
                    let j = other.col_indices[kk];
                    acc[j] = acc[j] + a_val * other.values[kk];
                }
            }
            for (j, v) in acc.into_iter().enumerate() {
                if v != T::zero() {
                    triplets.push((i, j, v));
                }
            }
        }
        Self::from_triplets(m, n, &triplets)
    }

    /// Entrywise addition.
    ///
    /// # Errors
    ///
    /// [`LinearError::ShapeMismatch`] if the shapes differ.
    pub fn add_matrix(&self, other: &Self) -> Result<Self, LinearError> {
        if self.shape != other.shape {
            return Err(LinearError::ShapeMismatch {
                left: self.shape,
                right: other.shape,
            });
        }
        let (r, c) = self.shape;
        let mut triplets = Vec::new();
        for i in 0..r {
            for j in 0..c {
                let v = self.get_value_at(i, j) + other.get_value_at(i, j);
                if v != T::zero() {
                    triplets.push((i, j, v));
                }
            }
        }
        Self::from_triplets(r, c, &triplets)
    }

    /// Multiplies every stored entry by `scalar`.
    pub fn scalar_mult(&self, scalar: T) -> Self {
        Self {
            row_indices: self.row_indices.clone(),
            col_indices: self.col_indices.clone(),
            values: self.values.iter().map(|&v| v * scalar).collect(),
            shape: self.shape,
        }
    }
}

impl<T> Default for CsrMatrix<T> {
    fn default() -> Self {
        Self {
            row_indices: alloc::vec![0],
            col_indices: Vec::new(),
            values: Vec::new(),
            shape: (0, 0),
        }
    }
}
