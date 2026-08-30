#![cfg_attr(not(feature = "std"), no_std)]
//! DESIGN B prototype: `linear` owns ALGORITHMS ONLY, generic over a minimal
//! access trait that every other crate implements for its own representation.
//!
//! There is no matrix type in this crate. Nothing is copied.
//!
//! `no_std` + `alloc`, because `deep_causality_tensor` and
//! `deep_causality_sparse` are both `no-std`-capable and this crate must sit
//! below them.

extern crate alloc;

use deep_causality_algebra::Field;
use deep_causality_num::{One, Zero};

mod algorithms;
mod rows_of_rows;
pub use algorithms::{
    RrefResult, determinant, kernel_basis, rank_in_place, rref, solve_upper_augmented,
};

/// Read access: shape and elements.
///
/// `Scalar: Field` is stated here rather than on each algorithm, so an
/// implementor states it once.
pub trait MatrixView {
    type Scalar: Field;

    fn rows(&self) -> usize;
    fn cols(&self) -> usize;

    /// The entry at `(r, c)`, by value.
    ///
    /// By value, not `&Self::Scalar`: a bit-packed representation has no
    /// element to lend a reference to. `Field: Clone`, so this costs nothing
    /// a reference would have saved for the scalar types in this workspace.
    fn get(&self, r: usize, c: usize) -> Self::Scalar;
}

/// The three row operations Gaussian elimination performs, plus the pivot rule.
///
/// Everything here is a ROW operation. That is the whole point of the design:
/// the O(rows x cols) inner loop lives behind `axpy_rows`, so a representation
/// that can do a whole row at once (bit-packed 𝔽₂: one XOR per 64 columns;
/// a SIMD or BLAS-backed dense row) implements it that way and the generic
/// driver above never sees individual elements in the hot path.
pub trait RowOps: MatrixView {
    /// Exchange two rows. A permutation; not expressible as an `axpy`.
    fn swap_rows(&mut self, a: usize, b: usize);

    /// `row[r][from_col..] *= factor`. Normalises a pivot to 1 for RREF.
    fn scale_row(&mut self, r: usize, factor: Self::Scalar, from_col: usize);

    /// `row[dst][from_col..] += factor * row[src][from_col..]`.
    ///
    /// The inner loop of every elimination. `from_col` lets an implementation
    /// skip the already-eliminated prefix; a float implementation also needs it
    /// to avoid re-introducing round-off into columns that are exactly zero.
    fn axpy_rows(&mut self, dst: usize, src: usize, factor: Self::Scalar, from_col: usize);

    /// Which row to pivot on in `col`, searching at or below `from_row`.
    ///
    /// The pivot RULE is representation-specific and cannot be generic:
    /// an exact field takes the first non-zero, floating point takes the
    /// largest magnitude, and `Field` has neither an order nor an epsilon
    /// with which to express the latter. The default is the exact-field rule.
    fn pivot_in_column(&self, col: usize, from_row: usize) -> Option<usize> {
        (from_row..self.rows()).find(|&r| !self.get(r, col).is_zero())
    }
}

/// Constructing a result (a kernel basis, an inverse) needs an empty matrix of
/// a chosen shape. Split from `RowOps` because a read-only view — a CSR matrix
/// borrowed from a chain complex — can implement elimination-by-copy without
/// being constructible in place.
pub trait MatrixBuild: MatrixView + Sized {
    fn zeros(rows: usize, cols: usize) -> Self;
    fn set(&mut self, r: usize, c: usize, v: Self::Scalar);

    fn identity(n: usize) -> Self {
        let mut m = Self::zeros(n, n);
        for i in 0..n {
            m.set(i, i, Self::Scalar::one());
        }
        m
    }
}
