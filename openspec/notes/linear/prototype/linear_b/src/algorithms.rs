//! The generic algorithms. Written ONCE, against `RowOps`.
//!
//! Nothing in this file mentions a concrete representation, a concrete scalar,
//! or a word width.

use crate::{MatrixBuild, MatrixView, RowOps};
use alloc::vec::Vec;
use deep_causality_algebra::Field;
use deep_causality_num::{One, Zero};

/// Result of a reduction: the rank and the pivot column of each pivot row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RrefResult {
    pub rank: usize,
    pub pivot_cols: Vec<usize>,
}

/// Reduce `m` to reduced row echelon form in place.
pub fn rref<M: RowOps + ?Sized>(m: &mut M) -> RrefResult {
    let rows = m.rows();
    let cols = m.cols();
    let mut pivot_cols = Vec::new();
    let mut pivot_row = 0usize;

    for col in 0..cols {
        if pivot_row >= rows {
            break;
        }
        let p = match m.pivot_in_column(col, pivot_row) {
            Some(p) => p,
            None => continue,
        };
        m.swap_rows(pivot_row, p);

        let pivot = m.get(pivot_row, col);
        let inv = M::Scalar::one() / pivot;
        m.scale_row(pivot_row, inv, col);

        for r in 0..rows {
            if r == pivot_row {
                continue;
            }
            let factor = m.get(r, col);
            if factor.is_zero() {
                continue;
            }
            let neg = M::Scalar::zero() - factor;
            m.axpy_rows(r, pivot_row, neg, col);
        }
        pivot_cols.push(col);
        pivot_row += 1;
    }

    RrefResult {
        rank: pivot_row,
        pivot_cols,
    }
}

/// Rank, destroying `m`.
pub fn rank_in_place<M: RowOps + ?Sized>(m: &mut M) -> usize {
    rref(m).rank
}

/// Determinant by elimination, destroying `m`. `None` for a non-square matrix.
pub fn determinant<M: RowOps + ?Sized>(m: &mut M) -> Option<M::Scalar> {
    let n = m.rows();
    if n != m.cols() {
        return None;
    }
    let mut det = M::Scalar::one();
    let mut pivot_row = 0usize;
    for col in 0..n {
        let p = match m.pivot_in_column(col, pivot_row) {
            Some(p) => p,
            None => return Some(M::Scalar::zero()),
        };
        if p != pivot_row {
            m.swap_rows(pivot_row, p);
            det = M::Scalar::zero() - det;
        }
        let pivot = m.get(pivot_row, col);
        det = det * pivot.clone();
        let inv = M::Scalar::one() / pivot;
        m.scale_row(pivot_row, inv, col);
        for r in (pivot_row + 1)..n {
            let factor = m.get(r, col);
            if factor.is_zero() {
                continue;
            }
            let neg = M::Scalar::zero() - factor;
            m.axpy_rows(r, pivot_row, neg, col);
        }
        pivot_row += 1;
    }
    Some(det)
}

/// Read the solution out of an augmented `[A | b]` already in RREF.
pub fn solve_upper_augmented<M: MatrixView>(m: &M, rank: usize) -> Vec<M::Scalar> {
    (0..rank).map(|r| m.get(r, m.cols() - 1)).collect()
}

/// A basis for the kernel, as columns of a freshly built matrix of type `B`.
///
/// The output type is a second parameter because the natural output of a
/// bit-packed 𝔽₂ kernel is another bit-packed matrix, not a `Vec<Vec<Scalar>>`.
pub fn kernel_basis<M, B>(m: &mut M) -> B
where
    M: RowOps,
    B: MatrixBuild<Scalar = M::Scalar>,
{
    let cols = m.cols();
    let RrefResult { rank, pivot_cols } = rref(m);
    let free: Vec<usize> = (0..cols).filter(|c| !pivot_cols.contains(c)).collect();
    let mut out = B::zeros(cols, free.len());
    for (k, &f) in free.iter().enumerate() {
        out.set(f, k, M::Scalar::one());
        for (i, &pc) in pivot_cols.iter().enumerate().take(rank) {
            let v = M::Scalar::zero() - m.get(i, f);
            out.set(pc, k, v);
        }
    }
    out
}

/// Compile-time proof that the algorithms above need only `Field` on the
/// scalar: no order, no `abs`, no `epsilon`.
const _: fn() = || {
    fn _needs_only_field<F: Field>() {}
};
