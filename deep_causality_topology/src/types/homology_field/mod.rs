/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The field a homology rank is taken over, and the one rank helper that takes it.
//!
//! # Why the field is a parameter
//!
//! Rank is a property of a matrix **over a field**, and a boundary matrix has a different rank over
//! ℚ than over 𝔽₂. The two agree for the toric code, which is why the geometric-QEC example's
//! `[[32,2,4]]` comes out right, but that is a property of that code family rather than a general
//! fact: a qLDPC code with an even-weight dependency has a smaller 𝔽₂ rank, and a complex with
//! 2-torsion — ℝP², the Klein bottle — has a larger one.
//!
//! Leaving the choice implicit is how a homology dimension comes out wrong with nothing raised.
//! [`HomologyField`] makes it an argument, so the answer names the field it is an answer over.
//!
//! # What this replaces
//!
//! Two copies of the same helper — `SimplicialComplex`'s `rank_of_csr` and `CellComplex`'s
//! `rank_of_matrix` — each of which densified a `CsrMatrix<i8>` into a `Vec<f64>`, ran an SVD, and
//! counted singular values above `1e-5`. Every Betti number the crate reported rested on that
//! threshold. Both are gone; this is the one that remains, and it constructs no floating-point
//! anything.

use deep_causality_linear::{
    CsrMatrix, DenseMatrix, csr_to_packed_gf2_mod2, rank_exact, rank_gf2,
};

use crate::TopologyError;

/// The field a boundary-matrix rank — and so a Betti number — is taken over.
///
/// Passed at the call site rather than chosen by a default, a feature or a global. See
/// [`ChainComplex::betti_number_over`](crate::ChainComplex::betti_number_over).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HomologyField {
    /// Characteristic zero: rank over ℚ, which for an integer matrix is also its rank over ℝ.
    ///
    /// This is the number the retired SVD path was approximating, so a complex that reported a
    /// Betti number before reports the same one now — exactly, and without a tolerance.
    ///
    /// Computed by fraction-free elimination over ℤ, which never leaves the integers and so never
    /// rounds. Rank is a fraction-field notion, so the answer over ℤ is the answer over ℚ.
    Rational,
    /// Characteristic two: rank over 𝔽₂, by packed mod-2 elimination.
    ///
    /// The field to ask for when the complex is being read as a CSS code, where the chain groups
    /// are 𝔽₂ vector spaces and the rank over ℚ is the wrong question.
    ///
    /// Coefficients are reduced mod 2, so a boundary entry of `-1` is the same as `1` here. That is
    /// the definition of the mod-2 chain complex rather than a lossy conversion.
    Gf2,
}

impl HomologyField {
    /// The rank of a boundary matrix over this field.
    ///
    /// # Errors
    ///
    /// [`TopologyErrorEnum::LinearAlgebraError`](crate::TopologyErrorEnum::LinearAlgebraError) if
    /// the elimination overflows. Only [`HomologyField::Rational`] can raise it: the fraction-free
    /// intermediates are minors, which grow with the matrix, and reporting the overflow is what
    /// keeps a wrapped intermediate from being returned as a rank. The 𝔽₂ path is bit arithmetic
    /// and has nothing to overflow.
    pub fn rank_of(self, matrix: &CsrMatrix<i8>) -> Result<usize, TopologyError> {
        let (rows, cols) = matrix.shape();
        if rows == 0 || cols == 0 {
            return Ok(0);
        }
        match self {
            // Widened to `i64` before elimination. The entries are `i8`, and the fraction-free
            // intermediates are minors of the whole matrix rather than single entries, so `i8`
            // would report an overflow on the second pivot of almost any complex.
            Self::Rational => {
                let widened = widen_to_dense_i64(matrix);
                rank_exact(&widened).map_err(TopologyError::from)
            }
            Self::Gf2 => {
                let packed = csr_to_packed_gf2_mod2::<u64>(matrix);
                rank_gf2(&packed).map_err(TopologyError::from)
            }
        }
    }
}

/// The same matrix over `i64`, dense and row-major.
///
/// Dense because the elimination reads every position and the sparse read is a scan of the row;
/// exact because `i64` is, which is the property that matters. This is where the retired helpers
/// built a `Vec<f64>` and called `svd()`.
///
/// Widened from `i8` because the fraction-free intermediates are minors of the whole matrix rather
/// than single entries, and `i8` would report an overflow on the second pivot of almost anything.
fn widen_to_dense_i64(matrix: &CsrMatrix<i8>) -> DenseMatrix<i64> {
    let (rows, cols) = matrix.shape();
    let mut data = vec![0i64; rows * cols];
    let row_ptrs = matrix.row_indices();
    let col_idxs = matrix.col_indices();
    let values = matrix.values();
    for r in 0..rows {
        for idx in row_ptrs[r]..row_ptrs[r + 1] {
            data[r * cols + col_idxs[idx]] = values[idx] as i64;
        }
    }
    DenseMatrix::from_vec(data, rows, cols).expect("built from the shape")
}
