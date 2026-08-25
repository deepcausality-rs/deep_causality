/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Conversions among the three matrix representations.
//!
//! Explicit, never implicit. A conversion changes the cost model of everything done afterwards —
//! densifying a sparse matrix turns a stored-entry cost into a `rows * cols` one — so it is written
//! at the call site rather than performed inside an algorithm that looked cheap.
//!
//! # Total or fallible by construction
//!
//! | from | to | |
//! |---|---|---|
//! | sparse | dense | total; costs memory |
//! | dense | sparse | total |
//! | dense or sparse | packed 𝔽₂ | **fallible**: the target holds only `{0, 1}` |
//! | packed 𝔽₂ | dense `Gf2` | total |
//!
//! Only the packing direction can fail, and it fails for one reason: an entry outside `{0, 1}`.
//! The error names the position, so a caller does not re-scan to find it.

use crate::errors::linear_error::LinearError;
use crate::types::csr_matrix::CsrMatrix;
use crate::types::dense_matrix::DenseMatrix;
use crate::types::packed_gf2::PackedGf2;
use deep_causality_algebra::CommutativeSemiring;
use deep_causality_num::{Gf2, NaturalNumber};

/// Densifies a sparse matrix, materialising its structural zeros.
pub fn csr_to_dense<T>(m: &CsrMatrix<T>) -> DenseMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    let (r, c) = m.shape();
    let mut out = alloc::vec::Vec::with_capacity(r * c);
    for i in 0..r {
        for j in 0..c {
            out.push(m.get_value_at(i, j));
        }
    }
    DenseMatrix::from_vec(out, r, c).expect("the buffer is built from the shape")
}

/// Sparsifies a dense matrix, storing only its non-zeros.
pub fn dense_to_csr<T>(m: &DenseMatrix<T>) -> CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    use crate::traits::matrix_view::MatrixView;
    let (r, c) = (m.rows(), m.cols());
    let mut triplets = alloc::vec::Vec::new();
    for i in 0..r {
        for j in 0..c {
            let v = m.get(i, j).expect("indices come from the shape");
            if v != T::zero() {
                triplets.push((i, j, v));
            }
        }
    }
    CsrMatrix::from_triplets(r, c, &triplets).expect("indices come from the shape")
}

/// Packs a dense matrix of 𝔽₂ entries.
pub fn dense_gf2_to_packed<W>(m: &DenseMatrix<Gf2>) -> Result<PackedGf2<W>, LinearError>
where
    W: NaturalNumber,
{
    use crate::traits::matrix_view::MatrixView;
    let (r, c) = (m.rows(), m.cols());
    let mut flat = alloc::vec::Vec::with_capacity(r * c);
    for i in 0..r {
        for j in 0..c {
            flat.push(m.get(i, j)?);
        }
    }
    PackedGf2::from_slice(&flat, r, c)
}

/// Unpacks into a dense matrix of 𝔽₂ entries.
pub fn packed_to_dense_gf2<W>(m: &PackedGf2<W>) -> DenseMatrix<Gf2>
where
    W: NaturalNumber,
{
    use crate::traits::matrix_view::MatrixView;
    let (r, c) = (m.rows(), m.cols());
    let mut out = alloc::vec::Vec::with_capacity(r * c);
    for i in 0..r {
        for j in 0..c {
            out.push(m.get(i, j).expect("indices come from the shape"));
        }
    }
    DenseMatrix::from_vec(out, r, c).expect("the buffer is built from the shape")
}

/// Packs an integer matrix by reducing every entry modulo 2.
///
/// This is the conversion `deep_causality_topology`'s boundary operators need, and it is **total**
/// rather than fallible: every integer has a residue mod 2, and `-1` and `1` are both the 𝔽₂ one.
/// It is distinct from [`csr_to_packed_gf2_strict`] because reducing and rejecting are different
/// intentions, and a caller who means one should not silently get the other.
pub fn csr_to_packed_gf2_mod2<W>(m: &CsrMatrix<i8>) -> PackedGf2<W>
where
    W: NaturalNumber,
{
    let (r, c) = m.shape();
    let mut flat = alloc::vec::Vec::with_capacity(r * c);
    for i in 0..r {
        for j in 0..c {
            flat.push(i64::from(m.get_value_at(i, j)));
        }
    }
    PackedGf2::from_i64_mod2(&flat, r, c).expect("the buffer is built from the shape")
}

/// Packs an integer matrix, rejecting any entry outside `{0, 1}`.
///
/// # Errors
///
/// [`LinearError::NotBinary`], naming the first offending position.
pub fn csr_to_packed_gf2_strict<W>(m: &CsrMatrix<i8>) -> Result<PackedGf2<W>, LinearError>
where
    W: NaturalNumber,
{
    let (r, c) = m.shape();
    let mut flat = alloc::vec::Vec::with_capacity(r * c);
    for i in 0..r {
        for j in 0..c {
            let v = m.get_value_at(i, j);
            if v != 0 && v != 1 {
                return Err(LinearError::NotBinary((i, j)));
            }
            flat.push(i64::from(v));
        }
    }
    PackedGf2::from_i64_mod2(&flat, r, c)
}
